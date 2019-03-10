extern crate log;
extern crate log4rs;

use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Logger, Root};

pub fn main() -> Result<log4rs::Handle, log::SetLoggerError> {
    let stdout = ConsoleAppender::builder().build();

    let requests = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build("log/requests.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("requests", Box::new(requests)))
        .logger(Logger::builder().build("app::backend::db", LevelFilter::Info))
        .logger(Logger::builder()
            .appender("requests")
            .additive(false)
            .build("app::requests", LevelFilter::Info))
        .build(Root::builder().appender("stdout").build(LevelFilter::Warn))
        .unwrap();

    // use handle to change logger configuration at runtime
    log4rs::init_config(config)
}