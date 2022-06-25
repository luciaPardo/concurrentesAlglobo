// Actor: Hotel

use std::{collections::HashMap, io::Read, sync::Arc};

use actix::dev::MessageResponse;
use actix::{Actor, Context, Handler};
use helpers::protocol::Protocol;
use helpers::TransactionMessage;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

enum TransactionState {
    Accepted { client: String },
    Abort,
    Commit,
}

struct Hotel {
    reservas_aceptadas: u32,
    transaction_log: HashMap<u32, TransactionState>,
}

impl Hotel {
    pub fn new() -> Self {
        Self {
            reservas_aceptadas: 0,
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
    type Result = Result<bool, std::io::Error>;

    fn handle(&mut self, msg: TransactionMessage, _ctx: &mut Context<Self>) -> Self::Result {
        println!("[HOTEL] handle: {:?}", msg);
        match msg {
            TransactionMessage::Prepare {
                transaction_id,
                client,
            } => {
                self.transaction_log
                    .insert(transaction_id, TransactionState::Accepted { client });
                // TODO: mandar OK a AlGlobo
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
                            self.reservas_aceptadas + 1
                        );
                        self.reservas_aceptadas += 1;
                        self.transaction_log
                            .insert(transaction_id, TransactionState::Commit);
                    }
                    Some(TransactionState::Commit) => {
                        // Mandar OK (ya commiteada)
                    }
                    Some(TransactionState::Abort) => {
                        // Mandar Abort
                    }
                    None => {
                        // transaction id no existe???
                    }
                }
            }
            _ => panic!("Invalid"),
        }

        Ok(true)
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
        let mut protocol = Protocol::new(stream);
        let addr = addr.clone();
        handles.push(actix_rt::spawn(async move {
            loop {
                let message = protocol.receive().await;
                if let Some(message) = message {
                    addr.send(message).await.unwrap().unwrap();
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
