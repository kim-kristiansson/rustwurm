//! Rustwurm game server
//!
//! Uses the protocol selected at compile time via Cargo features.

use rustwurm::{Server, SelectedProtocol};

fn main() {
    println!("Starting Rustwurm server...");

    let mut server = match Server::<SelectedProtocol>::bind_default() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to start server: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = server.run() {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}