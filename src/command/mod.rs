extern crate backtrace;
extern crate serde;
extern crate serde_json;
extern crate toml;

use self::backtrace::Backtrace;
use self::serde_json::Error as JsonErr;
use self::toml::de;
use clap::{App, ArgMatches};

use std::fmt;
use std::io;

pub mod list;
pub mod register;
pub mod start;

pub trait Runnable<'a> {
    fn new(args: &'a ArgMatches<'a>) -> Self;
    fn run(&self) -> DmgrResult;
}

pub trait Subcommand {
    const NAME: &'static str;

    fn sub_cmd() -> App<'static, 'static>;
}

pub type DmgrResult<T = ()> = Result<T, DmgrErr>;

#[derive(Debug)]
pub struct DmgrErr {
    msg: String,
    pub stacktrace: Backtrace,
}

impl DmgrErr {
    pub fn new(content: &str) -> Self {
        let bt = backtrace::Backtrace::new();
        DmgrErr {
            msg: String::from(content),
            stacktrace: bt,
        }
    }
}

impl fmt::Display for DmgrErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<io::Error> for DmgrErr {
    fn from(err: io::Error) -> Self {
        DmgrErr::new(err.to_string().as_str())
    }
}

impl From<String> for DmgrErr {
    fn from(err: String) -> Self {
        DmgrErr::new(err.as_str())
    }
}

impl From<de::Error> for DmgrErr {
    fn from(err: de::Error) -> Self {
        DmgrErr::new(&err.to_string())
    }
}

impl From<JsonErr> for DmgrErr {
    fn from(err: JsonErr) -> Self {
        DmgrErr::new(&err.to_string())
    }
}
