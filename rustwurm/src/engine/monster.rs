use crate::world::Position;

pub type MonsterId = u32;

#[derive(Debug)]
pub struct Monster {
    pub id: MonsterId,
    pub pos: Position,
    pub hp: i32,
    pub max_hp: i32,
    pub name: String,
    pub xp_reward: i32,
    pub damage: i32,
}

impl Monster {
    pub fn new(id: MonsterId, pos: Position, name: impl Into<String>) -> Self {
        Self {
            id,
            pos,
            hp: 30,
            max_hp: 30,
            name: name.into(),
            xp_reward: 20,
            damage: 5,
        }
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.hp = (self.hp - amount).max(0);
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }
}