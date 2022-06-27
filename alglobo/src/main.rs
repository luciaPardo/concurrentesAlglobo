use airline_client::AirlineClient;
use bank_client::BankClient;
use hotel_client::HotelClient;
use replication::Replication;
mod airline_client;
mod bank_client;
mod hotel_client;
mod payments_queue;
use payments_queue::PaymentsQueue;
use std::sync::Arc;
use std::{thread, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
mod leader_election;
mod replication;

use helpers::alglobo_transaction::AlgloboTransaction;

#[actix_rt::main]
async fn main() {
    let replication_manager = Replication::new();
    loop {
        if replication_manager.is_leader() {
            println!("Replica {} is the current leader.", std::process::id());
            replica_main().await;
            return;
        } else {
            println!("Replica {} is not the leader.", std::process::id());
            replication_manager.wait_until_becoming_leader();
            println!(
                "Current leader has failed. Replica {} will take over.",
                std::process::id()
            );
        }
    }
}

async fn replica_main() {
    let mut hotel = HotelClient::new().await;

    // TODO: replace bank and airline with correct clients
    let mut airline = AirlineClient::new().await;
    let mut bank = HotelClient::new().await;
    let payments_queue = PaymentsQueue::new(1000);
    loop {
        while let Some(tx) = payments_queue.pop() {
            if !hotel.create_transaction(&tx).await {
                println!("Hotel did not like transaction {:?}", tx);
            }
            if !airline.create_transaction(&tx).await {
                println!("Airline did not like transaction {:?}", tx);
                hotel.abort(tx.id).await;
            }
            if !bank.create_transaction(&tx).await {
                println!("Bank did not like transaction {:?}", tx);
                hotel.abort(tx.id).await;
                airline.abort(tx.id).await;
            }

            hotel.commit(tx.id).await;
            airline.commit(tx.id).await;
            bank.commit(tx.id).await;
            println!("Transaction {:?} approved", tx);

            std::thread::sleep(Duration::from_millis(100));
        }
    }
}
