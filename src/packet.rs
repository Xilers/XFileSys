use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum MsgOpcode {
    Handshake = 0,
    PlainMsg = 1,
    Terminate = 2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MsgPacket {
    pub len: usize,
    pub id: String,
    // opcode: MsgOpcode,
    pub data: String,
}
impl MsgPacket {
    pub fn new(id: &str, data: &str) -> Self {
        MsgPacket {
            len: id.len() + data.len(),
            id: id.to_string(),
            // opcode: MsgOpcode::PlainMsg,
            data: data.to_string(),
        }
    }

    pub fn dummy() -> Self {
        MsgPacket {
            len: 0,
            id: String::new(),
            // opcode: MsgOpcode::PlainMsg,
            data: String::new(),
        }
    }

    pub fn parse(mut base: Self, data: String) -> Self {
        base.data = data;
        base.len = base.id.len() + base.data.len();
        base
    }
}
