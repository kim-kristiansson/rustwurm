//! Login packet parsing for Tibia 1.03
//!
//! The game login packet has a unique fixed format that differs from
//! the standard opcode-based packets.
//!
//! # Format (Section 4.2)
//!
//! ```text
//! GameLogin103Packet (67 bytes body):
//!     Offset  Size  Field
//!     ------  ----  -----
//!     0       5     Magic bytes: [00 00 01 01 00]
//!     5       2     Protocol version: u16_le = 0x0067 (103)
//!     7       30    Character name (fixed-length, null-padded)
//!     37      30    Password (fixed-length, null-padded)
//! ```
//!
//! Wire format: `[u16_le Length = 67][Body...]`

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
///
/// Login packets are identified by:
/// 1. Body length of exactly 67 bytes
/// 2. Starting with the magic bytes [00 00 01 01 00]
pub fn is_login_packet(frame: &Frame) -> bool {
    // Login packet body is exactly 67 bytes
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

/// Build a login packet (for client implementation or testing)
///
/// Creates a login packet with the standard Tibia 1.03 format.
pub fn build_login(name: &str, password: &str) -> Frame {
    FrameBuilder::new()
        .bytes(&LOGIN_MAGIC)
        .u16(PROTOCOL_VERSION)
        .fixed_string(name, LOGIN_NAME_LENGTH)
        .fixed_string(password, LOGIN_PASSWORD_LENGTH)
        .build()
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
    fn test_login_body_structure() {
        let frame = build_login("Test", "Pass");

        // Verify the structure
        assert_eq!(frame.body.len(), 67);

        // Magic bytes at offset 0-4
        assert_eq!(&frame.body[0..5], &LOGIN_MAGIC);

        // Protocol version at offset 5-6 (little-endian 0x0067 = 103)
        assert_eq!(frame.body[5], 0x67);
        assert_eq!(frame.body[6], 0x00);

        // Name starts at offset 7
        assert_eq!(&frame.body[7..11], b"Test");
        // Padding should be zeros
        assert!(frame.body[11..37].iter().all(|&b| b == 0));

        // Password starts at offset 37
        assert_eq!(&frame.body[37..41], b"Pass");
        // Padding should be zeros
        assert!(frame.body[41..67].iter().all(|&b| b == 0));
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
        let frame = FrameBuilder::with_opcode(0x05).build();
        assert!(!is_login_packet(&frame));
    }

    #[test]
    fn test_wrong_magic() {
        // Create a 67-byte packet with wrong magic
        let frame = FrameBuilder::new()
            .bytes(&[0x00, 0x00, 0x00, 0x00, 0x00]) // Wrong magic
            .u16(PROTOCOL_VERSION)
            .fixed_string("Test", LOGIN_NAME_LENGTH)
            .fixed_string("Pass", LOGIN_PASSWORD_LENGTH)
            .build();

        assert!(!is_login_packet(&frame)); // Should fail magic check
    }

    #[test]
    fn test_wrong_size() {
        // Create a packet with wrong size
        let frame = FrameBuilder::new()
            .bytes(&LOGIN_MAGIC)
            .u16(PROTOCOL_VERSION)
            .bytes(b"short") // Too short
            .build();

        assert!(!is_login_packet(&frame)); // Should fail size check
    }

    #[test]
    fn test_login_wire_format() {
        let frame = build_login("Test", "Pass");

        let mut buffer = Vec::new();
        frame.write_to(&mut buffer).unwrap();

        // Length field should equal body length (67 bytes)
        let length = u16::from_le_bytes([buffer[0], buffer[1]]);
        assert_eq!(length, 67);

        // Total wire size = 2 (length) + 67 (body) = 69 bytes
        assert_eq!(buffer.len(), 69);
    }

    #[test]
    fn test_special_characters_in_name() {
        // Test with various ASCII characters
        let frame = build_login("Player_123", "p@$$w0rd!");
        let creds = parse_login(&frame).unwrap();

        assert_eq!(creds.name, "Player_123");
        assert_eq!(creds.password, "p@$$w0rd!");
    }

    #[test]
    fn test_empty_credentials() {
        let frame = build_login("", "");
        let creds = parse_login(&frame).unwrap();

        assert_eq!(creds.name, "");
        assert_eq!(creds.password, "");
    }
}