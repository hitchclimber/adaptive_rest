use log::{Level, Log};
use tokio::sync::mpsc::UnboundedSender;

use crate::util::{error::InternalError, result::InternalResult};

pub struct TuiLogger {
    sender: UnboundedSender<String>,
    level: Level,
}

impl TuiLogger {
    pub fn init(sender: UnboundedSender<String>, level: Level) -> InternalResult<()> {
        let logger = Box::new(TuiLogger { sender, level });
        log::set_max_level(level.to_level_filter());
        log::set_boxed_logger(logger).map_err(|_| InternalError::LoggerInitError)?;
        Ok(())
    }
}

impl Log for TuiLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let msg = format!("[{}] {}", record.level(), record.args());
            let _ = self.sender.send(msg);
        }
    }

    fn flush(&self) {}
}
