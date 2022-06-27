use crate::TransactionMessage;
use crate::alglobo_transaction::Pago;
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
        self.send(TransactionMessage::Commit { transaction_id })
            .await;
    }

    pub async fn prepare(&mut self, transaction: &Pago) -> bool {
        self.send(TransactionMessage::Prepare { 
            transaction:transaction.clone()
        })
        .await;
        match self.receive().await {
            Some(TransactionMessage::Response { success }) => success,
            res => panic!("Invalid prepare response: {:?}", res),
        }
    }

    pub async fn abort(&mut self, transaction_id: u32) {
        self.send(TransactionMessage::Abort { transaction_id })
            .await;
    }

    async fn send(&mut self, msg: TransactionMessage) {
        let payload = msg.to_bytes();
        let sz = payload.len() as u32;
        self.stream.write_all(&sz.to_le_bytes()).await.unwrap();
        self.stream.write_all(&payload).await.unwrap();
    }

    pub async fn receive(&mut self) -> Option<TransactionMessage> {
        let mut sz = [0u8; 4];
        if self.stream.read_exact(&mut sz).await.is_ok() {
            let mut buf = vec![0u8; u32::from_le_bytes(sz) as usize];
            self.stream.read_exact(&mut buf).await.unwrap();
            let message = TransactionMessage::from_bytes(&buf);
            Some(message)
        } else {
            None
        }
    }
    pub async fn send_ok(&mut self) {
        self.send(TransactionMessage::Response { success: true })
            .await;
    }

    pub async fn send_failure(&mut self) {
        self.send(TransactionMessage::Response { success: false })
            .await;
    }
}
