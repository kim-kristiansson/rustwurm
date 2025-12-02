//! Protocol traits for version-agnostic wire format handling
//!
//! Each Tibia protocol version implements these traits to translate
//! between wire formats and engine messages.

use std::io::{Read, Write};
use crate::engine::{ClientMessage, ServerMessage};
use crate::error::ProtocolResult;

/// Trait for reading client messages from the wire
pub trait ClientCodec {
    /// Read a single client message from the stream
    /// Returns None if the packet was handled internally (e.g., keep-alive)
    fn read_message(&mut self, reader: &mut dyn Read) -> ProtocolResult<Option<ClientMessage>>;
}

/// Trait for writing server messages to the wire
pub trait ServerCodec {
    /// Write a server message to the stream
    fn write_message(&mut self, writer: &mut dyn Write, msg: &ServerMessage) -> ProtocolResult<()>;
}

/// Combined protocol codec
pub trait Protocol: ClientCodec + ServerCodec + Default + Send {
    /// Protocol version identifier (e.g., "1.03")
    fn version() -> &'static str;
}