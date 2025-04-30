mod app;
mod branding;
mod client;
mod helpers;
mod pages;
mod state;
mod types;
mod widget;

use std::fs;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dirs =
        directories::ProjectDirs::from("", "", "akron").expect("Failed to build project dir path");
    let data_dir = dirs.data_dir();
    fs::create_dir_all(data_dir).unwrap();

    let config_path = data_dir.join("config.json");
    let config = state::Config::load(config_path)
    app::State::new(config).run();
    Ok(())
}
