//! TCP game server

use std::io;
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};
use std::thread::sleep;

use crate::engine::{Game, ServerMessage};
use crate::error::ServerResult;
use crate::protocol::{Protocol, ClientCodec, ServerCodec};

use super::session::Session;

const DEFAULT_PORT: u16 = 7171;
const TICK_RATE_MS: u64 = 100;

pub struct Server<P: Protocol> {
    game: Game,
    listener: TcpListener,
    sessions: Vec<Session<P>>,
}

impl<P: Protocol> Server<P> {
    pub fn bind(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;

        println!(
            "[SERVER] Listening on {} (protocol v{})",
            addr,
            P::version()
        );

        Ok(Self {
            game: Game::new(),
            listener,
            sessions: Vec::new(),
        })
    }

    pub fn bind_default() -> io::Result<Self> {
        Self::bind(&format!("127.0.0.1:{}", DEFAULT_PORT))
    }

    pub fn run(&mut self) -> ServerResult<()> {
        let tick_duration = Duration::from_millis(TICK_RATE_MS);
        let mut last_tick = Instant::now();

        loop {
            // Accept new connections
            self.accept_connections();

            // Process client messages
            self.process_clients();

            // Game tick
            let now = Instant::now();
            if now.duration_since(last_tick) >= tick_duration {
                self.game.tick();
                self.broadcast_events();
                last_tick = now;
            }

            // Don't spin too fast
            sleep(Duration::from_millis(1));
        }
    }

    fn accept_connections(&mut self) {
        loop {
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    println!("[SERVER] New connection from {}", addr);
                    if let Err(e) = stream.set_nonblocking(true) {
                        eprintln!("[SERVER] Failed to set non-blocking: {}", e);
                        continue;
                    }
                    self.sessions.push(Session::new(stream));
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => eprintln!("[SERVER] Accept error: {}", e),
            }
        }
    }

    fn process_clients(&mut self) {
        let mut to_remove = Vec::new();

        for (i, session) in self.sessions.iter_mut().enumerate() {
            match session.protocol.read_message(&mut session.stream) {
                Ok(Some(msg)) => {
                    println!("[SERVER] Received: {:?}", msg);
                    let responses = self.game.handle_client_message(msg);

                    for response in responses {
                        if let ServerMessage::LoginOk { player_id } = &response {
                            session.player_id = Some(*player_id);
                        }

                        if let Err(e) = session.protocol.write_message(&mut session.stream, &response) {
                            eprintln!("[SERVER] Write error: {}", e);
                            to_remove.push(i);
                        }
                    }
                }
                Ok(None) => {
                    // No message (internal packet or would-block)
                }
                Err(e) => {
                    if !matches!(
                        e,
                        crate::error::ProtocolError::Io(ref io_err)
                            if io_err.kind() == io::ErrorKind::WouldBlock
                    ) {
                        eprintln!("[SERVER] Client error: {}", e);
                        to_remove.push(i);
                    }
                }
            }
        }

        // Remove disconnected clients (in reverse to preserve indices)
        for i in to_remove.into_iter().rev() {
            if let Some(player_id) = self.sessions[i].player_id {
                self.game.remove_player(player_id);
            }
            self.sessions.remove(i);
        }
    }

    fn broadcast_events(&mut self) {
        let events = self.game.drain_events();

        for event in events {
            // Convert events to server messages and broadcast
            // For now, just log them
            println!("[SERVER] Event: {:?}", event);
        }
    }
}