//! Rustwurm - A Tibia-inspired game server
//!
//! Supports multiple protocol versions via compile-time feature selection.
//!
//! # Features
//!
//! - `protocol-tibia103` (default) - Tibia 1.03 protocol
//! - `protocol-tibia300` - Tibia 3.0 protocol (placeholder)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────┐     ┌──────────────────┐
//! │  Client (TCP)   │────▶│  Protocol Codec  │
//! └─────────────────┘     └────────┬─────────┘
//!                                  │
//!                                  ▼
//!                         ┌──────────────────┐
//!                         │  Engine Messages │
//!                         └────────┬─────────┘
//!                                  │
//!                                  ▼
//!                         ┌──────────────────┐
//!                         │   Game Engine    │
//!                         └────────┬─────────┘
//!                                  │
//!                                  ▼
//!                         ┌──────────────────┐
//!                         │   World State    │
//!                         └──────────────────┘
//! ```
//!
//! # Example
//!
//! ```ignore
//! use rustwurm::{Server, SelectedProtocol};
//!
//! let mut server = Server::<SelectedProtocol>::bind_default()?;
//! server.run()?;
//! ```

pub mod engine;
pub mod error;
pub mod net;
pub mod protocol;
pub mod world;

// Re-exports for convenience
pub use engine::{Game, GameCommand, GameEvent, PlayerId};
pub use error::{GameError, ProtocolError, ServerError};
pub use net::Server;
pub use protocol::{Protocol, SelectedProtocol};