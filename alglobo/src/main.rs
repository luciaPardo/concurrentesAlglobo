use airline_client::AirlineClient;
use bank_client::BankClient;
use hotel_client::HotelClient;
use leader_election::bully::BullyLeaderElection;
use replication::Replication;
mod airline_client;
mod bank_client;
mod hotel_client;
mod payments_queue;
use payments_queue::PaymentsQueue;
mod output_logger;
use output_logger::OutputLogger;
use std::sync::Arc;
use std::{thread, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
mod leader_election;
mod replication;

#[actix_rt::main]
async fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let replica_id = if args.len() == 1 {
        0u32
    } else if args.len() == 2 {
        args[1]
            .parse()
            .expect("Invalid replica id. Usage: ./alglobo [replica id]")
    } else {
        panic!("Usage: ./alglobo [replica id]");
    };
    let mut replication_manager = Replication::new(BullyLeaderElection::new(replica_id));
    loop {
        if replication_manager.is_leader() {
            println!(
                "[{}] Replica {} is the current leader.",
                replica_id,
                std::process::id()
            );
            replica_main().await;
            return;
        } else {
            println!(
                "[{}] Replica {} is not the leader.",
                replica_id,
                std::process::id()
            );
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
    let mut logger = OutputLogger::new("./failed.csv".into(), "./processed.csv".into());

    let mut airline = AirlineClient::new().await;
    let mut bank = BankClient::new().await;
    let payments_queue = PaymentsQueue::new(1000);
    loop {
        while let Some(tx) = payments_queue.pop() {
            if !hotel.create_transaction(&tx).await {
                println!("Hotel did not like transaction {:?}", tx);
                logger.log_failed(&tx);
                continue;
            }
            if !airline.create_transaction(&tx).await {
                println!("Airline did not like transaction {:?}", tx);
                hotel.abort(tx.id).await;
                logger.log_failed(&tx);
                continue;
            }
            if !bank.create_transaction(&tx).await {
                println!("Bank did not like transaction {:?}", tx);
                hotel.abort(tx.id).await;
                airline.abort(tx.id).await;
                logger.log_failed(&tx);
                continue;
            }

            hotel.commit(tx.id).await;
            airline.commit(tx.id).await;
            bank.commit(tx.id).await;
            println!("Transaction {:?} approved", tx);
            logger.log_success(&tx);

            std::thread::sleep(Duration::from_millis(100));
        }
    }
}
