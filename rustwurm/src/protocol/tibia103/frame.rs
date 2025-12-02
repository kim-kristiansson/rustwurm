//! Packet framing for Tibia 1.03 protocol
//!
//! All packets are framed as: `[u16_le Length][Body...]`
//! where Length = Body.len() + 2 (includes the length field itself)

use std::io::{self, Read, Write};
use crate::error::{ProtocolError, ProtocolResult};

/// Minimum packet length (just the length field + empty body)
pub const MIN_PACKET_LENGTH: u16 = 2;

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
    /// Length includes itself, so body_len = Length - 2
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

        // Body length is total length minus the 2-byte length field
        let body_len = (length - 2) as usize;

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
        // Calculate total length (body + 2 for length field)
        let length = (self.body.len() + 2) as u16;

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

    /// Check if this frame has an opcode prefix
    pub fn has_opcode(&self) -> bool {
        self.body.len() >= 2
    }

    /// Extract the opcode from the body (first 2 bytes)
    pub fn opcode(&self) -> Option<u16> {
        if self.body.len() >= 2 {
            Some(u16::from_le_bytes([self.body[0], self.body[1]]))
        } else {
            None
        }
    }

    /// Get the payload (body without opcode)
    pub fn payload(&self) -> &[u8] {
        if self.body.len() > 2 {
            &self.body[2..]
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

    /// Start building a frame with the given opcode
    pub fn with_opcode(opcode: u16) -> Self {
        let mut builder = Self::new();
        builder.body.extend_from_slice(&opcode.to_le_bytes());
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
        let mut builder = FrameBuilder::with_opcode(0x000A);
        builder.write_u32(12345);
        builder.write_cstring("hello");
        let original = builder.build();

        let mut buffer = Vec::new();
        original.write_to(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = Frame::read_from(&mut cursor).unwrap();

        assert_eq!(original.body, decoded.body);
        assert_eq!(decoded.opcode(), Some(0x000A));
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
}