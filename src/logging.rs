use std::path::PathBuf;

use log::LevelFilter;
use log4rs::{append::{console::ConsoleAppender, file::FileAppender}, config::{Appender, Root}, Config, Handle};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    dir: Option<PathBuf>,
    #[serde(default="LogConfig::default_levelfilter")]
    level: LevelFilter,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self { dir: None, level: LevelFilter::Info }
    }
}

impl LogConfig {
    pub fn setup(self: &Self, name: &str) -> Handle {
        log4rs::init_config(self.as_log4rs_config(name)).unwrap()
    }

    pub(self) fn as_log4rs_config(self: &Self, name: &str) -> Config {
        let stdout = ConsoleAppender::builder().build();

        let mut config = Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)));
        let mut root = Root::builder().appender("stdout");

        if let Some(dir) = &self.dir {
            let logfile = FileAppender::builder()
                .build(dir.join(format!("{}.log", name)))
                .unwrap();
            config = config.appender(Appender::builder().build("logfile", Box::new(logfile)));
            root = root.appender("logfile");
        }

        config.build(root.build(self.level)).unwrap()
    }

    fn default_levelfilter() -> LevelFilter {
        LevelFilter::Info
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_log_config() {
        let cfg = serde_yaml::from_str::<LogConfig>(r#"---
        level: Debug
        dir: C:\tmp\
        "#);
        dbg!(&cfg);
        assert!(cfg.is_ok());
        let cfg = cfg.unwrap();
        assert_eq!(cfg.level, LevelFilter::Debug);
        assert_eq!(cfg.dir, Some(PathBuf::from("C:\\tmp\\")));
    }

    #[test]
    fn it_parses_log_config_without_dir() {
        let cfg = serde_yaml::from_str::<LogConfig>(r#"---
        level: Debug
        "#);
        dbg!(&cfg);
        assert!(cfg.is_ok());
        let cfg = cfg.unwrap();
        assert_eq!(cfg.level, LevelFilter::Debug);
        assert_eq!(cfg.dir, None);
    }

    #[test]
    fn as_to_lof4rs_config() {
        let cfg = serde_yaml::from_str::<LogConfig>(r#"---
        level: Debug
        dir: C:\tmp\
        "#).unwrap();
        let cfg = cfg.as_log4rs_config("test");
        dbg!(&cfg);
        assert_eq!(cfg.root().appenders(), vec!("stdout", "logfile"));
        assert_eq!(cfg.appenders().iter().map(|a| a.name()).collect::<Vec<&str>>(), vec!("stdout", "logfile"));
    }

    #[test]
    fn as_to_lof4rs_config_without_dir() {
        let cfg = serde_yaml::from_str::<LogConfig>(r#"---
        level: Debug
        "#).unwrap();
        let cfg = cfg.as_log4rs_config("test");
        dbg!(&cfg);
        assert_eq!(cfg.root().appenders(), vec!("stdout"));
        assert_eq!(cfg.appenders().iter().map(|a| a.name()).collect::<Vec<&str>>(), vec!("stdout"));
    }
}