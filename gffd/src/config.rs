use serde::{Deserialize, Deserializer, Serialize};
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::fs;
fn deserialize_env_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    if buf.starts_with("!env ") {
        let val = std::env::var(&buf[5..]).unwrap_or("".to_string());
        if val == "".to_string() {
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
    pub server_options: ServerConfig,
}

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub callback_url: String,
}

impl Debug for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = serde_json::to_string_pretty(self).unwrap();
        write!(f, "{}", str).unwrap();
        Ok(())
    }
}
impl Debug for ServerConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = serde_json::to_string_pretty(self).unwrap();
        write!(f, "{}", str).unwrap();
        Ok(())
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            calendar_main_id: "".to_string(),
            calendar_filter_id: "".to_string(),
            calendar_auth_file: "google_auth.json".to_string(),
            server_options: ServerConfig::default(),
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
    pub fn read_config_file(directory: &String) -> Result<Self, std::io::Error> {
        let cfg_file = format!("{}/config.json", directory);
        match fs::exists(&cfg_file) {
            Ok(false) => {
                let default = Config::default();
                fs::write(&cfg_file, serde_json::to_string_pretty(&default)?)?;
                Ok(default)
            }
            Ok(true) => {
                let bytes = fs::read(&cfg_file)?;
                let cfg: Config = serde_json::from_slice(&bytes[..])?;
                Ok(cfg)
            }
            Err(e) => {
                println!("Unable to access cfg_file.  Check permissions");
                panic!("failed to read config dir");
            }
        }
    }
}
