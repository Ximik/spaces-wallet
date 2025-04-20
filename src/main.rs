mod app;
mod client;
mod helpers;
mod screen;
mod types;
mod widget;

use app::App;

use clap::Parser;
use spaces_client::config::{ExtendedNetwork, default_spaces_rpc_port};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Bitcoin network to use
    #[arg(long, env = "SPACED_CHAIN", default_value = "mainnet")]
    chain: ExtendedNetwork,
    /// Spaced RPC URL [default: based on specified chain]
    #[arg(long)]
    spaced_rpc_url: Option<String>,
}

fn default_spaced_rpc_url(chain: &ExtendedNetwork) -> String {
    format!("http://127.0.0.1:{}", default_spaces_rpc_port(chain))
}

pub fn main() -> iced::Result {
    let mut args = Args::parse();
    if args.spaced_rpc_url.is_none() {
        args.spaced_rpc_url = Some(default_spaced_rpc_url(&args.chain));
    }

    App::run(args)
}
