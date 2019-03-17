extern crate log4rs;
extern crate log;

mod logging;
mod runner;

use log::{error,warn,info,debug,trace};

fn main() {
    logging::init();

    if true { // TODO: temp way to quickly enabled/disable execution of logging
        error!("Error");
        warn!("Warn");
        info!("info");
        debug!("debug");
        trace!("trace");
    }

    runner::run();
}
