use crate::map::Map;
use crate::entities::{Player, Monster, Npc};
use crate::engine::{PlayerId};

pub enum PlayerCommand {
    Move(PlayerId, i32, i32),
    Attack(PlayerId)
}

pub struct Game {
    map: Map,
    players: Vec<Player>,
    monsters: Vec<Monster>,
    npcs: Vec<Npc>,
    last_message: String,
}

impl Game {
    pub fn new() -> Self {
        const START_MAP: &str = include_str!("./maps/start.map");

        let (map, (px, py), monsters, npcs) = Map::from_ascii(START_MAP);
        let players = vec![Player::new(px, py)];

        Self {
            map,
            players,
            monsters,
            npcs,
            last_message: String::from("Welcome to Rustwurm!"),
        }
    }

    pub fn draw(&self) {
        let player = &self.players[0];
        self.map.draw(player, &self.monsters, &self.npcs);

        println!();
        println!("HP: {}   Level: {}   XP: {}", player.hp, player.lvl, player.xp);
        println!("Last: {}", self.last_message);
    }

    fn attack(&mut self, player_id: PlayerId) {
        let Some(player) = self.players.get(player_id as usize) else {
            return;
        };

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
                .any(|(dx, dy)| player.x + dx == m.x && player.y + dy == m.y)
        }) {
            let name = self.monsters[index].name;
            self.monsters[index].hp -= dmg;

            if self.monsters[index].hp <= 0 {
                let xp_gain = self.monsters[index].xp_reward;
                let name = self.monsters[index].name;

                if let Some(player_mut) = self.players.get_mut(player_id as usize) {
                    player_mut.xp += xp_gain;
                }

                self.monsters.remove(index);
                self.last_message = format!("You killed the {} and gained {} XP!", name, xp_gain);

                self.check_lvl_up(player_id);
            } else {
                self.last_message = format!("You hit the {} for {} damage.", name, dmg);
            }
        } else {
            self.last_message = "You swing at the air.".to_string();
        }
    }

    fn update_monsters(&mut self) {
        if self.players.is_empty() {
            return;
        }

        for monster in &mut self.monsters {
            let mut target_index: Option<usize> = None;
            let mut best_dist2: i32 = i32::MAX;

            for(i, player) in self.players.iter().enumerate() {
                let dx = player.x - monster.x;
                let dy = player.y - monster.y;
                let dist2 = dx * dx + dy * dy;
                if dist2 < best_dist2 {
                    best_dist2 = dist2;
                    target_index = Some(i);
                }
            }

            let Some(target_idx) = target_index else {
                continue;
            };

            let target = &self.players[target_idx];

            let dx = target.x - monster.x;
            let dy = target.y - monster.y;

            let step_x = dx.signum();
            let step_y = dy.signum();

            if step_x != 0 {
                let new_x = monster.x + step_x;
                if self.map.is_walkable(new_x, monster.y)
                    && !self.players.iter().any(|p| p.x == new_x && p.y == monster.y)
                {
                    monster.x = new_x;
                }
            } else if step_y != 0 {
                let new_y = monster.y + step_y;
                if self.map.is_walkable(monster.x, new_y)
                    && !self.players.iter().any(|p| p.x == monster.x && p.y == monster.y)
                {
                    monster.y = new_y;
                }
            }

            let target = &mut self.players[target_idx];
            let dist_x = (target.x - monster.x).abs();
            let dist_y = (target.y - monster.y).abs();

            if dist_x <= 1 && dist_y <= 1 {
                let dmg = 5;
                target.hp -= dmg;
                self.last_message = format!("The {} hits you for {} damage.", monster.name, dmg);
            }
        }
    }

    pub fn is_player_dead(&self, player_id: PlayerId) -> bool {
        self.players
            .get(player_id as usize)
            .map(|p| p.hp <= 0)
            .unwrap_or(true)
    }

    pub fn player_hp(&self, player_id: PlayerId) -> i32 {
        self.players
            .get(player_id as usize)
            .map(|p| p.hp)
            .unwrap_or(0)
    }

    pub fn monster_count(&self) -> i32 {
        self.monsters.len() as i32
    }

    fn apply_player_command(&mut self, cmd: PlayerCommand) {
        match cmd {
            PlayerCommand::Move(player_id, dx, dy) => {
                if let Some(player) = self.players.get_mut(player_id as usize) {
                    player.try_move(dx, dy, &self.map);
                }
            }
            PlayerCommand::Attack(player_id) => {
                self.attack(player_id);
            }
        }
    }

    fn check_lvl_up(&mut self, player_id: PlayerId) {
        if let Some(player) = self.players.get_mut(player_id as usize) {
            let required_xp = player.lvl * 20;
            if player.xp >= required_xp {
                player.lvl += 1;
                player.hp = 100;
                self.last_message = format!(
                    "You advanced to level {}!",
                    player.lvl
                )
            }
        }
    }

    pub fn tick(&mut self, cmd: Option<PlayerCommand>) {
        if let Some(cmd) = cmd {
            self.apply_player_command(cmd);
        }

        self.update_monsters();
    }
}
