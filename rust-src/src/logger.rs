use colored::Colorize;
use log::{Level, Metadata, Record};

pub struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let debug_prefix = match record.level() {
            Level::Error => "ERROR".bright_red(),
            Level::Warn => "WARN".bright_yellow(),
            Level::Info => "INFO".green(),
            Level::Debug => "DEBUG".bright_blue(),
            Level::Trace => "TRACE".into(),
        };
        println!(
            "{} - {}: {}",
            debug_prefix,
            record.module_path().unwrap_or_default().bright_black(),
            record.args()
        );
    }

    fn flush(&self) {}
}
