#[derive(Debug)]
pub enum ControlMessage {
    Ok,
    Election,
    Coordinator,
    Ping,
    Pong,
}

impl ControlMessage {
    pub fn to_bytes(&self, id: u32) -> Vec<u8> {
        let opcode = match self {
            ControlMessage::Ok => b'O',
            ControlMessage::Election => b'E',
            ControlMessage::Coordinator => b'C',
            ControlMessage::Ping => b'P',
            ControlMessage::Pong => b'p',
        };

        let mut result = vec![opcode];
        result.extend(id.to_le_bytes());
        result
    }

    pub fn from_bytes(data: &[u8]) -> (ControlMessage, u32) {
        let opcode = match data[0] {
            b'O' => ControlMessage::Ok,
            b'E' => ControlMessage::Election,
            b'C' => ControlMessage::Coordinator,
            b'P' => ControlMessage::Ping,
            b'p' => ControlMessage::Pong,
            e => panic!("Invalid opcode: {:?}", e),
        };

        let id = u32::from_le_bytes(data[1..].try_into().unwrap());
        (opcode, id)
    }

    pub const fn size_of() -> usize {
        5
    }
}
