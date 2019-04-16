use command::DmgrResult;
use config::ServiceConfigContent;
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Service {
    pub name: String,
    pub repo_path: PathBuf,
    pub config_file: PathBuf,
    pub start_process: Option<String>,
    pub start_dev_mode: Option<String>,
    pub start_container: Option<String>,
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

        //        let name= path_to_svc_name(&canonical_path).to_os_string();
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

    //    pub fn from(s: &str) -> DmgrResult<Self> {
    //        Self::from_path(&PathBuf::from(s))
    //    }

    pub fn from_name(s: &str) -> DmgrResult<Self> {
        Self::from_path(&PathBuf::from(s))
    }

    pub fn from_alias(s: &str) -> DmgrResult<Self> {
        Self::from_path(&PathBuf::from(s))
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
}

// TODO: this is implemented in register.rs too
fn path_to_svc_name(p: &PathBuf) -> &OsStr {
    p.file_stem().unwrap_or(OsStr::new("UNKNOWN"))
}

fn repo_path_for(canonical_path: &PathBuf) -> PathBuf {
    let repo = canonical_path.parent().and_then(|p| p.parent());
    PathBuf::from(repo.unwrap())
}
