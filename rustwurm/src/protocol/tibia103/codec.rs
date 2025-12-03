//! Tibia 1.03 protocol codec
//!
//! This codec translates between wire format and engine messages.
//! It handles the special login packet format and standard opcode-based packets.

use std::io::{Read, Write};

use crate::engine::{ClientMessage, ServerMessage, PlayerId};
use crate::error::{ProtocolError, ProtocolResult};
use crate::protocol::traits::{ClientCodec, ServerCodec, Protocol};

use super::constants::{ClientOpcode, EquipmentSlot, MessageType, MAP_WIDTH, MAP_HEIGHT};
use super::frame::Frame;
use super::primitives::PayloadReader;
use super::login::{is_login_packet, parse_login};
use super::server_packets::{self, Position};

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

        let Some(opcode) = ClientOpcode::from_u8(opcode_raw) else {
            // Unknown opcode - log and skip
            eprintln!(
                "[v1.03] Unknown client opcode: {:#04x}, payload_len={}",
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
                ClientMessage::Cancel { player_id }
            }

            // Turn commands (facing direction, no movement)
            ClientOpcode::TurnNorth => {
                ClientMessage::Turn { player_id, direction: crate::engine::Direction::North }
            }
            ClientOpcode::TurnEast => {
                ClientMessage::Turn { player_id, direction: crate::engine::Direction::East }
            }
            ClientOpcode::TurnSouth => {
                ClientMessage::Turn { player_id, direction: crate::engine::Direction::South }
            }
            ClientOpcode::TurnWest => {
                ClientMessage::Turn { player_id, direction: crate::engine::Direction::West }
            }

            // Say something
            ClientOpcode::Say => {
                let _speak_type = reader.read_u8()?;
                let message = reader.read_string()?;
                ClientMessage::Say { player_id, message }
            }

            // Attack
            ClientOpcode::Attack => {
                let target_id = reader.read_u32()?;
                if target_id == 0 {
                    // Stop attacking
                    ClientMessage::Cancel { player_id }
                } else {
                    ClientMessage::AttackTarget { player_id, target_id }
                }
            }

            // Follow
            ClientOpcode::Follow => {
                let _target_id = reader.read_u32()?;
                // TODO: Add Follow message to engine
                return Ok(None);
            }

            // Logout
            ClientOpcode::Logout => {
                ClientMessage::Logout { player_id }
            }

            // Auto-walk (path)
            ClientOpcode::AutoWalk => {
                // TODO: Parse path and implement auto-walk
                return Ok(None);
            }

            // Item interactions
            ClientOpcode::MoveItem | ClientOpcode::UseItem |
            ClientOpcode::UseItemWith | ClientOpcode::UseItemOnCreature |
            ClientOpcode::RotateItem => {
                // TODO: Parse item location and implement
                return Ok(None);
            }

            // Channel operations
            ClientOpcode::RequestChannels | ClientOpcode::OpenChannel |
            ClientOpcode::CloseChannel => {
                // TODO: Implement channels
                return Ok(None);
            }

            // Login shouldn't appear here (handled in AwaitingLogin state)
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

                // The client expects a full initialization sequence:
                // 1. InitGame (0x0A) - player ID, session flags, can report bugs
                // 2. PlayerDataBasic (0x9F) - player ID and position
                // 3. PlayerData (0xA0) - HP, mana, level, etc.
                // 4. PlayerSkills (0xA1) - all skill levels
                // 5. FullMap (0x64) - 18x14 tiles around player
                // 6. Equipment (0x78) - items in slots
                // 7. TextMessage (0xB4) - welcome message

                // Default spawn position (center of a simple map)
                let spawn_x: u16 = 100;
                let spawn_y: u16 = 100;
                let spawn_z: u8 = 7;

                let center = Position::new(spawn_x, spawn_y, spawn_z);

                vec![
                    // 1. InitGame - tells client their creature ID
                    server_packets::init_game(*player_id),

                    // 2. PlayerDataBasic - position info
                    server_packets::player_data_basic(*player_id, spawn_x, spawn_y, spawn_z),

                    // 3. PlayerData - stats
                    server_packets::player_data(
                        100,    // HP
                        100,    // Max HP
                        150,    // Capacity
                        0,      // Experience
                        1,      // Level
                        50,     // Mana
                        50,     // Max Mana
                        0,      // Magic Level
                        0,      // Magic Level %
                    ),

                    // 4. PlayerSkills
                    server_packets::player_skills_default(),

                    // 5. FullMap - send basic grass/wall map
                    server_packets::full_map_simple(center, |x, y| {
                        // Simple 20x20 room with walls around edges
                        let in_bounds = x >= 90 && x <= 110 && y >= 90 && y <= 110;
                        let is_wall = x == 90 || x == 110 || y == 90 || y == 110;
                        in_bounds && !is_wall
                    }),

                    // 6. Equipment - give player a backpack
                    server_packets::equipped_item(0x013D, EquipmentSlot::Backpack),

                    // 7. Welcome message
                    server_packets::info_message("Welcome to Rustwurm!"),
                ]
            }

            ServerMessage::LoginFailed { reason } => {
                vec![server_packets::login_failed(reason)]
            }

            ServerMessage::TextMessage { message } => {
                vec![server_packets::info_message(message)]
            }

            ServerMessage::PlayerStats { hp, max_hp, level, xp, mana, max_mana, .. } => {
                vec![server_packets::player_data(
                    *hp as u16,
                    *max_hp as u16,
                    100, // cap
                    *xp as u32,
                    *level as u16,
                    *mana as u16,
                    *max_mana as u16,
                    0, // magic level
                    0, // magic level %
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
    use super::super::frame::FrameBuilder;

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

        // Should have multiple packets now (init, data basic, data, skills, map, equip, msg)
        assert!(output.len() > 100, "Login response should be substantial, got {} bytes", output.len());

        // Codec should be in game state now
        assert_eq!(codec.state, ConnectionState::InGame);
        assert_eq!(codec.player_id, Some(1));
    }

    #[test]
    fn test_movement_packet() {
        let mut codec = Codec::new();
        codec.player_id = Some(42);
        codec.state = ConnectionState::InGame;

        // Build a move north packet (opcode 0x65, no payload)
        let frame = FrameBuilder::with_opcode(0x65).build();
        let mut data = Vec::new();
        frame.write_to(&mut data).unwrap();

        // Should be: 01 00 65 (length=1, opcode)
        assert_eq!(data, vec![0x01, 0x00, 0x65]);

        let mut cursor = Cursor::new(data);
        let msg = codec.read_message(&mut cursor).unwrap();

        assert!(matches!(msg, Some(ClientMessage::Move { player_id: 42, dx: 0, dy: -1 })));
    }

    #[test]
    fn test_say_packet() {
        let mut codec = Codec::new();
        codec.player_id = Some(1);
        codec.state = ConnectionState::InGame;

        // Build a say packet: opcode 0x96, speak_type 0x01, string "Hi"
        let mut builder = FrameBuilder::with_opcode(0x96);
        builder.write_u8(0x01); // speak type = say
        builder.write_string("Hi");
        let frame = builder.build();

        let mut data = Vec::new();
        frame.write_to(&mut data).unwrap();

        let mut cursor = Cursor::new(data);
        let msg = codec.read_message(&mut cursor).unwrap();

        if let Some(ClientMessage::Say { player_id, message }) = msg {
            assert_eq!(player_id, 1);
            assert_eq!(message, "Hi");
        } else {
            panic!("Expected Say message");
        }
    }
}