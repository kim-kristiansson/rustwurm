use std::io::{self, Write};

use rustwurm::{Game, PlayerCommand};

fn main() {
    let mut game = Game::new();

    println!("Use WASD to move, 'k' to attack, 'q' to quit.");

    loop{
        print!("\x1B[2J\x1B[1;1H");

        game.draw();

        if game.is_player_dead(0) {
            println!();
            println!("You died!");
            break;
        }

        print!("Command (w/a/s/d/k/q): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err(){
            break;
        }

        let cmd_char = input.chars().next().unwrap_or('\n');

        let cmd = match cmd_char {
            'w' | 'W' => Some(PlayerCommand::Move(0, 0, -1)),
            's' | 'S' => Some(PlayerCommand::Move(0, 0, 1)),
            'a' | 'A' => Some(PlayerCommand::Move(0, -1, 0)),
            'd' | 'D' => Some(PlayerCommand::Move(0, 1, 0)),
            'k' | 'K' => Some(PlayerCommand::Attack(0)),
            'q' | 'Q' => break,
            _ => None,
        };

        game.tick(cmd);
    }
}
