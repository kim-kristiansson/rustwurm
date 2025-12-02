//! Messages exchanged between client and server
//!
//! These are protocol-agnostic representations of game messages.
//! Protocol implementations translate between wire format and these types.

use super::player::PlayerId;

/// Messages from client to server
#[derive(Debug, Clone)]
pub enum ClientMessage {
    /// Login request with credentials
    Login {
        name: String,
        password: String,
    },

    /// Player wants to disconnect
    Logout {
        player_id: PlayerId,
    },

    /// Movement in a direction (dx, dy)
    Move {
        player_id: PlayerId,
        dx: i32,
        dy: i32,
    },

    /// Attack (target determined by adjacent creatures)
    Attack {
        player_id: PlayerId,
    },

    /// Attack specific creature by ID
    AttackTarget {
        player_id: PlayerId,
        target_id: u32,
    },

    /// Say something in chat
    Say {
        player_id: PlayerId,
        message: String,
    },

    /// Turn to face a direction (without moving)
    Turn {
        player_id: PlayerId,
        direction: Direction,
    },

    /// Use an item
    UseItem {
        player_id: PlayerId,
        item_id: u16,
    },

    /// Request to stop current action
    Cancel {
        player_id: PlayerId,
    },
}

/// Cardinal directions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn to_delta(self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::East => (1, 0),
            Direction::South => (0, 1),
            Direction::West => (-1, 0),
        }
    }

    pub fn from_delta(dx: i32, dy: i32) -> Option<Self> {
        match (dx.signum(), dy.signum()) {
            (0, -1) => Some(Direction::North),
            (1, 0) => Some(Direction::East),
            (0, 1) => Some(Direction::South),
            (-1, 0) => Some(Direction::West),
            _ => None,
        }
    }
}

/// Messages from server to client
#[derive(Debug, Clone)]
pub enum ServerMessage {
    /// Login successful
    LoginOk {
        player_id: PlayerId,
    },

    /// Login failed with reason
    LoginFailed {
        reason: String,
    },

    /// Player position update
    PlayerMoved {
        player_id: PlayerId,
        x: i32,
        y: i32,
    },

    /// Full player stats
    PlayerStats {
        player_id: PlayerId,
        hp: i32,
        max_hp: i32,
        level: i32,
        xp: i32,
        mana: i32,
        max_mana: i32,
    },

    /// Creature position update
    CreatureMoved {
        creature_id: u32,
        x: i32,
        y: i32,
    },

    /// Creature health changed
    CreatureHealth {
        creature_id: u32,
        health_percent: u8,
    },

    /// Creature appeared on screen
    CreatureAppear {
        creature_id: u32,
        name: String,
        x: i32,
        y: i32,
        health_percent: u8,
    },

    /// Creature left the screen
    CreatureDisappear {
        creature_id: u32,
    },

    /// Text message to display
    TextMessage {
        message: String,
    },

    /// Chat message from a player/creature
    CreatureSay {
        creature_id: u32,
        name: String,
        message: String,
    },

    /// Player died
    PlayerDied {
        player_id: PlayerId,
    },

    /// Map data (for initial load or teleport)
    MapDescription {
        center_x: i32,
        center_y: i32,
        tiles: Vec<TileInfo>,
    },
}

/// Simplified tile information for map messages
#[derive(Debug, Clone)]
pub struct TileInfo {
    pub x: i32,
    pub y: i32,
    pub ground_id: u16,
    pub items: Vec<u16>,
    pub creature_id: Option<u32>,
}

impl TileInfo {
    pub fn empty(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            ground_id: 0,
            items: Vec::new(),
            creature_id: None,
        }
    }

    pub fn with_ground(x: i32, y: i32, ground_id: u16) -> Self {
        Self {
            x,
            y,
            ground_id,
            items: Vec::new(),
            creature_id: None,
        }
    }
}