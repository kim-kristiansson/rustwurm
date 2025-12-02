//! Tibia 3.0 protocol implementation (placeholder)
//!
//! This module will implement the Tibia 3.0 wire protocol when needed.
//!
//! # Differences from 1.03
//!
//! The 3.0 protocol includes:
//! - XTEA encryption
//! - Different packet structure
//! - Account-based login flow
//! - Extended creature/item data

use std::io::{Read, Write};
use crate::engine::{ClientMessage, ServerMessage};
use crate::error::ProtocolResult;
use crate::protocol::traits::{ClientCodec, ServerCodec, Protocol};

/// Tibia 3.0 protocol codec (not yet implemented)
pub struct Codec;

impl Default for Codec {
    fn default() -> Self {
        Self
    }
}

impl ClientCodec for Codec {
    fn read_message(&mut self, _reader: &mut dyn Read) -> ProtocolResult<Option<ClientMessage>> {
        unimplemented!("Tibia 3.0 protocol not yet implemented")
    }
}

impl ServerCodec for Codec {
    fn write_message(&mut self, _writer: &mut dyn Write, _msg: &ServerMessage) -> ProtocolResult<()> {
        unimplemented!("Tibia 3.0 protocol not yet implemented")
    }
}

impl Protocol for Codec {
    fn version() -> &'static str {
        "3.0"
    }
}