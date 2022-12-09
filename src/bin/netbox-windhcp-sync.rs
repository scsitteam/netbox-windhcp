use log::error;
use netbox_windhcp::{Config, Sync, cli};

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

    let cli_args = cli::Sync::init();
    
    match Sync::new(config.sync, cli_args.noop).run().await {
        Ok(_) => {},
        Err(e) => error!("{}", e),
    }
}
