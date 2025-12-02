use std::collections::HashMap;

use crate::world::{Map, MapSpawns, Position};
use super::player::{Player, PlayerId};
use super::monster::{Monster, MonsterId, Npc};
use super::messages::{ClientMessage, ServerMessage};

/// Direction offsets for adjacent tile checks
const ADJACENT_OFFSETS: [(i32, i32); 8] = [
    (-1, -1), (0, -1), (1, -1),
    (-1,  0),          (1,  0),
    (-1,  1), (0,  1), (1,  1),
];

/// Commands that can be issued to the game engine (for local play)
#[derive(Debug, Clone)]
pub enum GameCommand {
    Move(PlayerId, i32, i32),
    Attack(PlayerId),
}

/// Events emitted by the game engine
#[derive(Debug, Clone)]
pub enum GameEvent {
    PlayerMoved { player_id: PlayerId, x: i32, y: i32 },
    PlayerAttacked { player_id: PlayerId, target_id: MonsterId, damage: i32 },
    MonsterKilled { player_id: PlayerId, monster_id: MonsterId, xp_gained: i32 },
    PlayerDamaged { player_id: PlayerId, damage: i32, source: String },
    PlayerLevelUp { player_id: PlayerId, new_level: i32 },
    PlayerDied { player_id: PlayerId },
    Message { text: String },
}

pub struct Game {
    map: Map,
    players: HashMap<PlayerId, Player>,
    monsters: HashMap<MonsterId, Monster>,
    npcs: Vec<Npc>,
    next_player_id: PlayerId,
    next_monster_id: MonsterId,
    events: Vec<GameEvent>,
}

impl Game {
    pub fn new() -> Self {
        const START_MAP: &str = include_str!("../maps/start.map");

        let (map, spawns) = Map::from_ascii(START_MAP);
        let mut game = Self {
            map,
            players: HashMap::new(),
            monsters: HashMap::new(),
            npcs: Vec::new(),
            next_player_id: 0,
            next_monster_id: 0,
            events: Vec::new(),
        };

        game.spawn_from_map(spawns);
        game
    }

    fn spawn_from_map(&mut self, spawns: MapSpawns) {
        // Spawn monsters
        for pos in spawns.monsters {
            let id = self.next_monster_id;
            self.next_monster_id += 1;
            self.monsters.insert(id, Monster::new(id, pos, "Monster"));
        }

        // Spawn NPCs
        for pos in spawns.npcs {
            self.npcs.push(Npc::new(pos, "Villager"));
        }
    }

    /// Add a player to the game (returns their ID)
    pub fn add_player(&mut self, name: String) -> PlayerId {
        let id = self.next_player_id;
        self.next_player_id += 1;

        // Find spawn position (simple: use map center or first walkable)
        let spawn_pos = Position::new(
            (self.map.width / 2) as i32,
            (self.map.height / 2) as i32,
        );

        self.players.insert(id, Player::new(id, name, spawn_pos));
        self.emit(GameEvent::Message {
            text: "Welcome to Rustwurm!".to_string()
        });

        id
    }

    /// Remove a player from the game
    pub fn remove_player(&mut self, player_id: PlayerId) {
        self.players.remove(&player_id);
    }

    /// Process a client message and return any server responses
    pub fn handle_client_message(&mut self, msg: ClientMessage) -> Vec<ServerMessage> {
        match msg {
            ClientMessage::Move { player_id, dx, dy } => {
                self.move_player(player_id, dx, dy);
            }
            ClientMessage::Attack { player_id } => {
                self.player_attack(player_id);
            }
            ClientMessage::Login { name, .. } => {
                let id = self.add_player(name);
                return vec![ServerMessage::LoginOk { player_id: id }];
            }
            ClientMessage::Logout { player_id } => {
                self.remove_player(player_id);
            }
            ClientMessage::Say { message, .. } => {
                self.emit(GameEvent::Message { text: message });
            }
        }

        vec![]
    }

    /// Process a local game command
    pub fn handle_command(&mut self, cmd: GameCommand) {
        match cmd {
            GameCommand::Move(player_id, dx, dy) => {
                self.move_player(player_id, dx, dy);
            }
            GameCommand::Attack(player_id) => {
                self.player_attack(player_id);
            }
        }
    }

    fn move_player(&mut self, player_id: PlayerId, dx: i32, dy: i32) {
        if let Some(player) = self.players.get_mut(&player_id) {
            if player.try_move(dx, dy, &self.map) {
                self.emit(GameEvent::PlayerMoved {
                    player_id,
                    x: player.pos.x,
                    y: player.pos.y,
                });
            }
        }
    }

    fn player_attack(&mut self, player_id: PlayerId) {
        let Some(player) = self.players.get(&player_id) else {
            return;
        };
        let player_pos = player.pos;

        // Find adjacent monster
        let target = self.monsters.iter()
            .find(|(_, m)| player_pos.is_adjacent(&m.pos))
            .map(|(&id, _)| id);

        let Some(monster_id) = target else {
            self.emit(GameEvent::Message {
                text: "You swing at the air.".to_string()
            });
            return;
        };

        let damage = 10;
        let monster = self.monsters.get_mut(&monster_id).unwrap();
        monster.take_damage(damage);

        self.emit(GameEvent::PlayerAttacked {
            player_id,
            target_id: monster_id,
            damage,
        });

        if monster.is_dead() {
            let xp = monster.xp_reward;
            let name = monster.name.clone();
            self.monsters.remove(&monster_id);

            if let Some(player) = self.players.get_mut(&player_id) {
                let leveled_up = player.gain_xp(xp);

                self.emit(GameEvent::MonsterKilled {
                    player_id,
                    monster_id,
                    xp_gained: xp,
                });

                if leveled_up {
                    self.emit(GameEvent::PlayerLevelUp {
                        player_id,
                        new_level: player.level,
                    });
                }
            }
        }
    }

    /// Run one game tick (monster AI, etc.)
    pub fn tick(&mut self) {
        self.update_monsters();
    }

    fn update_monsters(&mut self) {
        if self.players.is_empty() {
            return;
        }

        // Collect monster actions first to avoid borrow issues
        let mut monster_moves: Vec<(MonsterId, Position)> = Vec::new();
        let mut monster_attacks: Vec<(MonsterId, PlayerId, i32)> = Vec::new();

        for (&monster_id, monster) in &self.monsters {
            // Find closest player
            let closest = self.players.values()
                .min_by_key(|p| p.pos.distance_squared(&monster.pos));

            let Some(target) = closest else { continue };

            // Check if adjacent - attack
            if monster.pos.is_adjacent(&target.pos) {
                monster_attacks.push((monster_id, target.id, monster.damage));
                continue;
            }

            // Move toward player
            let dx = (target.pos.x - monster.pos.x).signum();
            let dy = (target.pos.y - monster.pos.y).signum();

            let new_pos = if dx != 0 {
                let candidate = monster.pos.offset(dx, 0);
                if self.map.is_position_walkable(candidate)
                    && !self.players.values().any(|p| p.pos == candidate)
                {
                    Some(candidate)
                } else {
                    None
                }
            } else {
                None
            };

            let new_pos = new_pos.or_else(|| {
                if dy != 0 {
                    let candidate = monster.pos.offset(0, dy);
                    if self.map.is_position_walkable(candidate)
                        && !self.players.values().any(|p| p.pos == candidate)
                    {
                        Some(candidate)
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

            if let Some(pos) = new_pos {
                monster_moves.push((monster_id, pos));
            }
        }

        // Apply moves
        for (monster_id, new_pos) in monster_moves {
            if let Some(monster) = self.monsters.get_mut(&monster_id) {
                monster.pos = new_pos;
            }
        }

        // Apply attacks
        for (monster_id, player_id, damage) in monster_attacks {
            if let Some(player) = self.players.get_mut(&player_id) {
                player.take_damage(damage);

                let monster_name = self.monsters.get(&monster_id)
                    .map(|m| m.name.clone())
                    .unwrap_or_else(|| "Monster".to_string());

                self.emit(GameEvent::PlayerDamaged {
                    player_id,
                    damage,
                    source: monster_name,
                });

                if player.is_dead() {
                    self.emit(GameEvent::PlayerDied { player_id });
                }
            }
        }
    }

    fn emit(&mut self, event: GameEvent) {
        self.events.push(event);
    }

    /// Drain and return all pending events
    pub fn drain_events(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.events)
    }

    // Accessor methods for rendering/queries

    pub fn get_player(&self, id: PlayerId) -> Option<&Player> {
        self.players.get(&id)
    }

    pub fn players(&self) -> impl Iterator<Item = &Player> {
        self.players.values()
    }

    pub fn monsters(&self) -> impl Iterator<Item = &Monster> {
        self.monsters.values()
    }

    pub fn npcs(&self) -> &[Npc] {
        &self.npcs
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn is_player_dead(&self, player_id: PlayerId) -> bool {
        self.players.get(&player_id)
            .map(|p| p.is_dead())
            .unwrap_or(true)
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}