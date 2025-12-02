//! Raw packet format for Tibia 1.03 protocol
//!
//! Packet structure:
//! [u16_le length][u16_le opcode][payload...]
//!
//! Length includes opcode + payload (so minimum length is 2)

use std::io::{self, Read, Write};
use crate::error::{ProtocolError, ProtocolResult};

/// Known client opcodes for v1.03
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ClientOpcode {
    Login = 0x01,
    Logout = 0x02,
    Move = 0x03,
    Attack = 0x04,
    // Add more as discovered
}

impl ClientOpcode {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x01 => Some(Self::Login),
            0x02 => Some(Self::Logout),
            0x03 => Some(Self::Move),
            0x04 => Some(Self::Attack),
            _ => None,
        }
    }
}

/// Known server opcodes for v1.03
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ServerOpcode {
    LoginOk = 0x01,
    LoginFailed = 0x02,
    PlayerMoved = 0x03,
    PlayerStats = 0x04,
    TextMessage = 0x10,
}

/// Raw packet (before interpretation)
#[derive(Debug)]
pub struct RawPacket {
    pub opcode: u16,
    pub payload: Vec<u8>,
}

impl RawPacket {
    pub fn new(opcode: u16, payload: Vec<u8>) -> Self {
        Self { opcode, payload }
    }

    pub fn read_from(reader: &mut dyn Read) -> ProtocolResult<Self> {
        // Read length (u16 LE)
        let mut len_buf = [0u8; 2];
        reader.read_exact(&mut len_buf)?;
        let length = u16::from_le_bytes(len_buf) as usize;

        if length < 2 {
            return Err(ProtocolError::PacketTooShort {
                expected: 2,
                actual: length
            });
        }

        // Read body (opcode + payload)
        let body_len = length;
        let mut body = vec![0u8; body_len];
        reader.read_exact(&mut body)?;

        let opcode = u16::from_le_bytes([body[0], body[1]]);
        let payload = body[2..].to_vec();

        Ok(Self { opcode, payload })
    }

    pub fn write_to(&self, writer: &mut dyn Write) -> ProtocolResult<()> {
        let body_len = 2 + self.payload.len();

        // Write length
        let len_bytes = (body_len as u16).to_le_bytes();
        writer.write_all(&len_bytes)?;

        // Write opcode
        let opcode_bytes = self.opcode.to_le_bytes();
        writer.write_all(&opcode_bytes)?;

        // Write payload
        writer.write_all(&self.payload)?;
        writer.flush()?;

        Ok(())
    }
}

/// Helper for reading primitives from payload
pub struct PayloadReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> PayloadReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn read_u8(&mut self) -> ProtocolResult<u8> {
        if self.remaining() < 1 {
            return Err(ProtocolError::PacketTooShort {
                expected: self.pos + 1,
                actual: self.data.len()
            });
        }
        let value = self.data[self.pos];
        self.pos += 1;
        Ok(value)
    }

    pub fn read_u16(&mut self) -> ProtocolResult<u16> {
        if self.remaining() < 2 {
            return Err(ProtocolError::PacketTooShort {
                expected: self.pos + 2,
                actual: self.data.len()
            });
        }
        let value = u16::from_le_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Ok(value)
    }

    pub fn read_u32(&mut self) -> ProtocolResult<u32> {
        if self.remaining() < 4 {
            return Err(ProtocolError::PacketTooShort {
                expected: self.pos + 4,
                actual: self.data.len()
            });
        }
        let bytes = [
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ];
        self.pos += 4;
        Ok(u32::from_le_bytes(bytes))
    }

    pub fn read_i32(&mut self) -> ProtocolResult<i32> {
        Ok(self.read_u32()? as i32)
    }

    pub fn read_string(&mut self) -> ProtocolResult<String> {
        let len = self.read_u16()? as usize;
        if self.remaining() < len {
            return Err(ProtocolError::PacketTooShort {
                expected: self.pos + len,
                actual: self.data.len()
            });
        }
        let bytes = &self.data[self.pos..self.pos + len];
        self.pos += len;

        String::from_utf8(bytes.to_vec())
            .map_err(|_| ProtocolError::InvalidPacket("Invalid UTF-8 string".to_string()))
    }
}

/// Helper for writing primitives to payload
pub struct PayloadWriter {
    data: Vec<u8>,
}

impl PayloadWriter {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn write_u8(&mut self, value: u8) {
        self.data.push(value);
    }

    pub fn write_u16(&mut self, value: u16) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_u32(&mut self, value: u32) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_i32(&mut self, value: i32) {
        self.write_u32(value as u32);
    }

    pub fn write_string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.write_u16(bytes.len() as u16);
        self.data.extend_from_slice(bytes);
    }

    pub fn finish(self) -> Vec<u8> {
        self.data
    }
}

impl Default for PayloadWriter {
    fn default() -> Self {
        Self::new()
    }
}