use crate::protocol::Protocol;
use crate::TransactionMessage;
use actix::{Actor, Handler};
use std::sync::Arc;
use tokio::net::TcpListener;

pub async fn run_entity<E: Actor<Context = actix::Context<E>> + Handler<TransactionMessage>>(
    listener: TcpListener,
    entity: E,
) {
    let addr = Arc::new(entity.start());
    let mut handles = Vec::new();
    while let Ok((stream, _)) = listener.accept().await {
        let addr = addr.clone();
        let mut protocol = Protocol::new(stream);
        handles.push(actix_rt::spawn(async move {
            loop {
                let message = protocol.receive().await;
                if let Some(message) = message {
                    if let Ok(Some(result)) = addr.send(message).await.unwrap() {
                        // We don't really care if we could send the response or not. At this point
                        // there is nothing we can do if the client does not want to hear our
                        // response.
                        if result {
                            let _ = protocol.send_ok().await;
                        } else {
                            let _ = protocol.send_failure().await;
                        }
                    }
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
