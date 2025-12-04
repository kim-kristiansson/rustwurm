//! World module - map and position types

mod map;
mod tile;

// Re-export from submodules
pub use map::MapSpawns;
pub use tile::Tile as TileType;

/// A position in the game world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Position {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn ground(x: i32, y: i32) -> Self {
        Self { x, y, z: 7 }
    }

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

    pub fn distance_squared(&self, other: &Position) -> i32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    pub fn is_adjacent(&self, other: &Position) -> bool {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        dx <= 1 && dy <= 1 && (dx + dy) > 0
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::ground(50, 50)
    }
}

/// Tile types for the map
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Void,
    Floor,
    Wall,
    Water,
}

impl Tile {
    pub fn is_walkable(&self) -> bool {
        matches!(self, Tile::Floor)
    }

    pub const Ground: Tile = Tile::Floor;
}

/// Simple game map
pub struct Map {
    pub width: usize,   // Made public
    pub height: usize,  // Made public
    tiles: Vec<Tile>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let tiles = vec![Tile::Floor; width * height];
        Self { width, height, tiles }
    }

    pub fn test_map(width: usize, height: usize) -> Self {
        let mut map = Self::new(width, height);
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

    /// Parse a map from ASCII art format
    pub fn from_ascii(ascii: &str) -> (Self, MapSpawns) {
        let lines: Vec<&str> = ascii.lines().filter(|l| !l.trim().is_empty()).collect();
        let height = lines.len();
        let width = lines.first().map(|l| l.chars().count()).unwrap_or(0);

        let mut tiles = vec![Tile::Floor; width * height];
        let mut player_start = Position::ground(1, 1);
        let mut monsters = Vec::new();
        let mut npcs = Vec::new();

        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let idx = y * width + x;
                let pos = Position::ground(x as i32, y as i32);
                match ch {
                    '#' => tiles[idx] = Tile::Wall,
                    'P' => player_start = pos,
                    'D' => monsters.push(pos),
                    'N' => npcs.push(pos),
                    _ => tiles[idx] = Tile::Floor,
                }
            }
        }

        let map = Map { width, height, tiles };
        let spawns = MapSpawns { player_start, monsters, npcs };
        (map, spawns)
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            Some(y as usize * self.width + x as usize)
        } else {
            None
        }
    }

    pub fn get_tile(&self, pos: Position) -> Option<Tile> {
        self.index(pos.x, pos.y).map(|i| self.tiles[i])
    }

    pub fn tile_at(&self, x: i32, y: i32) -> Tile {
        self.index(x, y).map(|i| self.tiles[i]).unwrap_or(Tile::Void)
    }

    pub fn set_tile(&mut self, x: i32, y: i32, tile: Tile) {
        if let Some(i) = self.index(x, y) {
            self.tiles[i] = tile;
        }
    }

    pub fn is_walkable(&self, pos: Position) -> bool {
        self.tile_at(pos.x, pos.y).is_walkable()
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::test_map(100, 100)
    }
}