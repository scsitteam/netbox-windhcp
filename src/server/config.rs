use std::{time::Duration, net::SocketAddr};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct WebhookConfig {
    pub listen: SocketAddr,
    pub sync_interval: Option<i64>,
    pub sync_standoff_time: Option<u64>,
    pub sync_timeout: Option<u64>,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let cfg = serde_yaml::from_str::<WebhookConfig>(r#"---
        listen: 127.0.0.1:12345
        "#);
        dbg!(&cfg);
        assert!(cfg.is_ok());
    }
}