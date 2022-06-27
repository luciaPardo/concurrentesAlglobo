use helpers::alglobo_transaction::AlgloboTransaction;
use std::collections::HashSet;
use std::sync::Condvar;
use std::sync::Mutex;
extern crate csv;

use csv::Reader;
use std::collections::VecDeque;

struct PaymentsQueue {
    queue: Queue,
}

impl PaymentsQueue {
    pub fn new(size: usize) -> Self {
        let queue = Queue::new(size);
        let mut processed = HashSet::<u32>::new();
        Self::get_ids("./failed.csv", &mut processed);
        Self::get_ids("./processed.csv", &mut processed);
        let mut reader = Reader::from_path("./payments.csv").expect("failed to read payments file");
        for result in reader.deserialize() {
            let record: AlgloboTransaction = result.expect("failed to parse payments file");
            if processed.contains(&record.id) {
                continue;
            } else {
                queue.push(record);
            }
        }
        Self { queue }
    }
    pub fn get_ids(path: &str, processed: &mut HashSet<u32>) {
        let mut reader = Reader::from_path(path).expect("failed to read file {:?}", path);
        for result in reader.deserialize() {
            let record: AlgloboTransaction = result.expect("failed to parse file {:?}", path);
            processed.insert(record.id);
        }
    }
    pub fn pop(&self) -> Option<AlgloboTransaction> {
        let transaction = self.queue.pop_front();
        transaction
    }
}
struct Queue {
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
        let mut data = self.data.lock().expect("poison");
        while data.len() == 0 {
            data = self.cv_empty.wait(data).expect("poison");
            break;
        }
        data.pop_front()
    }
}
