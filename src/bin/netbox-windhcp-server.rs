use netbox_windhcp::{server, Config};

fn main() {
    let config = match Config::load_from_file() {
        Ok(config) => config,
        Err(e) => {
            println!("Error reading config: {}", e);
            return;
        }
    };

    config.log.setup("server");

    #[cfg(target_os = "windows")]
    let service = server::service::running_as_service();
    #[cfg(not(target_os = "windows"))]
    let service = false;

    if service {
        #[cfg(target_os = "windows")]
        server::service::run();
    } else {
        server::run(None);
    }
}
