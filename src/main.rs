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
    config.run()?;
    let config = config::Config::load(&config_path)?;

    // run backend, retrieve the client connection

    // app::App::new(&config).run()
    Ok(())
}
