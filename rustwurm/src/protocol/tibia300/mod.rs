//! Tibia 3.0 protocol implementation (placeholder)
//!
//! TODO: Implement when needed

use std::io::{Read, Write};
use crate::engine::{ClientMessage, ServerMessage};
use crate::error::ProtocolResult;
use crate::protocol::traits::{ClientCodec, ServerCodec, Protocol};

pub struct Codec;

impl Default for Codec {
    fn default() -> Self {
        Self
    }
}

impl ClientCodec for Codec {
    fn read_message(&mut self, _reader: &mut dyn Read) -> ProtocolResult<Option<ClientMessage>> {
        todo!("v3.0 protocol not yet implemented")
    }
}

impl ServerCodec for Codec {
    fn write_message(&mut self, _writer: &mut dyn Write, _msg: &ServerMessage) -> ProtocolResult<()> {
        todo!("v3.0 protocol not yet implemented")
    }
}

impl Protocol for Codec {
    fn version() -> &'static str {
        "3.0"
    }
}