//! Server packet builders for Tibia 1.03
//!
//! These functions construct the packets sent from server to client.

use super::constants::{GameServerOpcode, EquipmentSlot, MessageType, TILE_TERMINATOR, MAP_TERMINATOR};
use super::frame::{Frame, FrameBuilder};

/// Build a SelfAppear packet (login OK for game server)
///
/// Sent when login credentials are accepted.
/// Contains the player's creature ID for identification.
pub fn login_ok(player_id: u32) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::SelfAppear.as_u8());
    builder.write_u32(player_id);
    builder.build()
}

/// Build a TextMessage packet for login failure
/// (Game server doesn't have a dedicated "login failed" opcode - it uses text message)
pub fn login_failed(reason: &str) -> Frame {
    text_message(reason, MessageType::Warning)
}

/// Build an EquippedItem packet
///
/// Informs the client about an item in an equipment slot
pub fn equipped_item(item_id: u16, slot: EquipmentSlot) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::EquippedItem.as_u8());
    builder.write_u8(slot as u8);
    builder.write_u16(item_id);
    builder.build()
}

/// Build a TextMessage packet
pub fn text_message(message: &str, msg_type: MessageType) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::TextMessage.as_u8());
    builder.write_u8(msg_type as u8);
    builder.write_string(message);
    builder.build()
}

/// Build a simple text message (info type)
pub fn info_message(message: &str) -> Frame {
    text_message(message, MessageType::Info)
}

/// Build a PlayerStats packet
pub fn player_stats(hp: u16, max_hp: u16, cap: u16, exp: u32, level: u16, mana: u16, max_mana: u16) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::PlayerStats.as_u8());
    builder.write_u16(hp);
    builder.write_u16(max_hp);
    builder.write_u16(cap);
    builder.write_u32(exp);
    builder.write_u16(level);
    builder.write_u16(mana);
    builder.write_u16(max_mana);
    builder.build()
}

/// Build a CreatureHealth packet
pub fn creature_health(creature_id: u32, health_percent: u8) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::CreatureHealth.as_u8());
    builder.write_u32(creature_id);
    builder.write_u8(health_percent);
    builder.build()
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
    pub light_intensity: u8,
    pub light_color: u8,
    pub speed: u16,
}

impl CreatureInfo {
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            health_percent: 100,
            direction: 2, // South
            outfit: OutfitInfo::default(),
            light_intensity: 0,
            light_color: 0,
            speed: 220,
        }
    }
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
        let mut builder = FrameBuilder::with_opcode(GameServerOpcode::MapDescription.as_u8());
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
            // Creature marker (0x0062 for new creature)
            self.builder.write_u16(0x0062);
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
            self.builder.write_u8(creature.light_intensity);
            self.builder.write_u8(creature.light_color);
            self.builder.write_u16(creature.speed);
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
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::CreatureMove.as_u8());
    from.write_to(&mut builder);
    builder.write_u8(0x01); // Stack position (simplified)
    to.write_to(&mut builder);
    builder.build()
}

/// Build a "add thing to tile" packet for a creature appearing
pub fn creature_appear(pos: Position, creature: &CreatureInfo) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::TileAddThing.as_u8());
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
    builder.write_u8(creature.light_intensity);
    builder.write_u8(creature.light_color);
    builder.write_u16(creature.speed);

    builder.build()
}

/// Build a "remove thing from tile" packet
pub fn tile_remove_thing(pos: Position, stack_pos: u8) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::TileRemoveThing.as_u8());
    pos.write_to(&mut builder);
    builder.write_u8(stack_pos);
    builder.build()
}

/// Build a magic effect packet
pub fn magic_effect(pos: Position, effect_type: u8) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::MagicEffect.as_u8());
    pos.write_to(&mut builder);
    builder.write_u8(effect_type);
    builder.build()
}

/// Build a distance/projectile effect packet
pub fn distance_effect(from: Position, to: Position, effect_type: u8) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::DistanceEffect.as_u8());
    from.write_to(&mut builder);
    to.write_to(&mut builder);
    builder.write_u8(effect_type);
    builder.build()
}

/// Build a creature say packet
pub fn creature_say(creature_id: u32, name: &str, speak_type: u8, message: &str) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::CreatureSay.as_u8());
    builder.write_u32(creature_id);
    builder.write_string(name);
    builder.write_u8(speak_type);
    builder.write_string(message);
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_ok() {
        let frame = login_ok(12345);
        assert_eq!(frame.opcode(), Some(GameServerOpcode::SelfAppear.as_u8()));
        // Should have player ID in payload
        assert_eq!(frame.payload().len(), 4);
    }

    #[test]
    fn test_text_message() {
        let frame = info_message("Hello, World!");
        assert_eq!(frame.opcode(), Some(GameServerOpcode::TextMessage.as_u8()));
        // Should contain message type + length-prefixed string
        assert!(frame.payload().len() > 14);
    }

    #[test]
    fn test_equipped_item() {
        let frame = equipped_item(0x013D, EquipmentSlot::Backpack);
        assert_eq!(frame.opcode(), Some(GameServerOpcode::EquippedItem.as_u8()));
        // slot (1) + item_id (2) = 3 bytes payload
        assert_eq!(frame.payload(), &[0x03, 0x3D, 0x01]);
    }

    #[test]
    fn test_player_stats() {
        let frame = player_stats(100, 100, 150, 0, 1, 50, 50);
        assert_eq!(frame.opcode(), Some(GameServerOpcode::PlayerStats.as_u8()));
        // hp(2) + max_hp(2) + cap(2) + exp(4) + level(2) + mana(2) + max_mana(2) = 16 bytes
        assert_eq!(frame.payload().len(), 16);
    }

    #[test]
    fn test_creature_health() {
        let frame = creature_health(0xABCD1234, 75);
        assert_eq!(frame.opcode(), Some(GameServerOpcode::CreatureHealth.as_u8()));
        // creature_id (4) + health_percent (1) = 5 bytes
        assert_eq!(frame.payload().len(), 5);
        assert_eq!(frame.payload()[4], 75);
    }
}