#[macro_use]
extern crate serde;
extern crate serde_json;
extern crate dirs;

use std::fs::File;
use std::path::{PathBuf};
use serde_json as json;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerAddr {
    host: String,
    port: u16,
}

impl ServerAddr {
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerConfig {
    pub local: ServerAddr,
    pub remote: ServerAddr,
}

pub fn load_config() -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let homedir = dirs::home_dir().expect("Cannot get home directory!");
    let confdir: PathBuf = [homedir.to_str().unwrap(), ".config", "relayer"].iter().collect();
    let mut config_path = confdir.clone();
    config_path.push("config.json");
    let config = File::open(config_path)?;
    let server_config: ServerConfig = json::from_reader(config)?;
    Ok(server_config)
}
