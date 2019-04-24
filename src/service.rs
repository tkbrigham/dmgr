extern crate home;
extern crate serde_json;

use sysinfo::SystemExt;

use command::DmgrResult;
use config::Runfile;
use config::ServiceConfigContent;
use std::ffi::OsStr;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::net::TcpListener;
use std::path::PathBuf;
use std::thread;
use std::thread::JoinHandle;

#[derive(Debug, Clone)]
pub struct Service {
    pub name: String,
    pub repo_path: PathBuf,
    pub config_file: PathBuf,
    pub start_process: Option<ServiceCommand>,
    pub start_dev_mode: Option<ServiceCommand>,
    pub start_container: Option<ServiceCommand>,
    pub http_check: Option<String>,
    pub health_checks: Vec<String>,
    pub image_name: Option<String>, // consider making getter that falls back to name
    pub image_tag: Option<String>,
    //    log_file: PathBuf, TODO make function
    pub aliases: Vec<String>,
    pub ports: Vec<u16>,
    pub requires_sudo: bool,
    pub register_by_default: bool,
}

impl Service {
    pub fn from_path(path: &PathBuf) -> DmgrResult<Self> {
        let e = dmgr_err!("unable to find service config {:?}", path);
        let canonical_path = path.canonicalize().map_err(|_| e)?;
        let config_content = ServiceConfigContent::from_path(&canonical_path)?;

        let svc = Service {
            name: path_to_svc_name(&canonical_path)
                .to_os_string()
                .into_string()?,
            repo_path: repo_path_for(&canonical_path),
            config_file: PathBuf::from(canonical_path),
            start_process: config_content.start_process,
            start_dev_mode: config_content.start_dev_mode,
            start_container: config_content.start_container,
            http_check: config_content.http_check,
            health_checks: config_content.health_checks.unwrap_or(vec![]),
            image_name: config_content.image_name,
            image_tag: None,
            aliases: config_content.aliases.unwrap_or(vec![]),
            ports: config_content.ports.unwrap_or(vec![]),
            requires_sudo: config_content.requires_sudo.unwrap_or(false),
            register_by_default: config_content.register_by_default.unwrap_or(true),
        };

        Ok(svc)
    }

    pub fn log_file(&self) -> DmgrResult<PathBuf> {
        let home = home::home_dir().ok_or(dmgr_err!("could not determine home dir"))?;
        Ok(home
            .join(".solo")
            .join("log")
            .join(format!("{}.log", self.name)))
    }

    pub fn run_file(&self) -> DmgrResult<PathBuf> {
        let home = home::home_dir().ok_or(dmgr_err!("could not determine home dir"))?;
        Ok(home
            .join(".solo")
            .join("run")
            .join(format!("{}.json", self.name)))
    }

    pub fn pid(&self) -> DmgrResult<i32> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(self.run_file()?)?;

        let runfile: Runfile = serde_json::from_reader(file)?;

        Ok(runfile.pid)
    }

    //    pub fn from(s: &str) -> DmgrResult<Self> {
    //        Self::from_path(&PathBuf::from(s))
    //    }

    pub fn from_name(s: &str) -> DmgrResult<Self> {
        Self::from_path(&PathBuf::from(s))
    }

    pub fn from_alias(s: &str) -> DmgrResult<Self> {
        Self::from_path(&PathBuf::from(s))
    }

    pub fn is_running(&self) -> bool {
        self.ports_all_open() && self.pid_is_running()
    }

    pub fn row(self) -> Vec<String> {
        vec![self.name.clone(), self.is_running().to_string().clone()]
    }

    pub fn ports_all_open(&self) -> bool {
        let mut threads: Vec<JoinHandle<bool>> = vec![];

        for port in self.ports.clone() {
            threads.push(thread::spawn(move || port_is_available(port)));
        }

        threads
            .into_iter()
            .fold(true, |b, t| b && t.join().unwrap())
    }

    pub fn pid_is_running(&self) -> bool {
        let system = sysinfo::System::new();
        let p = match self.pid() {
            Ok(pid) => pid,
            Err(_) => return false,
        };

        match system.get_process(p) {
            Some(_) => true,
            None => false,
        }
    }

    fn new() -> Service {
        Service {
            name: String::from(""),
            repo_path: PathBuf::new(),
            config_file: PathBuf::new(),
            start_process: None,
            start_dev_mode: None,
            start_container: None,
            http_check: None,
            health_checks: vec![],
            image_name: None,
            image_tag: None,
            aliases: vec![],
            ports: vec![],
            requires_sudo: false,
            register_by_default: true,
        }
    }

    //    fn http_check_passing() -> bool {
    //
    //    }
    //
    //    fn port_check() -> bool {
    //
    //    }
    //
    //    fn passes_pid_check() {}

    pub fn update_runfile(&self, r: Runfile) -> DmgrResult {
        let mut file = File::create(self.run_file()?)?;
        let mut content = serde_json::to_string_pretty(&r)?;
        content.push_str("\n");

        file.write_all(content.as_bytes())?;
        Ok(())
    }
}

pub type ServiceCommand = String;

// TODO: this is implemented in register.rs too
fn path_to_svc_name(p: &PathBuf) -> &OsStr {
    p.file_stem().unwrap_or(OsStr::new("UNKNOWN"))
}

fn repo_path_for(canonical_path: &PathBuf) -> PathBuf {
    let repo = canonical_path.parent().and_then(|p| p.parent());
    PathBuf::from(repo.unwrap())
}

fn port_is_available(port: u16) -> bool {
    match TcpListener::bind(("0.0.0.0", port)) {
        Ok(_) => true,
        Err(_) => false,
    }
}
