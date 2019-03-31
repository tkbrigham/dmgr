extern crate log4rs;
extern crate log;
extern crate clap;

mod logging;
mod runner;
mod command;
mod args;

use log::{error,warn,info,debug,trace};

use command::{Runnable,Subcommand,DmgrError,DmgrSuccess};
use command::list::ListRunner;
use command::start::StartRunner;
use clap::ArgMatches;

fn main() {
    logging::init();

    if false { // TODO: temp way to quickly enabled/disable execution of logging
        error!("Error");
        warn!("Warn");
        info!("info");
        debug!("debug");
        trace!("trace");
    }

    if false {
        runner::run();
    }

    let app = args::new();
    let matches = app.get_matches();

    println!("Matches = {:?}", matches);

    fn run(matches: ArgMatches) -> Result<DmgrSuccess,DmgrError> {
        match matches.subcommand() {
            (ListRunner::NAME, Some(list_args)) => ListRunner { args: list_args }.run(),
            (StartRunner::NAME, Some(start_args)) => StartRunner { args: start_args }.run(),
            _ => Err(DmgrError::new("unknown"))
        }
    }

    let t = run(matches);
    println!("{:?}", t);
}
