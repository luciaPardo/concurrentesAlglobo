use crate::alglobo_transaction::AlgloboTransaction;
use crate::TransactionMessage;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use std::io::Result;

pub struct Protocol {
    stream: TcpStream,
}

impl Protocol {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
    pub async fn commit(&mut self, transaction_id: u32) -> Result<bool> {
        self.send(TransactionMessage::Commit { transaction_id })
            .await?;
        Ok(self.read_response().await)
    }

    pub async fn prepare(&mut self, transaction: &AlgloboTransaction) -> Result<bool> {
        self.send(TransactionMessage::Prepare {
            transaction: transaction.clone(),
        })
        .await?;
        Ok(self.read_response().await)
    }

    async fn read_response(&mut self) -> bool {
        match self.receive().await {
            Some(TransactionMessage::Response { success }) => success,
            None => false,
            res => panic!("Invalid prepare response: {:?}", res),
        }
    }

    pub async fn abort(&mut self, transaction_id: u32) -> Result<bool> {
        self.send(TransactionMessage::Abort { transaction_id })
            .await?;
        Ok(self.read_response().await)
    }

    async fn send(&mut self, msg: TransactionMessage) -> Result<()> {
        let payload = msg.to_bytes();
        let sz = payload.len() as u32;
        let mut full_payload = Vec::new();
        full_payload.extend_from_slice(&sz.to_le_bytes());
        full_payload.extend(payload);
        self.stream.write_all(&full_payload).await
    }

    pub async fn receive(&mut self) -> Option<TransactionMessage> {
        let mut sz = [0u8; 4];
        if self.stream.read_exact(&mut sz).await.is_ok() {
            let mut buf = vec![0u8; u32::from_le_bytes(sz) as usize];
            if self.stream.read_exact(&mut buf).await.is_ok() {
                let message = TransactionMessage::from_bytes(&buf);
                Some(message)
            } else {
                None
            }
        } else {
            None
        }
    }
    pub async fn send_ok(&mut self) -> Result<()> {
        self.send(TransactionMessage::Response { success: true })
            .await
    }

    pub async fn send_failure(&mut self) -> Result<()> {
        self.send(TransactionMessage::Response { success: false })
            .await
    }
}
