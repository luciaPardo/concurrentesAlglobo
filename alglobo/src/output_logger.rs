use helpers::alglobo_transaction::AlgloboTransaction;
extern crate csv;
use csv::Writer;
use std::fs::File;

pub struct OutputLogger {
    failed_writer: Writer<File>,
    processed_writer: Writer<File>,
}

impl OutputLogger {
    pub fn new(failed_path: String, processed_path: String) -> Self {
        let failed_writer =
            Writer::from_path(failed_path).expect("error opening into processed file");
        let processed_writer =
            Writer::from_path(processed_path).expect("error opening into failed file");
        Self {
            failed_writer,
            processed_writer,
        }
    }

    pub fn log_success(&mut self, transaction: &AlgloboTransaction) {
        self.processed_writer
            .serialize(transaction)
            .expect("error when logging successful transaction");
        self.processed_writer.flush().expect("failed to flush");
    }

    pub fn log_failed(&mut self, transaction: &AlgloboTransaction) {
        self.failed_writer
            .serialize(transaction)
            .expect("error when logging failed transaction");
        self.failed_writer.flush().expect("failed to flush");
    }
}
