mod leader_election;
mod output_logger;
mod payments_queue;
mod replication;
mod transactional_entity;

use std::{
    error::Error,
    time::{Duration, SystemTime},
};

use actix_rt::net::TcpStream;
use helpers::{event::Event, event_protocol::EventProtocol};
use leader_election::{bully::BullyLeaderElection, leader_election_trait::LeaderElection};
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
    let mut replication_manager = Replication::new(BullyLeaderElection::discovery().unwrap());
    while !replication_manager.has_finished() {
        if replication_manager.is_leader() {
            println!(
                "[{}] I am the current leader.",
                replication_manager.get_current_id()
            );
            replica_main(&mut replication_manager).await?;
        } else {
            println!(
                "[{}] This replica is not the leader.",
                replication_manager.get_current_id(),
            );
            replication_manager.wait_until_becoming_leader();
        }
    }
    println!("Leader has informed there is no more work to do. Graceful quit.");
    Ok(())
}

async fn replica_main(
    manager: &mut Replication<impl LeaderElection>,
) -> Result<(), Box<dyn Error>> {
    let mut payments_queue =
        PaymentsQueue::new("./payments.csv", "./processed.csv", "./failed.csv")
            .expect("Could not load payments file");
    let mut logger = OutputLogger::new("./failed.csv".into(), "./processed.csv".into())
        .expect("Cannot start transaction logger");

    let mut hotel = TransactionalEntity::new(HOTEL_HOST).await?;
    let mut airline = TransactionalEntity::new(AIRLINE_HOST).await?;
    let mut bank = TransactionalEntity::new(BANK_HOST).await?;
    let socket_event = TcpStream::connect(STATS_HOST).await?;
    let mut event_protocol = EventProtocol::new(socket_event);

    while let Some(tx) = payments_queue.pop() {
        if !manager.is_leader() {
            // Leader has changed
            return Ok(());
        }

        std::thread::sleep(Duration::from_millis(3000));
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
    manager.graceful_quit();
    Ok(())
}
