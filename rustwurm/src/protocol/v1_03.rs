use std::io::{self, Read, Write};

use crate::engine::{EngineClientMsg, EngineServerMsg};
use crate::protocol::WireProtocol;

#[derive(Debug)]
struct RawPacket {
    opcode: u16,
    payload: Vec<u8>,
}

impl RawPacket {
    fn read_from(reader: &mut dyn Read) -> io::Result<Self> {
        // [u16_le length][u16_le opcode][payload...]
        let mut len_buf = [0u8; 2];
        reader.read_exact(&mut len_buf)?;
        let length = u16::from_le_bytes(len_buf) as usize;

        if length < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("length too small: {}", length),
            ));
        }

        let body_len = length - 2;
        let mut body = vec![0u8; body_len];
        reader.read_exact(&mut body)?;

        let opcode = u16::from_le_bytes([body[0], body[1]]);
        let payload = body[2..].to_vec();

        Ok(Self { opcode, payload })
    }

    fn write_to(&self, writer: &mut dyn Write) -> io::Result<()> {
        let body_len = 2 + self.payload.len();
        let total_len = 2 + body_len;

        let len_bytes = (total_len as u16).to_le_bytes();
        let opcode_bytes = self.opcode.to_le_bytes();

        writer.write_all(&len_bytes)?;
        writer.write_all(&opcode_bytes)?;
        writer.write_all(&self.payload)?;
        writer.flush()
    }
}

pub struct Protocol {
    // per-connection state later (keys, etc.)
}

impl Protocol {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::new()
    }
}

impl WireProtocol for Protocol {
    fn read_client_msg(
        &mut self,
        reader: &mut dyn Read,
    ) -> io::Result<Option<EngineClientMsg>> {
        let packet = RawPacket::read_from(reader)?;

        // TEMP: just log and return no EngineClientMsg so things compile
        println!(
            "[Tibia1] recv opcode={:#06x}, payload_len={}",
            packet.opcode,
            packet.payload.len()
        );

        Ok(None)
    }

    fn write_server_msg(
        &mut self,
        writer: &mut dyn Write,
        msg: &EngineServerMsg,
    ) -> io::Result<()> {
        // TEMP: do nothing, just log and pretend success
        println!("[Tibia1] send msg: {:?}", msg);


        Ok(())
    }
}
