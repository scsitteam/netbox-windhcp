use std::{time::Duration, env::{self, consts::EXE_SUFFIX}, path::PathBuf, error::Error};

use chrono::Utc;
use log::{info, debug, error};
use tokio::{sync::broadcast::{self, error::RecvError}, time::{sleep, error::Elapsed}, process::Command};

use super::{config::WebhookConfig, shared::{SharedServerStatus, Message}};

fn get_sync_binary() -> Result<PathBuf, Box<dyn Error>> {
    let sync_exe = env::current_exe()?
        .parent().ok_or(SyncError::CommandNotFound)?
        .join(format!("netbox-windhcp-sync{}", EXE_SUFFIX));
    if ! sync_exe.is_file() {
        error!("Sync binary at {} not found.", sync_exe.display());
        return Err(Box::new(SyncError::CommandNotFound));
    }
    Ok(sync_exe)
}

pub async fn worker(config: &WebhookConfig, status: &SharedServerStatus, message_tx: &broadcast::Sender<Message>, mut message_rx: broadcast::Receiver<Message>) {
    let status: SharedServerStatus = status.clone();
    let sync_standoff_time = config.sync_standoff_time();
    let sync_timeout = config.sync_timeout();
    
    let sync_command = match get_sync_binary() {
        Ok(bin) => bin,
        Err(e) => {
            error!("Sync binary not found. {}", e);
            message_tx.send(Message::Shutdown).unwrap();
            return;
        }
    };

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
    CommandNotFound,
    IoError(std::io::Error),
    Timeout(Elapsed),
    CommandStatus(i32),
    CommandNoStatus,
}

impl Error for SyncError {}
impl From<std::io::Error> for SyncError {
    fn from(e: std::io::Error) -> Self {
        SyncError::IoError(e)
    }
}
impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::CommandNotFound => write!(f, "Sync binary not found"),
            SyncError::IoError(e) => write!(f, "Process io::Error {:?}", e),
            SyncError::Timeout(_) => write!(f, "Process timed out."),
            SyncError::CommandStatus(s) => write!(f, "Process exited with status code {}", s),
            SyncError::CommandNoStatus => write!(f, "Process exited without a status"),
        }
    }
}

async fn run_sync_command(command: &PathBuf, timeout: &Duration) -> Result<(), SyncError> {
    info!("Run Sync Command: {}", &command.display());
    let mut child = Command::new(&command)
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