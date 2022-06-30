use std::{collections::VecDeque, sync::Condvar};

use helpers::alglobo_transaction::AlgloboTransaction;
use std::sync::Mutex;

pub struct Queue {
    data: Mutex<VecDeque<AlgloboTransaction>>,
    cv_full: Condvar,
    cv_empty: Condvar,
    max: usize,
}

impl Queue {
    pub fn new(k: usize) -> Self {
        Self {
            data: Mutex::new(VecDeque::new()),
            cv_full: Condvar::new(),
            cv_empty: Condvar::new(),
            max: k,
        }
    }
    pub fn push(&self, value: AlgloboTransaction) {
        let mut data = self.data.lock().expect("poison");
        while data.len() == self.max {
            data = self.cv_full.wait(data).expect("poison");
        }
        data.push_back(value);
        self.cv_empty.notify_one()
    }
    pub fn pop_front(&self) -> Option<AlgloboTransaction> {
        let mut cola = self.data.lock().expect("poison");
        cola.pop_front()
    }
}
