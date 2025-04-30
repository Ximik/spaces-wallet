use iced::{
    Center, Element, Fill, Task,
    widget::{checkbox, column, container},
};

use spaces_client::config::ExtendedNetwork;

use crate::{
    Config,
    client::{Client, ClientResult, ServerInfo},
    widget::{
        form::{pick_list, submit_button, text_input, text_label},
        text::error_block,
    },
};

#[derive(Debug)]
pub struct State {
    config: Config,
    client: Option<Client>,
    error: Option<String>,
    rpc_url: Option<String>,
    network: ExtendedNetwork,
}

#[derive(Debug, Clone)]
pub enum Message {
    UrlToggle(bool),
    UrlInput(String),
    NetworkSelect(ExtendedNetwork),
    Connect,
    ConnectResult(ClientResult<ServerInfo>),
}

pub enum Action {
    Exit(Config, Client),
    Task(Task<Message>),
}

impl Action {
    fn none() -> Action {
        Action::Task(Task::none())
    }
}

const DEFAULT_RPC_URL: &str = "http://127.0.0.1:7225";

impl State {
    pub fn run(config: Config, autoload: bool) -> (Self, Task<Message>) {
        let rpc_url = config.spaced_rpc_url.clone();
        let network = config.network;
        let task = if autoload {
            Task::done(Message::Connect)
        } else {
            Task::none()
        };
        (
            Self {
                config,
                client: None,
                rpc_url,
                network,
                error: None,
            },
            task,
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        self.error = None;
        match message {
            Message::UrlToggle(some) => {
                self.rpc_url = if some {
                    Some(DEFAULT_RPC_URL.into())
                } else {
                    None
                };
                Action::none()
            }
            Message::UrlInput(spaced_rpc_url) => {
                self.rpc_url = Some(spaced_rpc_url);
                Action::none()
            }
            Message::NetworkSelect(network) => {
                self.network = network;
                Action::none()
            }
            Message::Connect => {
                if let Some(rpc_url) = self.rpc_url.as_ref() {
                    match Client::new(rpc_url) {
                        Ok(client) => {
                            let task =
                                Action::Task(client.get_server_info().map(Message::ConnectResult));
                            self.client = Some(client);
                            task
                        }
                        Err(err) => Action::Task(Task::done(Message::ConnectResult(Err(err)))),
                    }
                } else {
                    unimplemented!()
                }
            }
            Message::ConnectResult(result) => match result {
                Ok(info) => {
                    if info.network == self.network.to_string() {
                        if self.config.spaced_rpc_url != self.rpc_url
                            || self.config.network != self.network
                        {
                            self.config.spaced_rpc_url = self.rpc_url.clone();
                            self.config.network = self.network;
                            self.config.wallet = None;
                        }
                        self.config.save();
                        Action::Exit(self.config.clone(), self.client.take().unwrap())
                    } else {
                        self.client = None;
                        self.error = Some("Wrong network".to_string());
                        Action::none()
                    }
                }
                Err(err) => {
                    self.client = None;
                    self.error = Some(err);
                    Action::none()
                }
            },
        }
    }

    pub fn view(&self) -> Element<Message> {
        container(
            column![
                error_block(self.error.as_ref()),
                column![
                    checkbox("Use standalone spaced node", self.rpc_url.is_some())
                        .on_toggle(Message::UrlToggle),
                    text_label("JSON-RPC address"),
                    text_input(DEFAULT_RPC_URL, self.rpc_url.as_ref().map_or("", |v| v),)
                        .on_input_maybe(self.rpc_url.as_ref().map(|_| Message::UrlInput)),
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
                container(submit_button(
                    "Connect",
                    if self.client.is_some() || self.rpc_url.as_ref().is_some_and(|s| s.is_empty())
                    {
                        None
                    } else {
                        Some(Message::Connect)
                    }
                ))
                .align_x(Center)
                .width(Fill)
            ]
            .spacing(10),
        )
        .padding(100)
        .into()
    }
}
