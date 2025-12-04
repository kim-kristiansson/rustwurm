//! Tibia 1.03 protocol implementation
//!
//! This module implements the wire protocol for Tibia version 1.03,
//! the earliest documented version of the Tibia protocol (circa 1997).
//!
//! # Architecture
//!
//! - [`constants`] - Protocol constants, opcodes, magic bytes
//! - [`frame`] - Packet framing (length-prefixed, with server header prefix)
//! - [`primitives`] - Low-level data type readers
//! - [`login`] - Login packet handling
//! - [`server_packets`] - Server packet builders
//! - [`codec`] - Main codec implementation
//!
//! # Protocol Overview
//!
//! ## Key Characteristics
//!
//! - No encryption (XTEA/RSA introduced in later versions)
//! - TCP-based on port 7171
//! - Little-endian byte order
//! - 8-bit X/Y coordinates (max map size 256×256)
//! - Single map layer (Z = 7, ground level)
//!
//! ## Client → Server Packets
//!
//! Client packets use simple framing:
//! ```text
//! [u16_le Length][Opcode (1B)][Payload...]
//! ```
//!
//! Key opcodes:
//! - `0x05`: Walk (direction byte)
//! - `0x09`: Chat (length-prefixed message)
//! - `0x0A`: Turn (direction byte)
//! - `0x14`: Push/Move item
//! - `0x34`: Set attack target
//! - `0xFF`: Logout
//!
//! ## Server → Client Packets
//!
//! Server packets have a 4-zero-byte prefix before the opcode:
//! ```text
//! [u16_le Length][0x00 0x00 0x00 0x00][Opcode (1B)][Payload...]
//! ```
//! where Length = 4 + 1 + payload.len()
//!
//! Key opcodes:
//! - `0x01`: Login confirmation
//! - `0x02`: Error message
//! - `0x04`: Info popup
//! - `0x0A`: Full map description
//! - `0x0B-0x0E`: Map scroll (N/E/S/W)
//! - `0x14`: Equipped item (Note: Item ID before Slot!)
//! - `0x65`: Chat message
//! - `0x68`: Status message
//!
//! ## Login Packet
//!
//! The game login packet has a special fixed format (67 bytes body):
//! ```text
//! [u16_le Length = 67]
//! [Magic: 00 00 01 01 00]
//! [Protocol: u16_le = 0x0067]
//! [Name: 30 bytes, null-padded]
//! [Password: 30 bytes, null-padded]
//! ```
//!
//! ## Login Response Sequence
//!
//! On successful login, the server sends:
//! 1. Login (0x01) - confirmation
//! 2. EquippedItem (0x14) - for each inventory slot
//! 3. Map (0x0A) - surrounding tiles
//! 4. Info (0x04) - welcome message
//!
//! # Usage
//!
//! The primary interface is the [`Codec`] struct:
//!
//! ```ignore
//! use rustwurm::protocol::tibia103::Codec;
//! use rustwurm::protocol::Protocol;
//!
//! let codec = Codec::new();
//! println!("Protocol version: {}", Codec::version());
//! ```

pub mod constants;
pub mod frame;
pub mod primitives;
pub mod login;
pub mod server_packets;
pub mod codec;

// Re-exports for convenient access
pub use codec::Codec;
pub use constants::{
    ClientOpcode, ServerOpcode, EquipmentSlot, ChatType, Direction,
    FightMode, FightStance,
    LOGIN_MAGIC, PROTOCOL_VERSION, LOGIN_BODY_LENGTH,
    MAP_WIDTH, MAP_HEIGHT, SERVER_HEADER_PREFIX_SIZE,
};
pub use frame::{Frame, FrameBuilder, ServerFrame, ServerFrameBuilder};
pub use login::LoginCredentials;
pub use server_packets::{Position, OutfitColors, TileData, MapBuilder};