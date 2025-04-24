use std::path::{PathBuf, Path};

use log::LevelFilter;
use log4rs::{
    append::{console::ConsoleAppender, rolling_file::{RollingFileAppender, policy::compound::{CompoundPolicy, trigger::size::SizeTrigger, roll::fixed_window::FixedWindowRoller}}},
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config, Handle,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    #[serde(default = "LogConfig::default_dir")]
    dir: Option<PathBuf>,
    #[serde(default = "LogConfig::default_levelfilter")]
    level: LevelFilter,
    #[serde(default = "LogConfig::default_max_size")]
    max_size: u64,
    #[serde(default = "LogConfig::default_keep_log")]
    keep_logs: u32,
}

impl Default for LogConfig {
    fn default() -> Self {
        let dir = if Path::new(concat!("C:\\ProgramData\\", env!("CARGO_PKG_NAME"))).exists() {
            Some(PathBuf::from(concat!("C:\\ProgramData\\", env!("CARGO_PKG_NAME"))))
        } else {
            None
        };

        Self { dir, level: LevelFilter::Info, max_size: 10*1024*1024, keep_logs: 10 }
    }
}

impl LogConfig {
    pub fn setup(&self, name: &str) -> Handle {
        log4rs::init_config(self.as_log4rs_config(name)).unwrap()
    }

    pub(self) fn as_log4rs_config(&self, name: &str) -> Config {
        let stdout = ConsoleAppender::builder()
            .encoder(Box::new(PatternEncoder::new(
                "{d(%Y-%m-%d %H:%M:%S)} {h({l})} {t} - {m}{n}",
            )))
            .build();

        let mut config = Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)));
        let mut root = Root::builder().appender("stdout");

        if let Some(dir) = &self.dir {
            let policy = CompoundPolicy::new(
                Box::new(SizeTrigger::new(self.max_size)),
                Box::new(
                    FixedWindowRoller::builder()
                        .build(dir.join(format!("{}.{{}}.log", name)).to_str().unwrap(), self.keep_logs)
                        .unwrap()
                )
            );

            let logfile = RollingFileAppender::builder()
                .append(true)
                .encoder(Box::new(PatternEncoder::new(
                    "{d(%Y-%m-%d %H:%M:%S)} {l} {t} - {m}{n}",
                )))
                .build(
                    dir.join(format!("{}.log", name)),
                    Box::new(policy)
                )
                .unwrap();
            config = config.appender(Appender::builder().build("logfile", Box::new(logfile)));
            root = root.appender("logfile");
        }

        config.build(root.build(self.level)).unwrap()
    }

    fn default_dir() -> Option<PathBuf> {
        Self::default().dir
    }

    fn default_levelfilter() -> LevelFilter {
        LevelFilter::Info
    }

    fn default_max_size() -> u64 {
        10*1024*1024
    }

    fn default_keep_log() -> u32 {
        10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_log_config() {
        let cfg = serde_yaml_ng::from_str::<LogConfig>(r#"---
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
        let cfg = serde_yaml_ng::from_str::<LogConfig>(r#"---
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
        let cfg = serde_yaml_ng::from_str::<LogConfig>(r#"---
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
        let cfg = serde_yaml_ng::from_str::<LogConfig>(r#"---
        level: Debug
        "#).unwrap();
        let cfg = cfg.as_log4rs_config("test");
        dbg!(&cfg);
        assert_eq!(cfg.root().appenders(), vec!("stdout"));
        assert_eq!(cfg.appenders().iter().map(|a| a.name()).collect::<Vec<&str>>(), vec!("stdout"));
    }
}
