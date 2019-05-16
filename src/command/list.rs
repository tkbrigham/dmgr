use clap::{App, Arg, ArgMatches, SubCommand};
use prettytable::format;
use prettytable::Table;
use prettytable::*;

use command::DmgrResult;
use command::{Runnable, Subcommand};
use config::ServiceRegistry;
use std::thread;
use std::time::Instant;

#[derive(Debug)]
pub struct ListRunner<'a> {
    pub args: &'a ArgMatches<'a>,
}

impl<'a> Subcommand for ListRunner<'a> {
    const NAME: &'static str = "list";

    fn sub_cmd() -> App<'static, 'static> {
        SubCommand::with_name(Self::NAME)
            .about("lists services")
            .alias("ls")
            // TODO: are these necessary?
            .arg(Arg::with_name("all").long("all").short("a"))
            .arg(Arg::with_name("hidden only").long("hidden").short("h"))
    }
}

impl<'a> Runnable<'a> for ListRunner<'a> {
    fn new(args: &'a ArgMatches) -> Self {
        ListRunner { args }
    }
    fn run(&self) -> DmgrResult {
        let start = Instant::now();
        println!("start: {:?}", start.elapsed());
        let reg = ServiceRegistry::get()?;
        println!("after reg: {:?}", start.elapsed());
        let mut threads = vec![];
        let services = reg.services();
        println!("after services: {:?}", start.elapsed());

        for service in services {
            println!("starting thread for {:?}: {:?}", &service.name, start.elapsed());
            threads.push(thread::spawn(move || service.row()));
        }

        let mut rows = vec![];
        for thread in threads {
            println!("joining thread: {:?}", start.elapsed());
            rows.push(thread.join());
        }

        let header: Vec<&str> = vec!["Service", "Status", "Ports"];
        let t = TableBuilder::new().header(header);
        println!("after table builder: {:?}", start.elapsed());

        rows.into_iter()
            .filter_map(Result::ok)
            .fold(t, |t, r| t.add_row(r))
            .build()
            .printstd();

        println!("after rows.into_iter(): {:?}", start.elapsed());

        Ok(())
    }
}

struct TableBuilder {
    pub table: Table,
    rows: Vec<Vec<String>>,
}

impl TableBuilder {
    fn new() -> TableBuilder {
        let mut table = Table::new();
        let format = format::FormatBuilder::new()
            .column_separator(' ')
            .separator(
                format::LinePosition::Title,
                format::LineSeparator::new('-', ' ', ' ', ' '),
            )
            .padding(0, 1)
            .build();
        table.set_format(format);

        TableBuilder {
            table,
            rows: vec![],
        }
    }

    fn header<T>(mut self, header: Vec<T>) -> Self
    where
        T: AsRef<str> + std::fmt::Display,
    {
        self.table.set_titles(Row::from(header));
        self
    }

    fn add_row<T>(mut self, row: Vec<T>) -> Self
    where
        T: AsRef<str> + std::fmt::Display,
    {
        self.table.add_row(Row::from(row));
        self
    }

    fn build(self) -> Table {
        self.table
    }
}
