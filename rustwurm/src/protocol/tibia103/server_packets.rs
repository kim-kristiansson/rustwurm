//! Server packet builders for Tibia 1.03
//!
//! These functions construct the packets sent from server to client.

use super::constants::{GameServerOpcode, EquipmentSlot, MessageType, TILE_TERMINATOR, MAP_WIDTH, MAP_HEIGHT};
use super::frame::{Frame, FrameBuilder};

// =============================================================================
// LOGIN / INITIALIZATION PACKETS
// =============================================================================

/// Build an InitGame packet (0x0A)
///
/// First packet sent after successful login.
/// Structure: playerId (u32) + sessionFlags (u16) + canReportBugs (u8)
pub fn init_game(player_id: u32) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::SelfAppear.as_u8());
    builder.write_u32(player_id);      // Player's creature ID
    builder.write_u16(0);              // Session flags (0 for 1.03)
    builder.write_u8(0);               // Can report bugs (0 = no)
    builder.build()
}

/// Build a PlayerDataBasic packet (0x9F)
///
/// Tells the client the player's creature ID and starting position.
pub fn player_data_basic(player_id: u32, x: u16, y: u16, z: u8) -> Frame {
    let mut builder = FrameBuilder::with_opcode(0x9F);
    builder.write_u32(player_id);
    builder.write_u16(x);
    builder.write_u16(y);
    builder.write_u8(z);
    builder.build()
}

/// Build a PlayerData packet (0xA0) - player stats
///
/// HP, MaxHP, Capacity, Experience, Level, Mana, MaxMana, MagicLevel, MagicLevelPercent
pub fn player_data(
    hp: u16,
    max_hp: u16,
    capacity: u16,
    experience: u32,
    level: u16,
    mana: u16,
    max_mana: u16,
    magic_level: u8,
    magic_level_percent: u8,
) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::PlayerStats.as_u8()); // 0xA0
    builder.write_u16(hp);
    builder.write_u16(max_hp);
    builder.write_u16(capacity);
    builder.write_u32(experience);
    builder.write_u16(level);
    builder.write_u16(mana);
    builder.write_u16(max_mana);
    builder.write_u8(magic_level);
    builder.write_u8(magic_level_percent);
    builder.build()
}

/// Build a PlayerSkills packet (0xA1)
///
/// All skill levels and percentages to next level.
/// Skills: Fist, Club, Sword, Axe, Distance, Shielding, Fishing
pub fn player_skills(skills: &[(u8, u8); 7]) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::PlayerSkills.as_u8()); // 0xA1
    for &(level, percent) in skills {
        builder.write_u8(level);
        builder.write_u8(percent);
    }
    builder.build()
}

/// Build default player skills (all level 10, 0%)
pub fn player_skills_default() -> Frame {
    player_skills(&[
        (10, 0), // Fist
        (10, 0), // Club
        (10, 0), // Sword
        (10, 0), // Axe
        (10, 0), // Distance
        (10, 0), // Shielding
        (10, 0), // Fishing
    ])
}

// =============================================================================
// MAP PACKETS
// =============================================================================

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

/// Build a FullMap packet (0x64)
///
/// Sends the 18x14 tile area around the player.
/// This is the minimal version - just ground tiles, no creatures/items yet.
pub fn full_map(center: Position, get_ground_tile: impl Fn(i32, i32) -> u16) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::MapDescription.as_u8()); // 0x64

    // Write center position
    center.write_to(&mut builder);

    // Map dimensions for Tibia 1.03: 18 wide x 14 tall, single floor
    let half_width = (MAP_WIDTH / 2) as i32;
    let half_height = (MAP_HEIGHT / 2) as i32;

    let start_x = center.x as i32 - half_width;
    let start_y = center.y as i32 - half_height;

    // Write tiles row by row (top to bottom, left to right)
    for dy in 0..MAP_HEIGHT as i32 {
        for dx in 0..MAP_WIDTH as i32 {
            let world_x = start_x + dx;
            let world_y = start_y + dy;

            // Get ground tile ID
            let ground_id = get_ground_tile(world_x, world_y);

            // Write ground item
            builder.write_u16(ground_id);

            // End of tile stack (0xFF 0xFF means no more items on this tile)
            builder.write_bytes(&TILE_TERMINATOR);
        }
    }

    builder.build()
}

/// Build a simple FullMap with grass (item 100) and walls (item 101)
pub fn full_map_simple(center: Position, is_walkable: impl Fn(i32, i32) -> bool) -> Frame {
    full_map(center, |x, y| {
        if is_walkable(x, y) {
            100 // Grass tile
        } else {
            101 // Stone wall
        }
    })
}

/// Represents a single tile for map encoding (extended version)
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

// =============================================================================
// INVENTORY / EQUIPMENT PACKETS
// =============================================================================

/// Build an EquippedItem packet (0x78)
///
/// Informs the client about an item in an equipment slot.
pub fn equipped_item(item_id: u16, slot: EquipmentSlot) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::EquippedItem.as_u8());
    builder.write_u8(slot as u8);
    builder.write_u16(item_id);
    builder.build()
}

/// Build a ClearInventorySlot packet (0x79)
pub fn clear_inventory_slot(slot: EquipmentSlot) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::EquippedItemClear.as_u8());
    builder.write_u8(slot as u8);
    builder.build()
}

// =============================================================================
// TEXT / CHAT PACKETS
// =============================================================================

/// Build a TextMessage packet (0xB4)
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

/// Build a warning message (red text)
pub fn warning_message(message: &str) -> Frame {
    text_message(message, MessageType::Warning)
}

/// Build a TextMessage packet for login failure
pub fn login_failed(reason: &str) -> Frame {
    text_message(reason, MessageType::Warning)
}

// =============================================================================
// CREATURE PACKETS
// =============================================================================

/// Build a CreatureHealth packet (0x8C)
pub fn creature_health(creature_id: u32, health_percent: u8) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::CreatureHealth.as_u8());
    builder.write_u32(creature_id);
    builder.write_u8(health_percent);
    builder.build()
}

/// Build a creature move packet (0x6D)
pub fn creature_move(creature_id: u32, from: Position, to: Position) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::CreatureMove.as_u8());
    from.write_to(&mut builder);
    builder.write_u8(0x01); // Stack position (simplified)
    to.write_to(&mut builder);
    builder.build()
}

/// Build a "add thing to tile" packet (0x6A) for a creature appearing
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

/// Build a "remove thing from tile" packet (0x6C)
pub fn tile_remove_thing(pos: Position, stack_pos: u8) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::TileRemoveThing.as_u8());
    pos.write_to(&mut builder);
    builder.write_u8(stack_pos);
    builder.build()
}

// =============================================================================
// EFFECT PACKETS
// =============================================================================

/// Build a magic effect packet (0x83)
pub fn magic_effect(pos: Position, effect_type: u8) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::MagicEffect.as_u8());
    pos.write_to(&mut builder);
    builder.write_u8(effect_type);
    builder.build()
}

/// Build a distance/projectile effect packet (0x84)
pub fn distance_effect(from: Position, to: Position, effect_type: u8) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::DistanceEffect.as_u8());
    from.write_to(&mut builder);
    to.write_to(&mut builder);
    builder.write_u8(effect_type);
    builder.build()
}

/// Build a creature say packet (0xAA)
pub fn creature_say(creature_id: u32, name: &str, speak_type: u8, message: &str) -> Frame {
    let mut builder = FrameBuilder::with_opcode(GameServerOpcode::CreatureSay.as_u8());
    builder.write_u32(creature_id);
    builder.write_string(name);
    builder.write_u8(speak_type);
    builder.write_string(message);
    builder.build()
}

// =============================================================================
// LEGACY ALIASES (for compatibility)
// =============================================================================

/// Legacy alias for player_data
pub fn player_stats(hp: u16, max_hp: u16, cap: u16, exp: u32, level: u16, mana: u16, max_mana: u16) -> Frame {
    player_data(hp, max_hp, cap, exp, level, mana, max_mana, 0, 0)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_game() {
        let frame = init_game(12345);
        assert_eq!(frame.opcode(), Some(GameServerOpcode::SelfAppear.as_u8()));
        // player_id (4) + sessionFlags (2) + canReportBugs (1) = 7 bytes payload
        assert_eq!(frame.payload().len(), 7);
    }

    #[test]
    fn test_player_data_basic() {
        let frame = player_data_basic(12345, 100, 200, 7);
        assert_eq!(frame.opcode(), Some(0x9F));
        // player_id (4) + x (2) + y (2) + z (1) = 9 bytes payload
        assert_eq!(frame.payload().len(), 9);
    }

    #[test]
    fn test_player_skills() {
        let frame = player_skills_default();
        assert_eq!(frame.opcode(), Some(0xA1));
        // 7 skills * 2 bytes each = 14 bytes payload
        assert_eq!(frame.payload().len(), 14);
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
    fn test_creature_health() {
        let frame = creature_health(0xABCD1234, 75);
        assert_eq!(frame.opcode(), Some(GameServerOpcode::CreatureHealth.as_u8()));
        // creature_id (4) + health_percent (1) = 5 bytes
        assert_eq!(frame.payload().len(), 5);
        assert_eq!(frame.payload()[4], 75);
    }
}