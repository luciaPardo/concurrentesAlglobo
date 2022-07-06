use crate::event::Event;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct EventProtocol {
    stream: TcpStream,
}

impl EventProtocol {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub async fn send_event(&mut self, event: Event) {
        let payload = event.to_bytes();
        let sz = payload.len() as u32;
        let mut full_payload = Vec::new();
        full_payload.extend_from_slice(&sz.to_le_bytes());
        full_payload.extend(payload);

        // Stats are not mission-critical, so we can ignore this
        // errors.
        let _ = self.stream.write_all(&full_payload).await;
    }

    pub async fn recv_event(&mut self) -> Option<Event> {
        let mut sz = [0u8; 4];
        if self.stream.read_exact(&mut sz).await.is_ok() {
            let mut buf = vec![0u8; u32::from_le_bytes(sz) as usize];
            if self.stream.read_exact(&mut buf).await.is_err() {
                return None;
            }
            let message = Event::from_bytes(&buf);
            Some(message)
        } else {
            None
        }
    }
}
