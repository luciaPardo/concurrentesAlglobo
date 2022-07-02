mod leader_election;
mod output_logger;
mod payments_queue;
mod queue;
mod replication;
mod transactional_entity;

use std::{
    error::Error,
    time::{Duration, SystemTime},
};

use actix_rt::net::TcpStream;
use helpers::{event::Event, event_protocol::EventProtocol};
use leader_election::bully::BullyLeaderElection;
use output_logger::OutputLogger;
use payments_queue::PaymentsQueue;
use replication::Replication;
use transactional_entity::TransactionalEntity;

const HOTEL_HOST: &str = "0.0.0.0:9999";
const AIRLINE_HOST: &str = "0.0.0.0:9998";
const BANK_HOST: &str = "0.0.0.0:9997";
const STATS_HOST: &str = "0.0.0.0:9996";

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().collect::<Vec<String>>();
    let replica_id = if args.len() == 1 {
        0u8
    } else if args.len() == 2 {
        args[1]
            .parse()
            .expect("Invalid replica id. Usage: ./alglobo [replica id]")
    } else {
        panic!("Usage: ./alglobo [replica id]");
    };
    let mut replication_manager = Replication::new(BullyLeaderElection::new(replica_id).unwrap());
    while !replication_manager.has_finished() {
        if replication_manager.is_leader() {
            println!(
                "[{}] Replica {} is the current leader.",
                replica_id,
                std::process::id()
            );
            let result = replica_main().await;
            if result.is_ok() {
                replication_manager.graceful_quit();
            }
            return result;
        } else {
            println!(
                "[{}] Replica {} is not the leader.",
                replica_id,
                std::process::id()
            );
            replication_manager.wait_until_becoming_leader();
        }
    }
    println!("Leader has informed there is no more work to do. Graceful quit.");
    Ok(())
}

async fn replica_main() -> Result<(), Box<dyn Error>> {
    let payments_queue =
        PaymentsQueue::new(1000, "./payments.csv", "./processed.csv", "./failed.csv")
            .expect("Could not load payments file");
    let mut logger = OutputLogger::new("./failed.csv".into(), "./processed.csv".into())
        .expect("Cannot start transaction logger");

    let mut hotel = TransactionalEntity::new(HOTEL_HOST).await;
    let mut airline = TransactionalEntity::new(AIRLINE_HOST).await;
    let mut bank = TransactionalEntity::new(BANK_HOST).await;
    let socket_event = TcpStream::connect(STATS_HOST).await.unwrap();
    let mut event_protocol = EventProtocol::new(socket_event);

    while let Some(tx) = payments_queue.pop() {
        std::thread::sleep(Duration::from_millis(1000));
        let payment_time = SystemTime::now();
        if !hotel.create_transaction(&tx).await {
            println!("Hotel did not like transaction {}", tx.id);
            logger.log_failed(&tx);
            continue;
        }
        if !airline.create_transaction(&tx).await {
            println!("Airline did not like transaction {}", tx.id);
            hotel.abort(tx.id).await;
            logger.log_failed(&tx);
            continue;
        }
        if !bank.create_transaction(&tx).await {
            println!("Bank did not like transaction {}", tx.id);
            hotel.abort(tx.id).await;
            airline.abort(tx.id).await;
            logger.log_failed(&tx);
            continue;
        }

        hotel.commit(tx.id).await;
        airline.commit(tx.id).await;
        bank.commit(tx.id).await;

        println!("Transaction {} approved", tx.id);
        let new_sys_time = SystemTime::now();
        let difference = new_sys_time
            .duration_since(payment_time)
            .expect("Clock Error")
            .as_millis();
        event_protocol
            .send_event(Event::PaymentSuccess {
                duration: difference as u32,
            })
            .await;
        logger.log_success(&tx);
    }
    println!("All payments have been processed");
    Ok(())
}
