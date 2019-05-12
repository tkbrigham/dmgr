extern crate home;
extern crate libc;
extern crate serde_json;

use libc::kill;
use log::debug;

use command::DmgrErr;
use command::DmgrResult;
use config::Runfile;
use config::ServiceConfigContent;
use std::ffi::OsStr;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::Ipv4Addr;
use std::net::Shutdown;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::net::TcpStream;
use std::ops::BitAnd;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

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

    pub fn from_name(s: &str) -> DmgrResult<Self> {
        Self::from_path(&PathBuf::from(s))
    }

    pub fn from_alias(s: &str) -> DmgrResult<Self> {
        Self::from_path(&PathBuf::from(s))
    }

    pub fn row(&self) -> Vec<String> {
        let name_clone = self.name.clone();
        //        println!("[{}] finished name_clone after {:?}", name_clone, now.elapsed());

        let status = self.typed_status().to_string();
        //        println!("[{}] finished status after {:?}", name_clone, now.elapsed());

        let ports = format!("{:?}", self.ports);
        //        println!("[{}] finished ports after {:?}", name_clone, now.elapsed());

        vec![name_clone, status, ports]
    }

    fn typed_status(&self) -> ServiceStatus {
        let has_active_pid = self.has_active_pid();

        if self.is_http() {
            let ownership_status = if has_active_pid {
                ServiceStatus::Owned
            } else {
                ServiceStatus::Disowned
            };
            self.ports
                .clone()
                .into_iter()
                .fold(ownership_status, |status, port| {
                    status & self.status_for_port(port)
                })
        } else {
            if has_active_pid {
                ServiceStatus::Running
            } else {
                ServiceStatus::Stopped
            }
        }
    }

    pub fn is_http(&self) -> bool {
        !self.ports.is_empty()
    }

    pub fn status_for_port(&self, port: u16) -> ServiceStatus {
        let addr = format!("0.0.0.0:{}", port);
        let mut stream = match TcpStream::connect(&addr) {
            Ok(s) => s,
            Err(_) => return ServiceStatus::Stopped,
        };

        stream
            .set_nodelay(true)
            .expect("could not configure TCP stream");

        match &self.http_check {
            None => ServiceStatus::Running,
            Some(endpoint) => {
                let succ = get_success(&mut stream, endpoint);
                if succ.is_ok() {
                    ServiceStatus::Running
                } else {
                    ServiceStatus::Waiting
                }
            }
        }
    }

    // Could be disowned, could be owned by dmgr
    pub fn is_running(&self) -> bool {
        self.has_active_pid() || self.has_ports_defined_and_open()
    }

    pub fn is_disowned(&self) -> bool {
        !self.has_active_pid() && self.has_ports_defined_and_open()
    }

    pub fn is_ready(&self) -> bool {
        self.is_running() && !self.is_waiting()
    }

    pub fn is_waiting(&self) -> bool {
        self.is_running()
            && (self.has_http_check_defined_and_failing()
                || self.has_ports_defined_and_all_closed())
    }

    pub fn has_http_check_defined_and_failing(&self) -> bool {
        self.http_check.is_some() && !self.http_check_passing()
    }

    pub fn has_ports_defined_and_all_closed(&self) -> bool {
        self.ports.len() > 0 && self.open_ports().len() == 0
    }

    pub fn has_ports_defined_and_open(&self) -> bool {
        self.ports.len() > 0 && self.open_ports().len() > 0
    }

    pub fn open_ports(&self) -> Vec<&u16> {
        self.ports
            .iter()
            .filter(|&&port| {
                let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port));
                tcp_is_available(&addr)
            })
            .collect()
    }

    pub fn http_check_passing(&self) -> bool {
        let endpoint = match self.http_check {
            Some(ref e) => e,
            None => return true,
        };

        let addrs: Vec<SocketAddr> = self
            .ports
            .clone()
            .into_iter()
            .map(|p| SocketAddr::from(([0, 0, 0, 0], p)))
            .collect();

        let mut stream = match TcpStream::connect(&addrs[..]) {
            Ok(s) => s,
            Err(_) => return false,
        };

        get_success(&mut stream, endpoint).is_ok()
    }

    pub fn has_active_pid(&self) -> bool {
        let p = match self.pid() {
            Ok(pid) => pid,
            Err(_) => return false,
        };

        println!("service {:?} has p {:?}", self.name, &p);

        unsafe {
            let code = kill(p, 0);
            println!("code is: {:?}", &code);
            code == 0
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
    let code = parts
        .next()
        .ok_or(dmgr_err!("could not determine HTTP status code"))?;
    u16::from_str(code).map_err(|e| DmgrErr::new(&e.to_string()))
}

fn status_code_ok(code: u16) -> bool {
    code < 400
}

fn get_success(stream: &mut TcpStream, endpoint: &String) -> DmgrResult {
    let req = format!("GET {} HTTP/1.1\r\n", endpoint);
    stream.write(req.as_bytes())?;

    let host_header = "Host: *\r\n";
    stream.write(host_header.as_bytes())?;

    stream.write("\r\n".as_bytes())?;
    stream.shutdown(Shutdown::Write)?;

    let mut buf = String::new();
    let mut buffered = BufReader::new(stream);

    buffered.read_line(&mut buf)?;
    let resp = &buf.trim();
    let code = http_status_code(resp)?;

    if !status_code_ok(code) {
        fail!("received error code from server: {:?}", code);
    }

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

fn tcp_is_available(addr: &SocketAddr) -> bool {
    let timeout = Duration::from_millis(50);
    TcpStream::connect_timeout(addr, timeout).is_ok()
}

#[derive(Debug, PartialEq)]
pub enum ServiceStatus {
    Stopped,
    Waiting,
    Running,
    Owned,
    Disowned,
    WaitingDisowned,
    RunningDisowned,
}

impl ToString for ServiceStatus {
    fn to_string(&self) -> String {
        let status = match self {
            &ServiceStatus::Stopped => "-",
            &ServiceStatus::Waiting => "waiting",
            &ServiceStatus::Running => "running",
            &ServiceStatus::WaitingDisowned => "waiting*",
            &ServiceStatus::RunningDisowned => "running*",
            s => {
                debug!("unexpected status to render: {:?}", s);
                "-"
            }
        };

        String::from(status)
    }
}

impl BitAnd for ServiceStatus {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        if self == rhs {
            return self;
        }

        match (self, rhs) {
            (ServiceStatus::Owned, e) | (e, ServiceStatus::Owned) => e, // owned is the identity property
            (ServiceStatus::Stopped, ServiceStatus::Running)
            | (ServiceStatus::Running, ServiceStatus::Stopped) => ServiceStatus::Waiting, // combining mixed signals means waiting
            (ServiceStatus::Waiting, ServiceStatus::Stopped)
            | (ServiceStatus::Stopped, ServiceStatus::Waiting)
            | (ServiceStatus::Waiting, ServiceStatus::Running)
            | (ServiceStatus::Running, ServiceStatus::Waiting) => ServiceStatus::Waiting,
            (ServiceStatus::Disowned, ServiceStatus::Stopped)
            | (ServiceStatus::Stopped, ServiceStatus::Disowned) => ServiceStatus::Stopped,
            (ServiceStatus::Disowned, ServiceStatus::Waiting)
            | (ServiceStatus::Waiting, ServiceStatus::Disowned) => ServiceStatus::WaitingDisowned,
            (ServiceStatus::Disowned, ServiceStatus::Running)
            | (ServiceStatus::Running, ServiceStatus::Disowned) => ServiceStatus::RunningDisowned,
            (a, b) => {
                debug!("unexpected combination of statuses: {:?}, {:?}", a, b);
                ServiceStatus::Stopped
            }
        }
    }
}
