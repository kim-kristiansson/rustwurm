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

        for x in 0..width {
            tiles[x] = Tile::Wall;                               // top row
            tiles[x + (height - 1) * width] = Tile::Wall;        // bottom row
        }

        for y in 0..height {
            tiles[y * width] = Tile::Wall;                       // left column
            tiles[y * width + (width - 1)] = Tile::Wall;         // right column
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

    fn draw(&self, player: &Player) {
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                if player.x == x && player.y == y {
                    print!("@"); // player
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

fn main() {
    let mut map = Map::new(20, 10);
    let mut player = Player::new(5, 5);

    println!("Use WASD to move, q to quit.");

    loop {
        print!("\x1B[2J\x1B[1;1H");

        map.draw(&player);

        print!("Command (w/a/s/d/q): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let cmd = input.chars().next().unwrap_or('\n');

        match cmd {
            'w' => player.try_move(0, -1, &map),
            's' => player.try_move(0, 1, &map),
            'a' => player.try_move(-1, 0, &map),
            'd' => player.try_move(1, 0, &map),
            'q' => break,
            _ => {}
        }
    }
}
