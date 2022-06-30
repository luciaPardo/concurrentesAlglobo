use std::net::UdpSocket;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use super::control_message::ControlMessage;
use super::leader_election_trait::LeaderElection;

fn id_to_ctrladdr(id: u32) -> String {
    "127.0.0.1:1234".to_owned() + &*id.to_string()
}

const REPLICAS: u32 = 4;
const TIMEOUT: Duration = Duration::from_secs(10);
const LEADER_HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(1);

pub struct BullyLeaderElection {
    id: u32,
    socket: UdpSocket,
    leader_id: Arc<(Mutex<Option<u32>>, Condvar)>,
    got_ok: Arc<(Mutex<bool>, Condvar)>,
    got_pong: Arc<(Mutex<Option<u32>>, Condvar)>,
    stop: Arc<(Mutex<bool>, Condvar)>,
}

impl BullyLeaderElection {
    pub fn new(id: u32) -> BullyLeaderElection {
        let mut ret = BullyLeaderElection {
            id,
            socket: UdpSocket::bind(id_to_ctrladdr(id)).unwrap(),
            leader_id: Arc::new((Mutex::new(Some(id)), Condvar::new())),
            got_ok: Arc::new((Mutex::new(false), Condvar::new())),
            got_pong: Arc::new((Mutex::new(None), Condvar::new())),
            stop: Arc::new((Mutex::new(false), Condvar::new())),
        };

        let mut clone = ret.clone();
        thread::spawn(move || clone.responder());

        thread::sleep(Duration::from_secs(2));
        ret.find_new();
        ret
    }

    fn am_i_leader(&self) -> bool {
        self.get_leader_id() == self.id
    }

    fn get_leader_id(&self) -> u32 {
        self.leader_id
            .1
            .wait_while(self.leader_id.0.lock().unwrap(), |leader_id| {
                leader_id.is_none()
            })
            .unwrap()
            .unwrap()
    }

    fn find_new(&mut self) {
        if *self.stop.0.lock().unwrap() {
            return;
        }
        if self.leader_id.0.lock().unwrap().is_none() {
            // ya esta buscando lider
            return;
        }
        println!("[{}] buscando lider", self.id);
        *self.got_ok.0.lock().unwrap() = false;
        *self.leader_id.0.lock().unwrap() = None;
        self.send_election();
        let got_ok =
            self.got_ok
                .1
                .wait_timeout_while(self.got_ok.0.lock().unwrap(), TIMEOUT, |got_it| !*got_it);
        if !*got_ok.unwrap().0 {
            self.make_me_leader()
        } else {
            self.leader_id
                .1
                .wait_while(self.leader_id.0.lock().unwrap(), |leader_id| {
                    leader_id.is_none()
                })
                .expect("poison")
                .expect("poison2");
        }
    }

    fn id_to_msg(&self, header: u8) -> Vec<u8> {
        let mut msg = vec![header];
        msg.extend_from_slice(&self.id.to_le_bytes());
        msg
    }

    fn send_election(&self) {
        // P envía el mensaje ELECTION a todos los procesos que tengan número mayor
        let msg = self.id_to_msg(b'E');
        for peer_id in (self.id + 1)..REPLICAS {
            self.socket.send_to(&msg, id_to_ctrladdr(peer_id)).unwrap();
        }
    }

    fn make_me_leader(&self) {
        // El nuevo coordinador se anuncia con un mensaje COORDINATOR
        let mut leader_lock = self.leader_id.0.lock().unwrap();
        println!("[{}] me anuncio como lider", self.id);
        let msg = self.id_to_msg(b'C');
        for peer_id in 0..REPLICAS {
            if peer_id != self.id {
                self.socket.send_to(&msg, id_to_ctrladdr(peer_id)).unwrap();
            }
        }
        *leader_lock = Some(self.id);
    }

    fn responder(&mut self) {
        println!("[{}] Responder started", self.id);
        while !*self.stop.0.lock().unwrap() {
            let mut buf = [0; ControlMessage::size_of()];
            let (size, _) = self.socket.recv_from(&mut buf).unwrap();
            let (message, id_from) = ControlMessage::from_bytes(&buf[0..size]);
            if *self.stop.0.lock().unwrap() {
                break;
            }
            println!(" Message : {:?}", message);
            match message {
                ControlMessage::Ok => {
                    println!("[{}] recibí OK de {}", self.id, id_from);
                    *self.got_ok.0.lock().unwrap() = true;
                    self.got_ok.1.notify_all();
                }
                ControlMessage::Election => {
                    println!("[{}] recibí Election de {}", self.id, id_from);
                    if id_from < self.id {
                        self.socket
                            .send_to(
                                &ControlMessage::Ok.to_bytes(self.id),
                                id_to_ctrladdr(id_from),
                            )
                            .unwrap();
                        let mut me = self.clone();
                        thread::spawn(move || me.find_new());
                    }
                }
                ControlMessage::Coordinator => {
                    println!("[{}] recibí nuevo coordinador {}", self.id, id_from);
                    *self.leader_id.0.lock().unwrap() = Some(id_from);
                    self.leader_id.1.notify_all();
                }
                ControlMessage::Ping => {
                    println!("[{}] recibí Ping de {}", self.id, id_from);
                    self.socket
                        .send_to(
                            &ControlMessage::Pong.to_bytes(self.id),
                            id_to_ctrladdr(id_from),
                        )
                        .unwrap();
                }
                ControlMessage::Pong => {
                    println!("[{}] recibí Pong de {}", self.id, id_from);
                    *self.got_pong.0.lock().unwrap() = Some(id_from);
                }
                ControlMessage::GracefulQuit => {
                    *self.stop.0.lock().unwrap() = true;
                    break;
                }
            }
        }
        *self.stop.0.lock().unwrap() = true;
        self.stop.1.notify_all();
        println!("[{}] Responder finished", self.id);
    }

    fn clone(&self) -> BullyLeaderElection {
        BullyLeaderElection {
            id: self.id,
            socket: self.socket.try_clone().unwrap(),
            leader_id: self.leader_id.clone(),
            got_ok: self.got_ok.clone(),
            got_pong: self.got_pong.clone(),
            stop: self.stop.clone(),
        }
    }
}

impl LeaderElection for BullyLeaderElection {
    fn is_leader(&self) -> bool {
        self.am_i_leader()
    }

    fn wait_until_becoming_leader(&mut self) {
        while !self.is_leader() {
            std::thread::sleep(LEADER_HEALTH_CHECK_TIMEOUT);
            if self.has_finished() {
                break;
            }
            println!("checking if current leader is healthy");

            let mut got_pong = self.got_pong.0.lock().unwrap();
            *got_pong = None;
            self.socket
                .send_to(
                    &ControlMessage::Ping.to_bytes(self.id),
                    id_to_ctrladdr(self.get_leader_id()),
                )
                .unwrap();

            println!("wait for leader response");
            let (got_pong, _) = self
                .got_pong
                .1
                .wait_timeout_while(got_pong, LEADER_HEALTH_CHECK_TIMEOUT, |e| e.is_none())
                .unwrap();

            let find_new_leader = match *got_pong {
                Some(e) if e != self.get_leader_id() => true,
                None => true,
                _ => false,
            };
            drop(got_pong);

            if find_new_leader {
                self.find_new_leader();
            }
        }
    }

    fn find_new_leader(&mut self) {
        self.find_new()
    }

    fn has_finished(&self) -> bool {
        *self.stop.0.lock().unwrap()
    }

    fn graceful_quit(&mut self) {
        *self.stop.0.lock().unwrap() = true;

        for replica_id in 0..REPLICAS {
            if replica_id == self.id {
                continue;
            }

            self.socket
                .send_to(
                    &ControlMessage::GracefulQuit.to_bytes(self.id),
                    id_to_ctrladdr(replica_id),
                )
                .unwrap();
        }
    }
}
