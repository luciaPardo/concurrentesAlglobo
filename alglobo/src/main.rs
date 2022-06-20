use std::{io::Write, net::TcpStream, time::Duration};

use helpers::TransactionMessage;

fn send_transaction_message(stream: &mut TcpStream, msg: TransactionMessage) {
    let payload = msg.to_bytes();
    let sz = payload.len() as u32;
    stream.write_all(&sz.to_le_bytes()).unwrap();
    stream.write_all(&payload).unwrap();
}

fn main() {
    let mut stream = TcpStream::connect("0.0.0.0:9999").unwrap();

    for i in 0..10 {
        send_transaction_message(
            &mut stream,
            TransactionMessage::Prepare {
                transaction_id: 12 + i,
                client: "lucho".into(),
            },
        );
        send_transaction_message(
            &mut stream,
            TransactionMessage::Commit {
                transaction_id: 12 + i,
            },
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

// [P TID L U C H O] [C TID]
