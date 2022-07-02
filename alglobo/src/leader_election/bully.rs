use std::{
    io::{ErrorKind, Result},
    net::UdpSocket,
    ops::Deref,
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};

use super::leader_election_trait::LeaderElection;
use super::{
    atomic_value::AtomicValue,
    control_message::{ControlMessage, PeerId},
};

const BASE_PEER_PORT: u16 = 27000;

const RECV_TIMEOUT: Duration = Duration::from_secs(1);
const LEADER_HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(1);
const LEADER_ELECTION_TIMEOUT: Duration = Duration::from_secs(10);

const MIN_PEER_ID: PeerId = 1;
const MAX_PEER_ID: PeerId = 30;

pub struct BullyLeaderElectionInner {
    id: PeerId,
    socket: UdpSocket,
    leader_id: AtomicValue<Option<PeerId>>,
    got_ok: AtomicValue<bool>,
    got_pong: AtomicValue<Option<PeerId>>,
    stop: AtomicValue<bool>,
    current_state: AtomicValue<State>,
}

pub struct BullyLeaderElection {
    inner: BullyLeaderElectionInner,
    responder_thread: Option<JoinHandle<()>>,
}

impl Deref for BullyLeaderElection {
    type Target = BullyLeaderElectionInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum State {
    Idle,
    FindingLeader { since: SystemTime },
    WaitingForCoordinator,
}

impl State {
    pub fn finding_leader_now() -> Self {
        Self::FindingLeader {
            since: SystemTime::now(),
        }
    }
}

impl BullyLeaderElection {
    pub fn new(id: PeerId) -> Result<BullyLeaderElection> {
        let inner = BullyLeaderElectionInner::new(id)?;
        let mut thread_inner = inner.clone();
        let responder_thread = thread::spawn(move || thread_inner.responder());
        let mut ret = Self {
            inner,
            responder_thread: Some(responder_thread),
        };

        ret.find_new_leader();
        Ok(ret)
    }
}

impl BullyLeaderElectionInner {
    pub fn new(id: PeerId) -> Result<BullyLeaderElectionInner> {
        let socket = UdpSocket::bind(Self::build_peer_address(id))?;
        socket
            .set_read_timeout(Some(RECV_TIMEOUT))
            .expect("Could not set socket as non-blocking.");

        Ok(Self {
            id,
            socket,
            leader_id: AtomicValue::new(None),
            got_ok: AtomicValue::new(false),
            got_pong: AtomicValue::new(None),
            stop: AtomicValue::new(false),
            current_state: AtomicValue::new(State::Idle),
        })
    }

    fn get_leader_id(&self) -> PeerId {
        self.leader_id
            .wait_while(|leader_id| leader_id.is_none())
            .expect("Mutex poisoned")
    }

    fn responder(&mut self) {
        println!("Responder thread started");
        while !*self.stop.load() {
            // Ticks are not spaced evenly, but it is guaranteed that ticks are
            // not spaced for more than READ_TIMEOUT.
            self.on_tick();

            // Try to fetch a new message.
            match self.recv_message() {
                Ok(Some((message, peer_from))) => self.process_message(message, peer_from),
                Ok(None) => continue,
                Err(e) => {
                    println!("Error on responder thread: {:?}", e);
                    break;
                }
            }
        }
        self.stop.store(true);
        println!("Responder thread finished");
    }

    /// Processes a message received from a peer.
    ///
    /// This method should not block for a long time.
    fn process_message(&self, message: ControlMessage, peer_from: PeerId) {
        println!("Processing message {:?} from {}", message, peer_from);
        let current_state = *self.current_state.load();
        match message {
            ControlMessage::Ok => match current_state {
                State::FindingLeader { .. } => {
                    self.current_state.store(State::WaitingForCoordinator)
                }
                _ => {
                    println!(
                        "Received OK from {} during an invalid state, ignoring...",
                        peer_from
                    );
                }
            },
            ControlMessage::Election => {
                if peer_from < self.id {
                    self.send_message(peer_from, ControlMessage::Ok);
                    if !matches!(current_state, State::FindingLeader { .. }) {
                        self.current_state.store(State::finding_leader_now());
                    }
                }
            }
            ControlMessage::Coordinator => {
                self.leader_id.store(Some(peer_from));
                self.current_state.store(State::Idle);
            }
            ControlMessage::Ping => {
                self.send_message(peer_from, ControlMessage::Pong);
            }
            ControlMessage::Pong => {
                self.got_pong.store(Some(peer_from));
            }
            ControlMessage::GracefulQuit => {
                self.stop.store(true);
            }
        }
    }

    fn on_tick(&self) {
        let current_state = *self.current_state.load();
        if let State::FindingLeader { since } = current_state {
            if SystemTime::now().duration_since(since).unwrap() > LEADER_ELECTION_TIMEOUT {
                // We were finding a leader for more than LEADER_ELECTION_TIMEOUT
                // => Finish the election by declaring ourselves as coordinator.
                println!("[{}] Announcing myself as new coordinator", self.id);
                self.broadcast_message(ControlMessage::Coordinator);
                self.leader_id.store(Some(self.id));
                self.current_state.store(State::Idle);
            }
        }
    }

    /// Waits for a new message during RECV_TIMEOUT seconds.
    fn recv_message(&self) -> Result<Option<(ControlMessage, PeerId)>> {
        let mut buf = [0; ControlMessage::size_of()];

        let (size, _) = match self.socket.recv_from(&mut buf) {
            Err(e) if e.kind() == ErrorKind::WouldBlock => return Ok(None),
            r => r?,
        };

        if size != ControlMessage::size_of() {
            // we received less bytes than needed for a control message???
            println!(
                "[{}] RECV MESSAGE: Invalid message received, ignoring...",
                self.id
            );
            Ok(None)
        } else {
            Ok(Some(ControlMessage::from_bytes(&buf[0..size])))
        }
    }

    fn send_message(&self, dst_peer: PeerId, message: ControlMessage) {
        self.socket
            .send_to(
                &message.to_bytes(self.id),
                Self::build_peer_address(dst_peer),
            )
            .unwrap();
    }

    fn broadcast_message(&self, message: ControlMessage) {
        self.broadcast_message_if(message, |_| true);
    }

    fn broadcast_message_if<F: Fn(PeerId) -> bool>(&self, message: ControlMessage, predicate: F) {
        for peer_id in MIN_PEER_ID..=MAX_PEER_ID {
            if peer_id == self.id {
                continue;
            }

            if predicate(peer_id) {
                self.send_message(peer_id, message);
            }
        }
    }

    fn build_peer_address(peer_id: PeerId) -> String {
        format!("127.0.0.1:{}", BASE_PEER_PORT + (peer_id as u16))
    }
}

impl Clone for BullyLeaderElectionInner {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            socket: self.socket.try_clone().unwrap(),
            leader_id: self.leader_id.clone(),
            got_ok: self.got_ok.clone(),
            got_pong: self.got_pong.clone(),
            stop: self.stop.clone(),
            current_state: self.current_state.clone(),
        }
    }
}

impl LeaderElection for BullyLeaderElection {
    fn is_leader(&self) -> bool {
        self.get_leader_id() == self.id
    }

    fn wait_until_becoming_leader(&mut self) {
        while !self.is_leader() {
            std::thread::sleep(LEADER_HEALTH_CHECK_TIMEOUT);
            if self.has_finished() {
                break;
            }
            println!("checking if current leader is healthy");
            self.got_pong.store(None);
            self.send_message(self.get_leader_id(), ControlMessage::Ping);
            println!("wait for leader response");
            let got_pong = self
                .got_pong
                .wait_timeout_while(LEADER_HEALTH_CHECK_TIMEOUT, |e| e.is_none());

            if !matches!(&got_pong, Some(e) if e.unwrap() == self.get_leader_id()) {
                drop(got_pong);
                self.find_new_leader();
            }
        }
    }

    fn find_new_leader(&mut self) {
        if let State::FindingLeader { .. } = *self.current_state.load() {
            return;
        }

        self.current_state.store(State::finding_leader_now());
        self.leader_id.store(None);
        self.broadcast_message_if(ControlMessage::Election, |peer_id| self.id < peer_id);
    }

    fn has_finished(&self) -> bool {
        *self.stop.load()
    }

    fn graceful_quit(&mut self) {
        self.stop.store(true);
        self.broadcast_message(ControlMessage::GracefulQuit);
    }
}

impl Drop for BullyLeaderElection {
    fn drop(&mut self) {
        if let Some(handle) = self.responder_thread.take() {
            self.stop.store(true);
            let _ = handle.join();
        }
    }
}
