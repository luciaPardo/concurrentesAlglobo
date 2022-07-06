use std::sync::Arc;

use actix::{Actor, Context, Handler};
use helpers::{event::Event, event_protocol::EventProtocol};

use tokio::net::TcpListener;

extern crate actix;

struct Stats {
    tot_time: u32,
    payments_count: u32,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            tot_time: 0,
            payments_count: 0,
        }
    }
}

impl Actor for Stats {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("[STATS] Iniciado");
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        println!("[STATS] Detenido");
    }
}

impl Handler<Event> for Stats {
    type Result = ();

    fn handle(&mut self, msg: Event, _ctx: &mut Context<Self>) -> Self::Result {
        println!("[STATS] handle: {:?}", msg);
        match msg {
            Event::TxSuccess {
                entity,
                duration_ms,
            } => {
                println!("Entity : {} took : {}", entity, duration_ms);
            }
            Event::TxFailure { entity, reason } => {
                println!("Entity : {} failed because : {}", entity, reason);
            }
            Event::PaymentSuccess { duration } => {
                self.tot_time += duration;
                self.payments_count += 1;
                println!("Payment took : {}", duration);
                println!(
                    "Payments average : {} milisecs",
                    self.tot_time / self.payments_count
                );
            }
            Event::PaymentFailed { reason } => {
                println!("Payment Failed because: {}", reason)
            }
        }
    }
}

#[actix_rt::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:9996")
        .await
        .expect("Could not open port 9996");
    let stats = Stats::new();
    let addr = Arc::new(stats.start());
    let mut handles = Vec::new();
    while let Ok((stream, _)) = listener.accept().await {
        let addr = addr.clone();
        let mut protocol = EventProtocol::new(stream);
        handles.push(actix_rt::spawn(async move {
            loop {
                if let Some(message) = protocol.recv_event().await {
                    addr.send(message).await.unwrap();
                }
            }
        }))
    }

    for handle in handles {
        handle.await.ok();
    }
}
