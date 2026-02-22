use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    // No config directory found
    #[error("No config directory {0}")]
    NoConfigDir(String),
    // Permission error reading config.json
    #[error("Config directory permission error for {0}")]
    PermissionError(String),
    // unreadable config.json
    #[error("Unable to read {0}")]
    ReadError(String),
    // Missing config.json
    #[error("No config.json in {0}")]
    MissingConfig(String),
    // Missing state_directory
    #[error("state_directory is set to {0} but is not avaiable")]
    MissingStateDir(String),
    // Invalid json
    #[error("{0} does not contain a valid config - {1}")]
    InvalidConfig(String, String),
}

fn deserialize_env_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    if let Some(var_name) = buf.strip_prefix("!env ") {
        let val = std::env::var(var_name).unwrap_or("".to_string());
        if val.is_empty() {
            println!("Environment variable {} is unset or empty", &buf[5..]);
        }
        Ok(val)
    } else {
        Ok(buf)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_env_string")]
    pub calendar_main_id: String,
    #[serde(deserialize_with = "deserialize_env_string")]
    pub calendar_filter_id: String,
    #[serde(deserialize_with = "deserialize_env_string")]
    pub calendar_auth_file: String,
    #[serde(deserialize_with = "deserialize_env_string")]
    pub cookie: String,
    #[serde(default)]
    pub state_directory: String,
    pub server_options: ServerConfig,
    pub screens: HashMap<String, ScreenConfig>,
    pub strands: HashMap<String, StrandConfig>,
    pub names: HashMap<String, String>,
    #[serde(skip)]
    directory: String,
    #[serde(skip)]
    debug: bool,
    #[serde(skip)]
    live: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StrandConfig {
    pub id: u32,
    pub colour: String,
    pub priority: u32,
}

impl Default for StrandConfig {
    fn default() -> Self {
        Self {
            id: 0,
            colour: "000000".to_string(),
            priority: 99,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ScreenConfig {
    pub id: u32,
    pub colour: u32,
}

impl Default for ScreenConfig {
    fn default() -> Self {
        Self { id: 0, colour: 7 }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub callback_url: String,
}

impl Debug for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?
        )
    }
}
impl Debug for ServerConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?
        )
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            calendar_main_id: "".to_string(),
            calendar_filter_id: "".to_string(),
            calendar_auth_file: "google_auth.json".to_string(),
            state_directory: "".to_string(),
            server_options: ServerConfig::default(),
            screens: HashMap::default(),
            strands: HashMap::default(),
            directory: ".".to_string(),
            cookie: "".to_string(),
            names: HashMap::default(),
            debug: false,
            live: false,
        }
    }
}
impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 3020,
            callback_url: "https://gff.darach.org.uk".to_string(),
        }
    }
}

impl Config {
    pub fn set_auth_file(&mut self, file: &str) {
        self.calendar_auth_file = file.to_string();
        if Path::new(file).is_relative()
            && let Some(abspath) = Path::new(&self.directory).join(Path::new(file)).to_str()
        {
            self.calendar_auth_file = abspath.to_owned();
        }
    }
    pub fn set_debug(&mut self) {
        self.debug = true;
    }
    pub fn is_debug(&self) -> bool {
        self.debug
    }
    pub fn set_live(&mut self) {
        self.live = true;
    }
    pub fn is_live(&self) -> bool {
        self.live
    }
    pub fn read_config_file(directory: &String) -> Result<Self, ConfigError> {
        if let Ok(true) = fs::exists(directory) {
        } else {
            return Err(ConfigError::NoConfigDir(directory.clone()));
        }
        let cfg_file = format!("{}/config.json", directory);
        match fs::exists(&cfg_file) {
            Ok(true) => {
                let bytes =
                    fs::read(&cfg_file).map_err(|_| ConfigError::ReadError(cfg_file.clone()))?;
                let mut cfg: Config = serde_json::from_slice(&bytes[..])
                    .map_err(|e| ConfigError::InvalidConfig(cfg_file.clone(), format!("{}", e)))?;
                let dir = fs::canonicalize(directory)
                    .map_err(|_| ConfigError::NoConfigDir(directory.clone()))?;
                cfg.directory = dir
                    .to_str()
                    .ok_or(ConfigError::NoConfigDir(directory.clone()))?
                    .to_owned();
                if !cfg.screens.contains_key("Unknown") {
                    cfg.screens
                        .insert("Unknown".to_string(), ScreenConfig::default());
                }
                if !cfg.strands.contains_key("None") {
                    cfg.strands
                        .insert("None".to_string(), StrandConfig::default());
                }
                if cfg.state_directory.is_empty() {
                    cfg.state_directory = cfg.directory.clone();
                }

                match fs::exists(&cfg.state_directory) {
                    Ok(true) => {}
                    _ => {
                        return Err(ConfigError::MissingStateDir(cfg.state_directory.clone()));
                    }
                }
                Ok(cfg)
            }
            Ok(false) => {
                Err(ConfigError::MissingConfig(directory.clone()))
                /*
                let default = Config::default();
                fs::write(&cfg_file, serde_json::to_string_pretty(&default)?)?;
                Ok(default)
                */
            }
            Err(_) => {
                println!("Unable to access cfg_file.  Check permissions");
                Err(ConfigError::PermissionError(cfg_file.clone()))
            }
        }
    }

    pub fn screen_from_id(&self, id: u32) -> (String, ScreenConfig) {
        let found = self.screens.iter().find(|(_k, v)| id == v.id);
        match found {
            Some((s, c)) => (s.clone(), c.clone()),
            None => ("".to_string(), ScreenConfig::default()),
        }
    }

    pub fn strand_from_badges(&self, badges: Vec<u32>) -> (String, StrandConfig) {
        let found = self
            .strands
            .iter()
            .filter(|(_k, v)| badges.contains(&v.id))
            .min_by(|a, b| a.1.priority.cmp(&b.1.priority));
        match found {
            Some((s, c)) => (s.clone(), c.clone()),
            None => ("".to_string(), StrandConfig::default()),
        }
    }
}
