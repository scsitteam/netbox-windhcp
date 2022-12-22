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
            std::process::exit(exitcode::CONFIG);
        }
    };

    config.log.setup("sync");

    #[cfg(target_os = "windows")]
    let cli_args = cli::Sync::init();

    #[cfg(target_os = "windows")]
    match Sync::new(config.sync, cli_args.noop).run().await {
        Ok(_) => std::process::exit(exitcode::OK),
        Err(e) => {
            error!("{}", e);
            std::process::exit(exitcode::DATAERR);
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        error!("Only works on Windows");
        std::process::exit(exitcode::DATAERR);
    }
}
