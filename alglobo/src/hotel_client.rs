use helpers::{alglobo_transaction::Pago, protocol::Protocol};
use tokio::net::TcpStream;

pub struct HotelClient {
    protocol: Protocol,
}

impl HotelClient {
    pub async fn new() -> Self {
        Self {
            protocol: Protocol::new(TcpStream::connect("0.0.0.0:9999").await.unwrap()),
        }
    }

    pub async fn create_transaction(&mut self, transaction: &Pago) -> bool {
        self.protocol
            .prepare(transaction.id, transaction.cliente.clone())
            .await
    }

    pub async fn commit(&mut self, transaction_id: u32) {
        self.protocol.commit(transaction_id).await
    }

    pub async fn abort(&mut self, transaction_id: u32) {
        self.protocol.abort(transaction_id).await
    }
}
