//! Protocol constants for Tibia 1.03
//!
//! These values are derived from the Tibia 1.03 Protocol Specification.
//! Note: Opcodes are single-byte values in this protocol version.
//!
//! IMPORTANT: Tibia 1.03 server-to-client packets have a 4-zero-byte prefix
//! before the opcode. This is handled in the frame layer.

/// Magic bytes at the start of a game login packet
pub const LOGIN_MAGIC: [u8; 5] = [0x00, 0x00, 0x01, 0x01, 0x00];

/// Protocol version identifier (0x0067 = 103 decimal)
pub const PROTOCOL_VERSION: u16 = 0x0067;

/// Fixed sizes for login packet fields
pub const LOGIN_PACKET_LENGTH: u16 = 67;
pub const LOGIN_BODY_LENGTH: usize = 67;
pub const LOGIN_NAME_LENGTH: usize = 30;
pub const LOGIN_PASSWORD_LENGTH: usize = 30;

/// Map viewport dimensions for Tibia 1.03
pub const MAP_WIDTH: usize = 18;
pub const MAP_HEIGHT: usize = 14;
pub const MAP_DEPTH: usize = 1;

/// Server packet header size (4 zero bytes before opcode)
pub const SERVER_HEADER_PREFIX_SIZE: usize = 4;

/// Tile terminator sequence (end of objects + end of tile)
pub const TILE_TERMINATOR: [u8; 2] = [0xFF, 0xFF];

/// Map data terminator sequence
pub const MAP_TERMINATOR: [u8; 2] = [0xFE, 0x00];

/// Character marker in map data
pub const CHARACTER_MARKER: u8 = 0xFB;

/// End of tile marker
pub const END_OF_TILE: u8 = 0xFF;

/// End of map marker
pub const END_OF_MAP: u8 = 0xFE;

// =============================================================================
// Client → Server Opcodes (Section 6.1)
// =============================================================================

/// Client → Server opcodes for Tibia 1.03
///
/// These are the actual opcodes used by the Tibia 1.03 client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ClientOpcode {
    /// Request online player list
    UserList = 0x03,
    /// Request player information
    PlayerInfo = 0x04,
    /// Walk one tile in direction (payload: direction u8)
    Walk = 0x05,
    /// Auto-walk to position (payload: x u8, y u8)
    AutoWalk = 0x06,
    /// Look at position/item (payload: x u8, y u8)
    LookAt = 0x07,
    /// Send chat message (payload: length u16, message bytes)
    Chat = 0x09,
    /// Turn character without moving (payload: direction u8)
    ChangeDirection = 0x0A,
    /// Send comment to server
    Comment = 0x0B,
    /// Move/throw item (payload: fromX, fromY, itemId, stackPos, toX, toY)
    Push = 0x14,
    /// Use an item (payload: type, x, y, itemId, stackPos, unknown)
    UseItem = 0x1E,
    /// Close container window (payload: localId u8)
    CloseContainer = 0x1F,
    /// Request data window
    RequestChangeData = 0x20,
    /// Update character data
    SetData = 0x21,
    /// Set text on item
    SetText = 0x23,
    /// Set house text
    HouseText = 0x24,
    /// Change fight mode/stance (payload: fightMode u8, stance u8)
    ChangeMode = 0x32,
    /// Stop attacking
    ExitBattle = 0x33,
    /// Set attack target (payload: creatureId u32)
    SetTarget = 0x34,
    /// Keep-alive response
    Echo = 0xC8,
    /// Disconnect
    Logout = 0xFF,
}

impl ClientOpcode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x03 => Some(Self::UserList),
            0x04 => Some(Self::PlayerInfo),
            0x05 => Some(Self::Walk),
            0x06 => Some(Self::AutoWalk),
            0x07 => Some(Self::LookAt),
            0x09 => Some(Self::Chat),
            0x0A => Some(Self::ChangeDirection),
            0x0B => Some(Self::Comment),
            0x14 => Some(Self::Push),
            0x1E => Some(Self::UseItem),
            0x1F => Some(Self::CloseContainer),
            0x20 => Some(Self::RequestChangeData),
            0x21 => Some(Self::SetData),
            0x23 => Some(Self::SetText),
            0x24 => Some(Self::HouseText),
            0x32 => Some(Self::ChangeMode),
            0x33 => Some(Self::ExitBattle),
            0x34 => Some(Self::SetTarget),
            0xC8 => Some(Self::Echo),
            0xFF => Some(Self::Logout),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

// =============================================================================
// Server → Client Opcodes (Section 5.1)
// =============================================================================

/// Server → Client opcodes for Tibia 1.03
///
/// All server packets are prefixed with 4 zero bytes before the opcode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ServerOpcode {
    /// No operation (Tibia 1.03 exclusive)
    Noop = 0x00,
    /// Login confirmation (no payload in 1.03, player ID in 3.0+)
    Login = 0x01,
    /// Error message display (payload: null-terminated string)
    Error = 0x02,
    /// Character data editing window
    DataWindow = 0x03,
    /// Information popup (payload: null-terminated string)
    Info = 0x04,
    /// Message of the day display
    MessageOfTheDay = 0x05,
    /// Full map description (payload: x u8, y u8, tile data)
    Map = 0x0A,
    /// Map scroll north (payload: 18x1 row of tiles)
    MoveOneTileNorth = 0x0B,
    /// Map scroll east (payload: 1x14 column of tiles)
    MoveOneTileEast = 0x0C,
    /// Map scroll south (payload: 18x1 row of tiles)
    MoveOneTileSouth = 0x0D,
    /// Map scroll west (payload: 1x14 column of tiles)
    MoveOneTileWest = 0x0E,
    /// Close container window
    CloseContainer = 0x12,
    /// Open container window (payload: localId, itemId, items, 0xFFFF)
    OpenContainer = 0x13,
    /// Add item to inventory slot (payload: itemId u16, slot u8)
    EquippedItem = 0x14,
    /// Remove item from inventory
    RemoveEquippedItem = 0x15,
    /// Update tile object (add/remove/update)
    UpdateObject = 0x19,
    /// Green text message
    GreenChat = 0x64,
    /// Chat message (payload: x, y, type, name + TAB + msg + NULL)
    Chat = 0x65,
    /// Online player list
    UserList = 0x66,
    /// Player information
    UserInfo = 0x67,
    /// Status bar message (payload: null-terminated string)
    StatusMessage = 0x68,
    /// Connection keep-alive
    Echo = 0xC8,
}

impl ServerOpcode {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::Noop),
            0x01 => Some(Self::Login),
            0x02 => Some(Self::Error),
            0x03 => Some(Self::DataWindow),
            0x04 => Some(Self::Info),
            0x05 => Some(Self::MessageOfTheDay),
            0x0A => Some(Self::Map),
            0x0B => Some(Self::MoveOneTileNorth),
            0x0C => Some(Self::MoveOneTileEast),
            0x0D => Some(Self::MoveOneTileSouth),
            0x0E => Some(Self::MoveOneTileWest),
            0x12 => Some(Self::CloseContainer),
            0x13 => Some(Self::OpenContainer),
            0x14 => Some(Self::EquippedItem),
            0x15 => Some(Self::RemoveEquippedItem),
            0x19 => Some(Self::UpdateObject),
            0x64 => Some(Self::GreenChat),
            0x65 => Some(Self::Chat),
            0x66 => Some(Self::UserList),
            0x67 => Some(Self::UserInfo),
            0x68 => Some(Self::StatusMessage),
            0xC8 => Some(Self::Echo),
            _ => None,
        }
    }
}

// =============================================================================
// Equipment Slots
// =============================================================================

/// Equipment slots (inventory positions)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EquipmentSlot {
    Helmet = 0x01,
    Necklace = 0x02,
    Backpack = 0x03,
    Armor = 0x04,
    RightHand = 0x05,
    LeftHand = 0x06,
    Legs = 0x07,
    Boots = 0x08,
}

impl EquipmentSlot {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::Helmet),
            0x02 => Some(Self::Necklace),
            0x03 => Some(Self::Backpack),
            0x04 => Some(Self::Armor),
            0x05 => Some(Self::RightHand),
            0x06 => Some(Self::LeftHand),
            0x07 => Some(Self::Legs),
            0x08 => Some(Self::Boots),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

// =============================================================================
// Chat Types (Section 10.1)
// =============================================================================

/// Chat message types for the Chat (0x65) server packet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ChatType {
    /// Red screen only (#a)
    RedScreenOnly = 0x41,
    /// Grey console only
    GreyConsoleOnly = 0x42,
    /// Green console, yellow screen
    GreenConsoleYellowScreen = 0x43,
    /// Grey console, white screen (#b)
    GreyConsoleWhiteScreen = 0x44,
    /// Grey console, yellow screen
    GreyConsoleYellowScreen = 0x45,
    /// Grey console, yellow screen (variant 2)
    GreyConsoleYellowScreen2 = 0x46,
    /// Red console, white screen (#g)
    RedConsoleWhiteScreen = 0x47,
    /// Red console, yellow screen
    RedConsoleYellowScreen = 0x48,
    /// Red console, yellow screen (variant 2)
    RedConsoleYellowScreen2 = 0x49,
    /// Red console, yellow screen (variant 3)
    RedConsoleYellowScreen3 = 0x4A,
    /// Green screen only (anonymous)
    GreenScreenOnly = 0x4D,
    /// Blue console, yellow screen
    BlueConsoleYellowScreen = 0x4E,
    /// Blue console, yellow screen (variant 2)
    BlueConsoleYellowScreen2 = 0x4F,
    /// Blue console, white screen (private message)
    BlueConsoleWhiteScreen = 0x50,
    /// Blue console, yellow screen (variant 3)
    BlueConsoleYellowScreen3 = 0x51,
    /// Blue console, yellow screen (variant 4)
    BlueConsoleYellowScreen4 = 0x52,
    /// Normal chat (default)
    Normal = 0x53,
    /// Whisper (#w)
    Whisper = 0x57,
    /// Yell (#y)
    Yell = 0x59,
}

impl ChatType {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

// =============================================================================
// Direction
// =============================================================================

/// Direction values for Walk (0x05) and ChangeDirection (0x0A)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    North = 0x00,
    East = 0x01,
    South = 0x02,
    West = 0x03,
}

impl Direction {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::North),
            0x01 => Some(Self::East),
            0x02 => Some(Self::South),
            0x03 => Some(Self::West),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
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

// =============================================================================
// Fight Mode / Stance
// =============================================================================

/// Fight modes for ChangeMode (0x32)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FightMode {
    Offensive = 0x01,
    Normal = 0x02,
    Defensive = 0x03,
}

/// Fight stances for ChangeMode (0x32)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FightStance {
    StandStill = 0x00,
    Chase = 0x01,
}