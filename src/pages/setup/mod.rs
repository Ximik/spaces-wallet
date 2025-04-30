use iced::{
    Center, Element, Fill, Task,
    widget::{button, checkbox, column, container, row},
};

use spaces_client::config::ExtendedNetwork;

use crate::{
    Config,
    client::{Client, ClientResult, ServerInfo},
    widget::{
        form::{pick_list, submit_button, text_input, text_label},
        icon::{Icon, button_icon},
        text::{error_block, text_big},
    },
};

#[derive(Debug)]
pub struct State {
    config: Config,
    client: Option<Client>,
    connected: bool,
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
    ListWalletsResult(ClientResult<Vec<String>>),
    Disconnect,
    CreateWallet,
    ImportWallet,
    ImportWalletPicked(Result<String, String>),
    SetWalletResult(Result<String, String>),
}

pub enum Action {
    Return(Config, Client),
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
                connected: false,
                rpc_url,
                network,
                error: None,
            },
            task,
        )
    }

    fn finish(&mut self) -> Action {
        self.config.save();
        Action::Return(self.config.clone(), self.client.take().unwrap())
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
                            let task = client.get_server_info().map(Message::ConnectResult);
                            self.client = Some(client);
                            Action::Task(task)
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
                        if self.config.wallet.is_some() {
                            self.finish()
                        } else {
                            Action::Task(
                                self.client
                                    .as_ref()
                                    .unwrap()
                                    .list_wallets()
                                    .map(Message::ListWalletsResult),
                            )
                        }
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
            Message::ListWalletsResult(result) => match result {
                Ok(wallets) => {
                    if wallets.is_empty() {
                        self.connected = true;
                        Action::none()
                    } else {
                        if self.config.wallet.is_none() && wallets.contains(&"default".to_string())
                        {
                            self.config.wallet = Some("default".to_string());
                        }
                        self.finish()
                    }
                }
                Err(err) => {
                    self.client = None;
                    self.error = Some(err);
                    Action::none()
                }
            },
            Message::Disconnect => {
                self.client = None;
                self.connected = false;
                Action::none()
            }
            Message::CreateWallet => Action::Task(
                self.client
                    .as_ref()
                    .unwrap()
                    .create_wallet("default".to_string())
                    .map(|r| Message::SetWalletResult(r.result.map(|_| r.label))),
            ),
            Message::ImportWallet => Action::Task(Task::perform(
                async move {
                    let result = rfd::AsyncFileDialog::new()
                        .add_filter("wallet file", &["json"])
                        .pick_file()
                        .await;
                    match result {
                        Some(file) => tokio::fs::read_to_string(file.path())
                            .await
                            .map_err(|e| e.to_string()),
                        None => Err("No file selected".to_string()),
                    }
                },
                Message::ImportWalletPicked,
            )),
            Message::ImportWalletPicked(result) => match result {
                Ok(contents) => Action::Task(
                    self.client
                        .as_ref()
                        .unwrap()
                        .import_wallet(&contents)
                        .map(Message::SetWalletResult),
                ),
                Err(err) => {
                    self.error = Some(err);
                    Action::none()
                }
            },
            Message::SetWalletResult(result) => match result {
                Ok(wallet) => {
                    self.config.wallet = Some(wallet);
                    self.finish()
                }
                Err(err) => {
                    self.error = Some(err);
                    Action::none()
                }
            },
        }
    }

    pub fn view(&self) -> Element<Message> {
        container(if !self.connected {
            column![
                text_big("Set up spaced connection"),
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
            .spacing(10)
        } else {
            column![
                row![
                    button_icon(Icon::ChevronLeft)
                        .style(button::text)
                        .on_press(Message::Disconnect),
                    text_big("Set up wallet"),
                ]
                .align_y(Center),
                error_block(self.error.as_ref()),
                row![
                    submit_button("Create wallet", Some(Message::CreateWallet)),
                    submit_button("Import wallet", Some(Message::ImportWallet)),
                ],
            ]
            .spacing(10)
        })
        .padding([60, 100])
        .into()
    }
}
