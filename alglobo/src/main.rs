use std::time::Duration;

use hotel_client::HotelClient;
use replication::Replication;
mod hotel_client;
mod leader_election;
// mod payments_queue;
mod replication;

use helpers::alglobo_transaction::Pago;

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
    let mut airline = HotelClient::new().await;
    let mut bank = HotelClient::new().await;
    loop {
        for i in 0..10 {
            let tx = Pago {
                id: i,
                cliente: "Lucía".into(),
                precio_hotel: 10,
                precio_aerolinea: 20,
            };

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
