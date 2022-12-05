use chrono::Utc;
use log::{info, error, debug};
use tokio::{sync::broadcast, task::JoinHandle, time::sleep};

use super::{
    config::WebhookConfig,
    shared::{SharedServerStatus, Message}
};

pub fn spawn(config: &WebhookConfig, status: &SharedServerStatus, sync_tx: &broadcast::Sender<Message>) -> JoinHandle<()> {
    let status = status.clone();
    let sync_tx = sync_tx.clone();
    let interval = chrono::Duration::seconds(config.sync_interval.unwrap_or(3600));

    tokio::spawn(async move {
        loop {
            let last_sync = {
                status.lock().await.last_sync
            }.unwrap_or(Utc::now() - interval);
            
            let next_sync = last_sync + interval;

            if next_sync <= Utc::now() {
                {
                    let mut status = status.lock().await;
                    status.needs_sync = true;
                }
                match sync_tx.send(Message::TriggerSync) {
                    Ok(_) => info!("Intervall Sync triggerd"),
                    Err(e) => error!("Triggering Intervall Sync: {:?}", e),
                }

                sleep(interval.to_std().unwrap()).await;

                continue;
            }
            
            let delta = next_sync.signed_duration_since(Utc::now());
            debug!("Wait for next Intervall Sync {}", delta);

            sleep(delta.to_std().unwrap()).await
        }
    })
}