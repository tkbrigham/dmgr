use clap::{App, Arg, ArgMatches, SubCommand};
use command::DmgrErr;
use command::{Runnable, Subcommand};

#[derive(Debug)]
pub struct StartRunner<'a> {
    pub args: &'a ArgMatches<'a>,
}

impl<'a> Runnable<'a> for StartRunner<'a> {
    fn new(args: &'a ArgMatches) -> Self {
        StartRunner { args }
    }
    fn run(&self) -> Result<(), DmgrErr> {
        Ok(())
    }
}

impl<'a> Subcommand for StartRunner<'a> {
    const NAME: &'static str = "start";

    fn sub_cmd() -> App<'static, 'static> {
        SubCommand::with_name(Self::NAME)
            .about("start a service")
            .arg(Arg::with_name("as container").long("container").short("c"))
    }
}
