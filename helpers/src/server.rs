#[actix_rt::main]
struct Server{
    listener: TcpListener,
}


impl Server{
    pub fn new() -> Self {
        Self {
            listener = TcpListener::bind("0.0.0.0:9999").await
            .expect("Could not open port 9999"),
        }
    }
    pub fn recv_transaction_msg(&mut self){
        let mut handles = Vec::new();
        while let Ok((mut stream, _)) = listener.accept().await {
            let addr = addr.clone();
            handles.push(actix_rt::spawn(async move {
                loop {
                    let mut sz = [0u8; 4];
                    if stream.read_exact(&mut sz).await.is_ok() {
                        let mut buf = vec![0u8; u32::from_le_bytes(sz) as usize];
                        stream.read_exact(&mut buf).await.unwrap();
                        let message = TransactionMessage::from_bytes(&buf);
                        println!("Received message: {:?}", message);
                        addr.do_send(message); //envía???
                    } else {
                        println!("Client disconnected");
                        break;
                    }
                }
            }))
        }

        for handle in handles {
            handle.await.ok();
        }
    }
}
