pub trait LeaderElection {
    /// Returns true if this process is the current leader.
    /// This method may block the current thread if there is an election
    /// in progress.
    fn is_leader() -> bool;

    /// Blocks the current thread until this process becomes the current
    /// leader.
    fn wait_until_becoming_leader();

    /// Starts the election process for a new leader.
    ///
    /// If the calling thread was the current leader this method
    /// will panic.
    fn find_new_leader();
}

pub trait LeaderElectionController {
    fn on_election(&self, id: u32);
    fn on_coordinator(&self, id: u32);
    fn on_ok(&self, id: u32);
}