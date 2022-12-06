use log::warn;
use netbox_windhcp_sync::Config;

fn main() {
    let config = match Config::load_from_file() {
        Ok(config) => config,
        Err(e) => {
            println!("Error reading config: {}", e);
            return;
        }
    };

    config.log.setup("sync");

    warn!("Config: {:?}", config);
}
