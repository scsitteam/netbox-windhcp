use std::time::Duration;

use chrono::Utc;
use log::{info, debug, error};
use tokio::{sync::broadcast::{self, error::RecvError}, time::{sleep, error::Elapsed}, process::Command};

use super::{config::WebhookConfig, shared::{SharedServerStatus, Message}};

pub async fn worker(config: &WebhookConfig, status: &SharedServerStatus, mut message_rx: broadcast::Receiver<Message>) {
    let status: SharedServerStatus = status.clone();
    let sync_command = config.sync_command.clone();
    let sync_standoff_time = Duration::from_secs(config.sync_standoff_time.unwrap_or(5));
    let sync_timeout = Duration::from_secs(config.sync_timeout.unwrap_or(60));

    debug!("Sync Runner Thread Started");
    loop {
        match message_rx.recv().await {
            Ok(Message::Shutdown) | Err(RecvError::Closed) => { break; },
            Ok(Message::TriggerSync) => {
                if ! status.lock().await.needs_sync {
                    debug!("Sync not required");
                    continue;
                }
                info!("Sync Triggerd");

                sleep(sync_standoff_time).await;

                {
                    let mut status = status.lock().await;
                    status.needs_sync = false;
                    status.syncing = true;
                }

                let sync_status = run_sync_command(&sync_command, &sync_timeout).await;

                match sync_status {
                    Ok(_) => {
                        info!("Sync succeeded");
                        {
                            let mut status = status.lock().await;
                            status.syncing = false;
                            status.last_sync = Some(Utc::now());

                        }
                    }
                    Err(e) => {
                        error!("Sync failed: {}", e);
                        {
                            let mut status = status.lock().await;
                            status.syncing = false;
                            status.last_sync = Some(Utc::now());

                        }
                    }
                }
            },
            Err(RecvError::Lagged(_)) => {},
        }
    }
    info!("Sync Thread Ended");
}

#[derive(Debug)]
enum SyncError {
    IoError(std::io::Error),
    Timeout(Elapsed),
    CommandStatus(i32),
    CommandNoStatus,
}
impl From<std::io::Error> for SyncError {
    fn from(e: std::io::Error) -> Self {
        SyncError::IoError(e)
    }
}
impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::IoError(e) => write!(f, "Process io::Error {:?}", e),
            SyncError::Timeout(_) => write!(f, "Process timed out."),
            SyncError::CommandStatus(s) => write!(f, "Process exited with status code {}", s),
            SyncError::CommandNoStatus => write!(f, "Process exited without a status"),
        }
    }
}

async fn run_sync_command(command: &Vec<String>, timeout: &Duration) -> Result<(), SyncError> {
    info!("Run Sync Command: {}", command.join(" "));
    let mut child = Command::new(&command[0])
        .args(&command[1..])
        .spawn()?;

    let status = match tokio::time::timeout(timeout.to_owned(), child.wait()).await {
        Ok(s) => s,
        Err(e) => {
            debug!("Sync Command reached timeout. Terminating process.");
            child.kill().await?;
            return Err(SyncError::Timeout(e))
        }
    }?;

    match status.code() {
        Some(0) => Ok(()),
        Some(n) => Err(SyncError::CommandStatus(n)),
        None => Err(SyncError::CommandNoStatus),
    }
}