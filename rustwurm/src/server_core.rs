use std::io;
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};
use std::thread::sleep;

use crate::Game;
use crate::WireProtocol;
use crate::engine::{EngineClientMsg, EngineServerMsg};

pub fn handle_client<P: WireProtocol + Default>(
    mut stream: TcpStream,
    game: &mut Game
) -> io::Result<()> {
    let mut proto = P::default();

    if let Some(msg) = proto.read_client_msg(&mut stream)? {
        match msg {
            EngineClientMsg::Move { player_id, dx, dy } => {
                // TODO: Translate to PlayerCommand and tick the game.
                println!(
                    "[SERVER] Move from player {} to dx: {}, dy: {}\n",
                    player_id, dx, dy
                );
            }
            EngineClientMsg::Attack { player_id} => {
                println!("[SERVER] Attack player from {}", player_id);
            }
            other => {
                println!("[SERVER] Unhandled client msg: {:?}", other);
            }
        }
    }

    let reply = EngineServerMsg::PlayerStats {
        player_id: 0,
        hp: game.player_hp(0),
        lvl: 1,
        xp: 0
    };

    proto.write_server_msg(&mut stream, &reply)?;

    Ok(())
}

pub fn run_server<P: WireProtocol + Default>() -> io::Result<()> {
    let mut game = Game::new();

    let listener = TcpListener::bind("127.0.0.1:7171")?;
    listener.set_nonblocking(true)?;

    println!("Server listening on port 7171");

    let tick_duration = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        match listener.accept() {
            Ok((stream, addr)) => {
                println!("[SERVER] New connection from {}", addr);
                let _ = handle_client::<P>(stream, &mut game);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => eprintln!("[SERVER] Accept error: {}", e),
        }

        let now= Instant::now();
        if now.duration_since(last_tick) > tick_duration {
            game.tick(None);
            last_tick = now;
        }

        sleep(Duration::from_millis(1));
    }
}