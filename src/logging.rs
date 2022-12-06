use log::LevelFilter;
use log4rs::{append::{console::ConsoleAppender, file::FileAppender}, config::{Appender, Root}, Config, Handle};

pub fn init(name: &str) -> Handle {
    let stdout = ConsoleAppender::builder().build();
    let logfile = FileAppender::builder()
        .build(format!("C:\\ProgramData\\netbox-windhcp-sync\\{}.log", name))
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("stdout").appender("logfile").build(LevelFilter::Debug))
        .unwrap();

    log4rs::init_config(config).unwrap()
}