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
        self.stream.write_all(&sz.to_le_bytes()).await.unwrap();
        self.stream.write_all(&payload).await.unwrap();
    }

    pub async fn recv_event(&mut self) -> Option<Event> {
        let mut sz = [0u8; 4];
        if self.stream.read_exact(&mut sz).await.is_ok() {
            let mut buf = vec![0u8; u32::from_le_bytes(sz) as usize];
            self.stream.read_exact(&mut buf).await.unwrap();
            let message = Event::from_bytes(&buf);
            Some(message)
        } else {
            None
        }
    }
}
