mod app;
mod branding;
mod client;
mod config;
mod helpers;
mod screen;
mod state;
mod types;
mod widget;

use std::fs;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dirs =
        directories::ProjectDirs::from("", "", "akron").expect("Failed to build project dir path");
    let data_dir = dirs.data_dir();
    fs::create_dir_all(data_dir)?;

    let config_path = data_dir.join("config.json");
    if config_path.exists() {
        let config = config::Config::load(config_path)?;
        // run spaced if not standalone
        let client = client::Client::new(config.spaced_rpc_url.as_ref().unwrap())?;
        app::App::new(config, client).run()?;
    } else {
        let config = config::Config::new(config_path);
        config.run()?;
    }
    Ok(())
}
