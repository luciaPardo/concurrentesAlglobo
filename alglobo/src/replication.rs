pub struct Replication {}

impl Replication {
    pub fn new() -> Self {
        Self {}
    }

    pub fn is_leader(&self) -> bool {
        true
    }

    pub fn wait_until_becoming_leader(&self) {}
}
