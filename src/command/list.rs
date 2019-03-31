use command::{Runnable,Subcommand};
use clap::{App,Arg,ArgMatches,SubCommand};
use command::{DmgrError,DmgrSuccess};

#[derive(Debug)]
pub struct ListRunner<'a> {
    pub args: &'a ArgMatches<'a>
}

impl<'a> Runnable<'a> for ListRunner<'a> {
    fn new(args: &'a ArgMatches) -> Self { ListRunner { args } }
    fn run(&self) -> Result<DmgrSuccess, DmgrError> { Ok(DmgrSuccess::new("list runner")) }
}

impl<'a> Subcommand for ListRunner<'a> {
    const NAME: &'static str = "list";

    fn sub_cmd() -> App<'static, 'static> {
        SubCommand::with_name(Self::NAME)
            .about("lists services")
            .arg(Arg::with_name("all")
                .long("all")
                .short("a")
            )
            .arg(Arg::with_name("hidden only")
                .long("hidden")
                .short("h")
            )
    }
}
