extern crate shlex;

use clap::{App, Arg, ArgMatches, SubCommand};
use log::{info, warn};
use sysinfo::ProcessExt;
use sysinfo::Signal;
use sysinfo::SystemExt;

use command::DmgrResult;
use command::{Runnable, Subcommand};
use config::ServiceRegistry;

#[derive(Debug)]
pub struct StopRunner<'a> {
    pub args: &'a ArgMatches<'a>,
}

impl<'a> Subcommand for StopRunner<'a> {
    const NAME: &'static str = "stop";

    fn sub_cmd() -> App<'static, 'static> {
        SubCommand::with_name(Self::NAME)
            .about("stop a service or group")
            .arg(
                Arg::with_name("service_or_group")
                    .help("service or group to stop")
                    .required_unless("all")
                    .conflicts_with("all"),
            )
            .arg(
                Arg::with_name("all")
                    .help("stop all registered services")
                    .long("all")
                    .short("a")
                    .conflicts_with("service_or_group"),
            )
    }
}

impl<'a> Runnable<'a> for StopRunner<'a> {
    fn new(args: &'a ArgMatches) -> Self {
        StopRunner { args }
    }

    fn run(&self) -> DmgrResult {
        println!("Matches = {:#?}", self.args);

        match self.args {
            //            a if a.is_present("all") => stop_all(a),
            //            c if c.is_present("container") => stop_container(c),
            //            d if d.is_present("dev_mode") => stop_dev_mode(d),
            default => stop(default),
        }
    }
}

fn stop<'a>(args: &'a ArgMatches) -> DmgrResult {
    let svc_name = args.value_of("service_or_group").unwrap();
    let svc = ServiceRegistry::get()?.get_service(svc_name)?;
    info!("stopping {:?}...", &svc.name);

    let pid = svc.pid()?;
    warn!("pid is {:?}", &pid);

    let system = sysinfo::System::new();
    let p = system
        .get_process(pid as i32)
        .ok_or(dmgr_err!("could not find process with pid {:?}", pid))?;
    //    info!("proc = {:?}", p);

    let sig = Signal::Continue;
    let kill_result = p.kill(sig);

    info!("kill result = {:?}", kill_result);

    Ok(())
}

//fn stop_all<'a>(args: &'a ArgMatches) -> DmgrResult {
//    Ok(())
//}
//
//fn stop_group(name: &str) -> DmgrResult {
//    Ok(())
//}
