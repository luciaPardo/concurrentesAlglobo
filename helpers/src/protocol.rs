use crate::TransactionMessage;
use std::string;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct Protocol {
    stream: TcpStream,
}

impl Protocol {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
    pub async fn commit(&mut self, transaction_id: u32) {
        let msg = TransactionMessage::Commit { transaction_id };
        let message = TransactionMessage::to_bytes(&msg);
        self.stream.write(&message).await.unwrap();
    }

    pub async fn prepare(&mut self, transaction_id: u32, client: String) -> bool {
        let msg = TransactionMessage::Prepare {
            transaction_id,
            client,
        };
        let message = TransactionMessage::to_bytes(&msg);
        self.stream.write(&message).await.unwrap();
        let mut data = [0 as u8; 6];
        self.stream.read_exact(&mut data).await.unwrap();
        match TransactionMessage::from_bytes(&data) {
            TransactionMessage::Response { success } => return success,
            res => panic!("Invalid prepare response: {:?}", res),
        }
    }

    pub async fn abort(&mut self, transaction_id: u32) {
        let msg = TransactionMessage::Abort { transaction_id };
        let message = TransactionMessage::to_bytes(&msg);
        self.stream.write(&message).await.unwrap();
    }

    pub async fn receive(&mut self) -> Option<TransactionMessage> {
        let mut sz = [0u8; 4];
        if self.stream.read_exact(&mut sz).await.is_ok() {
            let mut buf = vec![0u8; u32::from_le_bytes(sz) as usize];
            self.stream.read_exact(&mut buf).await.unwrap();
            let message = TransactionMessage::from_bytes(&buf);
            return Some(message);
        } else {
            None
        }
    }
}
