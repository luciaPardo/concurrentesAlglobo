use helpers::alglobo_transaction::AlgloboTransaction;
extern crate csv;
use csv::{Writer, WriterBuilder};
use std::{fs::File, io::Result, path::Path};

pub struct OutputLogger {
    failed_writer: Writer<File>,
    processed_writer: Writer<File>,
}

impl OutputLogger {
    pub fn new(failed_path: String, processed_path: String) -> Result<Self> {
        let failed_writer = Self::create_or_append(failed_path)?;
        let processed_writer = Self::create_or_append(processed_path)?;

        Ok(Self {
            failed_writer,
            processed_writer,
        })
    }

    /// Tries to open an existing file with append mode, or create it
    /// if it does not exist.
    ///
    /// This method also adds the header in the case the file does not
    /// exist. It would be great to have some method in csv::Writer that
    /// does so automatically, but it seems there isn't one yet.
    fn create_or_append<P: AsRef<Path>>(path: P) -> Result<Writer<File>> {
        // TODO: There is and edge case where the file exists but it has no
        // header written, it is incomplete or empty. We should do a better
        // handling for those cases.
        if let Ok(file) = File::options().append(true).open(&path) {
            Ok(WriterBuilder::new().has_headers(false).from_writer(file))
        } else {
            Ok(Writer::from_writer(File::create(path)?))
        }
    }

    pub fn log_success(&mut self, transaction: &AlgloboTransaction) {
        self.processed_writer
            .serialize(transaction)
            .expect("cannot write to successful transaction log");
        self.processed_writer
            .flush()
            .expect("cannot write to successful transaction log");
    }

    pub fn log_failed(&mut self, transaction: &AlgloboTransaction) {
        self.failed_writer
            .serialize(transaction)
            .expect("cannot write to failed transaction log");
        self.failed_writer
            .flush()
            .expect("cannot write to successful transaction log");
    }
}
