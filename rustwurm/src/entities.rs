use crate::map::Map;

pub struct Player {
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub lvl: i32,
    pub xp: i32
}

impl Player {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            hp: 100,
            lvl: 1,
            xp: 0
        }
    }

    pub fn try_move(&mut self, dx: i32, dy: i32, map: &Map) {
        let new_x = self.x + dx;
        let new_y = self.y + dy;

        if map.is_walkable(new_x, new_y) {
            self.x = new_x;
            self.y = new_y;
        }
    }
}

pub struct Monster {
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub name: &'static str,
    pub xp_reward: i32
}

impl Monster {
    pub fn new(x: i32, y: i32, name: &'static str, xp_reward: i32) -> Self {
        Self {
            x,
            y,
            hp: 30,
            name,
            xp_reward
        }
    }
}

pub struct Npc {
    pub x: i32,
    pub y: i32,
}

impl Npc {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
