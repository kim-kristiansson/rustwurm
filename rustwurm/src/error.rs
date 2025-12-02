use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Player {0} not found")]
    PlayerNotFound(String),

    #[error("Invalid position ({x}, {y}")]
    InvalidPosition { x: i32, y: i32 },

    #[error("Map error: {0}")]
    MapError(String),
}

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid packet: {0}")]
    InvalidPacket(String),

    #[error("Unknown opcode: {0:#06x}")]
    UnknownOpcode(u16),

    #[error("Packet too short: expected {expected}, got {actual}")]
    PacketTooShort { expected: usize, actual: usize },
}

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Protocol error: {0:}")]
    Protocol(#[from] ProtocolError),

    #[error("Game error: {0}")]
    Game(#[from] GameError),
}

pub type GameResult<T> = Result<T, GameError>;
pub type ProtocolResult<T> = Result<T, ProtocolError>;
pub type ServerResult<T> = Result<T, ServerError>;