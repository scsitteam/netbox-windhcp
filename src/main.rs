use log::{warn, debug};
use windows::Win32::System::Console::{GetStdHandle, STD_ERROR_HANDLE};


mod logging;
mod server;

fn main() {
    let log_handle = logging::init();

    let service = ! unsafe { GetStdHandle(STD_ERROR_HANDLE) }.is_ok();

    if service {
        server::service::run();
    } else {
        server::run(None);
    }
}
