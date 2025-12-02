//! Packet framing for Tibia 1.03 protocol
//!
//! All packets are framed as: `[u16_le Length][Body...]`
//! where Length = number of bytes after the length field (i.e., Body.len())

use std::io::{Read, Write};
use crate::error::{ProtocolError, ProtocolResult};

/// Minimum packet body length (at least an opcode byte)
pub const MIN_PACKET_LENGTH: u16 = 1;

/// Maximum packet length (prevent memory exhaustion)
pub const MAX_PACKET_LENGTH: u16 = 16384;

/// A framed packet with raw body bytes
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

    /// Read a framed packet from the stream
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

        // Body length equals the length field value
        let body_len = length as usize;

        // Read body
        let mut body = vec![0u8; body_len];
        if body_len > 0 {
            reader.read_exact(&mut body)?;
        }

        Ok(Self { body })
    }

    /// Write a framed packet to the stream
    ///
    /// Automatically prepends the length field
    pub fn write_to(&self, writer: &mut dyn Write) -> ProtocolResult<()> {
        // Length field = body length
        let length = self.body.len() as u16;

        if length > MAX_PACKET_LENGTH {
            return Err(ProtocolError::InvalidPacket(format!(
                "Packet too large to send: {} bytes (max {})",
                length, MAX_PACKET_LENGTH
            )));
        }

        // Write length
        writer.write_all(&length.to_le_bytes())?;

        // Write body
        writer.write_all(&self.body)?;
        writer.flush()?;

        Ok(())
    }

    /// Get the body as a slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.body
    }

    /// Check if this frame has at least one byte (opcode)
    pub fn has_opcode(&self) -> bool {
        !self.body.is_empty()
    }

    /// Extract the opcode from the body (first byte)
    ///
    /// Tibia 1.03 uses single-byte opcodes
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
}

/// Builder for constructing frames with opcode and payload
#[derive(Debug, Default)]
pub struct FrameBuilder {
    body: Vec<u8>,
}

impl FrameBuilder {
    pub fn new() -> Self {
        Self { body: Vec::new() }
    }

    /// Start building a frame with the given opcode (single byte)
    pub fn with_opcode(opcode: u8) -> Self {
        let mut builder = Self::new();
        builder.body.push(opcode);
        builder
    }

    /// Append raw bytes to the body
    pub fn write_bytes(&mut self, data: &[u8]) -> &mut Self {
        self.body.extend_from_slice(data);
        self
    }

    /// Append a single byte
    pub fn write_u8(&mut self, value: u8) -> &mut Self {
        self.body.push(value);
        self
    }

    /// Append a u16 (little-endian)
    pub fn write_u16(&mut self, value: u16) -> &mut Self {
        self.body.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Append a u32 (little-endian)
    pub fn write_u32(&mut self, value: u32) -> &mut Self {
        self.body.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Append an i32 (little-endian)
    pub fn write_i32(&mut self, value: i32) -> &mut Self {
        self.body.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Append a null-terminated string
    pub fn write_cstring(&mut self, s: &str) -> &mut Self {
        self.body.extend_from_slice(s.as_bytes());
        self.body.push(0);
        self
    }

    /// Append a length-prefixed string (u16 length + bytes)
    pub fn write_string(&mut self, s: &str) -> &mut Self {
        let bytes = s.as_bytes();
        self.body.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
        self.body.extend_from_slice(bytes);
        self
    }

    /// Append a fixed-width string, padded with nulls
    pub fn write_fixed_string(&mut self, s: &str, width: usize) -> &mut Self {
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

    /// Consume the body directly (alternative to build for internal use)
    pub fn into_body(self) -> Vec<u8> {
        self.body
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
    fn test_frame_round_trip() {
        let mut builder = FrameBuilder::with_opcode(0x0A);
        builder.write_u32(12345);
        builder.write_cstring("hello");
        let original = builder.build();

        let mut buffer = Vec::new();
        original.write_to(&mut buffer).unwrap();

        // Verify the length field is correct
        let length = u16::from_le_bytes([buffer[0], buffer[1]]);
        assert_eq!(length as usize, original.body.len());

        let mut cursor = Cursor::new(buffer);
        let decoded = Frame::read_from(&mut cursor).unwrap();

        assert_eq!(original.body, decoded.body);
        assert_eq!(decoded.opcode(), Some(0x0A));
    }

    #[test]
    fn test_fixed_string() {
        let mut builder = FrameBuilder::new();
        builder.write_fixed_string("test", 10);
        let frame = builder.build();

        assert_eq!(frame.body.len(), 10);
        assert_eq!(&frame.body[..4], b"test");
        assert!(frame.body[4..].iter().all(|&b| b == 0));
    }

    #[test]
    fn test_single_byte_opcode_packet() {
        // Movement packets are just a single opcode byte
        let frame = FrameBuilder::with_opcode(0x65).build();

        let mut buffer = Vec::new();
        frame.write_to(&mut buffer).unwrap();

        // Should be: 01 00 65 (length=1, opcode=0x65)
        assert_eq!(buffer, vec![0x01, 0x00, 0x65]);

        let mut cursor = Cursor::new(buffer);
        let decoded = Frame::read_from(&mut cursor).unwrap();
        assert_eq!(decoded.opcode(), Some(0x65));
        assert_eq!(decoded.payload().len(), 0);
    }

    #[test]
    fn test_packet_with_payload() {
        let mut builder = FrameBuilder::with_opcode(0x96); // Say opcode
        builder.write_u8(0x01); // speak type
        builder.write_string("Hello");
        let frame = builder.build();

        let mut buffer = Vec::new();
        frame.write_to(&mut buffer).unwrap();

        // Length should equal body length
        let length = u16::from_le_bytes([buffer[0], buffer[1]]);
        assert_eq!(length as usize, frame.body.len());

        let mut cursor = Cursor::new(buffer);
        let decoded = Frame::read_from(&mut cursor).unwrap();
        assert_eq!(decoded.opcode(), Some(0x96));
        assert_eq!(decoded.payload().len(), frame.body.len() - 1);
    }
}