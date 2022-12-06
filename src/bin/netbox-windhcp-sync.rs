use std::{env, process::Command};

use log::warn;
use netbox_windhcp_sync::{logging, config::Config};

fn main() {
    let _log_handle = logging::init("sync");

    let config = Config::load_from_file("netbox_windhcp_sync.cfg");
    warn!("Config: {:?}", config);

/*
    match env::current_exe() {
        Ok(exe_path) => {
            warn!("Path of this executable is: {}", exe_path.display());
            let bin = exe_path.parent().unwrap().join("netbox-windhcp-sync2.exe");
            warn!("Bin: {:?}", bin);

            let output = Command::new(bin)
                     .output()
                     .expect("Failed to execute command");
            
            warn!("output: {:?}", output);
        },
        Err(e) => warn!("failed to get current exe path: {e}"),
    };
    */
}
