extern crate home;
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

use self::serde_derive::{Deserialize, Serialize};

use std::collections::btree_map::BTreeMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::string::String;

use command::DmgrResult;
use constants;
use service::Service;

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceRegistry {
    pub path: PathBuf,
    pub content: ServiceRegistryContent,
}

pub type ServiceRegistryContent = BTreeMap<String, ServiceRegistryEntryJson>;

impl ServiceRegistry {
    pub fn get() -> DmgrResult<ServiceRegistry> {
        Self::from_path(Self::path())
    }

    fn path() -> PathBuf {
        PathBuf::from(home::home_dir().unwrap())
            .join(constants::SERVICE_CONFIG_DIR)
            .join(constants::SERVICE_REGISTRY_FILENAME)
    }

    fn from_path(path: PathBuf) -> DmgrResult<Self> {
        let e = err!("unable to find registry {:?}", path);
        path.to_str().map_or(e, |s| Self::from(s))
    }

    pub fn from(path: &str) -> DmgrResult<Self> {
        match path {
            json if json.ends_with(".json") => ServiceRegistry::from_json(json),
            _ => fail!("could not read '{:?}'", path),
        }
    }

    pub fn get_service(&self, name: &str) -> DmgrResult<Service> {
        let entry = self
            .content
            .get(name)
            .ok_or(dmgr_err!("unable to find service {:?}", name))?;
        let cfg_path = entry.repo_path.join(".solo").join(format!("{}.json", name));
        Service::from_path(&cfg_path)
    }

    pub fn add_svc(self, svc: &Service) -> DmgrResult<Self> {
        self.add_cfg(&svc.config_file)
    }

    pub fn add_cfg(mut self, path: &PathBuf) -> DmgrResult<Self> {
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();

        let mut ancestors = path.ancestors();
        ancestors.next();
        ancestors.next();

        let repo_path = ancestors.next().unwrap();

        let cfg_file = ServiceConfigContent::from_path(&path)?;

        let entry = ServiceRegistryEntryJson {
            aliases: cfg_file.aliases,
            image_tag: None,
            repo_path: PathBuf::from(repo_path),
        };

        self.content.insert(name, entry);
        Ok(self)
    }

    pub fn save(self) -> DmgrResult<Self> {
        let mut file = File::create(Self::path())?;
        let content = serde_json::to_string_pretty(&self.content)?;

        file.write_all(content.as_bytes())?;
        Ok(self)
    }

    fn from_json(path: &str) -> DmgrResult<ServiceRegistry> {
        let mut contents = String::new();
        let mut file = File::open(path)?;

        file.read_to_string(&mut contents)?;
        let content: ServiceRegistryContent = serde_json::from_str(contents.as_str())?;
        Ok(ServiceRegistry {
            content,
            path: PathBuf::from(path),
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceConfigContent {
    pub aliases: Option<Vec<String>>,
    pub image_name: Option<String>,
    pub ports: Option<Vec<u16>>,
    pub start_container: Option<String>,
    pub start_process: Option<String>,
    pub start_dev_mode: Option<String>,
    pub http_check: Option<String>,
    pub health_checks: Option<Vec<String>>,
    pub register_by_default: Option<bool>,
    pub requires_sudo: Option<bool>,
}

impl ServiceConfigContent {
    pub fn from_path(path: &PathBuf) -> DmgrResult<Self> {
        let e = err!("unable to find service config {:?}", path);
        path.to_str().map_or(e, |s| Self::from(s))
    }

    pub fn from(path: &str) -> DmgrResult<Self> {
        let mut contents = String::new();
        let mut file = File::open(path)?;

        file.read_to_string(&mut contents)?;
        Ok(serde_json::from_str(&contents)?)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceRegistryEntryJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    aliases: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_tag: Option<String>,
    repo_path: PathBuf,
}

impl From<Service> for ServiceRegistryEntryJson {
    fn from(svc: Service) -> Self {
        Self {
            aliases: Some(svc.aliases),
            image_tag: svc.image_tag,
            repo_path: svc.repo_path,
        }
    }
}

type Pid = u32;

#[derive(Debug, Deserialize, Serialize)]
pub struct Runfile {
    pub pid: Pid,
    pub is_container: bool,
}
