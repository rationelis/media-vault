use log::{LevelFilter, Metadata, Record, SetLoggerError};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};

pub struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            eprintln!("{} [{}] {}", record.level(), record.target(), record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init_logger(level: String) -> Result<(), SetLoggerError> {
    let level = match level.as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Off,
    };

    let config = Config::default();

    TermLogger::init(level, config, TerminalMode::Mixed, ColorChoice::Auto)
}
