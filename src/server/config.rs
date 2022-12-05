use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub webhook: WebhookConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebhookConfig {
    pub listen: std::net::SocketAddr,
    pub sync_command: Vec<String>,
    pub sync_interval: Option<i64>,
    pub sync_standoff_time: Option<u64>,
    pub sync_timeout: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let cfg = serde_yaml::from_str::<Config>(r#"---
        webhook:
            listen: 127.0.0.1:12345
        "#);
        dbg!(&cfg);
        assert!(cfg.is_ok());
    }
}