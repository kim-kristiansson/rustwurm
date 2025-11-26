use crate::entities::{Player, Monster, Npc};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tile {
    Floor,
    Wall,
}

pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Tile>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let mut tiles = vec![Tile::Floor; width * height];

        // Walls
        for x in 0..width {
            tiles[x] = Tile::Wall;                         // top row
            tiles[x + (height - 1) * width] = Tile::Wall;  // bottom row
        }

        for y in 0..height {
            tiles[y * width] = Tile::Wall;                 // left column
            tiles[y * width + (width - 1)] = Tile::Wall;   // right column
        }

        Self { width, height, tiles }
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 {
            return None;
        }
        let (x, y) = (x as usize, y as usize);
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(y * self.width + x)
        }
    }

    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        if let Some(idx) = self.index(x, y) {
            self.tiles[idx] == Tile::Floor
        } else {
            false
        }
    }


    pub fn from_ascii(ascii: &str) -> (Self, (i32, i32), Vec<Monster>, Vec<Npc>) {
        let lines: Vec<&str> = ascii
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect();

        let height = lines.len();
        let width = lines.first().map(|l| l.chars().count()).unwrap_or(0);

        let mut tiles = vec![Tile::Floor; width * height];
        let mut player_pos = (1_i32, 1_i32);
        let mut monsters = Vec::new();
        let mut npcs = Vec::new();

        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let idx = y * width + x;
                let (x_i32, y_i32) = (x as i32, y as i32);

                match ch {
                    '#' => {
                        tiles[idx] = Tile::Wall;
                    }
                    'P' => {
                        tiles[idx] = Tile::Floor;
                        player_pos = (x_i32, y_i32);
                    }
                    'D' => {
                        tiles[idx] = Tile::Floor;
                        monsters.push(Monster::new(x_i32, y_i32, "Monster", 20));
                    }
                    'N' => {
                        tiles[idx] = Tile::Floor;
                        npcs.push(Npc::new(x_i32, y_i32));
                    }
                    '.' | ' ' => {
                        tiles[idx] = Tile::Floor;
                    }
                    _ => {
                        tiles[idx] = Tile::Floor;
                    }
                }
            }
        }

        let map = Map {
            width,
            height,
            tiles,
        };

        (map, player_pos, monsters, npcs)
    }



    pub fn draw(&self, player: &Player, monsters: &[Monster], npcs: &[Npc]) {
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                if player.x == x && player.y == y {
                    print!("P"); // player
                } else if monsters.iter().any(|m| m.x == x && m.y == y) {
                    print!("D"); // monster
                } else if npcs.iter().any(|n| n.x == x && n.y == y) {
                    print!("N"); // npcs
                } else {
                    let ch = match self.tiles[self.index(x, y).unwrap()] {
                        Tile::Floor => '.',
                        Tile::Wall => '#',
                    };
                    print!("{ch}");
                }
            }
            println!();
        }
    }
}