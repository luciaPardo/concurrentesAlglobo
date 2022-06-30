use tokio::net::TcpStream;

use helpers::{alglobo_transaction::AlgloboTransaction, protocol::Protocol};

pub struct TransactionalEntity {
    protocol: Protocol,
}

impl TransactionalEntity {
    pub async fn new(host: &str) -> Self {
        Self {
            protocol: Protocol::new(TcpStream::connect(host).await.unwrap()),
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
