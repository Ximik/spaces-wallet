use iced::{
    Element, Task, exit,
    widget::{center, checkbox, column},
};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use spaces_client::config::ExtendedNetwork;

use crate::{
    branding::*,
    client::Client,
    widget::form::{pick_list, submit_button, text_input, text_label},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub path: PathBuf,
    pub setup: bool,
    pub spaced_rpc_url: Option<String>,
    pub network: ExtendedNetwork,
}

#[derive(Debug, Clone)]
pub enum Message {
    SpacedRpcUrlToggle(bool),
    SpacedRpcUrlInput(String),
    NetworkSelect(ExtendedNetwork),
    SavePress,
}

impl Config {
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let path: PathBuf = path.into();
        let config = if path.exists() {
            let config = fs::read_to_string(&path)?;
            Self {
                path,
                ..serde_json::from_str(&config)?
            }
        } else {
            Self {
                path,
                setup: true,
                spaced_rpc_url: None,
                network: ExtendedNetwork::Mainnet,
            }
        };
        Ok(config)
    }

    pub fn run(mut self) -> iced::Result {
        self.setup = false;
        iced::application(WINDOW_TITLE, Self::update, Self::view)
            .font(ICONS_FONT.clone())
            .window(iced::window::Settings {
                size: (350.0, 300.0).into(),
                icon: Some(WINDOW_ICON.clone()),
                ..Default::default()
            })
            .theme(|_| BITCOIN_THEME.clone())
            .run_with(move || (self, Task::none()))
    }
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SpacedRpcUrlToggle(some) => {
                self.spaced_rpc_url = if some { Some(String::new()) } else { None };
                Task::none()
            }
            Message::SpacedRpcUrlInput(spaced_rpc_url) => {
                self.spaced_rpc_url = Some(spaced_rpc_url);
                Task::none()
            }
            Message::NetworkSelect(network) => {
                self.network = network;
                Task::none()
            }
            Message::SavePress => {
                let config = serde_json::to_string_pretty(&self).unwrap();
                fs::write(&self.path, config).unwrap();
                exit()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        center(
            column![
                column![
                    checkbox("Use standalone spaced node", self.spaced_rpc_url.is_some())
                        .on_toggle(Message::SpacedRpcUrlToggle),
                    text_label("JSON-RPC address"),
                    text_input(
                        "http://127.0.0.1:7225",
                        self.spaced_rpc_url.as_ref().map_or("", |v| v),
                    )
                    .on_input_maybe(
                        self.spaced_rpc_url
                            .as_ref()
                            .map(|_| Message::SpacedRpcUrlInput)
                    ),
                ]
                .spacing(10),
                column![
                    text_label("Chain"),
                    pick_list(
                        [
                            ExtendedNetwork::Mainnet,
                            ExtendedNetwork::Testnet4,
                            ExtendedNetwork::Regtest
                        ],
                        Some(self.network),
                        Message::NetworkSelect
                    )
                ]
                .spacing(10),
                submit_button("Save", Some(Message::SavePress))
            ]
            .spacing(10),
        )
        .padding(20)
        .into()
    }
}
