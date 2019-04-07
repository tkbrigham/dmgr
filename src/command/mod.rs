pub mod list;
pub mod register;
pub mod start;

use clap::{App, ArgMatches};
use std::fmt;

pub trait Runnable<'a> {
    fn new(args: &'a ArgMatches<'a>) -> Self;
    fn run(&self) -> DmgrResult;
}

pub trait Subcommand {
    const NAME: &'static str;

    fn sub_cmd() -> App<'static, 'static>;
}

pub type DmgrResult = Result<(), DmgrErr>;

#[derive(Debug)]
pub struct DmgrErr {
    msg: String,
}

impl DmgrErr {
    pub fn new(content: &str) -> Self {
        DmgrErr {
            msg: String::from(content),
        }
    }
}

impl fmt::Display for DmgrErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
