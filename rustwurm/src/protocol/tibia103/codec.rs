//! Tibia 1.03 protocol codec
//!
//! This codec translates between wire format and engine messages.
//! It handles the special login packet format and standard opcode-based packets.

use std::io::{Read, Write};

use crate::engine::{ClientMessage, ServerMessage, PlayerId};
use crate::error::{ProtocolError, ProtocolResult};
use crate::protocol::traits::{ClientCodec, ServerCodec, Protocol};

use super::constants::{ClientOpcode, ServerOpcode, EquipmentSlot, MessageType};
use super::frame::Frame;
use super::primitives::PayloadReader;
use super::login::{is_login_packet, parse_login};
use super::server_packets;

/// Connection state for tracking protocol flow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionState {
    /// Waiting for login packet
    AwaitingLogin,
    /// Logged in, processing game packets
    InGame,
}

/// Tibia 1.03 protocol codec
pub struct Codec {
    state: ConnectionState,
    player_id: Option<PlayerId>,
}

impl Codec {
    pub fn new() -> Self {
        Self {
            state: ConnectionState::AwaitingLogin,
            player_id: None,
        }
    }

    /// Set the player ID after successful login
    pub fn set_player_id(&mut self, id: PlayerId) {
        self.player_id = Some(id);
        self.state = ConnectionState::InGame;
    }

    /// Parse a game packet (non-login)
    fn parse_game_packet(&self, frame: &Frame) -> ProtocolResult<Option<ClientMessage>> {
        let Some(opcode_raw) = frame.opcode() else {
            return Err(ProtocolError::InvalidPacket("Empty packet".to_string()));
        };

        let payload = frame.payload();
        let mut reader = PayloadReader::new(payload);

        // Get player ID (required for game packets)
        let player_id = self.player_id.ok_or_else(|| {
            ProtocolError::InvalidPacket("Game packet before login".to_string())
        })?;

        let Some(opcode) = ClientOpcode::from_u16(opcode_raw) else {
            // Unknown opcode - log and skip
            eprintln!(
                "[v1.03] Unknown client opcode: {:#06x}, payload_len={}",
                opcode_raw, payload.len()
            );
            return Ok(None);
        };

        let msg = match opcode {
            // Movement opcodes (no payload)
            ClientOpcode::MoveNorth => ClientMessage::Move { player_id, dx: 0, dy: -1 },
            ClientOpcode::MoveEast => ClientMessage::Move { player_id, dx: 1, dy: 0 },
            ClientOpcode::MoveSouth => ClientMessage::Move { player_id, dx: 0, dy: 1 },
            ClientOpcode::MoveWest => ClientMessage::Move { player_id, dx: -1, dy: 0 },

            // Diagonal movement
            ClientOpcode::MoveNorthEast => ClientMessage::Move { player_id, dx: 1, dy: -1 },
            ClientOpcode::MoveSouthEast => ClientMessage::Move { player_id, dx: 1, dy: 1 },
            ClientOpcode::MoveSouthWest => ClientMessage::Move { player_id, dx: -1, dy: 1 },
            ClientOpcode::MoveNorthWest => ClientMessage::Move { player_id, dx: -1, dy: -1 },

            // Stop/cancel
            ClientOpcode::StopWalk | ClientOpcode::CancelAction => {
                // No direct engine message, just acknowledge
                return Ok(None);
            }

            // Turn commands (facing direction, no movement)
            ClientOpcode::TurnNorth | ClientOpcode::TurnEast |
            ClientOpcode::TurnSouth | ClientOpcode::TurnWest => {
                // TODO: Add Turn message to engine
                return Ok(None);
            }

            // Say something
            ClientOpcode::Say => {
                let _speak_type = reader.read_u8()?;
                let message = reader.read_string()?;
                ClientMessage::Say { player_id, message }
            }

            // Attack
            ClientOpcode::Attack => {
                let _target_id = reader.read_u32()?;
                // For now, simplified attack (engine will find adjacent target)
                ClientMessage::Attack { player_id }
            }

            // Logout
            ClientOpcode::Logout => {
                ClientMessage::Logout { player_id }
            }

            // Login shouldn't appear here
            ClientOpcode::GameLogin => {
                return Err(ProtocolError::InvalidPacket(
                    "Unexpected login packet in game state".to_string()
                ));
            }
        };

        Ok(Some(msg))
    }
}

impl Default for Codec {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientCodec for Codec {
    fn read_message(&mut self, reader: &mut dyn Read) -> ProtocolResult<Option<ClientMessage>> {
        let frame = Frame::read_from(reader)?;

        match self.state {
            ConnectionState::AwaitingLogin => {
                // Check if this is a login packet
                if is_login_packet(&frame) {
                    let creds = parse_login(&frame)?;

                    if !creds.is_valid_version() {
                        return Err(ProtocolError::InvalidPacket(format!(
                            "Unsupported protocol version: {:#06x}",
                            creds.protocol_version
                        )));
                    }

                    Ok(Some(ClientMessage::Login {
                        name: creds.name,
                        password: creds.password,
                    }))
                } else {
                    Err(ProtocolError::InvalidPacket(
                        "Expected login packet".to_string()
                    ))
                }
            }

            ConnectionState::InGame => {
                self.parse_game_packet(&frame)
            }
        }
    }
}

impl ServerCodec for Codec {
    fn write_message(&mut self, writer: &mut dyn Write, msg: &ServerMessage) -> ProtocolResult<()> {
        let frames = match msg {
            ServerMessage::LoginOk { player_id } => {
                // Update codec state
                self.player_id = Some(*player_id);
                self.state = ConnectionState::InGame;

                // Send login response sequence
                vec![
                    server_packets::login_ok(),
                    // Example equipped items (backpack, sword, shield)
                    server_packets::equipped_item(0x013D, EquipmentSlot::Backpack),
                    server_packets::equipped_item(0x015A, EquipmentSlot::RightHand),
                    server_packets::equipped_item(0x025A, EquipmentSlot::LeftHand),
                    // Welcome message
                    server_packets::info_message("Welcome to Rustwurm!"),
                ]
            }

            ServerMessage::LoginFailed { reason } => {
                vec![server_packets::login_failed(reason)]
            }

            ServerMessage::TextMessage { message } => {
                vec![server_packets::info_message(message)]
            }

            ServerMessage::PlayerStats { hp, max_hp, level, xp, .. } => {
                vec![server_packets::player_stats(
                    *hp as u16,
                    *max_hp as u16,
                    100, // cap
                    *xp as u32,
                    *level as u16,
                    100, // mana
                    100, // max_mana
                )]
            }

            ServerMessage::CreatureHealth { creature_id, health_percent } => {
                vec![server_packets::creature_health(*creature_id, *health_percent)]
            }

            ServerMessage::PlayerMoved { x, y, .. } => {
                // For now, just log - full implementation needs map data
                eprintln!("[v1.03] Player moved to ({}, {})", x, y);
                vec![]
            }

            ServerMessage::PlayerDied { .. } => {
                vec![server_packets::text_message(
                    "You are dead.",
                    MessageType::Warning,
                )]
            }

            _ => {
                eprintln!("[v1.03] Unimplemented server message: {:?}", msg);
                vec![]
            }
        };

        // Write all frames
        for frame in frames {
            frame.write_to(writer)?;
        }

        Ok(())
    }
}

impl Protocol for Codec {
    fn version() -> &'static str {
        "1.03"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use super::super::login::build_login;

    #[test]
    fn test_login_flow() {
        let mut codec = Codec::new();

        // Build a login packet
        let login_frame = build_login("TestPlayer", "password123");
        let mut data = Vec::new();
        login_frame.write_to(&mut data).unwrap();

        // Parse it
        let mut cursor = Cursor::new(data);
        let msg = codec.read_message(&mut cursor).unwrap();

        assert!(matches!(msg, Some(ClientMessage::Login { .. })));

        if let Some(ClientMessage::Login { name, password }) = msg {
            assert_eq!(name, "TestPlayer");
            assert_eq!(password, "password123");
        }
    }

    #[test]
    fn test_login_response() {
        let mut codec = Codec::new();
        let mut output = Vec::new();

        codec.write_message(&mut output, &ServerMessage::LoginOk { player_id: 1 }).unwrap();

        // Should have multiple packets (login ok, equipped items, message)
        assert!(output.len() > 10);

        // Codec should be in game state now
        assert_eq!(codec.state, ConnectionState::InGame);
        assert_eq!(codec.player_id, Some(1));
    }
}