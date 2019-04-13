use clap::{App, Arg, ArgMatches, SubCommand};

use std::env;
use std::io;
use std::path::PathBuf;

use log::{info, warn};

use command::DmgrErr;
use command::DmgrResult;
use command::{Runnable, Subcommand};
use config::ServiceRegistry;
use constants;

#[derive(Debug)]
pub struct RegisterRunner<'a> {
    pub args: &'a ArgMatches<'a>,
}

impl<'a> Subcommand for RegisterRunner<'a> {
    const NAME: &'static str = "register";

    fn sub_cmd() -> App<'static, 'static> {
        SubCommand::with_name(Self::NAME)
            .about("manages registry")
            .arg(
                Arg::with_name("service")
                    .help("service to register; when empty, registers all default services")
                    .conflicts_with("recursive")
                    .conflicts_with("all"),
            )
            .arg(
                Arg::with_name("delete")
                    .help("delete service from registry")
                    .long("delete")
                    .short("d")
                    .requires("service")
                    .conflicts_with("recursive")
                    .conflicts_with("all"),
            )
            .arg(
                Arg::with_name("all")
                    .help("register all services in a directory (not just default ones)")
                    .long("all")
                    .short("a"),
            )
            .arg(
                Arg::with_name("recursive")
                    .help("recursively search for services to register")
                    .long("recursive")
                    .short("r")
                    .value_name("MAX_DEPTH")
                    .default_value("3"), // implicitly sets .takes_value(true)
            )
    }
}

impl<'a> Runnable<'a> for RegisterRunner<'a> {
    fn new(args: &'a ArgMatches) -> Self {
        RegisterRunner { args }
    }

    fn run(&self) -> DmgrResult {
        match self.args {
            //            r if r.occurrences_of("recursive") != 0 => register_recursive(r),
            //            a if a.is_present("all") => register_all(a),
            d if d.is_present("delete") => unregister(d),
            s if s.is_present("service") => register_single(s),
            default => register_default(default),
        }
    }
}

fn register_single<'a>(args: &'a ArgMatches) -> DmgrResult {
    let svc = args.value_of("service").unwrap();
    info!("registering service '{}'", svc);

    let cfg = find_svc_config(svc, env::current_dir())?;

    let registry = ServiceRegistry::get()?;
    registry.add_cfg(cfg)?.save()?;

    info!("successfully added service {:?}", svc);

    Ok(())
}

fn find_svc_config(svc: &str, cwd: io::Result<PathBuf>) -> Result<PathBuf, DmgrErr> {
    let config_file = cwd?
        .join(constants::SERVICE_CONFIG_DIR)
        .join(svc)
        .with_extension("json");

    config_file
        .canonicalize()
        .map_err(|_| dmgr_err!("unable to find a config file for service '{}'", svc))
}

fn unregister<'a>(args: &'a ArgMatches) -> DmgrResult {
    let svc = args.value_of("service").unwrap();
    let mut registry = ServiceRegistry::get()?;

    match registry.content.remove(svc) {
        Some(_) => info!("Successfully removed {:?} from service registry", svc),
        None => warn!("no entry {:?} found in {:?}", svc, registry.path),
    };

    registry.save()?;

    Ok(())
}

fn register_default<'a>(_args: &'a ArgMatches) -> DmgrResult {
    println!("running default register");
    Ok(())
}

//fn register_recursive<'a>(args: &'a ArgMatches) -> DmgrResult {
//    println!("running recursive");
//    Ok(())
//}
//
//fn register_all<'a>(args: &'a ArgMatches) -> DmgrResult {
//    println!("running all");
//    Ok(())
//}

// TODO
//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    mod find_svc_config {
//        use super::*;
//        use std::fmt::Error;
//        use command::DmgrErr;
//
//        #[test]
//        fn returns_err_if_cwd_is_err() {
//            let res: io::Result<PathBuf> = env::current_dir();
//            let cfg = find_svc_config("test", res);
//        }
//    }
//}
