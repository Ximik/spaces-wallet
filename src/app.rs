use iced::{
    Center, Element, Fill, Subscription, Task, Theme, clipboard, exit, time,
    widget::{Column, button, center, column, container, row, text, vertical_rule, vertical_space},
};
use spaces_client::{rpc::ServerInfo, wallets::WalletInfoWithProgress};

use crate::{
    branding::*,
    client::Client,
    config::Config,
    screen,
    types::*,
    widget::icon::{Icon, text_icon},
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
    Settings,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    NavigateTo(Route),
    ServerInfo(Result<ServerInfo, String>),
    ListWallets(Result<Vec<String>, String>),
    WalletLoad(Result<String, String>),
    WalletInfo(Result<WalletInfoWithProgress, String>),
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
    SettingsScreen(screen::settings::Message),
}

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
pub struct App {
    config: Config,
    client: Client,
    screen: Screen,
    tip_height: u32,
    blocks_height: u32,
    headers_height: u32,
    wallets: WalletsState,
    spaces: SpacesState,
    home_screen: screen::home::State,
    send_screen: screen::send::State,
    receive_screen: screen::receive::State,
    spaces_screen: screen::spaces::State,
    market_screen: screen::market::State,
    sign_screen: screen::sign::State,
    settings_screen: screen::settings::State,
}

impl App {
    pub fn new(config: Config, client: Client) -> Self {
        Self {
            config,
            client,
            screen: Screen::Home,
            tip_height: 0,
            blocks_height: 0,
            headers_height: 0,
            wallets: Default::default(),
            spaces: Default::default(),
            home_screen: Default::default(),
            send_screen: Default::default(),
            receive_screen: Default::default(),
            spaces_screen: Default::default(),
            market_screen: Default::default(),
            sign_screen: Default::default(),
            settings_screen: Default::default(),
        }
    }

    pub fn run(self) -> iced::Result {
        iced::application(WINDOW_TITLE, Self::update, Self::view)
            .font(ICONS_FONT.clone())
            .subscription(Self::subscription)
            .window(iced::window::Settings {
                min_size: Some((1300.0, 500.0).into()),
                icon: Some(WINDOW_ICON.clone()),
                ..Default::default()
            })
            .theme(|_| BITCOIN_THEME.clone())
            .run_with(move || {
                let task = Task::batch([self.get_server_info(), self.list_wallets()]);
                (self, task)
            })
    }

    fn get_server_info(&self) -> Task<Message> {
        let client = self.client.clone();
        Task::future(async move {
            let result = client.get_server_info().await;
            Message::ServerInfo(result)
        })
    }

    fn list_wallets(&self) -> Task<Message> {
        let client = self.client.clone();
        Task::future(async move {
            let result = client.list_wallets().await;
            Message::ListWallets(result)
        })
    }

    fn get_wallet_info(&self) -> Task<Message> {
        if let Some(wallet) = self.wallets.get_current() {
            let client = self.client.clone();
            let wallet_name = wallet.name.clone();
            Task::future(async move {
                let result = client.get_wallet_info(&wallet_name).await;
                Message::WalletInfo(result)
            })
        } else {
            Task::none()
        }
    }

    fn get_wallet_balance(&self) -> Task<Message> {
        if let Some(wallet) = self.wallets.get_current() {
            let client = self.client.clone();
            let wallet_name = wallet.name.clone();
            Task::future(async move {
                let result = client.get_wallet_balance(&wallet_name).await;
                Message::WalletBalance(wallet_name, result)
            })
        } else {
            Task::none()
        }
    }

    fn get_wallet_spaces(&self) -> Task<Message> {
        if let Some(wallet) = self.wallets.get_current() {
            let client = self.client.clone();
            let wallet_name = wallet.name.clone();
            Task::future(async move {
                let result = client.get_wallet_spaces(&wallet_name).await;
                Message::WalletSpaces(wallet_name, result)
            })
        } else {
            Task::none()
        }
    }

    fn get_wallet_transactions(&self) -> Task<Message> {
        if let Some(wallet) = self.wallets.get_current() {
            let client = self.client.clone();
            let wallet_name = wallet.name.clone();
            let count = self.home_screen.get_transactions_limit();
            Task::future(async move {
                let result = client.get_wallet_transactions(&wallet_name, count).await;
                Message::WalletTransactions(wallet_name, result)
            })
        } else {
            Task::none()
        }
    }

    fn get_wallet_address(&self, address_kind: AddressKind) -> Task<Message> {
        if let Some(wallet) = self.wallets.get_current() {
            let client = self.client.clone();
            let wallet_name = wallet.name.clone();
            Task::future(async move {
                let result = client.get_wallet_address(&wallet_name, address_kind).await;
                Message::WalletAddress(wallet_name, address_kind, result)
            })
        } else {
            Task::none()
        }
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
            Route::Settings => {
                self.screen = Screen::Settings;
                Task::none()
            }
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
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
                Task::batch(tasks)
            }
            Message::NavigateTo(route) => self.navigate_to(route),
            Message::ServerInfo(result) => {
                match result {
                    Ok(server_info) => {
                        self.tip_height = server_info.tip.height;
                        self.blocks_height = server_info.chain.blocks;
                        self.headers_height = server_info.chain.headers;
                    }
                    Err(_) => {
                        self.tip_height = 0;
                        self.blocks_height = 0;
                        self.headers_height = 0;
                    }
                }
                Task::none()
            }
            Message::ListWallets(result) => match result {
                Ok(wallets_names) => {
                    self.wallets.set_wallets(&wallets_names);
                    if self.wallets.get_current().is_none() {
                        if let Some(name) = self.config.wallet.as_ref() {
                            self.wallets.set_current(name);
                        }
                    }
                    if let Some(wallet) = self.wallets.get_current() {
                        let client = self.client.clone();
                        let wallet_name = wallet.name.clone();
                        Task::future(async move {
                            let result = client.load_wallet(&wallet_name).await;
                            Message::WalletLoad(result.map(|_| wallet_name))
                        })
                    } else {
                        self.navigate_to(Route::Settings)
                    }
                }
                Err(_) => self.list_wallets(),
            },
            Message::WalletLoad(result) => {
                if result.is_ok() {
                    Task::batch([self.get_wallet_info(), self.navigate_to(Route::Home)])
                } else {
                    Task::none()
                }
            }
            Message::WalletInfo(result) => {
                if let Ok(wallet_info) = result {
                    if let Some(wallet_state) = self.wallets.get_mut(&wallet_info.info.label) {
                        wallet_state.tip = wallet_info.info.tip;
                    }
                }
                Task::none()
            }
            Message::WalletBalance(wallet_name, result) => {
                if let Some(wallet_state) = self.wallets.get_mut(&wallet_name) {
                    if let Ok(balance) = result {
                        wallet_state.balance = balance.balance;
                    }
                }
                Task::none()
            }
            Message::WalletSpaces(wallet_name, result) => {
                if let Some(wallet_state) = self.wallets.get_mut(&wallet_name) {
                    if let Ok(spaces) = result {
                        let mut collect = |spaces: Vec<FullSpaceOut>| -> Vec<SLabel> {
                            spaces
                                .into_iter()
                                .map(|out| {
                                    let name = out.spaceout.space.as_ref().unwrap().name.clone();
                                    self.spaces.insert(name.clone(), Some(out));
                                    name
                                })
                                .collect()
                        };
                        wallet_state.winning_spaces = collect(spaces.winning);
                        wallet_state.outbid_spaces = collect(spaces.outbid);
                        wallet_state.owned_spaces = collect(spaces.owned);
                    }
                }
                Task::none()
            }
            Message::WalletTransactions(wallet_name, result) => {
                if let Some(wallet_state) = self.wallets.get_mut(&wallet_name) {
                    if let Ok(transactions) = result {
                        wallet_state.transactions = transactions;
                    }
                }
                Task::none()
            }
            Message::WalletAddress(wallet_name, address_kind, result) => {
                if let Some(wallet_state) = self.wallets.get_mut(&wallet_name) {
                    if let Ok(address) = result {
                        let address = Some(AddressState::new(address));
                        match address_kind {
                            AddressKind::Coin => wallet_state.coin_address = address,
                            AddressKind::Space => wallet_state.space_address = address,
                        }
                    }
                }
                Task::none()
            }
            Message::SpaceInfo(slabel, result) => {
                if let Ok(out) = result {
                    self.spaces.insert(slabel, out)
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
                    let wallet_name = self.wallets.get_current().unwrap().name.clone();
                    let count = self.home_screen.get_transactions_limit();
                    Task::future(async move {
                        let result = client.get_wallet_transactions(&wallet_name, count).await;
                        Message::WalletTransactions(wallet_name, result)
                    })
                }
                screen::home::Action::BumpFee { txid, fee_rate } => {
                    let client = self.client.clone();
                    let wallet_name = self.wallets.get_current().unwrap().name.clone();
                    Task::future(async move {
                        let result = client.bump_fee(&wallet_name, txid, fee_rate).await;
                        Message::HomeScreen(screen::home::Message::BumpFeeResult(result))
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
                    let wallet_name = self.wallets.get_current().unwrap().name.clone();
                    Task::future(async move {
                        let result = client
                            .send_coins(&wallet_name, recipient, amount, fee_rate)
                            .await;
                        Message::SendScreen(screen::send::Message::ClientResult(result))
                    })
                }
                screen::send::Action::SendSpace {
                    recipient,
                    slabel,
                    fee_rate,
                } => {
                    let client = self.client.clone();
                    let wallet_name = self.wallets.get_current().unwrap().name.clone();
                    Task::future(async move {
                        let result = client
                            .send_space(&wallet_name, recipient, slabel, fee_rate)
                            .await;
                        Message::SendScreen(screen::send::Message::ClientResult(result))
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
                    let wallet_name = self.wallets.get_current().unwrap().name.clone();
                    Task::future(async move {
                        let result = client
                            .open_space(&wallet_name, slabel, amount, fee_rate)
                            .await;
                        Message::SpacesScreen(screen::spaces::Message::ClientResult(result))
                    })
                }
                screen::spaces::Action::BidSpace {
                    slabel,
                    amount,
                    fee_rate,
                } => {
                    let client = self.client.clone();
                    let wallet_name = self.wallets.get_current().unwrap().name.clone();
                    Task::future(async move {
                        let result = client
                            .bid_space(&wallet_name, slabel, amount, fee_rate)
                            .await;
                        Message::SpacesScreen(screen::spaces::Message::ClientResult(result))
                    })
                }
                screen::spaces::Action::RegisterSpace { slabel, fee_rate } => {
                    let client = self.client.clone();
                    let wallet_name = self.wallets.get_current().unwrap().name.clone();
                    Task::future(async move {
                        let result = client.register_space(&wallet_name, slabel, fee_rate).await;
                        Message::SpacesScreen(screen::spaces::Message::ClientResult(result))
                    })
                }
                screen::spaces::Action::RenewSpace { slabel, fee_rate } => {
                    let client = self.client.clone();
                    let wallet_name = self.wallets.get_current().unwrap().name.clone();
                    Task::future(async move {
                        let result = client.renew_space(&wallet_name, slabel, fee_rate).await;
                        Message::SpacesScreen(screen::spaces::Message::ClientResult(result))
                    })
                }
                screen::spaces::Action::ShowTransactions => self.navigate_to(Route::Home),
                screen::spaces::Action::None => Task::none(),
            },
            Message::MarketScreen(message) => match self.market_screen.update(message) {
                screen::market::Action::Buy { listing, fee_rate } => {
                    let client = self.client.clone();
                    let wallet_name = self.wallets.get_current().unwrap().name.clone();
                    Task::future(async move {
                        let result = client.buy_space(&wallet_name, listing, fee_rate).await;
                        Message::MarketScreen(screen::market::Message::BuyResult(result))
                    })
                }
                screen::market::Action::Sell { slabel, price } => {
                    let client = self.client.clone();
                    let wallet_name = self.wallets.get_current().unwrap().name.clone();
                    Task::future(async move {
                        let result = client.sell_space(&wallet_name, slabel, price).await;
                        Message::MarketScreen(screen::market::Message::SellResult(result))
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
                    let wallet_name = self.wallets.get_current().unwrap().name.clone();
                    Task::future(async move {
                        let result = client.sign_event(&wallet_name, slabel, event).await;
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
                        Message::SignScreen(screen::sign::Message::EventFileSaved(result))
                    })
                }
                screen::sign::Action::None => Task::none(),
            },
            Message::SettingsScreen(message) => match self.settings_screen.update(message) {
                screen::settings::Action::SetCurrentWallet(name) => {
                    self.wallets.set_current(&name);
                    self.config.wallet = Some(name);
                    self.config.save();
                    self.list_wallets()
                }
                screen::settings::Action::ExportWallet(wallet_name) => {
                    let client = self.client.clone();
                    Task::future(async move {
                        let result = client.export_wallet(&wallet_name).await;
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
                        Message::SettingsScreen(screen::settings::Message::WalletFileSaved(result))
                    })
                }
                screen::settings::Action::CreateWallet(wallet_name) => {
                    self.config.wallet = None;
                    self.wallets.unset_current();
                    let client = self.client.clone();
                    Task::future(async move {
                        let result = client.create_wallet(&wallet_name).await;
                        Message::SettingsScreen(screen::settings::Message::WalletCreated(result))
                    })
                    .chain(self.list_wallets())
                }
                screen::settings::Action::ImportWallet => {
                    self.config.wallet = None;
                    self.wallets.unset_current();
                    let client = self.client.clone();
                    Task::future(async move {
                        let path = rfd::AsyncFileDialog::new()
                            .add_filter("wallet file", &["json"])
                            .pick_file()
                            .await
                            .map(|file| file.path().to_path_buf());

                        let result = if let Some(path) = path {
                            match tokio::fs::read_to_string(&path).await {
                                Ok(content) => client.import_wallet(&content).await,
                                Err(err) => Err(format!("Failed to read file: {}", err)),
                            }
                        } else {
                            Ok(())
                        };
                        Message::SettingsScreen(screen::settings::Message::WalletFileImported(
                            result,
                        ))
                    })
                    .chain(self.list_wallets())
                }
                screen::settings::Action::ResetBackend => {
                    self.config.remove();
                    exit()
                }
                screen::settings::Action::None => Task::none(),
            },
        }
    }

    fn view(&self) -> Element<Message> {
        let loading_text = || -> Option<String> {
            if self.headers_height == 0 {
                return Some("Loading bitcoin data".to_string());
            }
            if self.blocks_height + 1 < self.headers_height {
                return Some(format!(
                    "Syncing bitcoin data {} / {}",
                    self.blocks_height, self.headers_height,
                ));
            }
            if self.tip_height + 1 < self.blocks_height {
                return Some(format!(
                    "Syncing spaces data {} / {}",
                    self.tip_height, self.blocks_height,
                ));
            }
            if let Some(wallet) = self.wallets.get_current() {
                if wallet.state.tip + 1 < self.tip_height {
                    return Some(format!(
                        "Syncing wallet data {} / {}",
                        wallet.state.tip, self.tip_height,
                    ));
                }
            }
            None
        };

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
            .push_maybe(loading_text().map(|t| {
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
                                    self.blocks_height,
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
                                    self.blocks_height,
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
                            self.wallets.get_current().map(|w| w.name),
                        )
                        .map(Message::SettingsScreen),
                })
                .padding(20)
            ])
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(
            if self.tip_height != 0
                && self
                    .wallets
                    .get_current()
                    .is_some_and(|wallet| wallet.state.tip >= self.headers_height)
            {
                time::Duration::from_secs(30)
            } else {
                time::Duration::from_secs(5)
            },
        )
        .map(|_| Message::Tick)
    }
}
