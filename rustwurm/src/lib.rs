//! Rustwurm - A Tibia 1.03 server implementation
//!
//! This crate provides a server implementation compatible with the Tibia 1.03 protocol.
//!
//! # Quick Start
//!
//! ```ignore
//! use rustwurm::{Server, Game, SelectedProtocol};
//!
//! let game = Game::new();
//! let server = Server::new(game, "0.0.0.0:7171");
//! server.run();
//! ```

pub mod engine;
pub mod error;
pub mod protocol;
pub mod world;

// Re-export error types
pub use error::{GameError, GameResult, ProtocolError, ProtocolResult, ServerError, ServerResult};

// Re-export engine types
pub use engine::{ClientMessage, ServerMessage, Direction, Player, PlayerId, TileInfo};

// Re-export world types
pub use world::{Map, Position, Tile, TileType};

// Re-export protocol types
pub use protocol::{ClientCodec, ServerCodec, Protocol};

#[cfg(feature = "protocol-tibia103")]
pub use protocol::tibia103::Codec as SelectedProtocol;

#[cfg(all(feature = "protocol-tibia300", not(feature = "protocol-tibia103")))]
pub use protocol::tibia300::Codec as SelectedProtocol;

// Game command/event types for the main game loop
pub use engine::ClientMessage as GameCommand;
pub use engine::ServerMessage as GameEvent;

/// The main game state
///
/// Holds the world state, players, and game logic.
pub struct Game {
    pub map: Map,
    players: std::collections::HashMap<PlayerId, Player>,
    next_player_id: PlayerId,
}

impl Game {
    /// Create a new game instance
    pub fn new() -> Self {
        Self {
            map: Map::default(),
            players: std::collections::HashMap::new(),
            next_player_id: 1,
        }
    }

    /// Create a new game with a custom map
    pub fn with_map(map: Map) -> Self {
        Self {
            map,
            players: std::collections::HashMap::new(),
            next_player_id: 1,
        }
    }

    /// Add a new player to the game
    pub fn add_player(&mut self, name: String, pos: Position) -> PlayerId {
        let id = self.next_player_id;
        self.next_player_id += 1;

        let player = Player::new(id, name, pos);
        self.players.insert(id, player);
        id
    }

    /// Get a player by ID
    pub fn get_player(&self, id: PlayerId) -> Option<&Player> {
        self.players.get(&id)
    }

    /// Get a mutable reference to a player
    pub fn get_player_mut(&mut self, id: PlayerId) -> Option<&mut Player> {
        self.players.get_mut(&id)
    }

    /// Remove a player from the game
    pub fn remove_player(&mut self, id: PlayerId) -> Option<Player> {
        self.players.remove(&id)
    }

    /// Process a game command and return events
    pub fn process(&mut self, cmd: GameCommand) -> Vec<GameEvent> {
        match cmd {
            GameCommand::Login { name, .. } => {
                let pos = Position::ground(50, 50);
                let id = self.add_player(name, pos);
                vec![GameEvent::LoginOk { player_id: id }]
            }
            GameCommand::Move { player_id, dx, dy } => {
                if let Some(player) = self.get_player_mut(player_id) {
                    if player.try_move(dx, dy, &self.map) {
                        vec![GameEvent::PlayerMoved {
                            player_id,
                            x: player.pos.x,
                            y: player.pos.y,
                        }]
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            }
            GameCommand::Say { player_id, message } => {
                if let Some(player) = self.get_player(player_id) {
                    vec![GameEvent::CreatureSay {
                        creature_id: player_id,
                        name: player.name.clone(),
                        message,
                    }]
                } else {
                    vec![]
                }
            }
            GameCommand::Logout { player_id } => {
                self.remove_player(player_id);
                vec![]
            }
            _ => vec![],
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

/// TCP game server
///
/// Handles client connections and routes messages through the protocol codec.
pub struct Server {
    game: Game,
    address: String,
}

impl Server {
    /// Create a new server
    pub fn new(game: Game, address: impl Into<String>) -> Self {
        Self {
            game,
            address: address.into(),
        }
    }

    /// Get the bind address
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Get a reference to the game state
    pub fn game(&self) -> &Game {
        &self.game
    }

    /// Get a mutable reference to the game state
    pub fn game_mut(&mut self) -> &mut Game {
        &mut self.game
    }

    /// Run the server (blocking)
    ///
    /// This is a placeholder - actual networking would use tokio or similar.
    pub fn run(&self) {
        eprintln!("Server would bind to {}", self.address);
        eprintln!("Note: Actual async networking not yet implemented");
    }
}