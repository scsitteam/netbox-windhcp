use log::error;
use netbox_windhcp::Config;
#[cfg(target_os = "windows")]
use netbox_windhcp::{cli, Sync};

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

    #[cfg(target_os = "windows")]
    let cli_args = cli::Sync::init();

    #[cfg(target_os = "windows")]
    match Sync::new(config.sync, cli_args.noop).run().await {
        Ok(_) => {}
        Err(e) => error!("{}", e),
    }

    #[cfg(not(target_os = "windows"))]
    error!("Only works on Windows");
}
