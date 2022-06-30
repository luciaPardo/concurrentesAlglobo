use actix::{Actor, Context, Handler};
use std::{collections::HashMap, sync::Arc};
use tokio::net::TcpListener;

use helpers::protocol::Protocol;
use helpers::TransactionMessage;

enum TransactionState {
    Accepted { client: String },
    Abort,
    Commit,
}

struct Hotel {
    reservations: u32,
    transaction_log: HashMap<u32, TransactionState>,
}

impl Hotel {
    pub fn new() -> Self {
        Self {
            reservations: 0,
            transaction_log: HashMap::new(),
        }
    }
}

impl Actor for Hotel {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("[HOTEL] Iniciado");
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        println!("[HOTEL] Detenido");
    }
}

impl Handler<TransactionMessage> for Hotel {
    type Result = Result<Option<bool>, std::io::Error>;

    fn handle(&mut self, msg: TransactionMessage, _ctx: &mut Context<Self>) -> Self::Result {
        println!("[HOTEL] handle: {:?}", msg);
        match msg {
            TransactionMessage::Prepare { transaction } => {
                if transaction.client == "falla_hotel" {
                    println!("fallo el hotel");
                    return Ok(Some(false));
                }
                self.transaction_log.insert(
                    transaction.id,
                    TransactionState::Accepted {
                        client: transaction.client,
                    },
                );

                return Ok(Some(true));
            }
            TransactionMessage::Abort { transaction_id } => {
                self.transaction_log
                    .insert(transaction_id, TransactionState::Abort);
                // TODO: mandar Abort a AlGlobo
            }
            TransactionMessage::Commit { transaction_id } => {
                match self.transaction_log.get(&transaction_id) {
                    Some(TransactionState::Accepted { client }) => {
                        println!(
                            "Guardando reserva nro {} para {client}",
                            self.reservations + 1
                        );
                        self.reservations += 1;
                        self.transaction_log
                            .insert(transaction_id, TransactionState::Commit);
                    }
                    Some(TransactionState::Commit) => {
                        // Mandar OK (ya commiteada)
                    }
                    Some(TransactionState::Abort) => {}
                    None => {
                        // transaction id no existe???
                    }
                }
            }
            _ => panic!("Invalid"),
        }

        Ok(Some(true))
    }
}

#[actix_rt::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:9999")
        .await
        .expect("Could not open port 9999");
    let mut handles = Vec::new();
    let hotel = Hotel::new();
    let addr = Arc::new(hotel.start());

    while let Ok((stream, _)) = listener.accept().await {
        let addr = addr.clone();
        let mut protocol = Protocol::new(stream);
        handles.push(actix_rt::spawn(async move {
            loop {
                let message = protocol.receive().await;
                if let Some(message) = message {
                    if let Ok(Some(result)) = addr.send(message).await.unwrap() {
                        if result {
                            println!("sending ok");
                            protocol.send_ok().await;
                        } else {
                            println!("sending failure");
                            protocol.send_failure().await;
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
