//! Local single-player mode

use std::io::{self, Write};
use rustwurm::{Game, GameCommand, GameEvent};

fn main() {
    let mut game = Game::new();
    let player_id = game.add_player("LocalPlayer".to_string());

    println!("=== Rustwurm ===");
    println!("Use WASD to move, K to attack, Q to quit.\n");

    loop {
        // Clear screen and draw
        print!("\x1B[2J\x1B[1;1H");
        draw_game(&game, player_id);

        // Check death
        if game.is_player_dead(player_id) {
            println!("\nYou died! Game over.");
            break;
        }

        // Process events
        for event in game.drain_events() {
            match event {
                GameEvent::Message { text } => println!(">> {}", text),
                GameEvent::PlayerLevelUp { new_level, .. } => {
                    println!(">> Level up! You are now level {}!", new_level);
                }
                GameEvent::MonsterKilled { xp_gained, .. } => {
                    println!(">> Monster killed! +{} XP", xp_gained);
                }
                GameEvent::PlayerDamaged { damage, source, .. } => {
                    println!(">> {} hit you for {} damage!", source, damage);
                }
                _ => {}
            }
        }

        // Get input
        print!("\nCommand: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let cmd = match input.trim().chars().next() {
            Some('w' | 'W') => Some(GameCommand::Move(player_id, 0, -1)),
            Some('s' | 'S') => Some(GameCommand::Move(player_id, 0, 1)),
            Some('a' | 'A') => Some(GameCommand::Move(player_id, -1, 0)),
            Some('d' | 'D') => Some(GameCommand::Move(player_id, 1, 0)),
            Some('k' | 'K') => Some(GameCommand::Attack(player_id)),
            Some('q' | 'Q') => break,
            _ => None,
        };

        if let Some(cmd) = cmd {
            game.handle_command(cmd);
        }

        game.tick();
    }
}

fn draw_game(game: &Game, player_id: u32) {
    let map = game.map();
    let player = game.get_player(player_id);

    for y in 0..map.height as i32 {
        for x in 0..map.width as i32 {
            let ch = if player.map_or(false, |p| p.pos.x == x && p.pos.y == y) {
                'P'
            } else if game.monsters().any(|m| m.pos.x == x && m.pos.y == y) {
                'D'
            } else if game.npcs().iter().any(|n| n.pos.x == x && n.pos.y == y) {
                'N'
            } else {
                use rustwurm::world::Position;
                match map.get_tile(Position::new(x, y)) {
                    Some(rustwurm::world::Tile::Floor) => '.',
                    Some(rustwurm::world::Tile::Wall) => '#',
                    None => '?',
                }
            };
            print!("{}", ch);
        }
        println!();
    }

    if let Some(p) = player {
        println!();
        println!("HP: {}/{}  Level: {}  XP: {}", p.hp, p.max_hp, p.level, p.xp);
    }
}