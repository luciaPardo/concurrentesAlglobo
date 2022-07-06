use csv::Reader;
use std::collections::{HashSet, VecDeque};
use std::io::Result;

use helpers::alglobo_transaction::AlgloboTransaction;

pub struct PaymentsQueue {
    queue: VecDeque<AlgloboTransaction>,
}

impl PaymentsQueue {
    pub fn new(
        pending_payments_file: &str,
        processed_file_path: &str,
        failed_file_path: &str,
    ) -> Result<Self> {
        let mut queue = VecDeque::new();

        let processed_ids = Self::load_processed_ids(failed_file_path, processed_file_path)?;

        let mut reader = Reader::from_path(pending_payments_file)?;
        for result in reader.deserialize() {
            let record: AlgloboTransaction = result?;
            if processed_ids.contains(&record.id) {
                continue;
            } else {
                queue.push_back(record);
            }
        }

        Ok(Self { queue })
    }

    pub fn load_processed_ids(
        failed_file_path: &str,
        processed_file_path: &str,
    ) -> Result<HashSet<u32>> {
        let mut processed = HashSet::new();
        if !matches!(Self::try_get_ids(failed_file_path, &mut processed), Ok(e) if e > 0) {
            let _ = std::fs::remove_file(failed_file_path);
        }

        if !matches!(Self::try_get_ids(processed_file_path, &mut processed), Ok(e) if e > 0) {
            let _ = std::fs::remove_file(processed_file_path);
        }

        Ok(processed)
    }

    /// Tries to load IDs from the specified path into the processed set.
    ///
    /// This function will not fail if the file does not exist or cannot be
    /// read, but could fail if the file cannot be deserialized correctly.
    pub fn try_get_ids(path: &str, processed: &mut HashSet<u32>) -> Result<usize> {
        let mut ignored_transactions = 0;
        if let Ok(mut reader) = Reader::from_path(path) {
            for result in reader.deserialize() {
                let record: AlgloboTransaction = result?;
                processed.insert(record.id);
                ignored_transactions += 1;
            }
        }
        Ok(ignored_transactions)
    }

    pub fn pop(&mut self) -> Option<AlgloboTransaction> {
        self.queue.pop_front()
    }
}
