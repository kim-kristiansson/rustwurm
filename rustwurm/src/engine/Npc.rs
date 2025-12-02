use crate::world::Position;

#[derive(Debug)]
pub struct Npc {
    pub pos: Position,
    pub name: String,
}

impl Npc {
    pub fn new(pos: Position, name: impl Into<String>) -> Self {
        Self {
            pos,
            name: name.into(),
        }
    }
}