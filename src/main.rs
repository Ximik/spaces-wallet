mod app;
mod branding;
mod client;
mod config;
mod helpers;
mod screen;
mod types;
mod widget;

use std::fs;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dirs =
        directories::ProjectDirs::from("", "", "akron").expect("Failed to build project dir path");
    let data_dir = dirs.data_dir();
    fs::create_dir_all(data_dir)?;

    let config_path = data_dir.join("config.json");
    let config = config::Config::load(&config_path)?;
    if config.setup {
        config.run()?;
    } else {
        let client = client::Client::new(config.spaced_rpc_url.as_ref().unwrap());
        app::App::new(config, client).run()?;
    }
    Ok(())
}
