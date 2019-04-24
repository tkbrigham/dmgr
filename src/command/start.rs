extern crate shlex;

use clap::{App, Arg, ArgMatches, SubCommand};
use log::info;

use command::DmgrResult;
use command::{Runnable, Subcommand};
use config::ServiceRegistry;
use service::Service;
use std::fs::create_dir_all;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process::Command;
use config::Runfile;
use config::Pid;

#[derive(Debug)]
pub struct StartRunner<'a> {
    pub args: &'a ArgMatches<'a>,
}

impl<'a> Subcommand for StartRunner<'a> {
    const NAME: &'static str = "start";

    fn sub_cmd() -> App<'static, 'static> {
        SubCommand::with_name(Self::NAME)
            .about("start a service or group")
            .arg(
                Arg::with_name("service_or_group")
                    .help("service or group to start")
                    .required_unless("all")
                    .conflicts_with("all"),
            )
            .arg(
                Arg::with_name("all")
                    .help("start all registered services")
                    .long("all")
                    .short("a")
                    .conflicts_with("service_or_group"),
            )
            .arg(
                Arg::with_name("container")
                    .help("start service as a container")
                    .long("container")
                    .short("c")
                    .conflicts_with("dev_mode"),
            )
            .arg(
                Arg::with_name("dev_mode")
                    .help("start service in dev mode")
                    .long("dev_mode")
                    .short("d")
                    .conflicts_with("container"),
            )
            .arg(
                Arg::with_name("update_image")
                    .help("pull updated image for container")
                    .long("update_image")
                    .short("u")
                    .requires("container"),
            )
            .arg(
                Arg::with_name("attach")
                    .help("attach your console to command")
                    .long("attach")
                    .short("t")
                    .conflicts_with("all"),
            )
    }
}

impl<'a> Runnable<'a> for StartRunner<'a> {
    fn new(args: &'a ArgMatches) -> Self {
        StartRunner { args }
    }

    fn run(&self) -> DmgrResult {
        println!("Matches = {:#?}", self.args);

        match self.args {
            //            a if a.is_present("all") => start_all(a),
            //            c if c.is_present("container") => start_container(c),
            //            d if d.is_present("dev_mode") => start_dev_mode(d),
            default => start(default),
        }
    }
}

fn start<'a>(args: &'a ArgMatches) -> DmgrResult {
    let svc_name = args.value_of("service_or_group").unwrap();
    let attached = args.is_present("attach");
    println!("attached = {:?}", attached);
    let svc = ServiceRegistry::get()?.get_service(svc_name)?;

    // TODO: handle if arg is group
    start_as_process(svc, attached)
}

fn start_as_process(svc: Service, attached: bool) -> DmgrResult {
    info!("starting {:?} as process...", &svc.name);

    let cmd = cmd_for(&svc.start_process)?;
    if attached {
        start_attached(cmd)
    } else {
        spawn(svc, cmd)
    }
}

fn cmd_for(cmd: &Option<String>) -> DmgrResult<Command> {
    let start = cmd
        .clone()
        .ok_or(dmgr_err!("no start_process script found"))?;
    let args =
        shlex::split(start.as_str()).ok_or(dmgr_err!("problem parsing command: {:?}", start))?;

    let mut cmd = Command::new(&args[0]);
    cmd.args(args[1..].iter());
    Ok(cmd)
}

fn start_attached(mut cmd: Command) -> DmgrResult {
    let status = cmd.status()?;
    info!("exited with status {:?}", status.code().unwrap());
    Ok(())
}

fn spawn(svc: Service, mut cmd: Command) -> DmgrResult {
    let child = cmd.stderr(out_file(&svc)?)
        .stdout(out_file(&svc)?)
        .spawn()?;


    let runfile = Runfile { pid: child.id() as Pid, is_container: false };
    println!("da file = {:?}", runfile);
    svc.update_runfile(runfile)?;

//    wait_for_service(svc);

    Ok(())
}

//fn wait_for_service(svc: Service) -> DmgrResult {
//    Ok(())
//}

fn out_file(svc: &Service) -> DmgrResult<File> {
    let log_path = svc.log_file()?;
    if !log_path.exists() {
        ensure_parent_dir_exists(&log_path)?;
    }

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    Ok(file)
}

fn ensure_parent_dir_exists(filepath: &PathBuf) -> DmgrResult {
    let parent_dir = filepath.parent().ok_or(dmgr_err!(
        "could not determine parent dir of filepath {:?}",
        &filepath
    ))?;
    create_dir_all(parent_dir)?;
    Ok(())
}

//fn start_all<'a>(args: &'a ArgMatches) -> DmgrResult {
//    Ok(())
//}
//
//fn start_container<'a>(args: &'a ArgMatches) -> DmgrResult {
//    Ok(())
//}
//
//fn start_dev_mode<'a>(args: &'a ArgMatches) -> DmgrResult {
//    Ok(())
//}
//
//fn start_group(name: &str) -> DmgrResult {
//    Ok(())
//}
