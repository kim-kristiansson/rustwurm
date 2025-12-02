//! Protocol constants for Tibia 1.03
//!
//! These values are derived from analyzing the original Tibia 1.03 client/server.
//! Note: Opcodes are single-byte values in this protocol version.

/// Magic bytes at the start of a game login packet
pub const LOGIN_MAGIC: [u8; 5] = [0x00, 0x00, 0x01, 0x01, 0x00];

/// Protocol version identifier (0x0067 = 103 decimal)
pub const PROTOCOL_VERSION: u16 = 0x0067;

/// Fixed sizes for login packet fields
pub const LOGIN_PACKET_LENGTH: u16 = 65;
pub const LOGIN_BODY_LENGTH: usize = 65;
pub const LOGIN_NAME_LENGTH: usize = 30;
pub const LOGIN_PASSWORD_LENGTH: usize = 30;

/// Map viewport dimensions for Tibia 1.03
pub const MAP_WIDTH: usize = 18;
pub const MAP_HEIGHT: usize = 14;
pub const MAP_DEPTH: usize = 1;

/// Tile terminator sequence
pub const TILE_TERMINATOR: [u8; 2] = [0xFF, 0xFF];

/// Map data terminator sequence
pub const MAP_TERMINATOR: [u8; 2] = [0xFE, 0x00];

/// Client → Server opcodes (single byte)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ClientOpcode {
    /// Initial game login (special format, handled separately)
    GameLogin = 0x01,
    /// Player logout request
    Logout = 0x14,
    /// Automove / path walk
    AutoWalk = 0x64,
    /// Movement in a direction
    MoveNorth = 0x65,
    MoveEast = 0x66,
    MoveSouth = 0x67,
    MoveWest = 0x68,
    /// Stop auto-walk
    StopWalk = 0x69,
    /// Diagonal movement
    MoveNorthEast = 0x6A,
    MoveSouthEast = 0x6B,
    MoveSouthWest = 0x6C,
    MoveNorthWest = 0x6D,
    /// Turn in a direction
    TurnNorth = 0x6F,
    TurnEast = 0x70,
    TurnSouth = 0x71,
    TurnWest = 0x72,
    /// Move/throw item
    MoveItem = 0x78,
    /// Use item
    UseItem = 0x82,
    /// Use item with (crosshair)
    UseItemWith = 0x83,
    /// Use item on creature
    UseItemOnCreature = 0x84,
    /// Rotate item
    RotateItem = 0x85,
    /// Say something
    Say = 0x96,
    /// Request channel list
    RequestChannels = 0x97,
    /// Open/join channel
    OpenChannel = 0x98,
    /// Close channel
    CloseChannel = 0x99,
    /// Attack a creature
    Attack = 0xA1,
    /// Follow a creature
    Follow = 0xA2,
    /// Cancel current action
    CancelAction = 0xBE,
}

impl ClientOpcode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::GameLogin),
            0x14 => Some(Self::Logout),
            0x64 => Some(Self::AutoWalk),
            0x65 => Some(Self::MoveNorth),
            0x66 => Some(Self::MoveEast),
            0x67 => Some(Self::MoveSouth),
            0x68 => Some(Self::MoveWest),
            0x69 => Some(Self::StopWalk),
            0x6A => Some(Self::MoveNorthEast),
            0x6B => Some(Self::MoveSouthEast),
            0x6C => Some(Self::MoveSouthWest),
            0x6D => Some(Self::MoveNorthWest),
            0x6F => Some(Self::TurnNorth),
            0x70 => Some(Self::TurnEast),
            0x71 => Some(Self::TurnSouth),
            0x72 => Some(Self::TurnWest),
            0x78 => Some(Self::MoveItem),
            0x82 => Some(Self::UseItem),
            0x83 => Some(Self::UseItemWith),
            0x84 => Some(Self::UseItemOnCreature),
            0x85 => Some(Self::RotateItem),
            0x96 => Some(Self::Say),
            0x97 => Some(Self::RequestChannels),
            0x98 => Some(Self::OpenChannel),
            0x99 => Some(Self::CloseChannel),
            0xA1 => Some(Self::Attack),
            0xA2 => Some(Self::Follow),
            0xBE => Some(Self::CancelAction),
            _ => None,
        }
    }
}

/// Login Server → Client opcodes (single byte)
///
/// These opcodes are used on the login server connection (port 7171).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LoginServerOpcode {
    /// Login error with reason
    LoginError = 0x0A,
    /// MOTD (message of the day)
    Motd = 0x14,
    /// Character list
    CharacterList = 0x64,
}

impl LoginServerOpcode {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Game Server → Client opcodes (single byte)
///
/// These opcodes are used on the game server connection (port 7172).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GameServerOpcode {
    /// Player ID assignment (sent after successful game login)
    SelfAppear = 0x0A,
    /// Full map data (sent on login/teleport)
    MapDescription = 0x64,
    /// Partial map update (player moved north)
    MapSliceNorth = 0x65,
    /// Partial map update (player moved east)
    MapSliceEast = 0x66,
    /// Partial map update (player moved south)
    MapSliceSouth = 0x67,
    /// Partial map update (player moved west)
    MapSliceWest = 0x68,
    /// Add thing to tile
    TileAddThing = 0x6A,
    /// Transform thing on tile
    TileTransformThing = 0x6B,
    /// Remove thing from tile
    TileRemoveThing = 0x6C,
    /// Creature moved
    CreatureMove = 0x6D,
    /// Open container
    ContainerOpen = 0x6E,
    /// Close container
    ContainerClose = 0x6F,
    /// Add item to container
    ContainerAddItem = 0x70,
    /// Remove item from container
    ContainerRemoveItem = 0x71,
    /// Item equipped in a slot
    EquippedItem = 0x78,
    /// Inventory slot cleared
    EquippedItemClear = 0x79,
    /// Magic effect on tile
    MagicEffect = 0x83,
    /// Distance/projectile effect
    DistanceEffect = 0x84,
    /// Creature/player health update
    CreatureHealth = 0x8C,
    /// Player stats (HP, mana, level, etc.)
    PlayerStats = 0xA0,
    /// Player skills
    PlayerSkills = 0xA1,
    /// Creature says something
    CreatureSay = 0xAA,
    /// Text message to display
    TextMessage = 0xB4,
}

impl GameServerOpcode {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Equipment slots
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EquipmentSlot {
    Head = 0x01,
    Necklace = 0x02,
    Backpack = 0x03,
    Armor = 0x04,
    RightHand = 0x05,
    LeftHand = 0x06,
    Legs = 0x07,
    Feet = 0x08,
    Ring = 0x09,
    Ammo = 0x0A,
}

impl EquipmentSlot {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::Head),
            0x02 => Some(Self::Necklace),
            0x03 => Some(Self::Backpack),
            0x04 => Some(Self::Armor),
            0x05 => Some(Self::RightHand),
            0x06 => Some(Self::LeftHand),
            0x07 => Some(Self::Legs),
            0x08 => Some(Self::Feet),
            0x09 => Some(Self::Ring),
            0x0A => Some(Self::Ammo),
            _ => None,
        }
    }
}

/// Message types for TextMessage packets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    /// Yellow game message (info)
    Info = 0x12,
    /// White message in center (event)
    Event = 0x13,
    /// Green message (status/trade)
    Status = 0x14,
    /// Red warning message
    Warning = 0x15,
    /// Blue advance message
    Advance = 0x16,
}

/// Speak types for chat messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SpeakType {
    /// Normal say (default range)
    Say = 0x01,
    /// Whisper (short range)
    Whisper = 0x02,
    /// Yell (long range)
    Yell = 0x03,
    /// Private message
    Private = 0x04,
    /// Channel message
    Channel = 0x05,
    /// NPC speech (blue text)
    Npc = 0x06,
}

/// Direction values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl Direction {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::North),
            1 => Some(Self::East),
            2 => Some(Self::South),
            3 => Some(Self::West),
            _ => None,
        }
    }

    pub fn to_delta(self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::East => (1, 0),
            Direction::South => (0, 1),
            Direction::West => (-1, 0),
        }
    }
}