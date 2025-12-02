//! Protocol implementations
//!
//! This module provides protocol implementations for different Tibia versions.
//! The protocol version is selected at compile time via Cargo features.
//!
//! # Available Protocols
//!
//! - `protocol-tibia103` (default) - Tibia 1.03 protocol
//! - `protocol-tibia300` - Tibia 3.0 protocol (placeholder)
//!
//! # Usage
//!
//! The selected protocol is available as [`SelectedProtocol`]:
//!
//! ```ignore
//! use rustwurm::protocol::{SelectedProtocol, Protocol};
//!
//! println!("Using protocol version: {}", SelectedProtocol::version());
//! ```
//!
//! For direct access to a specific protocol version:
//!
//! ```ignore
//! use rustwurm::protocol::tibia103;
//!
//! let codec = tibia103::Codec::new();
//! ```

mod traits;
pub mod tibia103;
pub mod tibia300;

pub use traits::{ClientCodec, ServerCodec, Protocol};

// Re-export the selected protocol as the default
#[cfg(feature = "protocol-tibia103")]
pub type SelectedProtocol = tibia103::Codec;

#[cfg(all(feature = "protocol-tibia300", not(feature = "protocol-tibia103")))]
pub type SelectedProtocol = tibia300::Codec;

// Fallback if no protocol feature selected (shouldn't happen with default)
#[cfg(not(any(feature = "protocol-tibia103", feature = "protocol-tibia300")))]
compile_error!("At least one protocol feature must be enabled");