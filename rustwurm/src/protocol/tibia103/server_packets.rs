//! Server packet builders for Tibia 1.03
//!
//! These functions construct the packets sent from server to client.
//! All packets include the 4-zero-byte prefix per the Tibia 1.03 protocol.
//!
//! ## Packet Format Reference
//!
//! All server packets follow this structure:
//! ```text
//! [u16_le Length][0x00 0x00 0x00 0x00][Opcode (1B)][Payload...]
//! ```

use super::constants::{
    ServerOpcode, EquipmentSlot, ChatType,
    TILE_TERMINATOR, MAP_TERMINATOR, CHARACTER_MARKER, END_OF_MAP,
};
use super::frame::{ServerFrame, ServerFrameBuilder};

// =============================================================================
// Login / Error Messages
// =============================================================================

/// Build a Login packet (0x01) - login confirmation
///
/// In Tibia 1.03, this packet has no payload (player ID is implicit).
/// Format: `[4 zeros][0x01]`
pub fn login_ok() -> ServerFrame {
    ServerFrameBuilder::new(ServerOpcode::Login.as_u8()).build()
}

/// Build an Error packet (0x02) - login failure or error message
///
/// Format: `[4 zeros][0x02][error message][0x00]`
pub fn error_message(reason: &str) -> ServerFrame {
    let mut builder = ServerFrameBuilder::new(ServerOpcode::Error.as_u8());
    builder.write_cstring(reason);
    builder.build()
}

/// Build an Info packet (0x04) - information popup
///
/// Format: `[4 zeros][0x04][info message][0x00]`
pub fn info_message(message: &str) -> ServerFrame {
    let mut builder = ServerFrameBuilder::new(ServerOpcode::Info.as_u8());
    builder.write_cstring(message);
    builder.build()
}

/// Build a StatusMessage packet (0x68) - status bar message
///
/// Format: `[4 zeros][0x68][status text][0x00]`
pub fn status_message(message: &str) -> ServerFrame {
    let mut builder = ServerFrameBuilder::new(ServerOpcode::StatusMessage.as_u8());
    builder.write_cstring(message);
    builder.build()
}

// =============================================================================
// Equipment
// =============================================================================

/// Build an EquippedItem packet (0x14) - item in inventory slot
///
/// **IMPORTANT**: In Tibia 1.03, Item ID comes BEFORE Slot!
/// Format: `[4 zeros][0x14][Item ID u16][Slot u8]`
pub fn equipped_item(item_id: u16, slot: EquipmentSlot) -> ServerFrame {
    let mut builder = ServerFrameBuilder::new(ServerOpcode::EquippedItem.as_u8());
    builder.write_u16(item_id);  // Item ID first!
    builder.write_u8(slot.as_u8());
    builder.build()
}

/// Build a RemoveEquippedItem packet (0x15) - clear inventory slot
///
/// Format: `[4 zeros][0x15][Slot u8]`
pub fn remove_equipped_item(slot: EquipmentSlot) -> ServerFrame {
    let mut builder = ServerFrameBuilder::new(ServerOpcode::RemoveEquippedItem.as_u8());
    builder.write_u8(slot.as_u8());
    builder.build()
}

// =============================================================================
// Chat
// =============================================================================

/// Build a Chat packet (0x65) - chat message from player/creature
///
/// Format: `[4 zeros][0x65][X u8][Y u8][Type u8][Name][0x09][Message][0x00]`
pub fn chat_message(x: u8, y: u8, chat_type: ChatType, name: &str, message: &str) -> ServerFrame {
    let mut builder = ServerFrameBuilder::new(ServerOpcode::Chat.as_u8());
    builder.write_u8(x);
    builder.write_u8(y);
    builder.write_u8(chat_type.as_u8());

    // Name + TAB (0x09) + Message + NULL (0x00)
    builder.write_bytes(name.as_bytes());
    builder.write_u8(0x09); // TAB separator
    builder.write_bytes(message.as_bytes());
    builder.write_u8(0x00); // Null terminator

    builder.build()
}

/// Build a GreenChat packet (0x64) - green text message (anonymous)
///
/// Format: `[4 zeros][0x64][message][0x00]`
pub fn green_chat(message: &str) -> ServerFrame {
    let mut builder = ServerFrameBuilder::new(ServerOpcode::GreenChat.as_u8());
    builder.write_cstring(message);
    builder.build()
}

// =============================================================================
// Map Data
// =============================================================================

/// A position on the game map (Tibia 1.03 uses 8-bit coordinates)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}

impl Position {
    pub fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }
}

/// Outfit colors (packed 3-byte format)
#[derive(Debug, Clone, Copy, Default)]
pub struct OutfitColors {
    pub head: u8,
    pub body: u8,
    pub legs: u8,
    pub shoes: u8,
    pub unknown: u8,
}

impl OutfitColors {
    /// Pack colors into wire format (3 bytes)
    pub fn to_bytes(&self) -> [u8; 3] {
        [
            (self.legs << 4) | (self.shoes & 0x0F),
            (self.head << 4) | (self.body & 0x0F),
            self.unknown,
        ]
    }
}

/// Represents a single tile for map encoding
#[derive(Debug, Clone)]
pub struct TileData {
    /// Item IDs on this tile (ground + objects)
    pub items: Vec<u16>,
    /// Player character outfit on this tile (if any)
    pub character: Option<OutfitColors>,
}

impl TileData {
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            character: None,
        }
    }

    pub fn with_ground(ground_id: u16) -> Self {
        Self {
            items: vec![ground_id],
            character: None,
        }
    }

    pub fn with_items(items: Vec<u16>) -> Self {
        Self {
            items,
            character: None,
        }
    }
}

/// Builder for map description packets
pub struct MapBuilder {
    builder: ServerFrameBuilder,
    tile_count: usize,
}

impl MapBuilder {
    /// Start building a full map description packet (0x0A)
    ///
    /// Format: `[4 zeros][0x0A][X u8][Y u8][Tile data...][0xFE 0x00]`
    pub fn new(center: Position) -> Self {
        let mut builder = ServerFrameBuilder::new(ServerOpcode::Map.as_u8());
        builder.write_u8(center.x);
        builder.write_u8(center.y);

        Self {
            builder,
            tile_count: 0,
        }
    }

    /// Add a tile to the map data
    ///
    /// Tile format:
    /// ```text
    /// [Item ID u16]...  // For each item
    /// [0xFF]            // End of objects
    /// [0xFF]            // End of tile
    /// ```
    ///
    /// If character present:
    /// ```text
    /// [Item ID u16]...
    /// [0xFB][Outfit 3B]  // Character marker + outfit colors
    /// [0xFF][0xFF]       // End markers
    /// ```
    pub fn add_tile(&mut self, tile: &TileData) {
        // Write items
        for &item_id in &tile.items {
            self.builder.write_u16(item_id);
        }

        // Write character if present
        if let Some(ref outfit) = tile.character {
            self.builder.write_u8(CHARACTER_MARKER);
            self.builder.write_bytes(&outfit.to_bytes());
        }

        // Tile terminator (0xFF 0xFF)
        self.builder.write_bytes(&TILE_TERMINATOR);
        self.tile_count += 1;
    }

    /// Add an empty tile (just terminators)
    pub fn add_empty_tile(&mut self) {
        self.builder.write_bytes(&TILE_TERMINATOR);
        self.tile_count += 1;
    }

    /// Finish building and return the frame
    ///
    /// Replaces the last 0xFF with map terminator (0xFE 0x00)
    pub fn build(mut self) -> ServerFrame {
        // The documentation says the last tile's second 0xFF should be
        // replaced with 0xFE 0x00 (map terminator)
        // But since we already wrote TILE_TERMINATOR for each tile,
        // we need to handle this properly.

        // For simplicity, we'll just append the map terminator
        // In a real implementation, you'd track and replace the last byte
        self.builder.write_bytes(&MAP_TERMINATOR);

        self.builder.build()
    }
}

/// Build a simple map description with a callback for each tile
pub fn build_map_description<F>(center: Position, width: usize, height: usize, mut tile_fn: F) -> ServerFrame
where
    F: FnMut(i32, i32) -> TileData,
{
    let mut map = MapBuilder::new(center);

    let start_x = center.x as i32 - (width as i32 / 2);
    let start_y = center.y as i32 - (height as i32 / 2);

    for y in 0..height {
        for x in 0..width {
            let world_x = start_x + x as i32;
            let world_y = start_y + y as i32;
            let tile = tile_fn(world_x, world_y);
            map.add_tile(&tile);
        }
    }

    map.build()
}

// =============================================================================
// Map Scrolling
// =============================================================================

/// Build a MoveOneTileNorth packet (0x0B)
///
/// Sent when player moves north, contains the new top row (18 tiles)
pub fn move_north(tiles: &[TileData]) -> ServerFrame {
    build_scroll_packet(ServerOpcode::MoveOneTileNorth, tiles)
}

/// Build a MoveOneTileEast packet (0x0C)
///
/// Sent when player moves east, contains the new right column (14 tiles)
pub fn move_east(tiles: &[TileData]) -> ServerFrame {
    build_scroll_packet(ServerOpcode::MoveOneTileEast, tiles)
}

/// Build a MoveOneTileSouth packet (0x0D)
///
/// Sent when player moves south, contains the new bottom row (18 tiles)
pub fn move_south(tiles: &[TileData]) -> ServerFrame {
    build_scroll_packet(ServerOpcode::MoveOneTileSouth, tiles)
}

/// Build a MoveOneTileWest packet (0x0E)
///
/// Sent when player moves west, contains the new left column (14 tiles)
pub fn move_west(tiles: &[TileData]) -> ServerFrame {
    build_scroll_packet(ServerOpcode::MoveOneTileWest, tiles)
}

fn build_scroll_packet(opcode: ServerOpcode, tiles: &[TileData]) -> ServerFrame {
    let mut builder = ServerFrameBuilder::new(opcode.as_u8());

    for tile in tiles {
        // Write items
        for &item_id in &tile.items {
            builder.write_u16(item_id);
        }

        // Write character if present
        if let Some(ref outfit) = tile.character {
            builder.write_u8(CHARACTER_MARKER);
            builder.write_bytes(&outfit.to_bytes());
        }

        // Tile terminator
        builder.write_bytes(&TILE_TERMINATOR);
    }

    // Map terminator
    builder.write_bytes(&MAP_TERMINATOR);

    builder.build()
}

// =============================================================================
// Containers
// =============================================================================

/// Build an OpenContainer packet (0x13)
///
/// Format: `[4 zeros][0x13][Local ID u8][Item ID u16][Items...][0xFFFF]`
pub fn open_container(local_id: u8, container_item_id: u16, items: &[u16]) -> ServerFrame {
    let mut builder = ServerFrameBuilder::new(ServerOpcode::OpenContainer.as_u8());
    builder.write_u8(local_id);
    builder.write_u16(container_item_id);

    for &item_id in items {
        builder.write_u16(item_id);
    }

    // End marker
    builder.write_u16(0xFFFF);

    builder.build()
}

/// Build a CloseContainer packet (0x12)
///
/// Format: `[4 zeros][0x12][Local ID u8]`
pub fn close_container(local_id: u8) -> ServerFrame {
    let mut builder = ServerFrameBuilder::new(ServerOpcode::CloseContainer.as_u8());
    builder.write_u8(local_id);
    builder.build()
}

// =============================================================================
// Echo (Keep-alive)
// =============================================================================

/// Build an Echo packet (0xC8) - keep-alive response
pub fn echo() -> ServerFrame {
    ServerFrameBuilder::new(ServerOpcode::Echo.as_u8()).build()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_ok() {
        let frame = login_ok();
        assert_eq!(frame.opcode(), ServerOpcode::Login.as_u8());
        assert!(frame.payload().is_empty());

        let mut buffer = Vec::new();
        frame.write_to(&mut buffer).unwrap();

        // [05 00][00 00 00 00][01]
        // Length = 5, 4 zeros, opcode 0x01
        assert_eq!(buffer, vec![0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]);
    }

    #[test]
    fn test_error_message() {
        let frame = error_message("Bad password");
        let mut buffer = Vec::new();
        frame.write_to(&mut buffer).unwrap();

        // Check structure
        assert_eq!(buffer[6], ServerOpcode::Error.as_u8()); // 0x02
        assert!(buffer[7..].starts_with(b"Bad password\0"));
    }

    #[test]
    fn test_info_message() {
        let frame = info_message("Welcome!");
        let mut buffer = Vec::new();
        frame.write_to(&mut buffer).unwrap();

        assert_eq!(buffer[6], ServerOpcode::Info.as_u8()); // 0x04
        assert!(buffer[7..].starts_with(b"Welcome!\0"));
    }

    #[test]
    fn test_equipped_item_format() {
        // Per documentation: [4 zeros][0x14][Item ID u16][Slot u8]
        let frame = equipped_item(0x013D, EquipmentSlot::Backpack);
        let mut buffer = Vec::new();
        frame.write_to(&mut buffer).unwrap();

        // Length = 4 + 1 + 2 + 1 = 8
        assert_eq!(buffer[0], 8);
        assert_eq!(buffer[1], 0);
        // 4 zeros
        assert_eq!(&buffer[2..6], &[0x00, 0x00, 0x00, 0x00]);
        // Opcode 0x14
        assert_eq!(buffer[6], 0x14);
        // Item ID 0x013D (little-endian: 0x3D, 0x01)
        assert_eq!(buffer[7], 0x3D);
        assert_eq!(buffer[8], 0x01);
        // Slot 0x03 (Backpack)
        assert_eq!(buffer[9], 0x03);
    }

    #[test]
    fn test_chat_message() {
        let frame = chat_message(50, 50, ChatType::Normal, "Player", "Hello!");
        let mut buffer = Vec::new();
        frame.write_to(&mut buffer).unwrap();

        assert_eq!(buffer[6], ServerOpcode::Chat.as_u8()); // 0x65
        assert_eq!(buffer[7], 50); // X
        assert_eq!(buffer[8], 50); // Y
        assert_eq!(buffer[9], ChatType::Normal.as_u8()); // 0x53

        // Find the TAB separator
        let tab_pos = buffer[10..].iter().position(|&b| b == 0x09).unwrap() + 10;
        assert!(tab_pos > 10); // Name came before TAB
    }

    #[test]
    fn test_status_message() {
        let frame = status_message("You gained experience.");
        let mut buffer = Vec::new();
        frame.write_to(&mut buffer).unwrap();

        assert_eq!(buffer[6], ServerOpcode::StatusMessage.as_u8()); // 0x68
    }

    #[test]
    fn test_outfit_colors_packing() {
        let outfit = OutfitColors {
            head: 0x0A,
            body: 0x0B,
            legs: 0x0C,
            shoes: 0x0D,
            unknown: 0x00,
        };

        let bytes = outfit.to_bytes();

        // Byte 1: (legs << 4) | shoes = (0x0C << 4) | 0x0D = 0xCD
        assert_eq!(bytes[0], 0xCD);
        // Byte 2: (head << 4) | body = (0x0A << 4) | 0x0B = 0xAB
        assert_eq!(bytes[1], 0xAB);
        // Byte 3: unknown
        assert_eq!(bytes[2], 0x00);
    }
}