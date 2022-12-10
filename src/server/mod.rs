pub mod config;
mod interval;
#[cfg(target_os = "windows")]
pub mod service;
mod shared;
mod signal;
mod sync;
mod web;
mod webhook;

use log::debug;
use std::sync::{mpsc as std_mpsc, Arc};
use tokio::sync::{broadcast, Mutex};

use crate::{server::shared::ServerStatus, Config};

use self::shared::{Message, SharedServerStatus};

pub fn run(shutdown_rx: Option<std_mpsc::Receiver<Message>>) {
    let config = match Config::load_from_file() {
        Ok(config) => config.webhook,
        Err(e) => {
            println!("Error reading config: {}", e);
            return;
        }
    };

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let status: SharedServerStatus = Arc::new(Mutex::new(ServerStatus::new()));
            let (message_tx, message_rx) = broadcast::channel(16);

            let _interval_handle = self::interval::spawn(&config, &status, &message_tx);
            let _signal_handle = self::signal::spawn(&message_tx);

            if let Some(shutdown_rx) = shutdown_rx {
                let message_tx = message_tx.clone();
                debug!("Start Proxy");
                tokio::spawn(async move {
                    while let Ok(msg) = shutdown_rx.recv() {
                        debug!("Channel proxy got: {:?}", &msg);
                        message_tx.send(msg).unwrap();
                    }
                });
                debug!("Start Proxy Done");
            }

            tokio::join!(
                self::web::server(&config, &status, &message_tx),
                self::sync::worker(&config, &status, &message_tx, message_rx)
            );
        })
}
