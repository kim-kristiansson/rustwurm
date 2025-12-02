use std::io::{Read, Write};
use crate::engine::{EngineClientMsg, EngineServerMsg};

pub mod v1_03;

pub trait WireProtocol {
    fn read_client_msg(&mut self, reader: &mut dyn Read)
        -> std::io::Result<Option<EngineClientMsg>>;

    fn write_server_msg(&mut self, writer: &mut dyn Write, msg: &EngineServerMsg)
        -> std::io::Result<()>;
}