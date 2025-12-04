use std::collections::HashMap;

use crate::world::{Map, MapSpawns, Position};
use super::player::{Player, PlayerId};
use super::monster::{Monster, MonsterId};
use super::npc::Npc;
use super::messages::{ClientMessage, ServerMessage};

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
        let spawn_pos = Position::ground(
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
            ClientMessage::AttackTarget { player_id, target_id } => {
                self.player_attack_target(player_id, target_id);
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
            ClientMessage::Turn { .. } => {
                // TODO: Implement turning
            }
            ClientMessage::UseItem { .. } => {
                // TODO: Implement item usage
            }
            ClientMessage::Cancel { .. } => {
                // TODO: Implement action cancellation
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
        let moved = {
            let Some(player) = self.players.get_mut(&player_id) else {
                return;
            };
            if player.try_move(dx, dy, &self.map) {
                Some((player.pos.x, player.pos.y))
            } else {
                None
            }
        };

        if let Some((x, y)) = moved {
            self.emit(GameEvent::PlayerMoved { player_id, x, y });
        }
    }

    fn player_attack(&mut self, player_id: PlayerId) {
        // Get player position
        let player_pos = match self.players.get(&player_id) {
            Some(p) => p.pos,
            None => return,
        };

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

        self.attack_monster(player_id, monster_id);
    }

    fn player_attack_target(&mut self, player_id: PlayerId, target_id: u32) {
        // Verify player exists and monster exists and is adjacent
        let player_pos = match self.players.get(&player_id) {
            Some(p) => p.pos,
            None => return,
        };

        let monster = match self.monsters.get(&target_id) {
            Some(m) if player_pos.is_adjacent(&m.pos) => m,
            _ => {
                self.emit(GameEvent::Message {
                    text: "Target is out of range.".to_string()
                });
                return;
            }
        };

        // Monster exists and is adjacent
        drop(monster);
        self.attack_monster(player_id, target_id);
    }

    fn attack_monster(&mut self, player_id: PlayerId, monster_id: MonsterId) {
        let damage = 10;
        let (is_dead, xp) = {
            let monster = self.monsters.get_mut(&monster_id).unwrap();
            monster.take_damage(damage);
            (monster.is_dead(), monster.xp_reward)
        };

        self.emit(GameEvent::PlayerAttacked {
            player_id,
            target_id: monster_id,
            damage,
        });

        if is_dead {
            self.monsters.remove(&monster_id);

            let leveled_up = {
                match self.players.get_mut(&player_id) {
                    Some(player) => Some((player.gain_xp(xp), player.level)),
                    None => None,
                }
            };

            self.emit(GameEvent::MonsterKilled {
                player_id,
                monster_id,
                xp_gained: xp,
            });

            if let Some((true, new_level)) = leveled_up {
                self.emit(GameEvent::PlayerLevelUp {
                    player_id,
                    new_level,
                });
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
                if self.map.is_walkable(candidate)
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
                    if self.map.is_walkable(candidate)
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
            let (is_dead, monster_name) = {
                let monster_name = self.monsters.get(&monster_id)
                    .map(|m| m.name.clone())
                    .unwrap_or_else(|| "Monster".to_string());

                let is_dead = if let Some(player) = self.players.get_mut(&player_id) {
                    player.take_damage(damage);
                    player.is_dead()
                } else {
                    continue;
                };

                (is_dead, monster_name)
            };

            self.emit(GameEvent::PlayerDamaged {
                player_id,
                damage,
                source: monster_name,
            });

            if is_dead {
                self.emit(GameEvent::PlayerDied { player_id });
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