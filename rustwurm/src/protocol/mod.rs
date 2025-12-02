//! Protocol version implementations
//!
//! Select protocol version at compile time via Cargo features:
//! - `protocol-tibia103` (default)
//! - `protocol-tibia300`

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