use std::io::{self, Write};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Tile {
    Floor,
    Wall,
}

struct Map {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
}

impl Map {
    fn new(width: usize, height: usize) -> Self {
        let mut tiles = vec![Tile::Floor; width * height];

        // Vägg runt kanten
        for x in 0..width {
            tiles[x] = Tile::Wall;                         // översta raden
            tiles[x + (height - 1) * width] = Tile::Wall;  // nedersta raden
        }

        for y in 0..height {
            tiles[y * width] = Tile::Wall;                 // vänster kolumn
            tiles[y * width + (width - 1)] = Tile::Wall;   // höger kolumn
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

    fn is_walkable(&self, x: i32, y: i32) -> bool {
        if let Some(idx) = self.index(x, y) {
            self.tiles[idx] == Tile::Floor
        } else {
            false
        }
    }

    fn draw(&self, player: &Player, monsters: &[Monster], npcs: &[Npc]) {
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                if player.x == x && player.y == y {
                    print!("P"); // spelaren
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

struct Player {
    x: i32,
    y: i32,
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn try_move(&mut self, dx: i32, dy: i32, map: &Map) {
        let new_x = self.x + dx;
        let new_y = self.y + dy;

        if map.is_walkable(new_x, new_y) {
            self.x = new_x;
            self.y = new_y;
        }
    }
}

struct Monster {
    x: i32,
    y: i32,
}

impl Monster {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

struct Npc {
    x: i32,
    y: i32,
}

impl Npc {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

struct Game {
    map: Map,
    player: Player,
    monsters: Vec<Monster>,
    npcs: Vec<Npc>,
}

impl Game {
    fn new() -> Self {
        let map = Map::new(30, 12);
        let player = Player::new(5, 5);

        let monsters = vec![
            Monster::new(10, 5),
            Monster::new(15, 8),
            Monster::new(20, 3)
        ];

        let npcs = vec![
            Npc::new(25, 8),
            Npc::new(7, 10),
        ];

        Self{
            map,
            player,
            monsters,
            npcs
        }
    }

    fn draw(&self) {
        self.map.draw(&self.player, &self.monsters, &self.npcs);
    }

    fn handle_input(&mut self, cmd: char) -> bool {
        match cmd {
            'w' | 'W' => self.player.try_move(0, -1, &self.map),
            's' | 'S' => self.player.try_move(0, 1, &self.map),
            'a' | 'A' => self.player.try_move(-1, 0, &self.map),
            'd' | 'D' => self.player.try_move(1, 0, &self.map),
            'k' | 'K' => attack(&self.player, &mut self.monsters),
            'q' | 'Q' => return false,
            _ => {}
        }

        true
    }

    fn run(&mut self) {
        println!("Use WASD to move, 'k' to attack, 'q' to quit.");

        loop {
            print!("\x1B[2J\x1B[1;1H");

            self.draw();

            print!("Command (w/a/s/d/k/q): ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }

            let cmd = input.chars().next().unwrap_or('\n');

            if(!self.handle_input(cmd)) {
                break
            }
        }
    }
}

fn attack(player: &Player, monsters: &mut Vec<Monster>) {
    // slå åt ett håll runt spelaren – om något monster står där försvinner det
    let directions = [(0, -1), (0, 1), (-1, 0), (1, 0)];

    if let Some(index) = monsters.iter().position(|m| {
        directions
            .iter()
            .any(|(dx, dy)| player.x + dx == m.x && player.y + dy == m.y)
    }) {
        monsters.remove(index);
    }
}

fn main() {
    let mut game = Game::new();
    game.run();
}
