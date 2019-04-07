use clap::{App, Arg, ArgMatches, SubCommand};
use command::DmgrErr;
use command::DmgrResult;
use command::{Runnable, Subcommand};
use std::str;

#[derive(Debug)]
pub struct RegisterRunner<'a> {
    pub args: &'a ArgMatches<'a>,
}

impl<'a> Runnable<'a> for RegisterRunner<'a> {
    fn new(args: &'a ArgMatches) -> Self {
        RegisterRunner { args }
    }
    fn run(&self) -> DmgrResult {
        Ok(())
    }
}

impl<'a> Subcommand for RegisterRunner<'a> {
    const NAME: &'static str = "register";

    fn sub_cmd() -> App<'static, 'static> {
        SubCommand::with_name(Self::NAME)
            .about("manages registry")
            .arg(
                Arg::with_name("service")
                    .help("service to register; when empty, registers all default services")
                    .conflicts_with("recursive")
                    .conflicts_with("all"),
            )
            .arg(
                Arg::with_name("delete")
                    .help("delete service from registry")
                    .long("delete")
                    .short("d")
                    .requires("service")
                    .conflicts_with("recursive")
                    .conflicts_with("all"),
            )
            .arg(
                Arg::with_name("all")
                    .help("register all services in a directory (not just default ones)")
                    .long("all")
                    .short("a"),
            )
            .arg(
                Arg::with_name("recursive")
                    .help("recursively search for services to register")
                    .long("recursive")
                    .short("r")
                    .takes_value(true)
                    .value_name("MAX_DEPTH")
                    .default_value("3"),
            )
    }
}
