use log::error;
use netbox_windhcp::Config;
#[cfg(windows)]
use netbox_windhcp::{Sync, cli};

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

    #[cfg(windows)]
    let cli_args = cli::Sync::init();
    
    #[cfg(windows)]
    match Sync::new(config.sync, cli_args.noop).run().await {
        Ok(_) => {},
        Err(e) => error!("{}", e),
    }

    #[cfg(not(windows))]
    error!("Only works on Windows");
}
