use clap::{App, Arg, ArgMatches, SubCommand};
use prettytable::format;
use prettytable::Table;
use prettytable::*;

use command::DmgrResult;
use command::{Runnable, Subcommand};
use config::ServiceRegistry;
use std::thread;
use std::time;


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
        const SVC_REG: &str = "/Users/tkbrigham/.solo/service-registry.json";
        let reg = ServiceRegistry::from(SVC_REG)?;

        let mut threads = vec![];
        for (i, service) in reg.services().into_iter().enumerate() {
            threads.push(thread::spawn(move || {
                let now = time::Instant::now();

                println!("thread {} is booting", service.name);
                let row = service.row();

                let now = time::Instant::now();
                println!("thread {} took {:?}", service.name, now.elapsed());
                row
            }));
        }

        let mut rows = vec![];
        for thread in threads {
           rows.push(thread.join())
        }

        println!("results = {:?}", rows);
        let header: Vec<&str> = vec!["Service", "Status", "Ports"];
        let t = TableBuilder::new().header(header);
//
        rows.into_iter()
            .filter_map(Result::ok)
            .fold(t, |t, r| t.add_row(r))
            .build()
            .printstd();

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
