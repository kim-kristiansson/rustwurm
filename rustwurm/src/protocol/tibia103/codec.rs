//! Tibia 1.03 protocol codec
//!
//! This codec translates between wire format and engine messages.
//! It handles the special login packet format and standard opcode-based packets.
//!
//! ## Client → Server Opcodes (Section 6.1)
//!
//! - 0x05: Walk (direction in payload)
//! - 0x06: AutoWalk (x, y in payload)
//! - 0x09: Chat (length-prefixed message)
//! - 0x0A: ChangeDirection (direction in payload)
//! - 0x14: Push (move item)
//! - 0x34: SetTarget (creature ID)
//! - 0xFF: Logout

use std::io::{Read, Write};

use crate::engine::{ClientMessage, ServerMessage, PlayerId, Direction as EngineDirection};
use crate::error::{ProtocolError, ProtocolResult};
use crate::protocol::traits::{ClientCodec, ServerCodec, Protocol};

use super::constants::{ClientOpcode, EquipmentSlot, Direction, ChatType};
use super::frame::{Frame, ServerFrame};
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

        let Some(opcode) = ClientOpcode::from_u8(opcode_raw) else {
            // Unknown opcode - log and skip
            eprintln!(
                "[v1.03] Unknown client opcode: {:#04x}, payload_len={}",
                opcode_raw, payload.len()
            );
            return Ok(None);
        };

        let msg = match opcode {
            // Walk (0x05) - single direction byte
            ClientOpcode::Walk => {
                let dir_byte = reader.read_u8()?;
                let Some(dir) = Direction::from_u8(dir_byte) else {
                    return Err(ProtocolError::InvalidPacket(format!(
                        "Invalid walk direction: {:#04x}", dir_byte
                    )));
                };
                let (dx, dy) = dir.to_delta();
                ClientMessage::Move { player_id, dx, dy }
            }

            // AutoWalk (0x06) - x, y destination
            ClientOpcode::AutoWalk => {
                let _x = reader.read_u8()?;
                let _y = reader.read_u8()?;
                // TODO: Implement auto-walk pathfinding
                // For now, just ignore
                return Ok(None);
            }

            // LookAt (0x07) - x, y position
            ClientOpcode::LookAt => {
                let _x = reader.read_u8()?;
                let _y = reader.read_u8()?;
                // TODO: Implement look at
                return Ok(None);
            }

            // Chat (0x09) - length-prefixed message
            ClientOpcode::Chat => {
                let message = reader.read_string()?;
                ClientMessage::Say { player_id, message }
            }

            // ChangeDirection (0x0A) - direction byte
            ClientOpcode::ChangeDirection => {
                let dir_byte = reader.read_u8()?;
                let Some(dir) = Direction::from_u8(dir_byte) else {
                    return Err(ProtocolError::InvalidPacket(format!(
                        "Invalid turn direction: {:#04x}", dir_byte
                    )));
                };
                let engine_dir = match dir {
                    Direction::North => EngineDirection::North,
                    Direction::East => EngineDirection::East,
                    Direction::South => EngineDirection::South,
                    Direction::West => EngineDirection::West,
                };
                ClientMessage::Turn { player_id, direction: engine_dir }
            }

            // Push (0x14) - move item
            ClientOpcode::Push => {
                let _from_x = reader.read_u8()?;
                let _from_y = reader.read_u8()?;
                let _item_id = reader.read_u16()?;
                let _stack_pos = reader.read_u8()?;
                let _to_x = reader.read_u8()?;
                let _to_y = reader.read_u8()?;
                // TODO: Implement item movement
                return Ok(None);
            }

            // UseItem (0x1E)
            ClientOpcode::UseItem => {
                let _use_type = reader.read_u8()?;
                let _x = reader.read_u8()?;
                let _y = reader.read_u8()?;
                let item_id = reader.read_u16()?;
                let _stack_pos = reader.read_u8()?;
                let _unknown = reader.read_u8()?;
                ClientMessage::UseItem { player_id, item_id }
            }

            // CloseContainer (0x1F)
            ClientOpcode::CloseContainer => {
                let _local_id = reader.read_u8()?;
                // TODO: Implement container close
                return Ok(None);
            }

            // ChangeMode (0x32) - fight mode and stance
            ClientOpcode::ChangeMode => {
                let _fight_mode = reader.read_u8()?;
                let _stance = reader.read_u8()?;
                // TODO: Implement fight mode change
                return Ok(None);
            }

            // ExitBattle (0x33) - stop attacking
            ClientOpcode::ExitBattle => {
                ClientMessage::Cancel { player_id }
            }

            // SetTarget (0x34) - attack target
            ClientOpcode::SetTarget => {
                let target_id = reader.read_u32()?;
                if target_id == 0 {
                    ClientMessage::Cancel { player_id }
                } else {
                    ClientMessage::AttackTarget { player_id, target_id }
                }
            }

            // Echo (0xC8) - keep-alive
            ClientOpcode::Echo => {
                // Just acknowledge, no action needed
                return Ok(None);
            }

            // Logout (0xFF)
            ClientOpcode::Logout => {
                ClientMessage::Logout { player_id }
            }

            // Other opcodes not yet implemented
            ClientOpcode::UserList |
            ClientOpcode::PlayerInfo |
            ClientOpcode::Comment |
            ClientOpcode::RequestChangeData |
            ClientOpcode::SetData |
            ClientOpcode::SetText |
            ClientOpcode::HouseText => {
                eprintln!("[v1.03] Unimplemented opcode: {:?}", opcode);
                return Ok(None);
            }
        };

        Ok(Some(msg))
    }

    /// Write multiple server frames to the writer
    fn write_frames(&self, writer: &mut dyn Write, frames: Vec<ServerFrame>) -> ProtocolResult<()> {
        for frame in frames {
            frame.write_to(writer)?;
        }
        Ok(())
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
                // Debug: dump raw packet for analysis
                eprintln!("[DEBUG] Received packet in AwaitingLogin state:");
                eprintln!("[DEBUG]   Body length: {} bytes", frame.body.len());
                if frame.body.len() <= 100 {
                    eprintln!("[DEBUG]   Body (hex): {:02X?}", &frame.body);
                } else {
                    eprintln!("[DEBUG]   First 50 bytes: {:02X?}", &frame.body[..50]);
                }
                if let Some(op) = frame.opcode() {
                    eprintln!("[DEBUG]   First byte: {:#04x}", op);
                }

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
                    eprintln!("[DEBUG] Login check failed:");
                    eprintln!("[DEBUG]   Expected body length: 67, got: {}", frame.body.len());
                    if frame.body.len() >= 5 {
                        eprintln!("[DEBUG]   Expected magic: {:02X?}", super::constants::LOGIN_MAGIC);
                        eprintln!("[DEBUG]   Got first 5 bytes: {:02X?}", &frame.body[..5.min(frame.body.len())]);
                    }
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

                // Send login response sequence:
                // 1. Login confirmation (0x01)
                // 2. Equipped items (0x14)
                // 3. Info message (0x04)
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
                vec![server_packets::error_message(reason)]
            }

            ServerMessage::TextMessage { message } => {
                vec![server_packets::info_message(message)]
            }

            ServerMessage::CreatureSay { name, message, .. } => {
                // Use chat message with position 0,0 for now
                vec![server_packets::chat_message(0, 0, ChatType::Normal, name, message)]
            }

            ServerMessage::PlayerMoved { x, y, .. } => {
                // For now, just log - full implementation needs map data
                eprintln!("[v1.03] Player moved to ({}, {})", x, y);
                vec![]
            }

            ServerMessage::PlayerDied { .. } => {
                vec![server_packets::status_message("You are dead.")]
            }

            ServerMessage::PlayerStats { .. } => {
                // Tibia 1.03 doesn't have the standard stats packet
                // Stats were displayed differently in this version
                eprintln!("[v1.03] PlayerStats not supported in this protocol version");
                vec![]
            }

            ServerMessage::CreatureHealth { .. } => {
                // Tibia 1.03 didn't have creature health bars
                eprintln!("[v1.03] CreatureHealth not supported in this protocol version");
                vec![]
            }

            ServerMessage::MapDescription { .. } => {
                // TODO: Implement map description
                eprintln!("[v1.03] MapDescription not yet implemented");
                vec![]
            }

            ServerMessage::CreatureMoved { .. } |
            ServerMessage::CreatureAppear { .. } |
            ServerMessage::CreatureDisappear { .. } => {
                // TODO: Implement creature updates
                eprintln!("[v1.03] Creature updates not yet implemented");
                vec![]
            }
        };

        self.write_frames(writer, frames)
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

        // Should have multiple packets (login ok, equipped items, message)
        assert!(output.len() > 10);

        // Codec should be in game state now
        assert_eq!(codec.state, ConnectionState::InGame);
        assert_eq!(codec.player_id, Some(1));

        // Verify the first packet is login (0x01)
        // Skip length (2 bytes), 4 zeros, then opcode
        assert_eq!(output[6], 0x01);
    }

    #[test]
    fn test_walk_packet() {
        let mut codec = Codec::new();
        codec.player_id = Some(42);
        codec.state = ConnectionState::InGame;

        // Build a walk packet: opcode 0x05, direction 0x00 (North)
        let frame = FrameBuilder::with_opcode(0x05)
            .u8(0x00) // Direction: North
            .build();

        let mut data = Vec::new();
        frame.write_to(&mut data).unwrap();

        // Should be: 02 00 05 00 (length=2, opcode=0x05, direction=0x00)
        assert_eq!(data, vec![0x02, 0x00, 0x05, 0x00]);

        let mut cursor = Cursor::new(data);
        let msg = codec.read_message(&mut cursor).unwrap();

        assert!(matches!(msg, Some(ClientMessage::Move { player_id: 42, dx: 0, dy: -1 })));
    }

    #[test]
    fn test_walk_directions() {
        let mut codec = Codec::new();
        codec.player_id = Some(1);
        codec.state = ConnectionState::InGame;

        let test_cases = [
            (0x00, 0, -1),  // North
            (0x01, 1, 0),   // East
            (0x02, 0, 1),   // South
            (0x03, -1, 0),  // West
        ];

        for (dir_byte, expected_dx, expected_dy) in test_cases {
            let frame = FrameBuilder::with_opcode(0x05)
                .u8(dir_byte)
                .build();

            let mut data = Vec::new();
            frame.write_to(&mut data).unwrap();

            let mut cursor = Cursor::new(data);
            let msg = codec.read_message(&mut cursor).unwrap();

            if let Some(ClientMessage::Move { dx, dy, .. }) = msg {
                assert_eq!((dx, dy), (expected_dx, expected_dy),
                           "Direction {:#04x} failed", dir_byte);
            } else {
                panic!("Expected Move message for direction {:#04x}", dir_byte);
            }
        }
    }

    #[test]
    fn test_chat_packet() {
        let mut codec = Codec::new();
        codec.player_id = Some(1);
        codec.state = ConnectionState::InGame;

        // Build a chat packet: opcode 0x09, length-prefixed message
        let frame = FrameBuilder::with_opcode(0x09)
            .string("Hello, World!")
            .build();

        let mut data = Vec::new();
        frame.write_to(&mut data).unwrap();

        let mut cursor = Cursor::new(data);
        let msg = codec.read_message(&mut cursor).unwrap();

        if let Some(ClientMessage::Say { player_id, message }) = msg {
            assert_eq!(player_id, 1);
            assert_eq!(message, "Hello, World!");
        } else {
            panic!("Expected Say message");
        }
    }

    #[test]
    fn test_turn_packet() {
        let mut codec = Codec::new();
        codec.player_id = Some(1);
        codec.state = ConnectionState::InGame;

        // Build a turn packet: opcode 0x0A, direction 0x01 (East)
        let frame = FrameBuilder::with_opcode(0x0A)
            .u8(0x01)
            .build();

        let mut data = Vec::new();
        frame.write_to(&mut data).unwrap();

        let mut cursor = Cursor::new(data);
        let msg = codec.read_message(&mut cursor).unwrap();

        if let Some(ClientMessage::Turn { direction, .. }) = msg {
            assert_eq!(direction, EngineDirection::East);
        } else {
            panic!("Expected Turn message");
        }
    }

    #[test]
    fn test_logout_packet() {
        let mut codec = Codec::new();
        codec.player_id = Some(1);
        codec.state = ConnectionState::InGame;

        // Build a logout packet: opcode 0xFF
        let frame = FrameBuilder::with_opcode(0xFF).build();

        let mut data = Vec::new();
        frame.write_to(&mut data).unwrap();

        let mut cursor = Cursor::new(data);
        let msg = codec.read_message(&mut cursor).unwrap();

        assert!(matches!(msg, Some(ClientMessage::Logout { player_id: 1 })));
    }

    #[test]
    fn test_attack_packet() {
        let mut codec = Codec::new();
        codec.player_id = Some(1);
        codec.state = ConnectionState::InGame;

        // Build an attack packet: opcode 0x34, target ID
        let frame = FrameBuilder::with_opcode(0x34)
            .u32(12345)
            .build();

        let mut data = Vec::new();
        frame.write_to(&mut data).unwrap();

        let mut cursor = Cursor::new(data);
        let msg = codec.read_message(&mut cursor).unwrap();

        if let Some(ClientMessage::AttackTarget { target_id, .. }) = msg {
            assert_eq!(target_id, 12345);
        } else {
            panic!("Expected AttackTarget message");
        }
    }

    #[test]
    fn test_stop_attack_packet() {
        let mut codec = Codec::new();
        codec.player_id = Some(1);
        codec.state = ConnectionState::InGame;

        // SetTarget with ID 0 means stop attacking
        let frame = FrameBuilder::with_opcode(0x34)
            .u32(0)
            .build();

        let mut data = Vec::new();
        frame.write_to(&mut data).unwrap();

        let mut cursor = Cursor::new(data);
        let msg = codec.read_message(&mut cursor).unwrap();

        assert!(matches!(msg, Some(ClientMessage::Cancel { .. })));
    }
}