#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    Floor,
    Wall,
}

impl Tile {
    pub fn is_walkable(&self) -> bool {
        matches!(self, Tile::Floor)
    }

    pub fn from_char(ch: char) -> Self {
        match ch {
            '#' => Tile::Wall,
            _ => Tile::Floor,
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Tile::Floor => '.',
            Tile::Wall => '#',
        }
    }

    /// Get item ID for this tile type (for protocol)
    pub fn item_id(&self) -> u16 {
        match self {
            Tile::Floor => 100,  // Grass tile
            Tile::Wall => 101,   // Stone wall
        }
    }
}