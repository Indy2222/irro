use lazy_static::lazy_static;
use log::{Log, Metadata, Record};
use std::env;
use systemd::journal;

lazy_static! {
    static ref USE_JOURNALD: bool = env::var("INVOCATION_ID").is_ok();
}

pub struct IrroLogger;

impl Log for IrroLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if *USE_JOURNALD {
                journal::log_record(record);
            } else {
                println!("{}: {}", record.level(), record.args());
            }
        }
    }

    fn flush(&self) {}
}
