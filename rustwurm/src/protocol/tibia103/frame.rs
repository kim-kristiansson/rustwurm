//! Packet framing for Tibia 1.03 protocol
//!
//! ## Client → Server Packets
//!
//! Client packets use simple framing:
//! ```text
//! [u16_le Length][Opcode (1B)][Payload...]
//! ```
//! where Length = number of bytes after the length field (opcode + payload)
//!
//! ## Server → Client Packets
//!
//! Server packets have a special 4-zero-byte prefix before the opcode:
//! ```text
//! [u16_le Length][0x00 0x00 0x00 0x00][Opcode (1B)][Payload...]
//! ```
//! where Length = 4 (zeros) + 1 (opcode) + payload.len()

use std::io::{Read, Write};
use crate::error::{ProtocolError, ProtocolResult};
use super::constants::SERVER_HEADER_PREFIX_SIZE;

/// Minimum packet body length (at least an opcode byte)
pub const MIN_PACKET_LENGTH: u16 = 1;

/// Maximum packet length (prevent memory exhaustion)
pub const MAX_PACKET_LENGTH: u16 = 16384;

// =============================================================================
// Client Frame (reading from client)
// =============================================================================

/// A framed packet received from the client
///
/// Client packets have no prefix, just `[Length][Opcode][Payload]`
#[derive(Debug, Clone)]
pub struct Frame {
    /// Raw body bytes (everything after the length field)
    pub body: Vec<u8>,
}

impl Frame {
    /// Create a new frame with the given body
    pub fn new(body: Vec<u8>) -> Self {
        Self { body }
    }

    /// Create an empty frame
    pub fn empty() -> Self {
        Self { body: Vec::new() }
    }

    /// Read a framed packet from the stream (client → server format)
    ///
    /// Format: `[u16_le Length][Body...]`
    /// Length is the number of bytes in Body
    pub fn read_from(reader: &mut dyn Read) -> ProtocolResult<Self> {
        // Read length field (2 bytes, little-endian)
        let mut len_buf = [0u8; 2];
        reader.read_exact(&mut len_buf)?;
        let length = u16::from_le_bytes(len_buf);

        // Validate length
        if length < MIN_PACKET_LENGTH {
            return Err(ProtocolError::PacketTooShort {
                expected: MIN_PACKET_LENGTH as usize,
                actual: length as usize,
            });
        }

        if length > MAX_PACKET_LENGTH {
            return Err(ProtocolError::InvalidPacket(format!(
                "Packet too large: {} bytes (max {})",
                length, MAX_PACKET_LENGTH
            )));
        }

        // Read body
        let body_len = length as usize;
        let mut body = vec![0u8; body_len];
        if body_len > 0 {
            reader.read_exact(&mut body)?;
        }

        Ok(Self { body })
    }

    /// Check if this frame has at least one byte (opcode)
    pub fn has_opcode(&self) -> bool {
        !self.body.is_empty()
    }

    /// Extract the opcode from the body (first byte)
    pub fn opcode(&self) -> Option<u8> {
        self.body.first().copied()
    }

    /// Get the payload (body without the opcode byte)
    pub fn payload(&self) -> &[u8] {
        if self.body.len() > 1 {
            &self.body[1..]
        } else {
            &[]
        }
    }

    /// Write this frame to the stream (client → server format, no prefix)
    ///
    /// Format: `[u16_le Length][Body...]`
    pub fn write_to(&self, writer: &mut dyn Write) -> ProtocolResult<()> {
        let length = self.body.len() as u16;

        if length > MAX_PACKET_LENGTH {
            return Err(ProtocolError::InvalidPacket(format!(
                "Packet too large to send: {} bytes (max {})",
                length, MAX_PACKET_LENGTH
            )));
        }

        writer.write_all(&length.to_le_bytes())?;
        writer.write_all(&self.body)?;
        writer.flush()?;

        Ok(())
    }
}

// =============================================================================
// Server Frame (writing to client)
// =============================================================================

/// A framed packet to send to the client
///
/// Server packets include the 4-zero-byte prefix:
/// `[Length][0x00 0x00 0x00 0x00][Opcode][Payload]`
#[derive(Debug, Clone)]
pub struct ServerFrame {
    opcode: u8,
    payload: Vec<u8>,
}

impl ServerFrame {
    /// Create a new server frame with the given opcode
    pub fn new(opcode: u8) -> Self {
        Self {
            opcode,
            payload: Vec::new(),
        }
    }

    /// Create a server frame with opcode and pre-allocated payload capacity
    pub fn with_capacity(opcode: u8, capacity: usize) -> Self {
        Self {
            opcode,
            payload: Vec::with_capacity(capacity),
        }
    }

    /// Get the opcode
    pub fn opcode(&self) -> u8 {
        self.opcode
    }

    /// Get the payload
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Write the server frame to the stream
    ///
    /// Format: `[u16_le Length][0x00 0x00 0x00 0x00][Opcode (1B)][Payload...]`
    /// where Length = 4 + 1 + payload.len()
    pub fn write_to(&self, writer: &mut dyn Write) -> ProtocolResult<()> {
        // Calculate total body length: 4 (zeros) + 1 (opcode) + payload
        let body_len = SERVER_HEADER_PREFIX_SIZE + 1 + self.payload.len();

        if body_len > MAX_PACKET_LENGTH as usize {
            return Err(ProtocolError::InvalidPacket(format!(
                "Server packet too large: {} bytes (max {})",
                body_len, MAX_PACKET_LENGTH
            )));
        }

        // Write length field
        writer.write_all(&(body_len as u16).to_le_bytes())?;

        // Write 4 zero bytes (Tibia 1.03 server header prefix)
        writer.write_all(&[0x00, 0x00, 0x00, 0x00])?;

        // Write opcode
        writer.write_all(&[self.opcode])?;

        // Write payload
        if !self.payload.is_empty() {
            writer.write_all(&self.payload)?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Calculate the wire size of this frame
    pub fn wire_size(&self) -> usize {
        2 + SERVER_HEADER_PREFIX_SIZE + 1 + self.payload.len()
    }
}

// =============================================================================
// Server Frame Builder
// =============================================================================

/// Builder for constructing server frames
#[derive(Debug)]
pub struct ServerFrameBuilder {
    opcode: u8,
    payload: Vec<u8>,
}

impl ServerFrameBuilder {
    /// Start building a server frame with the given opcode
    pub fn new(opcode: u8) -> Self {
        Self {
            opcode,
            payload: Vec::new(),
        }
    }

    /// Append raw bytes to the payload
    pub fn write_bytes(&mut self, data: &[u8]) -> &mut Self {
        self.payload.extend_from_slice(data);
        self
    }

    /// Append a single byte
    pub fn write_u8(&mut self, value: u8) -> &mut Self {
        self.payload.push(value);
        self
    }

    /// Append a u16 (little-endian)
    pub fn write_u16(&mut self, value: u16) -> &mut Self {
        self.payload.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Append a u32 (little-endian)
    pub fn write_u32(&mut self, value: u32) -> &mut Self {
        self.payload.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Append a null-terminated string (C-string)
    pub fn write_cstring(&mut self, s: &str) -> &mut Self {
        self.payload.extend_from_slice(s.as_bytes());
        self.payload.push(0);
        self
    }

    /// Append a length-prefixed string (u16 length + bytes)
    pub fn write_string(&mut self, s: &str) -> &mut Self {
        let bytes = s.as_bytes();
        self.payload.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
        self.payload.extend_from_slice(bytes);
        self
    }

    /// Consume the builder and produce a ServerFrame
    pub fn build(self) -> ServerFrame {
        ServerFrame {
            opcode: self.opcode,
            payload: self.payload,
        }
    }
}

// =============================================================================
// Client Frame Builder (for testing / client implementation)
// =============================================================================

/// Builder for constructing client frames (no 4-zero prefix)
///
/// Uses consuming `self` pattern for clean method chaining:
/// ```ignore
/// FrameBuilder::new()
///     .opcode(0x05)
///     .u8(0x00)
///     .build()
/// ```
#[derive(Debug, Default, Clone)]
pub struct FrameBuilder {
    body: Vec<u8>,
}

impl FrameBuilder {
    pub fn new() -> Self {
        Self { body: Vec::new() }
    }

    /// Start building a frame with the given opcode (single byte)
    pub fn with_opcode(opcode: u8) -> Self {
        Self { body: vec![opcode] }
    }

    /// Add an opcode byte (consuming self for chaining)
    pub fn opcode(mut self, opcode: u8) -> Self {
        self.body.push(opcode);
        self
    }

    /// Append raw bytes to the body (consuming self for chaining)
    pub fn bytes(mut self, data: &[u8]) -> Self {
        self.body.extend_from_slice(data);
        self
    }

    /// Append a single byte (consuming self for chaining)
    pub fn u8(mut self, value: u8) -> Self {
        self.body.push(value);
        self
    }

    /// Append a u16 little-endian (consuming self for chaining)
    pub fn u16(mut self, value: u16) -> Self {
        self.body.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Append a u32 little-endian (consuming self for chaining)
    pub fn u32(mut self, value: u32) -> Self {
        self.body.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Append a null-terminated string (consuming self for chaining)
    pub fn cstring(mut self, s: &str) -> Self {
        self.body.extend_from_slice(s.as_bytes());
        self.body.push(0);
        self
    }

    /// Append a length-prefixed string (consuming self for chaining)
    pub fn string(mut self, s: &str) -> Self {
        let bytes = s.as_bytes();
        self.body.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
        self.body.extend_from_slice(bytes);
        self
    }

    /// Append a fixed-width string, padded with nulls (consuming self for chaining)
    pub fn fixed_string(mut self, s: &str, width: usize) -> Self {
        let bytes = s.as_bytes();
        let copy_len = bytes.len().min(width);

        // Write string bytes (truncated if necessary)
        self.body.extend_from_slice(&bytes[..copy_len]);

        // Pad with nulls
        for _ in copy_len..width {
            self.body.push(0);
        }

        self
    }

    // === Mutable reference versions for use in loops ===

    /// Append raw bytes (mutable reference version)
    pub fn write_bytes(&mut self, data: &[u8]) -> &mut Self {
        self.body.extend_from_slice(data);
        self
    }

    /// Append a single byte (mutable reference version)
    pub fn write_u8(&mut self, value: u8) -> &mut Self {
        self.body.push(value);
        self
    }

    /// Append a u16 (mutable reference version)
    pub fn write_u16(&mut self, value: u16) -> &mut Self {
        self.body.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Append a u32 (mutable reference version)
    pub fn write_u32(&mut self, value: u32) -> &mut Self {
        self.body.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Append a fixed-width string (mutable reference version)
    pub fn write_fixed_string(&mut self, s: &str, width: usize) -> &mut Self {
        let bytes = s.as_bytes();
        let copy_len = bytes.len().min(width);
        self.body.extend_from_slice(&bytes[..copy_len]);
        for _ in copy_len..width {
            self.body.push(0);
        }
        self
    }

    /// Consume the builder and produce a Frame
    pub fn build(self) -> Frame {
        Frame { body: self.body }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_client_frame_round_trip() {
        let frame = FrameBuilder::with_opcode(0x05)
            .u8(0x00) // Direction: North
            .build();

        let mut buffer = Vec::new();
        frame.write_to(&mut buffer).unwrap();

        // Should be: 02 00 05 00 (length=2, opcode=0x05, direction=0x00)
        assert_eq!(buffer, vec![0x02, 0x00, 0x05, 0x00]);

        let mut cursor = Cursor::new(buffer);
        let decoded = Frame::read_from(&mut cursor).unwrap();

        assert_eq!(decoded.opcode(), Some(0x05));
        assert_eq!(decoded.payload(), &[0x00]);
    }

    #[test]
    fn test_server_frame_with_prefix() {
        let frame = ServerFrameBuilder::new(0x01).build(); // Login opcode

        let mut buffer = Vec::new();
        frame.write_to(&mut buffer).unwrap();

        // Should be: 05 00 00 00 00 00 01
        // Length = 5 (4 zeros + 1 opcode)
        assert_eq!(buffer, vec![0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]);
    }

    #[test]
    fn test_server_frame_with_payload() {
        let mut builder = ServerFrameBuilder::new(0x04); // Info opcode
        builder.write_cstring("Hello");
        let frame = builder.build();

        let mut buffer = Vec::new();
        frame.write_to(&mut buffer).unwrap();

        // Length = 4 (zeros) + 1 (opcode) + 6 (Hello + null) = 11
        assert_eq!(buffer[0], 11);
        assert_eq!(buffer[1], 0);
        // 4 zero bytes
        assert_eq!(&buffer[2..6], &[0x00, 0x00, 0x00, 0x00]);
        // Opcode
        assert_eq!(buffer[6], 0x04);
        // "Hello" + null
        assert_eq!(&buffer[7..], b"Hello\0");
    }

    #[test]
    fn test_equipped_item_packet_format() {
        // Per documentation: [4 zeros][Opcode 0x14][Item ID u16][Slot u8]
        let mut builder = ServerFrameBuilder::new(0x14);
        builder.write_u16(0x013D); // Backpack item ID
        builder.write_u8(0x03);    // Backpack slot

        let frame = builder.build();
        let mut buffer = Vec::new();
        frame.write_to(&mut buffer).unwrap();

        // Length = 4 + 1 + 3 = 8
        assert_eq!(buffer[0], 8);
        assert_eq!(buffer[1], 0);
        // 4 zeros
        assert_eq!(&buffer[2..6], &[0x00, 0x00, 0x00, 0x00]);
        // Opcode 0x14
        assert_eq!(buffer[6], 0x14);
        // Item ID 0x013D (little-endian)
        assert_eq!(buffer[7], 0x3D);
        assert_eq!(buffer[8], 0x01);
        // Slot 0x03
        assert_eq!(buffer[9], 0x03);
    }

    #[test]
    fn test_fixed_string() {
        let frame = FrameBuilder::new()
            .fixed_string("test", 10)
            .build();

        assert_eq!(frame.body.len(), 10);
        assert_eq!(&frame.body[..4], b"test");
        assert!(frame.body[4..].iter().all(|&b| b == 0));
    }
}