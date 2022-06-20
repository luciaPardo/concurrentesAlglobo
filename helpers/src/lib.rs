extern crate actix;

use actix::Message;

#[derive(Eq, PartialEq, Debug, Message)]
#[rtype(result = "Result<bool, std::io::Error>")]
pub enum TransactionMessage {
    Prepare { transaction_id: u32, client: String },
    Abort { transaction_id: u32 },
    Commit { transaction_id: u32 },
}

impl TransactionMessage {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            TransactionMessage::Prepare {
                transaction_id,
                client,
            } => {
                let mut result = vec![b'P'];
                result.extend_from_slice(&u32::to_le_bytes(*transaction_id));
                let client_bytes = client.as_bytes();
                result.extend(client_bytes.iter());
                result
            }
            TransactionMessage::Abort { transaction_id } => {
                let mut result = vec![b'A'];
                result.extend_from_slice(&u32::to_le_bytes(*transaction_id));
                result
            }
            TransactionMessage::Commit { transaction_id } => {
                let mut result = vec![b'C'];
                result.extend_from_slice(&u32::to_le_bytes(*transaction_id));
                result
            }
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        match bytes[0] {
            b'P' => TransactionMessage::Prepare {
                transaction_id: u32::from_le_bytes(bytes[1..5].try_into().unwrap()),
                client: String::from_utf8_lossy(&bytes[5..]).into(),
            },
            b'A' => TransactionMessage::Abort {
                transaction_id: u32::from_le_bytes(bytes[1..].try_into().unwrap()),
            },
            b'C' => TransactionMessage::Commit {
                transaction_id: u32::from_le_bytes(bytes[1..].try_into().unwrap()),
            },
            _ => panic!("Invalid transaction message: {:?}", bytes),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let msg = TransactionMessage::Prepare {
            transaction_id: 1234,
            client: "test-client".into(),
        };

        assert_eq!(TransactionMessage::from_bytes(&msg.to_bytes()), msg);

        let msg = TransactionMessage::Commit {
            transaction_id: 99999,
        };
        assert_eq!(TransactionMessage::from_bytes(&msg.to_bytes()), msg);

        let msg = TransactionMessage::Abort {
            transaction_id: 1234556,
        };
        assert_eq!(TransactionMessage::from_bytes(&msg.to_bytes()), msg);
    }
}
