//! Client session management

use std::net::TcpStream;
use crate::engine::PlayerId;
use crate::protocol::Protocol;

/// Represents a connected client session
pub struct Session<P: Protocol> {
    pub stream: TcpStream,
    pub player_id: Option<PlayerId>,
    pub protocol: P,
}

impl<P: Protocol> Session<P> {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            player_id: None,
            protocol: P::default(),
        }
    }

    pub fn is_logged_in(&self) -> bool {
        self.player_id.is_some()
    }
}