mod shared;
pub mod config;
mod interval;
mod signal;
mod sync;
mod web;
mod webhook;
#[cfg(windows)]
pub mod service;

use std::sync::{mpsc as std_mpsc, Arc};
use log::debug;
use tokio::sync::{broadcast, Mutex};

use crate::server::{config::WebhookConfig, shared::ServerStatus};

use self::shared::{Message, SharedServerStatus};

pub fn run(shutdown_rx: Option<std_mpsc::Receiver<Message>>) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {

            let status: SharedServerStatus = Arc::new(Mutex::new(ServerStatus::new()));
            let config = WebhookConfig {
                listen: "127.0.0.1:12345".parse().unwrap(),
                sync_command: vec![String::from("sleep"), String::from("5")],
                sync_interval: Some(30),
                sync_standoff_time: Some(5),
                sync_timeout: Some(10),
            };
            let (message_tx, mut message_rx) = broadcast::channel(16);

            let _interval_handle = self::interval::spawn(&config, &status, &message_tx);
            let _signal_handle = self::signal::spawn(&message_tx);

            if let Some(shutdown_rx) = shutdown_rx {
                let message_tx = message_tx.clone();
                debug!("Start Proxy");
                tokio::spawn(async move {
                    while let Ok(msg) = shutdown_rx.recv() {
                        debug!("Channel proxy got: {:?}", &msg);
                        message_tx.send(msg).unwrap();
                    };
                });
                debug!("Start Proxy Done");
            }

            tokio::join!(
                self::web::server(&config, &status, &message_tx),
                self::sync::worker(&config, &status, message_rx)
            );

    })
}