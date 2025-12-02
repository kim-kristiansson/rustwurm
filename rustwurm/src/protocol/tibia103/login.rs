//! Login packet parsing for Tibia 1.03
//!
//! The game login packet has a unique fixed format that differs from
//! the standard opcode-based packets.
//!
//! # Format
//! ```text
//! GameLogin103Packet (67 bytes total):
//!     Length   : u16_le = 67
//!     Body (65 bytes):
//!         Magic    : 5 bytes  = [00 00 01 01 00]
//!         Protocol : u16_le   = 0x0067 (103)
//!         Name     : 30 bytes ASCII, null-terminated, null-padded
//!         Password : 30 bytes ASCII, null-terminated, null-padded
//! ```

use crate::error::{ProtocolError, ProtocolResult};
use super::constants::{
    LOGIN_MAGIC, PROTOCOL_VERSION,
    LOGIN_BODY_LENGTH, LOGIN_NAME_LENGTH, LOGIN_PASSWORD_LENGTH,
};
use super::primitives::PayloadReader;
use super::frame::{Frame, FrameBuilder};

/// Parsed game login credentials
#[derive(Debug, Clone)]
pub struct LoginCredentials {
    pub name: String,
    pub password: String,
    pub protocol_version: u16,
}

impl LoginCredentials {
    /// Check if this is a valid Tibia 1.03 login
    pub fn is_valid_version(&self) -> bool {
        self.protocol_version == PROTOCOL_VERSION
    }
}

/// Determine if a frame is a login packet by checking its structure
pub fn is_login_packet(frame: &Frame) -> bool {
    // Login packet body is exactly 65 bytes
    if frame.body.len() != LOGIN_BODY_LENGTH {
        return false;
    }

    // Check magic bytes
    frame.body.starts_with(&LOGIN_MAGIC)
}

/// Parse login credentials from a frame
///
/// The frame should have already been validated with `is_login_packet`
pub fn parse_login(frame: &Frame) -> ProtocolResult<LoginCredentials> {
    if frame.body.len() != LOGIN_BODY_LENGTH {
        return Err(ProtocolError::InvalidPacket(format!(
            "Login packet wrong size: expected {}, got {}",
            LOGIN_BODY_LENGTH,
            frame.body.len()
        )));
    }

    let mut reader = PayloadReader::new(&frame.body);

    // Verify magic bytes
    let magic = reader.read_bytes(5)?;
    if magic != LOGIN_MAGIC {
        return Err(ProtocolError::InvalidPacket(format!(
            "Invalid login magic: expected {:02X?}, got {:02X?}",
            LOGIN_MAGIC, magic
        )));
    }

    // Read protocol version
    let protocol_version = reader.read_u16()?;

    // Read name (30 bytes, null-padded)
    let name = reader.read_fixed_string(LOGIN_NAME_LENGTH)?;

    // Read password (30 bytes, null-padded)
    let password = reader.read_fixed_string(LOGIN_PASSWORD_LENGTH)?;

    Ok(LoginCredentials {
        name,
        password,
        protocol_version,
    })
}

/// Build a login packet (for client implementation)
pub fn build_login(name: &str, password: &str) -> Frame {
    let mut builder = FrameBuilder::new();
    builder.write_bytes(&LOGIN_MAGIC);
    builder.write_u16(PROTOCOL_VERSION);
    builder.write_fixed_string(name, LOGIN_NAME_LENGTH);
    builder.write_fixed_string(password, LOGIN_PASSWORD_LENGTH);
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_roundtrip() {
        let frame = build_login("TestPlayer", "secret123");

        assert!(is_login_packet(&frame));
        assert_eq!(frame.body.len(), LOGIN_BODY_LENGTH);

        let creds = parse_login(&frame).unwrap();
        assert_eq!(creds.name, "TestPlayer");
        assert_eq!(creds.password, "secret123");
        assert_eq!(creds.protocol_version, PROTOCOL_VERSION);
        assert!(creds.is_valid_version());
    }

    #[test]
    fn test_login_truncation() {
        // Names/passwords longer than 30 chars should be truncated
        let long_name = "a".repeat(50);
        let frame = build_login(&long_name, "pass");

        let creds = parse_login(&frame).unwrap();
        assert_eq!(creds.name.len(), 30);
        assert!(creds.name.chars().all(|c| c == 'a'));
    }

    #[test]
    fn test_non_login_packet() {
        // A normal opcode packet should not be detected as login
        let frame = FrameBuilder::with_opcode(0x0065).build();
        assert!(!is_login_packet(&frame));
    }
}