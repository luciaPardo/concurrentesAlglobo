use std::io::Result;
use tokio::net::TcpStream;

use helpers::{alglobo_transaction::AlgloboTransaction, protocol::Protocol};

pub struct TransactionalEntity {
    host: String,
    protocol: Protocol,
}

impl TransactionalEntity {
    pub async fn new(host: &str) -> Result<Self> {
        Ok(Self {
            host: host.into(),
            protocol: Protocol::new(TcpStream::connect(host).await?),
        })
    }

    pub async fn create_transaction(&mut self, transaction: &AlgloboTransaction) -> bool {
        if let Ok(success) = self.protocol.prepare(transaction).await {
            success
        } else {
            // Connection may have failed, try to reconnect to the entity
            if let Ok(connection) = TcpStream::connect(&self.host).await {
                self.protocol = Protocol::new(connection);
            }
            false
        }
    }

    // We assume the only operation that may fail is `create_transaction`.
    pub async fn commit(&mut self, transaction_id: u32) {
        let _ = self.protocol.commit(transaction_id).await;
    }

    pub async fn abort(&mut self, transaction_id: u32) {
        let _ = self.protocol.abort(transaction_id).await;
    }
}
