mod app;
mod client;
mod constants;
mod helpers;
mod screen;
mod types;
mod widget;

// use spaces_client::config::ExtendedNetwork;

pub fn main() -> iced::Result {
    let client = client::Client::new("http://127.0.0.1:7218");
    app::App::new(client).run()
}
