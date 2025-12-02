use crate::world::{Map, Position};

pub type PlayerId = u32;

#[derive(Debug)]
pub struct Player {
    pub id: PlayerId,
    pub pos: Position,
    pub hp: i32,
    pub max_hp: i32,
    pub level: i32,
    pub xp: i32,
    pub name: String,
}

impl Player {
    pub fn new(id: PlayerId, name: String, pos: Position) -> Self {
        Self {
            id,
            pos,
            hp: 100,
            max_hp: 100,
            level: 1,
            xp: 0,
            name,
        }
    }

    pub fn try_move(&mut self, dx: i32, dy: i32, map: &Map) -> bool {
        let new_pos = self.pos.offset(dx, dy);

        if map.is_position_walkable(new_pos) {
            self.pos = new_pos;
            true
        } else {
            false
        }
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.hp = (self.hp - amount).max(0);
    }

    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    pub fn gain_xp(&mut self, amount: i32) -> bool {
        self.xp += amount;
        self.check_level_up()
    }

    fn xp_for_next_level(&self) -> i32 {
        self.level * 20
    }

    fn check_level_up(&mut self) -> bool {
        if self.xp >= self.xp_for_next_level() {
            self.level += 1;
            self.max_hp += 10;
            self.hp = self.max_hp;
            true
        } else {
            false
        }
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }
}