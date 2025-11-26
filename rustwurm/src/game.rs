use std::io::{self, Write};

use crate::map::Map;
use crate::entities::{Player, Monster, Npc};

pub struct Game {
    map: Map,
    player: Player,
    monsters: Vec<Monster>,
    npcs: Vec<Npc>,
    last_message: String,
}

impl Game {
    pub fn new() -> Self {
        let map = Map::new(30, 12);
        let player = Player::new(5, 5);

        let monsters = vec![
            Monster::new(10, 5, "Rat", 5),
            Monster::new(15, 8, "Orc", 20),
            Monster::new(20, 3, "Dragon", 100),
        ];

        let npcs = vec![
            Npc::new(25, 8),
            Npc::new(7, 10),
        ];

        Self {
            map,
            player,
            monsters,
            npcs,
            last_message: String::from("Welcome to Rustwurm!"),
        }
    }

    pub fn draw(&self) {
        self.map.draw(&self.player, &self.monsters, &self.npcs);

        println!();
        println!("HP: {}   Level: {}   XP: {}", self.player.hp, self.player.lvl, self.player.xp);
        println!("Last: {}", self.last_message);
    }

    fn handle_input(&mut self, cmd: char) -> bool {
        match cmd {
            'w' | 'W' => self.player.try_move(0, -1, &self.map),
            's' | 'S' => self.player.try_move(0, 1, &self.map),
            'a' | 'A' => self.player.try_move(-1, 0, &self.map),
            'd' | 'D' => self.player.try_move(1, 0, &self.map),
            'k' | 'K' => self.attack(),
            'q' | 'Q' => return false,
            _ => {}
        }

        true
    }

    fn attack(&mut self) {
        let directions = [
            (0, -1), (1, -1),
            (-1, 0), (1, 0),
            (-1, 1), (0, 1),
            (1, 1)
        ];
        let dmg = 10;

        if let Some(index) = self.monsters.iter().position(|m| {
            directions
                .iter()
                .any(|(dx, dy)| self.player.x + dx == m.x && self.player.y + dy == m.y)
        }) {
            let name = self.monsters[index].name;
            self.monsters[index].hp -= dmg;

            if self.monsters[index].hp <= 0 {
                let xp_gain = self.monsters[index].xp_reward;
                let name = self.monsters[index].name;
                self.player.xp += xp_gain;

                self.monsters.remove(index);
                self.last_message = format!("You killed the {} and gained {} XP!", name, xp_gain);

                self.check_lvl_up();
            } else {
                self.last_message = format!("You hit the {} for {} damage.", name, dmg);
            }
        } else {
            self.last_message = "You swing at the air.".to_string();
        }
    }

    fn update_monsters(&mut self) {
        for monster in &mut self.monsters {
            let dx = self.player.x - monster.x;
            let dy = self.player.y - monster.y;

            let step_x = dx.signum();
            let step_y = dy.signum();

            if step_x != 0 {
                let new_x = monster.x + step_x;
                if self.map.is_walkable(new_x, monster.y)
                    && !(new_x == self.player.x && monster.y == self.player.y)
                {
                    monster.x = new_x;
                }
            } else if step_y != 0 {
                let new_y = monster.y + step_y;
                if self.map.is_walkable(monster.x, new_y)
                    && !(monster.x == self.player.x && new_y == self.player.y)
                {
                    monster.y = new_y;
                }
            }

            let dist_x = (self.player.x - monster.x).abs();
            let dist_y = (self.player.y - monster.y).abs();

            if dist_x <= 1 && dist_y <= 1 {
                let dmg = 5;
                self.player.hp -= dmg;
                self.last_message = format!("The {} hits you for {} damage.", monster.name, dmg);
            }
        }
    }

    fn check_lvl_up(&mut self) {
        let required_xp = self.player.lvl * 20;
        if self.player.xp >= required_xp {
            self.player.lvl += 1;
            self.player.hp = 100;
            self.last_message = format!(
                "You advanced to level {}!",
                self.player.lvl
            )
        }
    }

    pub fn run(&mut self) {
        println!("Use WASD to move, 'k' to attack, 'q' to quit.");

        loop {
            print!("\x1B[2J\x1B[1;1H");

            self.draw();

            if self.player.hp <= 0 {
                println!();
                println!("You died!");
                break;
            }

            print!("Command (w/a/s/d/k/q): ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }

            let cmd = input.chars().next().unwrap_or('\n');

            if !self.handle_input(cmd) {
                break;
            }

            self.update_monsters();
        }
    }
}
