extern crate clap;

use self::clap::*;

use command::list::ListRunner;
use command::register::RegisterRunner;
use command::start::StartRunner;
use command::stop::StopRunner;
use command::Subcommand;

pub fn new() -> App<'static, 'static> {
    app_from_crate!()
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::SubcommandRequired)
        .subcommand(ListRunner::sub_cmd())
        .subcommand(StartRunner::sub_cmd())
        .subcommand(RegisterRunner::sub_cmd())
        .subcommand(StopRunner::sub_cmd())
}
