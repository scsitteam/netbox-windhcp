use std::{ffi::OsString, time::Duration};

use log::{debug, error, info};
use std::sync::mpsc;
use windows_service::{
    define_windows_service,
    service_control_handler::{ServiceControlHandlerResult, self, ServiceStatusHandle},
    service::*,
    service_dispatcher
};

use crate::server::Message;

define_windows_service!(ffi_service_main, service_main);

const SERICE_NAME: &str = env!("CARGO_PKG_NAME");

pub fn run() {
    service_dispatcher::start(SERICE_NAME, ffi_service_main).unwrap();
}

pub fn service_main(arguments: Vec<OsString>) {
    debug!("Started service_main: {:?}", arguments);

    let (shutdown_tx, shutdown_rx) = mpsc::channel();

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        info!("Received ServiceConrtoll Event: {:?}", control_event);
        match control_event {
            ServiceControl::Stop => {
                // Handle stop event and return control back to the system.
                shutdown_tx.send(Message::Shutdown).unwrap();
                ServiceControlHandlerResult::NoError
            }
            // All services must accept Interrogate even if it's a no-op.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = match service_control_handler::register(SERICE_NAME, event_handler) {
        Ok(status_handle) => status_handle,
        Err(e) => {
            error!("Register service event handler failed: {:?}", e);
            return;
        },
    };
    
    debug!("Started service_main: {:?}", arguments);

    
    if set_service_status(&status_handle, ServiceState::Running).is_err() {
        return;
    }

    crate::server::run(Some(shutdown_rx));

    let _ = set_service_status(&status_handle, ServiceState::Stopped);
}

fn set_service_status(status_handle: &ServiceStatusHandle, current_state: ServiceState) -> Result<(), windows_service::Error> {
    let controls_accepted = match current_state {
        ServiceState::Stopped => ServiceControlAccept::empty(),
        ServiceState::Running => ServiceControlAccept::STOP,
        _ => ServiceControlAccept::STOP,
    };

    let status_stopped = ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state,
        controls_accepted,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    };
    match status_handle.set_service_status(status_stopped) {
        Ok(_) => {
            debug!("Switched service to {:?}", current_state);
            Ok(())
        }
        Err(e) => {
            error!("Updating service status to {:?} failed: {:?}", current_state, e);
            Err(e)
        }
    }
}

pub fn running_as_service() -> bool {
    ! unsafe { GetStdHandle(STD_ERROR_HANDLE) }.is_ok()
}