extern crate log4rs;
extern crate log;

mod logging;

use log::{error,warn,info,debug,trace};

fn main() {
    logging::init();

    error!("Error");
    warn!("Warn");
    info!("info");
    debug!("debug");
    trace!("trace");
}
