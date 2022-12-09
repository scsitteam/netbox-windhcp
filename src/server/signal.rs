use log::info;
use tokio::{sync::broadcast, task::JoinHandle, signal};

use super::shared::Message;

pub fn spawn(sync_tx: &broadcast::Sender<Message>) -> JoinHandle<()> {
    let sync_tx = sync_tx.clone();

    tokio::spawn(async move {
        while (signal::ctrl_c().await).is_ok() {
            info!("Received Ctrl+C send Shutdown message.");
            match sync_tx.send(Message::Shutdown) {
                Ok(_) => {},
                Err(_) => { break; },
            }
        }
    })
}