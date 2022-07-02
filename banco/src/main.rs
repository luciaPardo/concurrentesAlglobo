use std::{collections::HashMap, io::Read, sync::Arc};

use actix::dev::MessageResponse;
use actix::{Actor, Context, Handler};
use helpers::alglobo_transaction::AlgloboTransaction;
use helpers::entity_main::run_entity;
use helpers::protocol::{self, Protocol};
use helpers::TransactionMessage;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

enum TransactionState {
    Accepted { tx: AlgloboTransaction },
    Abort,
    Commit,
}

struct Bank {
    hotel_account: u32,
    airline_account: u32,
    transaction_log: HashMap<u32, TransactionState>,
}

impl Bank {
    pub fn new() -> Self {
        Self {
            hotel_account: 0,
            airline_account: 0,
            transaction_log: HashMap::new(),
        }
    }
}
impl Actor for Bank {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("[Bank] Iniciado");
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        println!("[Bank] Detenido");
    }
}

impl Handler<TransactionMessage> for Bank {
    type Result = Result<Option<bool>, std::io::Error>;

    fn handle(&mut self, msg: TransactionMessage, _ctx: &mut Context<Self>) -> Self::Result {
        println!("[Bank] handle: {:?}", msg);
        match msg {
            TransactionMessage::Prepare { transaction } => {
                if transaction.client == "falla_banco" {
                    return Ok(Some(false));
                }
                self.transaction_log.insert(
                    transaction.id,
                    TransactionState::Accepted { tx: transaction },
                );

                return Ok(Some(true));
            }
            TransactionMessage::Abort { transaction_id } => {
                self.transaction_log
                    .insert(transaction_id, TransactionState::Abort);
            }
            TransactionMessage::Commit { transaction_id } => {
                match self.transaction_log.get(&transaction_id) {
                    Some(TransactionState::Accepted { tx: transaction }) => {
                        self.hotel_account += transaction.hotel_price;
                        self.airline_account += transaction.airline_price;

                        println!(
                            "Sumando {} en la cuenta del hotel de cantidad {} de {}",
                            transaction.hotel_price, self.hotel_account, transaction.client
                        );
                        println!(
                            "Sumando {} en la cuenta de la aerolínea de cantidad {} de {}",
                            transaction.airline_price, self.airline_account, transaction.client
                        );

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
    let listener = TcpListener::bind("0.0.0.0:9997")
        .await
        .expect("Could not open port 9997");
    run_entity(listener, Bank::new()).await;
}
