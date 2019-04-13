extern crate home;
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

use self::serde_derive::{Deserialize, Serialize};

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::string::String;

use command::DmgrResult;
use constants;

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceRegistry {
    pub content: ServiceRegistryContent,
}

type ServiceRegistryContent = BTreeMap<String, ServiceRegistryEntryJson>;

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

    pub fn add_cfg(self, path: PathBuf) -> DmgrResult<Self> {
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();

        let repo_path = path.parent().and_then(|d| d.parent()).unwrap();
        let cfg_file = ServiceConfigContent::from_path(&path)?;

        let entry = ServiceRegistryEntryJson {
            aliases: cfg_file.aliases,
            image_tag: None,
            repo_path: PathBuf::from(repo_path),
        };

        let mut registry = Self::get()?;
        registry.content.insert(name, entry);

        Ok(registry)
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
        Ok(ServiceRegistry { content })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceConfigContent {
    aliases: Option<Vec<String>>,
    image_name: Option<String>,
    ports: Option<Vec<u16>>,
    start_container: Option<String>,
    start_process: Option<String>,
    start_dev_mode: Option<String>,
    http_check: Option<String>,
    health_checks: Option<Vec<String>>,
    register_by_default: Option<bool>,
    requires_sudo: Option<bool>,
}

impl ServiceConfigContent {
    fn from_path(path: &PathBuf) -> DmgrResult<Self> {
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
    aliases: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_tag: Option<String>,
    repo_path: PathBuf,
}

//toml::to_string(&config).unwrap()
