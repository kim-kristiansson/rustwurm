//! Tibia 1.03 protocol implementation
//!
//! This module implements the wire protocol for Tibia version 1.03.
//!
//! # Architecture
//!
//! - [`constants`] - Protocol constants, opcodes, magic bytes
//! - [`frame`] - Packet framing (length-prefixed)
//! - [`primitives`] - Low-level data type readers
//! - [`login`] - Login packet handling
//! - [`server_packets`] - Server packet builders
//! - [`codec`] - Main codec implementation
//!
//! # Usage
//!
//! The primary interface is the [`Codec`] struct, which implements
//! [`ClientCodec`] and [`ServerCodec`] for use with the server.
//!
//! ```ignore
//! use rustwurm::protocol::tibia103::Codec;
//! use rustwurm::protocol::Protocol;
//!
//! let codec = Codec::new();
//! println!("Protocol version: {}", Codec::version());
//! ```
//!
//! # Protocol Overview
//!
//! ## Packet Framing
//!
//! All packets are length-prefixed:
//! ```text
//! [u16_le Length][Body...]
//! ```
//! where `Length` is the number of bytes in `Body`.
//!
//! ## Login Packet
//!
//! The game login packet has a special fixed format (65 bytes body):
//! ```text
//! Magic    : [00 00 01 01 00]
//! Protocol : u16_le = 0x0067
//! Name     : 30 bytes (null-padded ASCII)
//! Password : 30 bytes (null-padded ASCII)
//! ```
//!
//! ## Game Packets
//!
//! Standard game packets use opcode-based format:
//! ```text
//! [u16_le Length][u8 Opcode][Payload...]
//! ```
//! Note: Opcodes are single bytes in Tibia 1.03.
//!
//! ## Login Response Sequence
//!
//! On successful login, the server sends:
//! 1. LoginOk (0x0A) with player's creature ID
//! 2. EquippedItem packets (0x78) for each slot
//! 3. MapDescription (0x64) with surrounding tiles
//! 4. TextMessage (0xB4) welcome text

pub mod constants;
pub mod frame;
pub mod primitives;
pub mod login;
pub mod server_packets;
pub mod codec;

// Re-exports for convenient access
pub use codec::Codec;
pub use constants::{ClientOpcode, ServerOpcode, EquipmentSlot, MessageType, SpeakType, Direction};
pub use frame::{Frame, FrameBuilder};
pub use login::LoginCredentials;
pub use server_packets::{Position, TileData, CreatureInfo, OutfitInfo};