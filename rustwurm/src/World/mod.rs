//! World module - map and position types

/// A position in the game world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Position {
    /// Create a position with explicit x, y, z coordinates
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Create a position at ground level (z=7 for Tibia 1.03)
    /// This is the most common constructor for 2D usage
    pub fn ground(x: i32, y: i32) -> Self {
        Self { x, y, z: 7 }
    }

    /// Alias for ground() - creates a position with default z level
    pub fn xy(x: i32, y: i32) -> Self {
        Self::ground(x, y)
    }

    pub fn offset(&self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            z: self.z,
        }
    }

    pub fn distance_to(&self, other: &Position) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::ground(50, 50)
    }
}

/// Tile types for the map
///
/// Provides both TileType and Tile names for compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Void,
    Floor,
    Wall,
    Water,
}

/// Alias for backwards compatibility
pub type TileType = Tile;

impl Tile {
    pub fn is_walkable(&self) -> bool {
        matches!(self, Tile::Floor)
    }

    /// Alias for Floor (for compatibility with code using "Ground")
    pub const Ground: Tile = Tile::Floor;
}

/// Simple game map
pub struct Map {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let tiles = vec![Tile::Floor; width * height];
        Self { width, height, tiles }
    }

    /// Create a simple test map with walls around the edges
    pub fn test_map(width: usize, height: usize) -> Self {
        let mut map = Self::new(width, height);

        // Add walls around the perimeter
        for x in 0..width {
            map.set_tile(x as i32, 0, Tile::Wall);
            map.set_tile(x as i32, (height - 1) as i32, Tile::Wall);
        }
        for y in 0..height {
            map.set_tile(0, y as i32, Tile::Wall);
            map.set_tile((width - 1) as i32, y as i32, Tile::Wall);
        }

        map
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            Some(y as usize * self.width + x as usize)
        } else {
            None
        }
    }

    pub fn get_tile(&self, x: i32, y: i32) -> Option<Tile> {
        self.index(x, y).map(|i| self.tiles[i])
    }

    /// Get tile at position, returning Void for out-of-bounds
    pub fn tile_at(&self, x: i32, y: i32) -> Tile {
        self.get_tile(x, y).unwrap_or(Tile::Void)
    }

    pub fn set_tile(&mut self, x: i32, y: i32, tile: Tile) {
        if let Some(i) = self.index(x, y) {
            self.tiles[i] = tile;
        }
    }

    pub fn is_walkable(&self, pos: Position) -> bool {
        self.tile_at(pos.x, pos.y).is_walkable()
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::test_map(100, 100)
    }
}