use csv::Reader;
use std::collections::HashSet;
use std::io::Result;

use crate::queue::Queue;
use helpers::alglobo_transaction::AlgloboTransaction;

pub struct PaymentsQueue {
    queue: Queue,
}

impl PaymentsQueue {
    pub fn new(
        size: usize,
        pending_payments_file: &str,
        processed_file_path: &str,
        failed_file_path: &str,
    ) -> Result<Self> {
        let queue = Queue::new(size);

        let processed_ids = Self::load_processed_ids(failed_file_path, processed_file_path)?;

        let mut reader = Reader::from_path(pending_payments_file)?;
        for result in reader.deserialize() {
            let record: AlgloboTransaction = result?;
            if processed_ids.contains(&record.id) {
                continue;
            } else {
                queue.push(record);
            }
        }

        Ok(Self { queue })
    }

    pub fn load_processed_ids(
        failed_file_path: &str,
        processed_file_path: &str,
    ) -> Result<HashSet<u32>> {
        let mut processed = HashSet::new();
        Self::try_get_ids(failed_file_path, &mut processed)?;
        Self::try_get_ids(processed_file_path, &mut processed)?;
        Ok(processed)
    }

    /// Tries to load IDs from the specified path into the processed set.
    ///
    /// This function will not fail if the file does not exist or cannot be
    /// read, but could fail if the file cannot be deserialized correctly.
    pub fn try_get_ids(path: &str, processed: &mut HashSet<u32>) -> Result<()> {
        if let Ok(mut reader) = Reader::from_path(path) {
            for result in reader.deserialize() {
                let record: AlgloboTransaction = result?;
                processed.insert(record.id);
            }
        }
        Ok(())
    }

    pub fn pop(&self) -> Option<AlgloboTransaction> {
        self.queue.pop_front()
    }
}
