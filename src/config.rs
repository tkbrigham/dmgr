extern crate serde_derive;
extern crate serde_json;
extern crate toml;

use self::serde_derive::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceRegistry {
    pub services: Vec<ServiceRegistryEntry>,
}

impl ServiceRegistry {
    pub fn from(path: &str) -> Option<Self> {
        match path {
            toml if toml.ends_with(".toml") => Some(ServiceRegistry::from_toml(toml)),
            json if json.ends_with(".json") => Some(ServiceRegistry::from_json(json)),
            _ => None,
        }
    }

    fn from_toml(path: &str) -> Self {
        let mut contents = String::new();
        let mut file = File::open(path).unwrap();

        file.read_to_string(&mut contents).unwrap();
        toml::from_str(contents.as_str()).unwrap()
    }

    fn from_json(path: &str) -> Self {
        let mut contents = String::new();
        let mut file = File::open(path).unwrap();

        file.read_to_string(&mut contents).unwrap();
        let map: HashMap<String, ServiceRegistryEntryJson> =
            serde_json::from_str(contents.as_str()).unwrap();
        let services: Vec<ServiceRegistryEntry> = map
            .into_iter()
            .map(|pair| ServiceRegistryEntry::from(pair))
            .collect();
        ServiceRegistry { services }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceRegistryEntry {
    pub name: String,
    pub aliases: Vec<String>,
    pub image_tag: Option<String>,
    pub config_path: String,
}

impl ServiceRegistryEntry {
    pub fn from(pair: (String, ServiceRegistryEntryJson)) -> Self {
        let name = &pair.0;
        let entry = pair.1;
        ServiceRegistryEntry {
            name: name.to_string(),
            aliases: entry.aliases,
            image_tag: entry.image_tag,
            config_path: format!("{}/.solo/{}.json", entry.repo_path, name),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Service {
    aliases: Vec<String>,
    ports: Vec<u16>,
    http_check: String,
    start_container: String,
    start_process: String,
    config_path: String,
}

impl Service {
    pub fn from(path: &str) -> Self {
        let cfg_line = format!("config_path = \"{}\"\n", path);
        let mut contents = String::from(cfg_line);
        let mut file = File::open(path).unwrap();

        file.read_to_string(&mut contents).unwrap();
        toml::from_str(contents.as_str()).unwrap()
    }

    pub fn into_registry_entry(self) -> ServiceRegistryEntry {
        ServiceRegistryEntry {
            name: self.name(),
            aliases: self.aliases,
            image_tag: None,
            config_path: self.config_path,
        }
    }

    pub fn name(&self) -> String {
        Path::new(&self.config_path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceRegistryEntryJson {
    aliases: Vec<String>,
    image_tag: Option<String>,
    repo_path: String,
}

//toml::to_string(&config).unwrap()
