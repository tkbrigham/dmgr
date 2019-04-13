#![allow(dead_code)]

extern crate clap;
extern crate log;
extern crate log4rs;
extern crate prettytable;

#[macro_use]
mod macros;

mod args;
mod command;
mod config;
mod constants;
mod logging;
mod runner;

use clap::ArgMatches;
use log::{debug, error, info, trace, warn};

use command::list::ListRunner;
use command::register::RegisterRunner;
use command::start::StartRunner;
use command::{DmgrErr, DmgrResult, Runnable, Subcommand};
use std::process;

fn main() -> DmgrResult {
    logging::init();

    // TODO: will print back trace
    if false {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    if false {
        // TODO: temp way to quickly enabled/disable execution of logging
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

    //    println!("Matches = {:#?}", matches);

    fn run(matches: ArgMatches) -> DmgrResult {
        match matches.subcommand() {
            (ListRunner::NAME, Some(args)) => ListRunner { args }.run(),
            (StartRunner::NAME, Some(args)) => StartRunner { args }.run(),
            (RegisterRunner::NAME, Some(args)) => RegisterRunner { args }.run(),
            _ => Err(DmgrErr::new("unknown")),
        }
    }

    if let Err(e) = run(matches) {
        error!("{}\n{:?}", e, e.stacktrace);
        process::exit(1)
    }

    Ok(())

    //    println!("{:?}", t);
    //
    //    println!("#################");
    //    let ffm_config = "/Users/tkbrigham/developer/socrata/feature-flag-monitor/.solo/feature-flag-monitor.toml";
    //    let my = ServiceConfig::from(ffm_config);
    //    println!("{:?}", my);
    //
    //    println!("#################");
    //    println!("{:?}", my.to_registry_entry());
    //
    //    println!("#################");
    //    let svc_registry_toml = "/Users/tkbrigham/.solo-registry.toml";
    //    let r = ServiceRegistry::from(svc_registry_toml);
    //    println!("{:#?}", r);
    //
    //    let svc_registry_json = "/Users/tkbrigham/.solo/service-registry.json";
    //    let r = ServiceRegistry::from(svc_registry_json);
    //    println!("{:#?}", r);
}
