use iced::time;
use iced::widget::{button, center, column, container, row, text, vertical_rule};
use iced::{Center, Element, Fill, Subscription, Task, clipboard};
use spaces_client::rpc::ServerInfo;

use crate::client::{Client, ClientError};
use crate::screen;
use crate::types::*;
use crate::widget::{
    icon::{Icon, text_icon},
    text::error_block,
};

#[derive(Debug, Clone)]
enum Route {
    Home,
    Send,
    Receive,
    Spaces,
    Space(SLabel),
    Market,
    Sign,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    NavigateTo(Route),
    ClientError(String),
    ServerInfo(Result<ServerInfo, String>),
    WalletLoad(Result<String, String>),
    WalletBalance(String, Result<Balance, String>),
    WalletSpaces(String, Result<ListSpacesResponse, String>),
    WalletTransactions(String, Result<Vec<TxInfo>, String>),
    WalletAddress(String, AddressKind, Result<String, String>),
    SpaceInfo(SLabel, Result<Option<FullSpaceOut>, String>),
    HomeScreen(screen::home::Message),
    SendScreen(screen::send::Message),
    ReceiveScreen(screen::receive::Message),
    SpacesScreen(screen::spaces::Message),
    MarketScreen(screen::market::Message),
    SignScreen(screen::sign::Message),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Home,
    Send,
    Receive,
    Spaces,
    Market,
    Sign,
}

#[derive(Debug)]
pub struct App {
    client: Client,
    client_error: Option<String>,
    screen: Screen,
    tip_height: u32,
    wallet_name: Option<String>,
    wallets: WalletsState,
    spaces: SpacesState,
    home_screen: screen::home::State,
    send_screen: screen::send::State,
    receive_screen: screen::receive::State,
    spaces_screen: screen::spaces::State,
    market_screen: screen::market::State,
    sign_screen: screen::sign::State,
}

impl App {
    pub fn run(args: crate::Args) -> iced::Result {
        let icon =
            iced::window::icon::from_rgba(include_bytes!("../assets/spaces.rgba").to_vec(), 64, 64)
                .expect("Failed to load icon");
        let icons_font = include_bytes!("../assets/icons.ttf").as_slice();
        iced::application(Self::title, Self::update, Self::view)
            .font(icons_font)
            .subscription(Self::subscription)
            .window(iced::window::Settings {
                min_size: Some((1300.0, 500.0).into()),
                icon: Some(icon),
                ..Default::default()
            })
            .theme(move |_| {
                iced::Theme::custom_with_fn(
                    "Bitcoin".into(),
                    iced::theme::Palette {
                        text: iced::Color::from_rgb8(77, 77, 77),
                        primary: iced::Color::from_rgb8(247, 147, 26),
                        ..iced::theme::Palette::LIGHT
                    },
                    |pallete| {
                        let mut pallete = iced::theme::palette::Extended::generate(pallete);
                        pallete.primary.base.text = iced::Color::WHITE;
                        pallete.primary.strong.text = iced::Color::WHITE;
                        pallete.primary.weak.text = iced::Color::WHITE;
                        pallete
                    },
                )
            })
            .run_with(move || Self::new(args))
    }

    fn title(&self) -> String {
        "Spaces Wallet".into()
    }

    fn new(args: crate::Args) -> (Self, Task<Message>) {
        let client = Client::new(&args.spaced_rpc_url.unwrap());
        let app = Self {
            client: client.clone(),
            client_error: None,
            screen: Screen::Home,
            tip_height: 0,
            wallet_name: None,
            wallets: Default::default(),
            spaces: Default::default(),
            home_screen: Default::default(),
            send_screen: Default::default(),
            receive_screen: Default::default(),
            spaces_screen: Default::default(),
            market_screen: Default::default(),
            sign_screen: Default::default(),
        };
        let task = Task::batch([
            app.get_server_info(),
            app.load_wallet("default".to_string()),
        ]);
        (app, task)
    }

    fn get_server_info(&self) -> Task<Message> {
        let client = self.client.clone();
        Task::future(async move {
            let result = client.get_server_info().await;
            Message::ServerInfo(result)
        })
    }

    fn load_wallet(&self, wallet_name: String) -> Task<Message> {
        let client = self.client.clone();
        Task::future(async move {
            _ = client.create_wallet(&wallet_name).await;
            let result = client.load_wallet(&wallet_name).await;
            Message::WalletLoad(result.map(|_| wallet_name))
        })
    }

    fn get_wallet_balance(&self) -> Task<Message> {
        if self.wallet_name.is_none() {
            return Task::none();
        }
        let client = self.client.clone();
        let wallet_name = self.wallet_name.as_ref().unwrap().clone();
        Task::future(async move {
            let result = client.get_wallet_balance(&wallet_name).await;
            Message::WalletBalance(wallet_name, result)
        })
    }

    fn get_wallet_spaces(&self) -> Task<Message> {
        if self.wallet_name.is_none() {
            return Task::none();
        }
        let client = self.client.clone();
        let wallet_name = self.wallet_name.as_ref().unwrap().clone();
        Task::future(async move {
            let result = client.get_wallet_spaces(&wallet_name).await;
            Message::WalletSpaces(wallet_name, result)
        })
    }

    fn get_wallet_transactions(&self) -> Task<Message> {
        if self.wallet_name.is_none() {
            return Task::none();
        }
        let client = self.client.clone();
        let wallet_name = self.wallet_name.as_ref().unwrap().clone();
        let count = self.home_screen.get_transactions_limit();
        Task::future(async move {
            let result = client.get_wallet_transactions(&wallet_name, count).await;
            Message::WalletTransactions(wallet_name, result)
        })
    }

    fn get_wallet_address(&self, address_kind: AddressKind) -> Task<Message> {
        if self.wallet_name.is_none() {
            return Task::none();
        }
        let client = self.client.clone();
        let wallet_name = self.wallet_name.as_ref().unwrap().clone();
        Task::future(async move {
            let result = client.get_wallet_address(&wallet_name, address_kind).await;
            Message::WalletAddress(wallet_name, address_kind, result)
        })
    }

    fn get_space_info(&self, slabel: SLabel) -> Task<Message> {
        let client = self.client.clone();
        Task::future(async move {
            let result = client.get_space_info(&slabel).await;
            Message::SpaceInfo(slabel, result)
        })
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
        }
    }

    fn set_client_error(&mut self, err: String) -> Task<Message> {
        self.wallet_name = None;
        self.client_error = Some(err);
        let client = self.client.clone();
        Task::perform(
            async move { client.get_server_info().await },
            Message::ServerInfo,
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                let mut tasks = vec![self.get_server_info()];
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
                Task::batch(tasks)
            }
            Message::NavigateTo(route) => self.navigate_to(route),
            Message::ClientError(err) => self.set_client_error(err),
            Message::WalletLoad(result) => match result {
                Ok(wallet_name) => {
                    self.wallet_name = Some(wallet_name.clone());
                    self.wallets.insert(wallet_name);
                    self.navigate_to(Route::Home)
                }
                Err(err) => self.set_client_error(err),
            },
            Message::ServerInfo(result) => match result {
                Ok(server_info) => {
                    self.tip_height = server_info.tip.height;
                    Task::none()
                }
                Err(err) => self.set_client_error(err),
            },
            Message::WalletBalance(wallet_name, result) => {
                if let Some(wallet_state) = self.wallets.get_mut(&wallet_name) {
                    match result {
                        Ok(balance) => {
                            wallet_state.balance = balance.balance;
                        }
                        Err(err) => return self.set_client_error(err),
                    }
                }
                Task::none()
            }
            Message::WalletSpaces(wallet_name, result) => {
                if let Some(wallet_state) = self.wallets.get_mut(&wallet_name) {
                    match result {
                        Ok(spaces) => {
                            let mut collect = |spaces: Vec<FullSpaceOut>| -> Vec<SLabel> {
                                spaces
                                    .into_iter()
                                    .map(|out| {
                                        let name =
                                            out.spaceout.space.as_ref().unwrap().name.clone();
                                        self.spaces.insert(name.clone(), Some(out));
                                        name
                                    })
                                    .collect()
                            };
                            wallet_state.winning_spaces = collect(spaces.winning);
                            wallet_state.outbid_spaces = collect(spaces.outbid);
                            wallet_state.owned_spaces = collect(spaces.owned);
                        }
                        Err(err) => return self.set_client_error(err),
                    }
                }
                Task::none()
            }
            Message::WalletTransactions(wallet_name, result) => {
                if let Some(wallet_state) = self.wallets.get_mut(&wallet_name) {
                    match result {
                        Ok(transactions) => {
                            wallet_state.transactions = transactions;
                        }
                        Err(err) => return self.set_client_error(err),
                    }
                }
                Task::none()
            }
            Message::WalletAddress(wallet_name, address_kind, result) => {
                if let Some(wallet_state) = self.wallets.get_mut(&wallet_name) {
                    match result {
                        Ok(address) => {
                            let address = Some(AddressState::new(address));
                            match address_kind {
                                AddressKind::Coin => wallet_state.coin_address = address,
                                AddressKind::Space => wallet_state.space_address = address,
                            }
                        }
                        Err(err) => return self.set_client_error(err),
                    }
                }
                Task::none()
            }
            Message::SpaceInfo(slabel, result) => {
                match result {
                    Ok(out) => self.spaces.insert(slabel, out),
                    Err(err) => return self.set_client_error(err),
                }
                Task::none()
            }
            Message::HomeScreen(message) => match self.home_screen.update(message) {
                screen::home::Action::WriteClipboard(s) => clipboard::write(s),
                screen::home::Action::ShowSpace { slabel } => {
                    self.navigate_to(Route::Space(slabel))
                }
                screen::home::Action::GetTransactions => {
                    let client = self.client.clone();
                    let wallet_name = self.wallet_name.as_ref().unwrap().clone();
                    let count = self.home_screen.get_transactions_limit();
                    Task::future(async move {
                        let result = client.get_wallet_transactions(&wallet_name, count).await;
                        Message::WalletTransactions(wallet_name, result)
                    })
                }
                screen::home::Action::BumpFee { txid, fee_rate } => {
                    let client = self.client.clone();
                    let wallet_name = self.wallet_name.as_ref().unwrap().clone();
                    Task::future(async move {
                        let result = client.bump_fee(&wallet_name, txid, fee_rate).await;
                        match result {
                            Ok(()) => {
                                Message::HomeScreen(screen::home::Message::BumpFeeResult(Ok(())))
                            }
                            Err(ClientError::Call(err)) => {
                                Message::HomeScreen(screen::home::Message::BumpFeeResult(Err(err)))
                            }
                            Err(ClientError::System(err)) => Message::ClientError(err),
                        }
                    })
                }
                screen::home::Action::None => Task::none(),
            },
            Message::SendScreen(message) => match self.send_screen.update(message) {
                screen::send::Action::SendCoins {
                    recipient,
                    amount,
                    fee_rate,
                } => {
                    let client = self.client.clone();
                    let wallet_name = self.wallet_name.as_ref().unwrap().clone();
                    Task::future(async move {
                        let result = client
                            .send_coins(&wallet_name, recipient, amount, fee_rate)
                            .await;
                        match result {
                            Ok(()) => {
                                Message::SendScreen(screen::send::Message::ClientResult(Ok(())))
                            }
                            Err(ClientError::Call(err)) => {
                                Message::SendScreen(screen::send::Message::ClientResult(Err(err)))
                            }
                            Err(ClientError::System(err)) => Message::ClientError(err),
                        }
                    })
                }
                screen::send::Action::SendSpace {
                    recipient,
                    slabel,
                    fee_rate,
                } => {
                    let client = self.client.clone();
                    let wallet_name = self.wallet_name.as_ref().unwrap().clone();
                    Task::future(async move {
                        let result = client
                            .send_space(&wallet_name, recipient, slabel, fee_rate)
                            .await;
                        match result {
                            Ok(()) => {
                                Message::SendScreen(screen::send::Message::ClientResult(Ok(())))
                            }
                            Err(ClientError::Call(err)) => {
                                Message::SendScreen(screen::send::Message::ClientResult(Err(err)))
                            }
                            Err(ClientError::System(err)) => Message::ClientError(err),
                        }
                    })
                }
                screen::send::Action::ShowTransactions => self.navigate_to(Route::Home),
                screen::send::Action::None => Task::none(),
            },
            Message::ReceiveScreen(message) => match self.receive_screen.update(message) {
                screen::receive::Action::WriteClipboard(s) => clipboard::write(s),
                screen::receive::Action::None => Task::none(),
            },
            Message::SpacesScreen(message) => match self.spaces_screen.update(message) {
                screen::spaces::Action::WriteClipboard(s) => clipboard::write(s),
                screen::spaces::Action::GetSpaceInfo { slabel } => self.get_space_info(slabel),
                screen::spaces::Action::OpenSpace {
                    slabel,
                    amount,
                    fee_rate,
                } => {
                    let client = self.client.clone();
                    let wallet_name = self.wallet_name.as_ref().unwrap().clone();
                    Task::future(async move {
                        let result = client
                            .open_space(&wallet_name, slabel, amount, fee_rate)
                            .await;
                        match result {
                            Ok(()) => {
                                Message::SpacesScreen(screen::spaces::Message::ClientResult(Ok(())))
                            }
                            Err(ClientError::Call(err)) => Message::SpacesScreen(
                                screen::spaces::Message::ClientResult(Err(err)),
                            ),
                            Err(ClientError::System(err)) => Message::ClientError(err),
                        }
                    })
                }
                screen::spaces::Action::BidSpace {
                    slabel,
                    amount,
                    fee_rate,
                } => {
                    let client = self.client.clone();
                    let wallet_name = self.wallet_name.as_ref().unwrap().clone();
                    Task::future(async move {
                        let result = client
                            .bid_space(&wallet_name, slabel, amount, fee_rate)
                            .await;
                        match result {
                            Ok(()) => {
                                Message::SpacesScreen(screen::spaces::Message::ClientResult(Ok(())))
                            }
                            Err(ClientError::Call(err)) => Message::SpacesScreen(
                                screen::spaces::Message::ClientResult(Err(err)),
                            ),
                            Err(ClientError::System(err)) => Message::ClientError(err),
                        }
                    })
                }
                screen::spaces::Action::RegisterSpace { slabel, fee_rate } => {
                    let client = self.client.clone();
                    let wallet_name = self.wallet_name.as_ref().unwrap().clone();
                    Task::future(async move {
                        let result = client.register_space(&wallet_name, slabel, fee_rate).await;
                        match result {
                            Ok(()) => {
                                Message::SpacesScreen(screen::spaces::Message::ClientResult(Ok(())))
                            }
                            Err(ClientError::Call(err)) => Message::SpacesScreen(
                                screen::spaces::Message::ClientResult(Err(err)),
                            ),
                            Err(ClientError::System(err)) => Message::ClientError(err),
                        }
                    })
                }
                screen::spaces::Action::RenewSpace { slabel, fee_rate } => {
                    let client = self.client.clone();
                    let wallet_name = self.wallet_name.as_ref().unwrap().clone();
                    Task::future(async move {
                        let result = client.renew_space(&wallet_name, slabel, fee_rate).await;
                        match result {
                            Ok(()) => {
                                Message::SpacesScreen(screen::spaces::Message::ClientResult(Ok(())))
                            }
                            Err(ClientError::Call(err)) => Message::SpacesScreen(
                                screen::spaces::Message::ClientResult(Err(err)),
                            ),
                            Err(ClientError::System(err)) => Message::ClientError(err),
                        }
                    })
                }
                screen::spaces::Action::ShowTransactions => self.navigate_to(Route::Home),
                screen::spaces::Action::None => Task::none(),
            },
            Message::MarketScreen(message) => match self.market_screen.update(message) {
                screen::market::Action::Buy { listing, fee_rate } => {
                    let client = self.client.clone();
                    let wallet_name = self.wallet_name.as_ref().unwrap().clone();
                    Task::future(async move {
                        let result = client.buy_space(&wallet_name, listing, fee_rate).await;
                        match result {
                            Ok(()) => {
                                Message::MarketScreen(screen::market::Message::BuyResult(Ok(())))
                            }
                            Err(ClientError::Call(err)) => {
                                Message::MarketScreen(screen::market::Message::BuyResult(Err(err)))
                            }
                            Err(ClientError::System(err)) => Message::ClientError(err),
                        }
                    })
                }
                screen::market::Action::Sell { slabel, price } => {
                    let client = self.client.clone();
                    let wallet_name = self.wallet_name.as_ref().unwrap().clone();
                    Task::future(async move {
                        let result = client.sell_space(&wallet_name, slabel, price).await;
                        match result {
                            Ok(listing) => Message::MarketScreen(
                                screen::market::Message::SellResult(Ok(listing)),
                            ),
                            Err(ClientError::Call(err)) => {
                                Message::MarketScreen(screen::market::Message::SellResult(Err(err)))
                            }
                            Err(ClientError::System(err)) => Message::ClientError(err),
                        }
                    })
                }
                screen::market::Action::WriteClipboard(s) => clipboard::write(s),
                screen::market::Action::ShowTransactions => self.navigate_to(Route::Home),
                screen::market::Action::None => Task::none(),
            },
            Message::SignScreen(message) => match self.sign_screen.update(message) {
                screen::sign::Action::FilePick => Task::future(async move {
                    let path = rfd::AsyncFileDialog::new()
                        .add_filter("JSON event", &["json"])
                        .pick_file()
                        .await
                        .map(|file| file.path().to_path_buf());

                    use spaces_wallet::bdk_wallet::serde_json;
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
                    Message::SignScreen(screen::sign::Message::EventFileLoaded(result))
                }),
                screen::sign::Action::Sign(slabel, event) => {
                    let client = self.client.clone();
                    let wallet_name = self.wallet_name.as_ref().unwrap().clone();
                    Task::future(async move {
                        let result = client.sign_event(&wallet_name, slabel, event).await;
                        match result {
                            Ok(event) => {
                                let file_path = rfd::AsyncFileDialog::new()
                                    .add_filter("JSON event", &["json"])
                                    .add_filter("All files", &["*"])
                                    .save_file()
                                    .await
                                    .map(|file| file.path().to_path_buf());

                                let result = if let Some(file_path) = file_path {
                                    use spaces_wallet::bdk_wallet::serde_json;
                                    let contents = serde_json::to_vec(&event).unwrap();
                                    tokio::fs::write(&file_path, contents)
                                        .await
                                        .map_err(|e| e.to_string())
                                } else {
                                    Ok(())
                                };
                                Message::SignScreen(screen::sign::Message::EventFileSaved(result))
                            }
                            Err(ClientError::Call(err)) => {
                                Message::SignScreen(screen::sign::Message::EventFileSaved(Err(err)))
                            }
                            Err(ClientError::System(err)) => Message::ClientError(err),
                        }
                    })
                }
                screen::sign::Action::None => Task::none(),
            },
        }
    }

    fn view(&self) -> Element<Message> {
        let navbar_button = |label, icon: Icon, route: Route, screen: Screen| {
            let button = button(
                row![text_icon(icon).size(20), text(label).size(16)]
                    .spacing(10)
                    .align_y(Center),
            )
            .style(if self.screen == screen {
                button::primary
            } else {
                button::text
            })
            .width(Fill);
            button.on_press(Message::NavigateTo(route))
        };

        let main: Element<Message> = if let Some(wallet) = self
            .wallet_name
            .as_ref()
            .and_then(|name| self.wallets.get(name))
        {
            row![
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
                ]
                .padding(10)
                .spacing(5)
                .width(200),
                vertical_rule(3),
                container(match &self.screen {
                    Screen::Home => self
                        .home_screen
                        .view(self.tip_height, wallet.balance, &wallet.transactions)
                        .map(Message::HomeScreen),
                    Screen::Send => self
                        .send_screen
                        .view(&wallet.owned_spaces)
                        .map(Message::SendScreen),
                    Screen::Receive => self
                        .receive_screen
                        .view(wallet.coin_address.as_ref(), wallet.space_address.as_ref(),)
                        .map(Message::ReceiveScreen),
                    Screen::Spaces => self
                        .spaces_screen
                        .view(
                            self.tip_height,
                            &self.spaces,
                            &wallet.winning_spaces,
                            &wallet.outbid_spaces,
                            &wallet.owned_spaces
                        )
                        .map(Message::SpacesScreen),
                    Screen::Market => self
                        .market_screen
                        .view(&wallet.owned_spaces)
                        .map(Message::MarketScreen),
                    Screen::Sign => self
                        .sign_screen
                        .view(&wallet.owned_spaces)
                        .map(Message::SignScreen),
                })
                .padding(20)
            ]
            .into()
        } else {
            center(text("Loading").align_x(Center)).into()
        };
        column![error_block(self.client_error.as_ref()), main].into()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(if self.wallet_name.is_some() {
            time::Duration::from_secs(30)
        } else {
            time::Duration::from_secs(5)
        })
        .map(|_| Message::Tick)
    }
}
