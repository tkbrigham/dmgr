extern crate backtrace;
extern crate serde;
extern crate serde_json;
extern crate toml;

use self::backtrace::Backtrace;
use self::serde_json::Error as JsonErr;
use self::toml::de;
use clap::{App, ArgMatches};

use std::ffi::OsString;
use std::fmt;
use std::io;
use std::path::StripPrefixError;

pub mod list;
pub mod register;
pub mod start;
pub mod stop;

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

impl From<String> for DmgrErr {
    fn from(err: String) -> Self {
        DmgrErr::new(err.as_str())
    }
}

impl From<io::Error> for DmgrErr {
    fn from(err: io::Error) -> Self {
        DmgrErr::new(err.to_string().as_str())
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

impl From<StripPrefixError> for DmgrErr {
    fn from(err: StripPrefixError) -> Self {
        DmgrErr::new(&err.to_string())
    }
}

impl From<OsString> for DmgrErr {
    fn from(err: OsString) -> Self {
        //        DmgrErr::new(&err.to_string().unwrap_or(format!("failed to convert {:?} to error", &err)))
        DmgrErr::new(
            &err.to_str()
                .unwrap_or(format!("failed to convert {:?} to error", &err).as_str()),
        )
    }
}
