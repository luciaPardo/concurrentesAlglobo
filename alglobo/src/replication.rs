use crate::leader_election::{control_message::PeerId, leader_election_trait::LeaderElection};

pub struct Replication<L: LeaderElection> {
    leader_election_strategy: L,
}

impl<L: LeaderElection> Replication<L> {
    pub fn new(leader_election_strategy: L) -> Self {
        Self {
            leader_election_strategy,
        }
    }

    pub fn is_leader(&self) -> bool {
        self.leader_election_strategy.is_leader()
    }

    pub fn wait_until_becoming_leader(&mut self) {
        self.leader_election_strategy.wait_until_becoming_leader()
    }

    pub fn has_finished(&self) -> bool {
        self.leader_election_strategy.has_finished()
    }

    /// Sends a graceful quit message to each replica.
    pub fn graceful_quit(&mut self) {
        self.leader_election_strategy.graceful_quit();
    }

    pub fn get_current_id(&self) -> PeerId {
        self.leader_election_strategy.get_current_id()
    }
}
