use super::tile::Tile;

/// Represents a position on the map
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn offset(&self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
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

/// Spawn data parsed from map files
#[derive(Debug)]
pub struct MapSpawns {
    pub player_start: Position,
    pub monsters: Vec<Position>,
    pub npcs: Vec<Position>,
}

pub struct Map {
    pub width: usize,
    pub height: usize,
    tiles: Vec<Tile>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let mut tiles = vec![Tile::Floor; width * height];

        // Create border walls
        for x in 0..width {
            tiles[x] = Tile::Wall;
            tiles[x + (height - 1) * width] = Tile::Wall;
        }
        for y in 0..height {
            tiles[y * width] = Tile::Wall;
            tiles[y * width + (width - 1)] = Tile::Wall;
        }

        Self { width, height, tiles }
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 {
            return None;
        }
        let (ux, uy) = (x as usize, y as usize);
        if ux >= self.width || uy >= self.height {
            None
        } else {
            Some(uy * self.width + ux)
        }
    }

    pub fn get_tile(&self, pos: Position) -> Option<Tile> {
        self.index(pos.x, pos.y).map(|idx| self.tiles[idx])
    }

    pub fn is_walkable(&self, pos: Position) -> bool {
        self.index(pos.x, pos.y)
            .map(|idx| self.tiles[idx].is_walkable())
            .unwrap_or(false)
    }

    /// Parse a map from ASCII art format
    /// Returns the map and spawn information
    pub fn from_ascii(ascii: &str) -> (Self, MapSpawns) {
        let lines: Vec<&str> = ascii
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect();

        let height = lines.len();
        let width = lines.first().map(|l| l.chars().count()).unwrap_or(0);

        let mut tiles = vec![Tile::Floor; width * height];
        let mut player_start = Position::new(1, 1);
        let mut monsters = Vec::new();
        let mut npcs = Vec::new();

        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let idx = y * width + x;
                let pos = Position::new(x as i32, y as i32);

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
        let spawns = MapSpawns {
            player_start,
            monsters,
            npcs,
        };

        (map, spawns)
    }

    /// Render the map to a string (for debugging/display)
    pub fn render(&self) -> String {
        let mut output = String::with_capacity(self.width * self.height + self.height);

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                output.push(self.tiles[idx].to_char());
            }
            output.push('\n');
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_adjacent() {
        let p1 = Position::new(5, 5);
        assert!(p1.is_adjacent(&Position::new(5, 4)));
        assert!(p1.is_adjacent(&Position::new(6, 5)));
        assert!(!p1.is_adjacent(&Position::new(5, 5))); // same position
        assert!(!p1.is_adjacent(&Position::new(7, 5))); // too far
    }

    #[test]
    fn test_map_walkable() {
        let map = Map::new(10, 10);
        assert!(!map.is_walkable(Position::new(0, 0))); // corner wall
        assert!(map.is_walkable(Position::new(5, 5)));  // center floor
    }
}