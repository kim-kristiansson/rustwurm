//! Tibia 1.03 protocol codec implementation

use std::io::{Read, Write};

use crate::engine::{ClientMessage, ServerMessage, PlayerId};
use crate::error::{ProtocolError, ProtocolResult};
use crate::protocol::traits::{ClientCodec, ServerCodec, Protocol};

use super::packets::{
    RawPacket, PayloadReader, PayloadWriter,
    ClientOpcode, ServerOpcode,
};

pub struct Codec {
    // Future: encryption keys, connection state, etc.
}

impl Codec {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Codec {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientCodec for Codec {
    fn read_message(&mut self, reader: &mut dyn Read) -> ProtocolResult<Option<ClientMessage>> {
        let packet = RawPacket::read_from(reader)?;
        let mut payload = PayloadReader::new(&packet.payload);

        let Some(opcode) = ClientOpcode::from_u16(packet.opcode) else {
            // Unknown opcode - log and skip
            eprintln!(
                "[v1.03] Unknown client opcode: {:#06x}, payload_len={}",
                packet.opcode,
                packet.payload.len()
            );
            return Ok(None);
        };

        let msg = match opcode {
            ClientOpcode::Login => {
                let name = payload.read_string()?;
                let password = payload.read_string()?;
                ClientMessage::Login { name, password }
            }
            ClientOpcode::Logout => {
                let player_id = payload.read_u32()?;
                ClientMessage::Logout { player_id }
            }
            ClientOpcode::Move => {
                let player_id = payload.read_u32()?;
                let dx = payload.read_i32()?;
                let dy = payload.read_i32()?;
                ClientMessage::Move { player_id, dx, dy }
            }
            ClientOpcode::Attack => {
                let player_id = payload.read_u32()?;
                ClientMessage::Attack { player_id }
            }
        };

        Ok(Some(msg))
    }
}

impl ServerCodec for Codec {
    fn write_message(&mut self, writer: &mut dyn Write, msg: &ServerMessage) -> ProtocolResult<()> {
        let (opcode, payload) = match msg {
            ServerMessage::LoginOk { player_id } => {
                let mut w = PayloadWriter::new();
                w.write_u32(*player_id);
                (ServerOpcode::LoginOk as u16, w.finish())
            }
            ServerMessage::LoginFailed { reason } => {
                let mut w = PayloadWriter::new();
                w.write_string(reason);
                (ServerOpcode::LoginFailed as u16, w.finish())
            }
            ServerMessage::PlayerMoved { player_id, x, y } => {
                let mut w = PayloadWriter::new();
                w.write_u32(*player_id);
                w.write_i32(*x);
                w.write_i32(*y);
                (ServerOpcode::PlayerMoved as u16, w.finish())
            }
            ServerMessage::PlayerStats { player_id, hp, max_hp, level, xp } => {
                let mut w = PayloadWriter::new();
                w.write_u32(*player_id);
                w.write_i32(*hp);
                w.write_i32(*max_hp);
                w.write_i32(*level);
                w.write_i32(*xp);
                (ServerOpcode::PlayerStats as u16, w.finish())
            }
            ServerMessage::TextMessage { message } => {
                let mut w = PayloadWriter::new();
                w.write_string(message);
                (ServerOpcode::TextMessage as u16, w.finish())
            }
            // TODO: implement remaining message types
            _ => {
                eprintln!("[v1.03] Unimplemented server message: {:?}", msg);
                return Ok(());
            }
        };

        let packet = RawPacket::new(opcode, payload);
        packet.write_to(writer)
    }
}

impl Protocol for Codec {
    fn version() -> &'static str {
        "1.03"
    }
}