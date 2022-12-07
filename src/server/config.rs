use std::{time::Duration, net::SocketAddr};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct WebhookConfig {
    pub listen: SocketAddr,
    sync_interval: Option<i64>,
    sync_standoff_time: Option<u64>,
    sync_timeout: Option<u64>,
    secret: Option<String>,
    cert: Option<String>,
    key: Option<String>,
}

impl WebhookConfig {
    pub fn sync_interval(&self) -> chrono::Duration {
        chrono::Duration::seconds(self.sync_interval.unwrap_or(900))
    }

    pub fn sync_standoff_time(&self) -> Duration {
        Duration::from_secs(self.sync_standoff_time.unwrap_or(5))
    }

    pub fn sync_timeout(&self) -> Duration {
        Duration::from_secs(self.sync_timeout.unwrap_or(60))
    }

    pub fn secret(&self) -> Option<&String> {
        self.secret.as_ref()
    }

    pub fn enable_tls(&self) -> bool {
        self.cert.is_some() && self.key.is_some()
    }

    pub fn cert(&self) -> Option<&String> {
        self.cert.as_ref()
    }

    pub fn key(&self) -> Option<&String> {
        self.key.as_ref()
    }
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self { listen: "9.9.9.9:1111".parse().unwrap(), sync_interval: Default::default(), sync_standoff_time: Default::default(), sync_timeout: Default::default(), secret: Default::default(), cert: Default::default(), key: Default::default() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_with_only_listen() {
        let cfg = serde_yaml::from_str::<WebhookConfig>(r#"---
        listen: 127.0.0.1:12345
        "#);
        assert_eq!(cfg.unwrap(), WebhookConfig {
            listen: "127.0.0.1:12345".parse().unwrap(),
            ..Default::default()
        });
    }

    #[test]
    fn it_parses_with_all_options() {
        let cfg = serde_yaml::from_str::<WebhookConfig>(r#"---
        listen: 127.0.0.1:12345
        sync_interval: 42
        sync_standoff_time: 42
        sync_timeout: 42
        secret: SECRET
        cert: cert.pem
        key: key.pem
        "#);
        assert_eq!(cfg.unwrap(), WebhookConfig {
            listen: "127.0.0.1:12345".parse().unwrap(),
            sync_interval: Some(42),
            sync_standoff_time: Some(42),
            sync_timeout: Some(42),
            secret: Some(String::from("SECRET")),
            cert: Some(String::from("cert.pem")),
            key: Some(String::from("key.pem")),
        });
    }

    #[test]
    fn it_returns_sync_interval_as_durations () {
        let cfg = WebhookConfig {sync_interval: Some(42), ..Default::default()};
        assert_eq!(cfg.sync_interval(), chrono::Duration::seconds(42))
    } 

    #[test]
    fn it_returns_sync_interval_default () {
        let cfg = WebhookConfig {..Default::default()};
        assert_eq!(cfg.sync_interval(), chrono::Duration::seconds(900))
    } 

    #[test]
    fn it_returns_sync_standoff_time_as_durations () {
        let cfg = WebhookConfig {sync_standoff_time: Some(42), ..Default::default()};
        assert_eq!(cfg.sync_standoff_time(), Duration::from_secs(42))
    }

    #[test]
    fn it_returns_sync_standoff_time_default () {
        let cfg = WebhookConfig {..Default::default()};
        assert_eq!(cfg.sync_standoff_time(), Duration::from_secs(5))
    }

    #[test]
    fn it_returns_sync_timeout_as_durations () {
        let cfg = WebhookConfig {sync_timeout: Some(42), ..Default::default()};
        assert_eq!(cfg.sync_timeout(), Duration::from_secs(42))
    }

    #[test]
    fn it_returns_sync_timeout_default () {
        let cfg = WebhookConfig {..Default::default()};
        assert_eq!(cfg.sync_timeout(), Duration::from_secs(60))
    }
}