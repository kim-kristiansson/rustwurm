use std::time::{Duration, Instant};
use std::thread::sleep;

use rustwurm::Game;

fn main() {
    let mut game = Game::new();

    let tick_duration = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    println!("Starting Rustwurm server loop (no network yer)...");

    loop {
        let now = Instant::now();
        if now.duration_since(last_tick) > tick_duration {
            game.tick(None);
            last_tick = now;

            println!(
                "[SERVER] Tick. Player HP: {}   Monsters: {}",
                game.player_hp(),
                game.monster_count()
            );
        }

        sleep(Duration::from_millis(1));
    }
}