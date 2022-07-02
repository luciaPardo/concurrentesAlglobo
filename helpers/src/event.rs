use actix::Message;




#[derive(Eq, PartialEq, Debug, Message)]
#[rtype(result = "()")]
pub enum Event {
    TxSuccess { entity: u8, duration_ms: u32 },
    TxFailure { entity: u8, reason: String },
    PaymentSuccess { duration: u32 },
    PaymentFailed { reason: String },
}

impl Event {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::TxSuccess {
                entity,
                duration_ms,
            } => {
                let mut result = vec![b'S', *entity];
                result.extend_from_slice(&u32::to_le_bytes(*duration_ms));
                result
            }
            Self::TxFailure { entity, reason } => {
                let mut result = vec![b'F', *entity];
                let reason_bytes = reason.as_bytes();
                result.extend(reason_bytes.iter());
                result
            }
            Self::PaymentSuccess { duration } => {
                let mut result = vec![b'P'];
                result.extend_from_slice(&u32::to_le_bytes(*duration));
                result
            }
            Self::PaymentFailed { reason } => {
                let mut result = vec![b'X'];
                let reason_bytes = reason.as_bytes();
                result.extend(reason_bytes.iter());
                result
            }
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        match bytes[0] {
            b'S' => Event::TxSuccess {
                entity: bytes[1],
                duration_ms: u32::from_le_bytes(bytes[2..].try_into().unwrap()),
            },

            b'F' => Event::TxFailure {
                entity: bytes[1],
                reason: String::from_utf8_lossy(&bytes[2..]).into(),
            },
            b'P' => Event::PaymentSuccess {
                duration: u32::from_le_bytes(bytes[1..].try_into().unwrap()),
            },
            b'X' => Event::PaymentFailed {
                reason: String::from_utf8_lossy(&bytes[1..]).into(),
            },
            _ => panic!("Invalid Event message: {:?}", bytes),
        }
    }
}
