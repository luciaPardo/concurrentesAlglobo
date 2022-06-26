use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::sync::Arc;
use std::{thread, time::Duration};


use helpers::{TransactionMessage, protocol::Protocol};

async fn send_transaction_message(stream: &mut TcpStream, msg: TransactionMessage) {
    let payload = msg.to_bytes();
    let sz = payload.len() as u32;
    stream.write_all(&sz.to_le_bytes()).await.unwrap();
    stream.write_all(&payload).await.unwrap();
}

#[actix_rt::main]
async fn main() {
    let mut stream = TcpStream::connect("0.0.0.0:9999").await.unwrap();
    let mut client = Protocol::new(stream);

    for i in 0..10 {
        client.prepare(i, "Lucía".into()).await;

       client.commit(i).await;
        std::thread::sleep(Duration::from_millis(100));
    }
}

// [P TID L U C H O] [C TID]
