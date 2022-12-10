use clap::Parser;

/// Netbxo to Windows DHCP Syncer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Sync {
    /// Do not change anything
    #[arg(short, long, default_value_t = false)]
    pub noop: bool,
}

impl Sync {
    pub fn init() -> Self { Sync::parse() }
}
