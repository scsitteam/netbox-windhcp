use std::error::Error;
use std::fs::File;

use serde::Deserialize;

use crate::logging::LogConfig;

use super::server::config::WebhookConfig;
#[cfg(windows)]
use super::sync::config::SyncConfig;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub webhook: WebhookConfig,
    #[cfg(windows)]
    pub sync: SyncConfig,
    #[serde(default)]
    pub log: LogConfig
}

impl Config {
    #[cfg(debug_assertions)]
    const CONFIG_FILE: &str = concat!("./", env!("CARGO_CRATE_NAME"), ".cfg");
    #[cfg(not(debug_assertions))]
    const CONFIG_FILE: &str = concat!("C:\\ProgramData\\", env!(CARGO_CRATE_NAME), "\\", env!(CARGO_CRATE_NAME), ".cfg");

    pub fn load_from_file() -> Result<Self, Box<dyn Error>> {
        let file = File::open(Self::CONFIG_FILE)?;

        Ok(serde_yaml::from_reader::<File, Config>(file)?)
    }
}


