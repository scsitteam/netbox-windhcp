use log::error;
use netbox_windhcp::{Config, Sync};

#[tokio::main]
async fn main() {
    let config = match Config::load_from_file() {
        Ok(config) => config,
        Err(e) => {
            println!("Error reading config: {}", e);
            return;
        }
    };

    config.log.setup("sync");
    
    match Sync::new(config.sync, true).run().await {
        Ok(_) => {},
        Err(e) => error!("{}", e),
    }
}
