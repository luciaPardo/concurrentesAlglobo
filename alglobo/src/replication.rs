use crate::leader_election::leader_election_trait::LeaderElection;

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
}
