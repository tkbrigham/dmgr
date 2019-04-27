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
use std::io::BufRead;
use std::str::FromStr;
use std::net::TcpListener;
use std::path::PathBuf;
use std::thread;
use std::thread::JoinHandle;
use std::net::TcpStream;
use std::net::SocketAddr;
use std::net::Shutdown;
use std::io::BufReader;
use command::DmgrErr;

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

    pub fn row(self) -> Vec<String> {
        vec![
            self.name.clone(),
            self.http_check_passing().to_string().clone(),
            format!("{:?}", self.ports),
        ]
    }

    pub fn is_running(&self) -> bool {
        self.pid_is_running() && !self.is_waiting()
    }

    pub fn is_waiting(&self) -> bool {
        let ports_defined_and_closed = self.ports.len() > 0 && !self.ports_all_open();
        let http_check_defined_and_failing = self.http_check.is_some() && !self.http_check_passing();
        self.pid_is_running() && (ports_defined_and_closed || http_check_defined_and_failing)
    }

    pub fn is_disowned(&self) -> bool {
        let ports_defined_and_open = self.ports.len() > 0 && self.ports_all_open();
        let http_check_defined_and_passing = self.http_check.is_some() && self.http_check_passing();
        !self.pid_is_running() && (ports_defined_and_open || http_check_defined_and_passing)
    }

    pub fn http_check_passing(&self) -> bool {
        let endpoint = match self.http_check {
            Some(ref e) => e,
            None => return true,
        };

        let addrs: Vec<SocketAddr> = self.ports
            .clone()
            .into_iter()
            .map(|p| SocketAddr::from(([0, 0, 0, 0], p)))
            .collect();

        get_success(addrs, endpoint).is_ok()
//
//
//        if let Ok(mut stream) = TcpStream::connect(&addrs[..]) {
//            stream.shutdown(Shutdown::Write).expect("shutdown call failed");
//
//            let mut buf = String::new();
//            let mut buffered = BufReader::new(stream);
//            match buffered.read_line(&mut buf);
//
//            println!("response = {:?}", buf.trim());
//
//            match http_status_code(buf.trim()) {
//                Ok(c) => {
//                    println!("da code = {:?}", c);
//                    let success = status_code_ok(c);
//                    println!("success = {:?}", success);
//                    success
//                },
//                Err(_) => false,
//            }
//        } else {
//            println!("ooh boy");
//            false
//        }
    }

    pub fn ports_all_open(&self) -> bool {
        let mut threads: Vec<JoinHandle<bool>> = vec![];

        for port in self.ports.clone() {
            threads.push(thread::spawn(move || tcp_is_available("0.0.0.0", port)));
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

fn http_status_code(line: &str) -> DmgrResult<u16> {
    let mut parts = line.split_whitespace();
    parts.next();
    let code = parts.next()
        .ok_or(dmgr_err!("could not determine HTTP status code"))?;
    u16::from_str(code)
        .map_err(|e| DmgrErr::new(&e.to_string()))
}

fn status_code_ok(code: u16) -> bool {
    code < 400
}

fn get_success(addrs: Vec<SocketAddr>, endpoint: &String) -> DmgrResult {
    let mut stream = TcpStream::connect(&addrs[..])?;
    stream.write(format!("GET {} HTTP/1.1\r\n", endpoint).as_bytes())?;
    stream.shutdown(Shutdown::Write)?;

    let mut buf = String::new();
    let mut buffered = BufReader::new(stream);
    buffered.read_line(&mut buf)?;

    let code = http_status_code(buf.trim())?;

    if !status_code_ok(code) { fail!("received error code from server: {:?}", code) }
    Ok(())
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

fn tcp_is_available(host: &str, port: u16) -> bool {
    TcpListener::bind((host, port)).is_ok()
}
