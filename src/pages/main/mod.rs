mod home;
mod market;
mod receive;
mod send;
mod settings;
mod sign;
mod spaces;
mod state;

use iced::{
    Center, Element, Fill, Subscription, Task, Theme, clipboard, time,
    widget::{Column, button, center, column, container, row, text, vertical_rule, vertical_space},
};

use crate::{
    Config,
    client::*,
    widget::icon::{Icon, text_icon},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Home,
    Send,
    Receive,
    Spaces,
    Market,
    Sign,
    Settings,
}

#[derive(Debug)]
pub struct State {
    config: Config,
    client: Client,
    screen: Screen,
    tip_height: u32,
    wallets: state::WalletsCollection,
    spaces: state::SpacesCollection,
    home_screen: home::State,
    send_screen: send::State,
    receive_screen: receive::State,
    spaces_screen: spaces::State,
    market_screen: market::State,
    sign_screen: sign::State,
    settings_screen: settings::State,
}

#[derive(Debug, Clone)]
pub enum Route {
    Home,
    Send,
    Receive,
    Spaces,
    Space(SLabel),
    Market,
    Sign,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    NavigateTo(Route),
    ServerInfo(ClientResult<ServerInfo>),
    ListWallets(ClientResult<Vec<String>>),
    WalletLoad(WalletResult<()>),
    WalletInfo(WalletResult<WalletInfoWithProgress>),
    WalletBalance(WalletResult<Balance>),
    WalletSpaces(WalletResult<ListSpacesResponse>),
    WalletTransactions(WalletResult<Vec<TxInfo>>),
    WalletAddress(WalletResult<(AddressKind, String)>),
    SpaceInfo(ClientResult<(SLabel, Option<FullSpaceOut>)>),
    HomeScreen(home::Message),
    SendScreen(send::Message),
    ReceiveScreen(receive::Message),
    SpacesScreen(spaces::Message),
    MarketScreen(market::Message),
    SignScreen(sign::Message),
    SettingsScreen(settings::Message),
}

pub enum Action {
    Return(Config),
    Task(Task<Message>),
}

impl State {
    pub fn run(config: Config, client: Client) -> (Self, Task<Message>) {
        let state = Self {
            config,
            client,
            screen: Screen::Home,
            tip_height: 0,
            wallets: Default::default(),
            spaces: Default::default(),
            home_screen: Default::default(),
            send_screen: Default::default(),
            receive_screen: Default::default(),
            spaces_screen: Default::default(),
            market_screen: Default::default(),
            sign_screen: Default::default(),
            settings_screen: Default::default(),
        };
        let task = Task::batch([state.get_server_info(), state.list_wallets()]);
        (state, task)
    }

    fn get_server_info(&self) -> Task<Message> {
        self.client.get_server_info().map(Message::ServerInfo)
    }

    fn list_wallets(&self) -> Task<Message> {
        self.client.list_wallets().map(Message::ListWallets)
    }

    fn get_wallet_info(&self) -> Task<Message> {
        if let Some(wallet) = self.wallets.get_current() {
            self.client
                .get_wallet_info(wallet.label.to_string())
                .map(Message::WalletInfo)
        } else {
            Task::none()
        }
    }

    fn get_wallet_balance(&self) -> Task<Message> {
        if let Some(wallet) = self.wallets.get_current() {
            self.client
                .get_wallet_balance(wallet.label.to_string())
                .map(Message::WalletBalance)
        } else {
            Task::none()
        }
    }

    fn get_wallet_spaces(&self) -> Task<Message> {
        if let Some(wallet) = self.wallets.get_current() {
            self.client
                .get_wallet_spaces(wallet.label.to_string())
                .map(Message::WalletSpaces)
        } else {
            Task::none()
        }
    }

    fn get_wallet_transactions(&self) -> Task<Message> {
        if let Some(wallet) = self.wallets.get_current() {
            self.client
                .get_wallet_transactions(
                    wallet.label.to_string(),
                    self.home_screen.get_transactions_limit(),
                )
                .map(Message::WalletTransactions)
        } else {
            Task::none()
        }
    }

    fn get_wallet_address(&self, address_kind: AddressKind) -> Task<Message> {
        if let Some(wallet) = self.wallets.get_current() {
            self.client
                .get_wallet_address(wallet.label.to_string(), address_kind)
                .map(Message::WalletAddress)
        } else {
            Task::none()
        }
    }

    fn get_space_info(&self, slabel: SLabel) -> Task<Message> {
        self.client.get_space_info(slabel).map(Message::SpaceInfo)
    }

    fn navigate_to(&mut self, route: Route) -> Task<Message> {
        match route {
            Route::Home => {
                if self.screen == Screen::Home {
                    self.home_screen.reset();
                } else {
                    self.screen = Screen::Home;
                }
                Task::batch([
                    self.get_wallet_balance(),
                    self.get_wallet_spaces(),
                    self.get_wallet_transactions(),
                ])
            }
            Route::Send => {
                self.screen = Screen::Send;
                self.get_wallet_spaces()
            }
            Route::Receive => {
                self.screen = Screen::Receive;
                Task::batch([
                    self.get_wallet_address(AddressKind::Coin),
                    self.get_wallet_address(AddressKind::Space),
                ])
            }
            Route::Spaces => {
                if self.screen == Screen::Spaces {
                    self.spaces_screen.reset();
                } else {
                    self.screen = Screen::Spaces;
                }
                if let Some(slabel) = self.spaces_screen.get_slabel() {
                    self.get_space_info(slabel)
                } else {
                    self.get_wallet_spaces()
                }
            }
            Route::Space(slabel) => {
                self.screen = Screen::Spaces;
                self.spaces_screen.set_slabel(&slabel);
                self.get_space_info(slabel)
            }
            Route::Market => {
                self.screen = Screen::Market;
                self.get_wallet_spaces()
            }
            Route::Sign => {
                self.screen = Screen::Sign;
                self.get_wallet_spaces()
            }
            Route::Settings => {
                self.screen = Screen::Settings;
                Task::none()
            }
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Tick => {
                let mut tasks = vec![self.get_server_info(), self.get_wallet_info()];
                match self.screen {
                    Screen::Home => {
                        tasks.push(self.get_wallet_balance());
                        tasks.push(self.get_wallet_transactions());
                    }
                    Screen::Spaces => {
                        tasks.push(self.get_wallet_spaces());
                        if let Some(slabel) = self.spaces_screen.get_slabel() {
                            tasks.push(self.get_space_info(slabel));
                        }
                    }
                    _ => {}
                }
                Action::Task(Task::batch(tasks))
            }
            Message::NavigateTo(route) => Action::Task(self.navigate_to(route)),
            Message::ServerInfo(result) => {
                if let Ok(server_info) = result {
                    self.tip_height = server_info.chain.headers;
                }
                Action::Task(Task::none())
            }
            Message::ListWallets(result) => Action::Task(match result {
                Ok(wallets_names) => {
                    self.wallets.set_wallets(&wallets_names);
                    if self.wallets.get_current().is_none() {
                        if let Some(name) = self.config.wallet.as_ref() {
                            self.wallets.set_current(name);
                        }
                    }
                    if let Some(wallet) = self.wallets.get_current() {
                        self.client
                            .load_wallet(wallet.label.clone())
                            .map(Message::WalletLoad)
                    } else {
                        self.navigate_to(Route::Settings)
                    }
                }
                Err(_) => self.list_wallets(),
            }),
            Message::WalletLoad(result) => Action::Task(if result.result.is_ok() {
                Task::batch([self.get_wallet_info(), self.navigate_to(Route::Home)])
            } else {
                Task::none()
            }),
            Message::WalletInfo(WalletResult {
                label: wallet,
                result,
            }) => {
                if let Ok(wallet_info) = result {
                    if let Some(wallet_state) = self.wallets.get_data_mut(&wallet) {
                        wallet_state.status = wallet_info.status;
                    }
                }
                Action::Task(Task::none())
            }
            Message::WalletBalance(WalletResult {
                label: wallet,
                result,
            }) => {
                if let Ok(balance) = result {
                    if let Some(wallet_state) = self.wallets.get_data_mut(&wallet) {
                        wallet_state.balance = balance.balance;
                    }
                }
                Action::Task(Task::none())
            }
            Message::WalletSpaces(WalletResult {
                label: wallet,
                result,
            }) => {
                if let Ok(spaces) = result {
                    if let Some(wallet_state) = self.wallets.get_data_mut(&wallet) {
                        let mut collect = |spaces: Vec<FullSpaceOut>| -> Vec<SLabel> {
                            spaces
                                .into_iter()
                                .map(|out| {
                                    let name = out.spaceout.space.as_ref().unwrap().name.clone();
                                    self.spaces.set(name.clone(), Some(out));
                                    name
                                })
                                .collect()
                        };
                        wallet_state.winning_spaces = collect(spaces.winning);
                        wallet_state.outbid_spaces = collect(spaces.outbid);
                        wallet_state.owned_spaces = collect(spaces.owned);
                    }
                }
                Action::Task(Task::none())
            }
            Message::WalletTransactions(WalletResult {
                label: wallet,
                result,
            }) => {
                if let Ok(transactions) = result {
                    if let Some(wallet_state) = self.wallets.get_data_mut(&wallet) {
                        wallet_state.transactions = transactions;
                    }
                }
                Action::Task(Task::none())
            }
            Message::WalletAddress(WalletResult {
                label: wallet,
                result,
            }) => {
                if let Ok((address_kind, address)) = result {
                    if let Some(wallet_state) = self.wallets.get_data_mut(&wallet) {
                        let address = Some(state::AddressData::new(address));
                        match address_kind {
                            AddressKind::Coin => wallet_state.coin_address = address,
                            AddressKind::Space => wallet_state.space_address = address,
                        }
                    }
                }
                Action::Task(Task::none())
            }
            Message::SpaceInfo(result) => {
                if let Ok((slabel, out)) = result {
                    self.spaces.set(slabel, out)
                }
                Action::Task(Task::none())
            }
            Message::HomeScreen(message) => Action::Task(match self.home_screen.update(message) {
                home::Action::WriteClipboard(s) => clipboard::write(s),
                home::Action::ShowSpace { slabel } => self.navigate_to(Route::Space(slabel)),
                home::Action::GetTransactions => self.get_wallet_transactions(),
                home::Action::BumpFee { txid, fee_rate } => self
                    .client
                    .bump_fee(
                        self.wallets.get_current().unwrap().label.clone(),
                        txid,
                        fee_rate,
                    )
                    .map(|r| {
                        Message::HomeScreen(home::Message::BumpFeeResult(r.result.map(|_| ())))
                    }),
                home::Action::None => Task::none(),
            }),
            Message::SendScreen(message) => Action::Task(match self.send_screen.update(message) {
                send::Action::SendCoins {
                    recipient,
                    amount,
                    fee_rate,
                } => self
                    .client
                    .send_coins(
                        self.wallets.get_current().unwrap().label.clone(),
                        recipient,
                        amount,
                        fee_rate,
                    )
                    .map(|r| {
                        Message::SendScreen(send::Message::ClientResult(r.result.map(|_| ())))
                    }),
                send::Action::SendSpace {
                    recipient,
                    slabel,
                    fee_rate,
                } => self
                    .client
                    .send_space(
                        self.wallets.get_current().unwrap().label.clone(),
                        recipient,
                        slabel,
                        fee_rate,
                    )
                    .map(|r| {
                        Message::SendScreen(send::Message::ClientResult(r.result.map(|_| ())))
                    }),
                send::Action::ShowTransactions => self.navigate_to(Route::Home),
                send::Action::None => Task::none(),
            }),
            Message::ReceiveScreen(message) => {
                Action::Task(match self.receive_screen.update(message) {
                    receive::Action::WriteClipboard(s) => clipboard::write(s),
                    receive::Action::None => Task::none(),
                })
            }
            Message::SpacesScreen(message) => {
                Action::Task(match self.spaces_screen.update(message) {
                    spaces::Action::WriteClipboard(s) => clipboard::write(s),
                    spaces::Action::GetSpaceInfo { slabel } => self.get_space_info(slabel),
                    spaces::Action::OpenSpace {
                        slabel,
                        amount,
                        fee_rate,
                    } => self
                        .client
                        .open_space(
                            self.wallets.get_current().unwrap().label.clone(),
                            slabel,
                            amount,
                            fee_rate,
                        )
                        .map(|r| {
                            Message::SpacesScreen(spaces::Message::ClientResult(
                                r.result.map(|_| ()),
                            ))
                        }),
                    spaces::Action::BidSpace {
                        slabel,
                        amount,
                        fee_rate,
                    } => self
                        .client
                        .bid_space(
                            self.wallets.get_current().unwrap().label.clone(),
                            slabel,
                            amount,
                            fee_rate,
                        )
                        .map(|r| {
                            Message::SpacesScreen(spaces::Message::ClientResult(
                                r.result.map(|_| ()),
                            ))
                        }),
                    spaces::Action::RegisterSpace { slabel, fee_rate } => self
                        .client
                        .register_space(
                            self.wallets.get_current().unwrap().label.clone(),
                            slabel,
                            fee_rate,
                        )
                        .map(|r| {
                            Message::SpacesScreen(spaces::Message::ClientResult(
                                r.result.map(|_| ()),
                            ))
                        }),
                    spaces::Action::RenewSpace { slabel, fee_rate } => self
                        .client
                        .renew_space(
                            self.wallets.get_current().unwrap().label.clone(),
                            slabel,
                            fee_rate,
                        )
                        .map(|r| {
                            Message::SpacesScreen(spaces::Message::ClientResult(
                                r.result.map(|_| ()),
                            ))
                        }),
                    spaces::Action::ShowTransactions => self.navigate_to(Route::Home),
                    spaces::Action::None => Task::none(),
                })
            }
            Message::MarketScreen(message) => {
                Action::Task(match self.market_screen.update(message) {
                    market::Action::Buy { listing, fee_rate } => self
                        .client
                        .buy_space(
                            self.wallets.get_current().unwrap().label.clone(),
                            listing,
                            fee_rate,
                        )
                        .map(|r| Message::MarketScreen(market::Message::BuyResult(r.result))),
                    market::Action::Sell { slabel, price } => self
                        .client
                        .sell_space(
                            self.wallets.get_current().unwrap().label.clone(),
                            slabel,
                            price,
                        )
                        .map(|r| Message::MarketScreen(market::Message::SellResult(r.result))),
                    market::Action::WriteClipboard(s) => clipboard::write(s),
                    market::Action::ShowTransactions => self.navigate_to(Route::Home),
                    market::Action::None => Task::none(),
                })
            }
            Message::SignScreen(message) => Action::Task(match self.sign_screen.update(message) {
                sign::Action::FilePick => Task::future(async move {
                    let path = rfd::AsyncFileDialog::new()
                        .add_filter("JSON event", &["json"])
                        .pick_file()
                        .await
                        .map(|file| file.path().to_path_buf());

                    let result = if let Some(path) = path {
                        match tokio::fs::read_to_string(&path).await {
                            Ok(content) => match serde_json::from_str::<NostrEvent>(&content) {
                                Ok(event) => Ok(Some((path.to_string_lossy().to_string(), event))),
                                Err(err) => Err(format!("Failed to parse JSON: {}", err)),
                            },
                            Err(err) => Err(format!("Failed to read file: {}", err)),
                        }
                    } else {
                        Ok(None)
                    };
                    Message::SignScreen(sign::Message::EventFileLoaded(result))
                }),
                sign::Action::Sign(slabel, event) => self
                    .client
                    .sign_event(
                        self.wallets.get_current().unwrap().label.clone(),
                        slabel,
                        event,
                    )
                    .then(|result| {
                        let result = result.result;
                        Task::future(async move {
                            let result = match result {
                                Ok(event) => {
                                    let file_path = rfd::AsyncFileDialog::new()
                                        .add_filter("JSON event", &["json"])
                                        .add_filter("All files", &["*"])
                                        .save_file()
                                        .await
                                        .map(|file| file.path().to_path_buf());

                                    if let Some(file_path) = file_path {
                                        use spaces_wallet::bdk_wallet::serde_json;
                                        let contents = serde_json::to_vec(&event).unwrap();
                                        tokio::fs::write(&file_path, contents)
                                            .await
                                            .map_err(|e| e.to_string())
                                    } else {
                                        Ok(())
                                    }
                                }
                                Err(err) => Err(err),
                            };
                            Message::SignScreen(sign::Message::EventFileSaved(result))
                        })
                    }),
                sign::Action::None => Task::none(),
            }),
            Message::SettingsScreen(message) => match self.settings_screen.update(message) {
                settings::Action::SetCurrentWallet(name) => {
                    self.wallets.set_current(&name);
                    self.config.wallet = Some(name);
                    self.config.save();
                    Action::Task(self.list_wallets())
                }
                settings::Action::ExportWallet(wallet_name) => {
                    Action::Task(self.client.export_wallet(wallet_name).then(|result| {
                        let result = result.result;
                        Task::future(async move {
                            let result = match result {
                                Ok(contents) => {
                                    let file_path = rfd::AsyncFileDialog::new()
                                        .add_filter("Wallet file", &["json"])
                                        .add_filter("All files", &["*"])
                                        .save_file()
                                        .await
                                        .map(|file| file.path().to_path_buf());

                                    if let Some(file_path) = file_path {
                                        tokio::fs::write(&file_path, contents)
                                            .await
                                            .map_err(|e| e.to_string())
                                    } else {
                                        Ok(())
                                    }
                                }
                                Err(err) => Err(err),
                            };
                            Message::SettingsScreen(settings::Message::WalletFileSaved(result))
                        })
                    }))
                }
                settings::Action::CreateWallet(wallet_name) => {
                    self.config.wallet = None;
                    self.wallets.unset_current();
                    Action::Task(
                        self.client
                            .create_wallet(wallet_name)
                            .map(|r| {
                                Message::SettingsScreen(settings::Message::WalletCreated(r.result))
                            })
                            .chain(self.list_wallets()),
                    )
                }
                settings::Action::FilePick => Action::Task(
                    Task::future(async move {
                        let result = rfd::AsyncFileDialog::new()
                            .add_filter("wallet file", &["json"])
                            .pick_file()
                            .await;
                        match result {
                            Some(file) => tokio::fs::read_to_string(file.path()).await.ok(),
                            None => None,
                        }
                    })
                    .map(|r| Message::SettingsScreen(settings::Message::WalletFileLoaded(r))),
                ),
                settings::Action::ImportWallet(contents) => {
                    self.config.wallet = None;
                    self.wallets.unset_current();
                    Action::Task(
                        self.client
                            .import_wallet(&contents)
                            .map(|r| {
                                Message::SettingsScreen(settings::Message::WalletFileImported(
                                    r.map(|_| ()),
                                ))
                            })
                            .chain(self.list_wallets()),
                    )
                }
                settings::Action::ResetBackend => Action::Return(self.config.clone()),
                settings::Action::None => Action::Task(Task::none()),
            },
        }
    }

    pub fn view(&self) -> Element<Message> {
        let navbar_button = |label, icon: Icon, route: Route, screen: Screen| {
            let button = button(
                row![text_icon(icon).size(20), text(label).size(16)]
                    .spacing(10)
                    .align_y(Center),
            )
            .style(move |theme: &Theme, status: button::Status| {
                let mut style = if self.screen == screen {
                    button::primary
                } else {
                    button::text
                }(theme, status);
                style.border = style.border.rounded(7);
                style
            })
            .width(Fill);
            button.on_press(Message::NavigateTo(route))
        };

        Column::new()
            .push_maybe(self.wallets.get_current().and_then(|wallet| {
                use spaces_client::wallets::WalletProgressUpdate;
                match wallet.state.status {
                    WalletProgressUpdate::SourceSync { total, completed } |
                    WalletProgressUpdate::CbfFilterSync { total, completed } |
                    WalletProgressUpdate::CbfProcessFilters { total, completed } |
                    WalletProgressUpdate::CbfDownloadMatchingBlocks { total, completed } |
                    WalletProgressUpdate::CbfProcessMatchingBlocks { total, completed } => {
                        Some(format!("Syncing {} / {}", completed, total))
                    },
                    WalletProgressUpdate::CbfApplyUpdate |
                    WalletProgressUpdate::Syncing => {
                        Some("Syncing".to_string())
                    },
                    WalletProgressUpdate::Complete => None,
                }
            }).map(|t| {
                container(text(t).align_x(Center).width(Fill))
                    .style(|theme: &Theme| {
                        container::Style::default()
                            .background(theme.extended_palette().secondary.base.color)
                    })
                    .width(Fill)
                    .padding([10, 0])
            }))
            .push(row![
                column![
                    navbar_button("Home", Icon::CurrencyBitcoin, Route::Home, Screen::Home,),
                    navbar_button("Send", Icon::ArrowDownFromArc, Route::Send, Screen::Send,),
                    navbar_button(
                        "Receive",
                        Icon::ArrowDownToArc,
                        Route::Receive,
                        Screen::Receive,
                    ),
                    navbar_button("Spaces", Icon::At, Route::Spaces, Screen::Spaces,),
                    navbar_button("Market", Icon::BuildingBank, Route::Market, Screen::Market,),
                    navbar_button("Sign", Icon::Signature, Route::Sign, Screen::Sign,),
                    vertical_space(),
                    navbar_button(
                        "Settings",
                        Icon::Settings,
                        Route::Settings,
                        Screen::Settings,
                    ),
                ]
                .padding(10)
                .spacing(5)
                .width(200),
                vertical_rule(3),
                container(match &self.screen {
                    Screen::Home =>
                        if let Some(wallet) = self.wallets.get_current() {
                            self.home_screen
                                .view(
                                    self.tip_height,
                                    wallet.state.balance,
                                    &wallet.state.transactions,
                                )
                                .map(Message::HomeScreen)
                        } else {
                            center("No wallet loaded").into()
                        },
                    Screen::Send =>
                        if let Some(wallet) = self.wallets.get_current() {
                            self.send_screen
                                .view(&wallet.state.owned_spaces)
                                .map(Message::SendScreen)
                        } else {
                            center("No wallet loaded").into()
                        },
                    Screen::Receive =>
                        if let Some(wallet) = self.wallets.get_current() {
                            self.receive_screen
                                .view(
                                    wallet.state.coin_address.as_ref(),
                                    wallet.state.space_address.as_ref(),
                                )
                                .map(Message::ReceiveScreen)
                        } else {
                            center("No wallet loaded").into()
                        },
                    Screen::Spaces =>
                        if let Some(wallet) = self.wallets.get_current() {
                            self.spaces_screen
                                .view(
                                    self.tip_height,
                                    &self.spaces,
                                    &wallet.state.winning_spaces,
                                    &wallet.state.outbid_spaces,
                                    &wallet.state.owned_spaces,
                                )
                                .map(Message::SpacesScreen)
                        } else {
                            center("No wallet loaded").into()
                        },
                    Screen::Market =>
                        if let Some(wallet) = self.wallets.get_current() {
                            self.market_screen
                                .view(wallet.state.owned_spaces.as_ref())
                                .map(Message::MarketScreen)
                        } else {
                            center("No wallet loaded").into()
                        },
                    Screen::Sign =>
                        if let Some(wallet) = self.wallets.get_current() {
                            self.sign_screen
                                .view(&wallet.state.owned_spaces)
                                .map(Message::SignScreen)
                        } else {
                            center("No wallet loaded").into()
                        },
                    Screen::Settings => self
                        .settings_screen
                        .view(
                            self.wallets.get_wallets(),
                            self.wallets.get_current().map(|w| w.label),
                        )
                        .map(Message::SettingsScreen),
                })
            ])
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        time::every(
            if self.tip_height != 0 && self.wallets.get_current().is_some_and(|w| w.is_synced()) {
                time::Duration::from_secs(30)
            } else {
                time::Duration::from_secs(5)
            },
        )
        .map(|_| Message::Tick)
    }
}
