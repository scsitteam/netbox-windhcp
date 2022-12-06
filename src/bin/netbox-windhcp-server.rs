use netbox_windhcp_sync::logging;
use netbox_windhcp_sync::server;

fn main() {
    let _log_handle = logging::init("server");

    #[cfg(windows)]
    let service = server::service::running_as_service();
    #[cfg(not(windows))]
    let service = false;

    if service {
        #[cfg(windows)]
        server::service::run();
    } else {
        server::run(None);
    }
}
