use log::{Log, Metadata, Record};
use systemd::journal;

pub struct IrroLogger;

impl Log for IrroLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            journal::log_record(record);
        }
    }

    fn flush(&self) {}
}
