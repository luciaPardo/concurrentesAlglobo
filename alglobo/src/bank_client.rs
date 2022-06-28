use helpers::{alglobo_transaction::AlgloboTransaction, protocol::Protocol};
use tokio::net::TcpStream;

pub struct BankClient {
    protocol: Protocol,
}

impl BankClient {
    pub async fn new() -> Self {
        Self {
            protocol: Protocol::new(TcpStream::connect("0.0.0.0:9997").await.unwrap()),
        }
    }

    pub async fn create_transaction(&mut self, transaction: &AlgloboTransaction) -> bool {
        self.protocol.prepare(transaction).await
    }

    pub async fn commit(&mut self, transaction_id: u32) {
        self.protocol.commit(transaction_id).await;
    }

    pub async fn abort(&mut self, transaction_id: u32) {
        self.protocol.abort(transaction_id).await;
    }
}
