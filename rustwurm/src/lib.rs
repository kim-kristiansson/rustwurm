//! Rustwurm - A Tibia 1.03 server implementation

pub mod engine;
pub mod error;
pub mod protocol;
pub mod world;

// Re-export error types
pub use error::{GameError, GameResult, ProtocolError, ProtocolResult, ServerError, ServerResult};

// Re-export engine types (including Game)
pub use engine::{
    ClientMessage, ServerMessage, Direction, Player, PlayerId, TileInfo,
    Game, GameCommand, GameEvent,
};

// Re-export world types
pub use world::{Map, Position, Tile, TileType};

// Re-export protocol types
pub use protocol::{ClientCodec, ServerCodec, Protocol};

#[cfg(feature = "protocol-tibia103")]
pub use protocol::tibia103::Codec as SelectedProtocol;

#[cfg(all(feature = "protocol-tibia300", not(feature = "protocol-tibia103")))]
pub use protocol::tibia300::Codec as SelectedProtocol;