pub mod start;
pub mod list;

use clap::{App,ArgMatches};

pub trait Runnable<'a> {
    fn new(args: &'a ArgMatches<'a>) -> Self;
    fn run(&self) -> Result<DmgrSuccess, DmgrError>;
}

pub trait Subcommand {
    const NAME: &'static str;

    fn sub_cmd() -> App<'static, 'static>;
}

#[derive(Debug)]
pub struct DmgrSuccess {
    content: String
}

impl DmgrSuccess {
    pub fn new(content: &str) -> Self { DmgrSuccess { content: String::from(content) } }
}

#[derive(Debug)]
pub struct DmgrError {
    content: String
}

impl DmgrError {
    pub fn new(content: &str) -> Self { DmgrError { content: String::from(content) } }
}
