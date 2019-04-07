extern crate home;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::console::Target;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::threshold::ThresholdFilter;

use std::path::PathBuf;

const LOG_LVL: LevelFilter = log::LevelFilter::Trace;
const LOG_PAT: &'static str = "{h({d(%Y-%m-%d %H:%M:%S%.3f)} - {l} - {m})}\n";

pub fn init() {
    let (dmgr_name, dmgr_appender) = dmgr_logger();
    let (console_name, console_appender) = console_logger();

    let config = Config::builder()
        .appender(console_appender)
        .appender(dmgr_appender)
        .build(
            Root::builder()
                .appender(console_name)
                .appender(dmgr_name)
                .build(LOG_LVL),
        )
        .unwrap();

    let _handle = log4rs::init_config(config); // may want to actually use this?
}

// logs all dmgr activity
fn dmgr_logger() -> (&'static str, Appender) {
    let name = "dmgr";
    let log_path: Result<PathBuf, &'static str> = match home::home_dir() {
        Some(path) => Ok(path.join(format!("{}.test.log", &name))),
        None => Err("problem determining home dir"),
    };

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(LOG_PAT)))
        .build(log_path.unwrap())
        .unwrap();

    (name, Appender::builder().build(name, Box::new(logfile)))
}

// logs to console
fn console_logger() -> (&'static str, Appender) {
    let name = "console";
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(LOG_PAT)))
        .target(Target::Stdout)
        .build();

    let appender = Appender::builder()
        .filter(Box::new(ThresholdFilter::new(LOG_LVL)))
        .build(name, Box::new(stdout));
    (name, appender)
}

#[allow(dead_code)]
// might need a special logger for formatting
fn table_logger() {}
