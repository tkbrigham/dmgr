extern crate clap;

use self::clap::*;

use command::Subcommand;
use command::list::ListRunner;
use command::start::StartRunner;

pub fn new() -> App<'static, 'static> {
    app_from_crate!()
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::SubcommandRequired)
        .subcommand(ListRunner::sub_cmd())
        .subcommand(StartRunner::sub_cmd())
}
