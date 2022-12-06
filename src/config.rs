use std::fs::File;

use log::LevelFilter;
use serde::Deserialize;

use crate::error::Error;

use super::server::config::WebhookConfig;
use super::sync::config::SyncConfig;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub webhook: WebhookConfig,
    pub sync: SyncConfig,
    pub log: LogConfig
}

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub dir: Option<String>,
    #[serde(default="default_levelfilter")]
    pub level: LevelFilter,
}

fn default_levelfilter() -> LevelFilter {
    LevelFilter::Info
}

impl Config {
    pub fn load_from_file(filename: &str) -> Result<Self, Error> {
        let file = match File::open(filename) {
            Ok(f) => f,
            Err(e) => return Err(Error::ConfigError(format!("opening file {}: {}", filename, e))),
        };

        match serde_yaml::from_reader::<File, Config>(file) {
            Ok(config) => Ok(config),
            Err(e) => return Err(Error::ConfigError(format!("parsing config file {}: {}", filename, e))),
        }
    }
}


