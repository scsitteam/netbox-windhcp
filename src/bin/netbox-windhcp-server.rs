use netbox_windhcp_sync::{server, Config};

fn main() {
    let config = match Config::load_from_file() {
        Ok(config) => config,
        Err(e) => {
            println!("Error reading config: {}", e);
            return;
        }
    };

    config.log.setup("server");

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
