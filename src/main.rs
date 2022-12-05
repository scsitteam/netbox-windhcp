#[cfg(windows)]
use windows::Win32::System::Console::{GetStdHandle, STD_ERROR_HANDLE};


mod logging;
mod server;

fn main() {
    let _log_handle = logging::init();

    #[cfg(windows)]
    let service = running_as_service();
    #[cfg(not(windows))]
    let service = false;

    if service {
        #[cfg(windows)]
        server::service::run();
    } else {
        server::run(None);
    }
}
