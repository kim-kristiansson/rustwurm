//! Primitive data type readers for parsing packet payloads
//!
//! Provides a cursor-like interface for reading typed data from byte slices.

use crate::error::{ProtocolError, ProtocolResult};

/// A reader for extracting primitive types from a byte slice
#[derive(Debug)]
pub struct PayloadReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> PayloadReader<'a> {
    /// Create a new reader over the given data
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Current position in the data
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Total length of the underlying data
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if we've consumed all data
    pub fn is_empty(&self) -> bool {
        self.pos >= self.data.len()
    }

    /// Number of bytes remaining to read
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    /// Peek at the next byte without consuming it
    pub fn peek_u8(&self) -> Option<u8> {
        if self.remaining() >= 1 {
            Some(self.data[self.pos])
        } else {
            None
        }
    }

    /// Skip n bytes
    pub fn skip(&mut self, n: usize) -> ProtocolResult<()> {
        if self.remaining() < n {
            return Err(ProtocolError::PacketTooShort {
                expected: self.pos + n,
                actual: self.data.len(),
            });
        }
        self.pos += n;
        Ok(())
    }

    /// Read a single byte
    pub fn read_u8(&mut self) -> ProtocolResult<u8> {
        if self.remaining() < 1 {
            return Err(ProtocolError::PacketTooShort {
                expected: self.pos + 1,
                actual: self.data.len(),
            });
        }
        let value = self.data[self.pos];
        self.pos += 1;
        Ok(value)
    }

    /// Read a u16 (little-endian)
    pub fn read_u16(&mut self) -> ProtocolResult<u16> {
        if self.remaining() < 2 {
            return Err(ProtocolError::PacketTooShort {
                expected: self.pos + 2,
                actual: self.data.len(),
            });
        }
        let value = u16::from_le_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Ok(value)
    }

    /// Read a u32 (little-endian)
    pub fn read_u32(&mut self) -> ProtocolResult<u32> {
        if self.remaining() < 4 {
            return Err(ProtocolError::PacketTooShort {
                expected: self.pos + 4,
                actual: self.data.len(),
            });
        }
        let value = u32::from_le_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(value)
    }

    /// Read an i8
    pub fn read_i8(&mut self) -> ProtocolResult<i8> {
        Ok(self.read_u8()? as i8)
    }

    /// Read an i16 (little-endian)
    pub fn read_i16(&mut self) -> ProtocolResult<i16> {
        Ok(self.read_u16()? as i16)
    }

    /// Read an i32 (little-endian)
    pub fn read_i32(&mut self) -> ProtocolResult<i32> {
        Ok(self.read_u32()? as i32)
    }

    /// Read a fixed number of bytes
    pub fn read_bytes(&mut self, n: usize) -> ProtocolResult<&'a [u8]> {
        if self.remaining() < n {
            return Err(ProtocolError::PacketTooShort {
                expected: self.pos + n,
                actual: self.data.len(),
            });
        }
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    /// Read all remaining bytes
    pub fn read_remaining(&mut self) -> &'a [u8] {
        let slice = &self.data[self.pos..];
        self.pos = self.data.len();
        slice
    }

    /// Read a null-terminated string (C-string)
    pub fn read_cstring(&mut self) -> ProtocolResult<String> {
        let start = self.pos;

        // Find the null terminator
        while self.pos < self.data.len() {
            if self.data[self.pos] == 0 {
                let bytes = &self.data[start..self.pos];
                self.pos += 1; // Skip the null terminator
                return String::from_utf8(bytes.to_vec())
                    .map_err(|_| ProtocolError::InvalidPacket("Invalid UTF-8 in string".to_string()));
            }
            self.pos += 1;
        }

        Err(ProtocolError::InvalidPacket("Unterminated string".to_string()))
    }

    /// Read a length-prefixed string (u16 length + bytes)
    pub fn read_string(&mut self) -> ProtocolResult<String> {
        let len = self.read_u16()? as usize;
        let bytes = self.read_bytes(len)?;

        String::from_utf8(bytes.to_vec())
            .map_err(|_| ProtocolError::InvalidPacket("Invalid UTF-8 in string".to_string()))
    }

    /// Read a fixed-width string (null-padded, null-terminated)
    pub fn read_fixed_string(&mut self, width: usize) -> ProtocolResult<String> {
        let bytes = self.read_bytes(width)?;

        // Find null terminator or use full width
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(width);
        let slice = &bytes[..end];

        String::from_utf8(slice.to_vec())
            .map_err(|_| ProtocolError::InvalidPacket("Invalid UTF-8 in fixed string".to_string()))
    }

    /// Read a position (x: u16, y: u16, z: u8)
    pub fn read_position(&mut self) -> ProtocolResult<(u16, u16, u8)> {
        let x = self.read_u16()?;
        let y = self.read_u16()?;
        let z = self.read_u8()?;
        Ok((x, y, z))
    }

    /// Check if the next bytes match the given pattern
    pub fn matches(&self, pattern: &[u8]) -> bool {
        if self.remaining() < pattern.len() {
            return false;
        }
        &self.data[self.pos..self.pos + pattern.len()] == pattern
    }

    /// Consume bytes if they match the pattern, otherwise return false
    pub fn consume_if(&mut self, pattern: &[u8]) -> bool {
        if self.matches(pattern) {
            self.pos += pattern.len();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_primitives() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let mut reader = PayloadReader::new(&data);

        assert_eq!(reader.read_u8().unwrap(), 0x01);
        assert_eq!(reader.read_u16().unwrap(), 0x0302);
        assert_eq!(reader.remaining(), 5);
    }

    #[test]
    fn test_read_cstring() {
        let data = b"hello\x00world";
        let mut reader = PayloadReader::new(data);

        assert_eq!(reader.read_cstring().unwrap(), "hello");
        assert_eq!(reader.remaining(), 5);
    }

    #[test]
    fn test_read_fixed_string() {
        let data = [b'h', b'i', 0, 0, 0, 0, 0, 0, 0, 0];
        let mut reader = PayloadReader::new(&data);

        assert_eq!(reader.read_fixed_string(10).unwrap(), "hi");
        assert!(reader.is_empty());
    }

    #[test]
    fn test_read_position() {
        let data = [0x64, 0x00, 0xC8, 0x00, 0x07]; // x=100, y=200, z=7
        let mut reader = PayloadReader::new(&data);

        let (x, y, z) = reader.read_position().unwrap();
        assert_eq!((x, y, z), (100, 200, 7));
    }
}