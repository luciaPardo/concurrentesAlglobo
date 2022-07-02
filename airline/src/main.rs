use actix::{Actor, Context, Handler};
use core::time;
use helpers::entity_main::run_entity;
use helpers::event_protocol::{self, EventProtocol};
use helpers::TransactionMessage;
use std::collections::HashMap;
use std::time::SystemTime;
use tokio::net::TcpListener;

enum TransactionState {
    Accepted { client: String },
    Abort,
    Commit,
}

struct Airline {
    reservations: u32,
    transaction_log: HashMap<u32, TransactionState>,
}

impl Airline {
    pub fn new() -> Self {
        Self {
            reservations: 0,
            transaction_log: HashMap::new(),
        }
    }
}
impl Actor for Airline {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("[AIRLINE] Iniciado");
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        println!("[AIRLINE] Detenido");
    }
}

impl Handler<TransactionMessage> for Airline {
    type Result = Result<Option<bool>, std::io::Error>;

    fn handle(&mut self, msg: TransactionMessage, _ctx: &mut Context<Self>) -> Self::Result {
        println!("[Airline] handle: {:?}", msg);
        match msg {
            TransactionMessage::Prepare { transaction } => {
                if transaction.client == "falla_airline" {
                    return Ok(Some(false));
                }
                let sys_time = SystemTime::now();
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
    let listener = TcpListener::bind("0.0.0.0:9998")
        .await
        .expect("Could not open port 9998");
    run_entity(listener, Airline::new()).await;
}
