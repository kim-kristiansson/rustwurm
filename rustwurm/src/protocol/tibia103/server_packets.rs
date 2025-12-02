//! Server packet builders for Tibia 1.03
//!
//! These functions construct the packets sent from server to client.

use super::constants::{ServerOpcode, EquipmentSlot, MessageType, TILE_TERMINATOR, MAP_TERMINATOR};
use super::frame::{Frame, FrameBuilder};

/// Build a LoginOk packet
///
/// Sent when login credentials are accepted
pub fn login_ok() -> Frame {
    FrameBuilder::with_opcode(ServerOpcode::LoginOk as u16).build()
}

/// Build a LoginFailed packet with reason
pub fn login_failed(reason: &str) -> Frame {
    FrameBuilder::with_opcode(ServerOpcode::LoginFailed as u16)
        .write_cstring(reason)
        .build()
}

/// Build an EquippedItem packet
///
/// Informs the client about an item in an equipment slot
pub fn equipped_item(item_id: u16, slot: EquipmentSlot) -> Frame {
    FrameBuilder::with_opcode(ServerOpcode::EquippedItem as u16)
        .write_u16(item_id)
        .write_u8(slot as u8)
        .build()
}

/// Build a TextMessage packet
pub fn text_message(message: &str, msg_type: MessageType) -> Frame {
    FrameBuilder::with_opcode(ServerOpcode::TextMessage as u16)
        .write_u8(msg_type as u8)
        .write_cstring(message)
        .build()
}

/// Build a simple text message (info type)
pub fn info_message(message: &str) -> Frame {
    text_message(message, MessageType::Info)
}

/// Build a PlayerStats packet
pub fn player_stats(hp: u16, max_hp: u16, cap: u16, exp: u32, level: u16, mana: u16, max_mana: u16) -> Frame {
    FrameBuilder::with_opcode(ServerOpcode::PlayerStats as u16)
        .write_u16(hp)
        .write_u16(max_hp)
        .write_u16(cap)
        .write_u32(exp)
        .write_u16(level)
        .write_u16(mana)
        .write_u16(max_mana)
        .build()
}

/// Build a CreatureHealth packet
pub fn creature_health(creature_id: u32, health_percent: u8) -> Frame {
    FrameBuilder::with_opcode(ServerOpcode::CreatureHealth as u16)
        .write_u32(creature_id)
        .write_u8(health_percent)
        .build()
}

/// A position on the game map
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: u16,
    pub y: u16,
    pub z: u8,
}

impl Position {
    pub fn new(x: u16, y: u16, z: u8) -> Self {
        Self { x, y, z }
    }

    /// Write position to a frame builder
    pub fn write_to(&self, builder: &mut FrameBuilder) {
        builder.write_u16(self.x);
        builder.write_u16(self.y);
        builder.write_u8(self.z);
    }
}

/// Represents a single tile for map encoding
#[derive(Debug, Clone)]
pub struct TileData {
    /// Ground item ID (0 for none)
    pub ground: u16,
    /// Items stacked on this tile
    pub items: Vec<u16>,
    /// Creature on this tile (if any)
    pub creature: Option<CreatureInfo>,
}

impl TileData {
    pub fn empty() -> Self {
        Self {
            ground: 0,
            items: Vec::new(),
            creature: None,
        }
    }

    pub fn with_ground(ground_id: u16) -> Self {
        Self {
            ground: ground_id,
            items: Vec::new(),
            creature: None,
        }
    }
}

/// Information about a creature for map data
#[derive(Debug, Clone)]
pub struct CreatureInfo {
    pub id: u32,
    pub name: String,
    pub health_percent: u8,
    pub direction: u8,
    pub outfit: OutfitInfo,
}

/// Creature outfit information
#[derive(Debug, Clone)]
pub struct OutfitInfo {
    pub look_type: u16,
    pub head: u8,
    pub body: u8,
    pub legs: u8,
    pub feet: u8,
}

impl Default for OutfitInfo {
    fn default() -> Self {
        Self {
            look_type: 128, // Default human look
            head: 0,
            body: 0,
            legs: 0,
            feet: 0,
        }
    }
}

/// Builder for map description packets
pub struct MapBuilder {
    builder: FrameBuilder,
}

impl MapBuilder {
    /// Start building a full map description packet
    pub fn new(center: Position) -> Self {
        let mut builder = FrameBuilder::with_opcode(ServerOpcode::MapDescription as u16);
        center.write_to(&mut builder);

        Self { builder }
    }

    /// Add a tile to the map data
    pub fn add_tile(&mut self, tile: &TileData) {
        // Write ground
        if tile.ground != 0 {
            self.builder.write_u16(tile.ground);
        }

        // Write stacked items
        for &item_id in &tile.items {
            self.builder.write_u16(item_id);
        }

        // Write creature if present
        if let Some(ref creature) = tile.creature {
            // Creature marker (0x0061 for known creature, 0x0062 for new)
            self.builder.write_u16(0x0062); // New creature
            self.builder.write_u32(0); // Remove ID (0 for new)
            self.builder.write_u32(creature.id);
            self.builder.write_string(&creature.name);
            self.builder.write_u8(creature.health_percent);
            self.builder.write_u8(creature.direction);
            self.builder.write_u16(creature.outfit.look_type);
            self.builder.write_u8(creature.outfit.head);
            self.builder.write_u8(creature.outfit.body);
            self.builder.write_u8(creature.outfit.legs);
            self.builder.write_u8(creature.outfit.feet);
        }

        // Tile terminator
        self.builder.write_bytes(&TILE_TERMINATOR);
    }

    /// Add an empty/skip tile
    pub fn add_empty_tile(&mut self) {
        self.builder.write_bytes(&TILE_TERMINATOR);
    }

    /// Finish building and return the frame
    pub fn build(mut self) -> Frame {
        // Map terminator
        self.builder.write_bytes(&MAP_TERMINATOR);
        self.builder.build()
    }
}

/// Build a map description with a simple callback for each tile
pub fn build_map_description<F>(center: Position, width: usize, height: usize, mut tile_fn: F) -> Frame
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

/// Build a creature move packet
pub fn creature_move(creature_id: u32, from: Position, to: Position) -> Frame {
    let mut builder = FrameBuilder::with_opcode(ServerOpcode::CreatureMove as u16);
    from.write_to(&mut builder);
    builder.write_u8(0x01); // Stack position (simplified)
    to.write_to(&mut builder);
    builder.build()
}

/// Build a creature appear packet
pub fn creature_appear(pos: Position, creature: &CreatureInfo) -> Frame {
    let mut builder = FrameBuilder::with_opcode(ServerOpcode::CreatureAppear as u16);
    pos.write_to(&mut builder);

    builder.write_u16(0x0062); // New creature marker
    builder.write_u32(0); // Remove ID
    builder.write_u32(creature.id);
    builder.write_string(&creature.name);
    builder.write_u8(creature.health_percent);
    builder.write_u8(creature.direction);
    builder.write_u16(creature.outfit.look_type);
    builder.write_u8(creature.outfit.head);
    builder.write_u8(creature.outfit.body);
    builder.write_u8(creature.outfit.legs);
    builder.write_u8(creature.outfit.feet);

    builder.build()
}

/// Build a creature disappear packet
pub fn creature_disappear(pos: Position, stack_pos: u8) -> Frame {
    let mut builder = FrameBuilder::with_opcode(ServerOpcode::CreatureDisappear as u16);
    pos.write_to(&mut builder);
    builder.write_u8(stack_pos);
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_ok() {
        let frame = login_ok();
        assert_eq!(frame.opcode(), Some(ServerOpcode::LoginOk as u16));
        assert_eq!(frame.payload().len(), 0);
    }

    #[test]
    fn test_text_message() {
        let frame = info_message("Hello, World!");
        assert_eq!(frame.opcode(), Some(ServerOpcode::TextMessage as u16));
        // Should contain message type + null-terminated string
        assert!(frame.payload().len() > 14);
    }

    #[test]
    fn test_equipped_item() {
        let frame = equipped_item(0x013D, EquipmentSlot::Backpack);
        assert_eq!(frame.opcode(), Some(ServerOpcode::EquippedItem as u16));
        assert_eq!(frame.payload(), &[0x3D, 0x01, 0x03]);
    }
}