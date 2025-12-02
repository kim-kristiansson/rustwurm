//! Protocol constants for Tibia 1.03
//!
//! These values are derived from analyzing the original Tibia 1.03 client/server.

/// Magic bytes at the start of a game login packet
pub const LOGIN_MAGIC: [u8; 5] = [0x00, 0x00, 0x01, 0x01, 0x00];

/// Protocol version identifier (0x0067 = 103 decimal)
pub const PROTOCOL_VERSION: u16 = 0x0067;

/// Fixed sizes for login packet fields
pub const LOGIN_PACKET_LENGTH: u16 = 67;
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

/// Client → Server opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ClientOpcode {
    /// Initial game login (special format, not opcode-based)
    GameLogin = 0x0000,
    /// Player logout request
    Logout = 0x0014,
    /// Movement in a direction
    MoveNorth = 0x0065,
    MoveEast = 0x0066,
    MoveSouth = 0x0067,
    MoveWest = 0x0068,
    /// Diagonal movement
    MoveNorthEast = 0x006A,
    MoveSouthEast = 0x006B,
    MoveSouthWest = 0x006C,
    MoveNorthWest = 0x006D,
    /// Stop auto-walk
    StopWalk = 0x0069,
    /// Turn in a direction
    TurnNorth = 0x006F,
    TurnEast = 0x0070,
    TurnSouth = 0x0071,
    TurnWest = 0x0072,
    /// Say something
    Say = 0x0096,
    /// Attack a creature
    Attack = 0x00A1,
    /// Cancel current action
    CancelAction = 0x00BE,
}

impl ClientOpcode {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x0014 => Some(Self::Logout),
            0x0065 => Some(Self::MoveNorth),
            0x0066 => Some(Self::MoveEast),
            0x0067 => Some(Self::MoveSouth),
            0x0068 => Some(Self::MoveWest),
            0x006A => Some(Self::MoveNorthEast),
            0x006B => Some(Self::MoveSouthEast),
            0x006C => Some(Self::MoveSouthWest),
            0x006D => Some(Self::MoveNorthWest),
            0x0069 => Some(Self::StopWalk),
            0x006F => Some(Self::TurnNorth),
            0x0070 => Some(Self::TurnEast),
            0x0071 => Some(Self::TurnSouth),
            0x0072 => Some(Self::TurnWest),
            0x0096 => Some(Self::Say),
            0x00A1 => Some(Self::Attack),
            0x00BE => Some(Self::CancelAction),
            _ => None,
        }
    }
}

/// Server → Client opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ServerOpcode {
    /// Login successful
    LoginOk = 0x0001,
    /// Login failed with reason
    LoginFailed = 0x0002,
    /// Full map data (sent on login/teleport)
    MapDescription = 0x000A,
    /// Partial map update (player moved north)
    MapSliceNorth = 0x000B,
    /// Partial map update (player moved east)
    MapSliceEast = 0x000C,
    /// Partial map update (player moved south)
    MapSliceSouth = 0x000D,
    /// Partial map update (player moved west)
    MapSliceWest = 0x000E,
    /// Item equipped in a slot
    EquippedItem = 0x0014,
    /// Creature/player health update
    CreatureHealth = 0x008C,
    /// Player stats (HP, mana, level, etc.)
    PlayerStats = 0x00A0,
    /// Player skills
    PlayerSkills = 0x00A1,
    /// Text message to display
    TextMessage = 0x0068,
    /// Creature says something
    CreatureSay = 0x00AA,
    /// Creature moved on map
    CreatureMove = 0x006D,
    /// Creature appeared
    CreatureAppear = 0x006A,
    /// Creature disappeared
    CreatureDisappear = 0x006B,
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
    /// Yellow game message
    Info = 0x12,
    /// White message in center
    Event = 0x13,
    /// Green message (player trade)
    Status = 0x14,
    /// Red warning message
    Warning = 0x15,
    /// Blue advance message
    Advance = 0x16,
}