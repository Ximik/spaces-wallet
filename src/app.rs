use std::fmt;
use std::sync::Arc;

use iced::time;
use iced::widget::{button, center, column, container, row, text, vertical_rule};
use iced::{clipboard, Center, Element, Fill, Subscription, Task};

use jsonrpsee::core::ClientError;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use spaced::rpc::{
    BidParams, OpenParams, RegisterParams, RpcClient, RpcWalletRequest, RpcWalletTxBuilder,
    SendCoinsParams, ServerInfo, TransferSpacesParams,
};
use wallet::bitcoin::transaction;

use crate::screen;
use crate::types::*;
use crate::widget::{
    icon::{text_icon, Icon},
    text::error_block,
};

#[derive(Debug, Clone)]
enum RpcError {
    Call { code: i32, message: String },
    Global { message: String },
}
impl From<ClientError> for RpcError {
    fn from(error: ClientError) -> Self {
        match error {
            ClientError::Call(e) => RpcError::Call {
                code: e.code(),
                message: e.message().to_string(),
            },
            _ => RpcError::Global {
                message: error.to_string(),
            },
        }
    }
}
impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                RpcError::Call { message, .. } => message,
                RpcError::Global { message } => message,
            }
        )
    }
}
type RpcResult<T> = Result<T, RpcError>;

#[derive(Debug, Clone)]
enum RpcRequest {
    GetServerInfo,
    GetSpaceInfo {
        slabel: SLabel,
    },
    LoadWallet {
        wallet_name: String,
    },
    GetBalance,
    GetWalletSpaces,
    GetTransactions {
        count: usize,
        skip: usize,
    },
    GetAddress {
        address_kind: AddressKind,
    },
    SendCoins {
        recipient: String,
        amount: Amount,
        fee_rate: Option<FeeRate>,
    },
    OpenSpace {
        slabel: SLabel,
        amount: Amount,
        fee_rate: Option<FeeRate>,
    },
    BidSpace {
        slabel: SLabel,
        amount: Amount,
        fee_rate: Option<FeeRate>,
    },
    RegisterSpace {
        slabel: SLabel,
        fee_rate: Option<FeeRate>,
    },
    TransferSpace {
        slabel: SLabel,
        recipient: String,
        fee_rate: Option<FeeRate>,
    },
}

#[derive(Debug, Clone)]
enum RpcResponse {
    GetServerInfo {
        result: RpcResult<ServerInfo>,
    },
    GetSpaceInfo {
        slabel: SLabel,
        result: RpcResult<Option<FullSpaceOut>>,
    },
    LoadWallet {
        wallet_name: String,
        result: RpcResult<()>,
    },
    GetBalance {
        wallet_name: String,
        result: RpcResult<Balance>,
    },
    GetTransactions {
        count: usize,
        skip: usize,
        wallet_name: String,
        result: RpcResult<Vec<TxInfo>>,
    },
    GetWalletSpaces {
        wallet_name: String,
        result: RpcResult<Vec<WalletOutput>>,
    },
    GetAddress {
        wallet_name: String,
        address_kind: AddressKind,
        result: RpcResult<String>,
    },
    SendCoins {
        result: RpcResult<()>,
    },
    OpenSpace {
        result: RpcResult<()>,
    },
    BidSpace {
        result: RpcResult<()>,
    },
    RegisterSpace {
        result: RpcResult<()>,
    },
    TransferSpace {
        result: RpcResult<()>,
    },
}

#[derive(Debug, Clone)]
enum Route {
    Home,
    Send,
    Receive,
    Spaces,
    Space(SLabel),
}

#[derive(Debug, Clone)]
enum Message {
    RpcRequest(RpcRequest),
    RpcResponse(RpcResponse),
    NavigateTo(Route),
    HomeScreen(screen::home::Message),
    SendScreen(screen::send::Message),
    ReceiveScreen(screen::receive::Message),
    SpacesScreen(screen::spaces::Message),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Home,
    Send,
    Receive,
    Spaces,
}

#[derive(Debug)]
pub struct App {
    rpc_client: Arc<HttpClient>,
    rpc_error: Option<String>,
    screen: Screen,
    tip_height: u32,
    wallet: Option<WalletState>,
    spaces: SpacesState,
    send_screen: screen::send::State,
    receive_screen: screen::receive::State,
    spaces_screen: screen::spaces::State,
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
                size: (1000.0, 500.0).into(),
                min_size: Some((1000.0, 500.0).into()),
                icon: Some(icon),
                ..Default::default()
            })
            .run_with(move || Self::new(args))
    }

    fn new(args: crate::Args) -> (Self, Task<Message>) {
        let rpc_client: Arc<HttpClient> = Arc::new(
            HttpClientBuilder::default()
                .build(args.spaced_rpc_url.unwrap())
                .unwrap(),
        );
        (
            Self {
                rpc_client,
                rpc_error: None,
                screen: Screen::Home,
                tip_height: 0,
                wallet: None,
                spaces: Default::default(),
                send_screen: Default::default(),
                receive_screen: Default::default(),
                spaces_screen: Default::default(),
            },
            Task::batch([
                Task::done(Message::RpcRequest(RpcRequest::LoadWallet {
                    wallet_name: args.wallet.into(),
                })),
                Task::done(Message::RpcRequest(RpcRequest::GetServerInfo)),
            ]),
        )
    }

    fn title(&self) -> String {
        "Spaces Wallet".into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::RpcRequest(request) => {
                let client = self.rpc_client.clone();
                match request {
                    RpcRequest::GetServerInfo => Task::perform(
                        async move {
                            let result = client.get_server_info().await.map_err(RpcError::from);
                            RpcResponse::GetServerInfo { result }
                        },
                        Message::RpcResponse,
                    ),
                    RpcRequest::GetSpaceInfo { slabel } => Task::perform(
                        async move {
                            use protocol::hasher::KeyHasher;
                            use spaced::store::Sha256;

                            let hash = hex::encode(Sha256::hash(slabel.as_ref()));
                            let result = client.get_space(&hash).await.map_err(RpcError::from);
                            RpcResponse::GetSpaceInfo { slabel, result }
                        },
                        Message::RpcResponse,
                    ),
                    RpcRequest::LoadWallet { wallet_name } => Task::perform(
                        async move {
                            let result = client
                                .wallet_load(&wallet_name)
                                .await
                                .map_err(RpcError::from);
                            RpcResponse::LoadWallet {
                                wallet_name,
                                result,
                            }
                        },
                        Message::RpcResponse,
                    ),
                    RpcRequest::GetBalance => {
                        if let Some(wallet) = self.wallet.as_ref() {
                            let wallet_name = wallet.name.clone();
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_get_balance(&wallet_name)
                                        .await
                                        .map_err(RpcError::from);
                                    RpcResponse::GetBalance {
                                        wallet_name,
                                        result,
                                    }
                                },
                                Message::RpcResponse,
                            )
                        } else {
                            Task::none()
                        }
                    }
                    RpcRequest::GetWalletSpaces => {
                        if let Some(wallet) = self.wallet.as_ref() {
                            let wallet_name = wallet.name.clone();
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_list_spaces(&wallet_name)
                                        .await
                                        .map_err(RpcError::from);
                                    RpcResponse::GetWalletSpaces {
                                        wallet_name,
                                        result,
                                    }
                                },
                                Message::RpcResponse,
                            )
                        } else {
                            Task::none()
                        }
                    }
                    RpcRequest::GetTransactions { count, skip } => {
                        if let Some(wallet) = self.wallet.as_ref() {
                            let wallet_name = wallet.name.clone();
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_list_transactions(&wallet_name, count, skip)
                                        .await
                                        .map_err(RpcError::from);
                                    RpcResponse::GetTransactions {
                                        wallet_name,
                                        count,
                                        skip,
                                        result,
                                    }
                                },
                                Message::RpcResponse,
                            )
                        } else {
                            Task::none()
                        }
                    }
                    RpcRequest::GetAddress { address_kind } => {
                        if let Some(wallet) = self.wallet.as_ref() {
                            let wallet_name = wallet.name.clone();
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_get_new_address(&wallet_name, address_kind)
                                        .await
                                        .map_err(RpcError::from);
                                    RpcResponse::GetAddress {
                                        wallet_name,
                                        address_kind,
                                        result,
                                    }
                                },
                                Message::RpcResponse,
                            )
                        } else {
                            Task::none()
                        }
                    }
                    RpcRequest::SendCoins {
                        recipient,
                        amount,
                        fee_rate,
                    } => {
                        if let Some(wallet) = self.wallet.as_ref() {
                            let wallet_name = wallet.name.clone();
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_send_request(
                                            &wallet_name,
                                            RpcWalletTxBuilder {
                                                bidouts: None,
                                                requests: vec![RpcWalletRequest::SendCoins(
                                                    SendCoinsParams {
                                                        amount,
                                                        to: recipient,
                                                    },
                                                )],
                                                fee_rate,
                                                dust: None,
                                                force: false,
                                                confirmed_only: false,
                                                skip_tx_check: false,
                                            },
                                        )
                                        .await
                                        .map(|_| ())
                                        .map_err(RpcError::from);
                                    RpcResponse::SendCoins { result }
                                },
                                Message::RpcResponse,
                            )
                        } else {
                            Task::none()
                        }
                    }
                    RpcRequest::OpenSpace {
                        slabel,
                        amount,
                        fee_rate,
                    } => {
                        if let Some(wallet) = self.wallet.as_ref() {
                            let wallet_name = wallet.name.clone();
                            Task::perform(
                                async move {
                                    let name = slabel.to_string();
                                    let amount = amount.to_sat();
                                    let result = client
                                        .wallet_send_request(
                                            &wallet_name,
                                            RpcWalletTxBuilder {
                                                bidouts: None,
                                                requests: vec![RpcWalletRequest::Open(
                                                    OpenParams { name, amount },
                                                )],
                                                fee_rate,
                                                dust: None,
                                                force: false,
                                                confirmed_only: false,
                                                skip_tx_check: false,
                                            },
                                        )
                                        .await
                                        .map(|_| ())
                                        .map_err(RpcError::from);
                                    RpcResponse::OpenSpace { result }
                                },
                                Message::RpcResponse,
                            )
                        } else {
                            Task::none()
                        }
                    }
                    RpcRequest::BidSpace {
                        slabel,
                        amount,
                        fee_rate,
                    } => {
                        if let Some(wallet) = self.wallet.as_ref() {
                            let wallet_name = wallet.name.clone();
                            Task::perform(
                                async move {
                                    let name = slabel.to_string();
                                    let amount = amount.to_sat();
                                    let result = client
                                        .wallet_send_request(
                                            &wallet_name,
                                            RpcWalletTxBuilder {
                                                bidouts: None,
                                                requests: vec![RpcWalletRequest::Bid(BidParams {
                                                    name,
                                                    amount,
                                                })],
                                                fee_rate,
                                                dust: None,
                                                force: false,
                                                confirmed_only: false,
                                                skip_tx_check: false,
                                            },
                                        )
                                        .await
                                        .map(|_| ())
                                        .map_err(RpcError::from);
                                    RpcResponse::BidSpace { result }
                                },
                                Message::RpcResponse,
                            )
                        } else {
                            Task::none()
                        }
                    }
                    RpcRequest::RegisterSpace { slabel, fee_rate } => {
                        if let Some(wallet) = self.wallet.as_ref() {
                            let wallet_name = wallet.name.clone();
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_send_request(
                                            &wallet_name,
                                            RpcWalletTxBuilder {
                                                bidouts: None,
                                                requests: vec![RpcWalletRequest::Register(
                                                    RegisterParams {
                                                        name: slabel.to_string(),
                                                        to: None,
                                                    },
                                                )],
                                                fee_rate,
                                                dust: None,
                                                force: false,
                                                confirmed_only: false,
                                                skip_tx_check: false,
                                            },
                                        )
                                        .await
                                        .map(|_| ())
                                        .map_err(RpcError::from);
                                    RpcResponse::RegisterSpace { result }
                                },
                                Message::RpcResponse,
                            )
                        } else {
                            Task::none()
                        }
                    }
                    RpcRequest::TransferSpace {
                        slabel,
                        recipient,
                        fee_rate,
                    } => {
                        if let Some(wallet) = self.wallet.as_ref() {
                            let wallet_name = wallet.name.clone();
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_send_request(
                                            &wallet_name,
                                            RpcWalletTxBuilder {
                                                bidouts: None,
                                                requests: vec![RpcWalletRequest::Transfer(
                                                    TransferSpacesParams {
                                                        spaces: vec![slabel.to_string()],
                                                        to: recipient,
                                                    },
                                                )],
                                                fee_rate,
                                                dust: None,
                                                force: false,
                                                confirmed_only: false,
                                                skip_tx_check: false,
                                            },
                                        )
                                        .await
                                        .map(|_| ())
                                        .map_err(RpcError::from);
                                    RpcResponse::TransferSpace { result }
                                },
                                Message::RpcResponse,
                            )
                        } else {
                            Task::none()
                        }
                    }
                }
            }
            Message::RpcResponse(response) => {
                self.rpc_error = None;
                match response {
                    RpcResponse::GetServerInfo { result } => {
                        match result {
                            Ok(server_info) => {
                                self.tip_height = server_info.tip.height;
                            }
                            Err(e) => {
                                self.rpc_error = Some(e.to_string());
                            }
                        }
                        Task::none()
                    }
                    RpcResponse::GetSpaceInfo { slabel, result } => {
                        match result {
                            Ok(out) => {
                                self.spaces.insert(
                                    slabel,
                                    out.map(|out| out.spaceout.space.unwrap().covenant),
                                );
                            }
                            Err(e) => {
                                self.rpc_error = Some(e.to_string());
                            }
                        }
                        Task::none()
                    }
                    RpcResponse::LoadWallet {
                        wallet_name,
                        result,
                    } => match result {
                        Ok(_) => {
                            self.wallet = Some(WalletState::new(wallet_name));
                            Task::done(Message::NavigateTo(Route::Home))
                        }
                        Err(e) => {
                            self.rpc_error = Some(e.to_string());
                            Task::none()
                        }
                    },
                    RpcResponse::GetBalance {
                        wallet_name,
                        result,
                    } => {
                        match result {
                            Ok(balance) => {
                                if let Some(wallet) = self.wallet.as_mut() {
                                    if wallet.name == wallet_name {
                                        wallet.balance = balance.balance;
                                    }
                                }
                            }
                            Err(e) => {
                                self.rpc_error = Some(e.to_string());
                            }
                        }
                        Task::none()
                    }
                    RpcResponse::GetWalletSpaces {
                        wallet_name,
                        result,
                    } => {
                        match result {
                            Ok(spaces) => {
                                if let Some(wallet) = self.wallet.as_mut() {
                                    if wallet.name == wallet_name {
                                        wallet.spaces = spaces
                                            .into_iter()
                                            .map(|out| {
                                                let space = out.space.unwrap();
                                                self.spaces.insert(
                                                    space.name.clone(),
                                                    Some(space.covenant),
                                                );
                                                space.name
                                            })
                                            .collect();
                                    }
                                }
                            }
                            Err(e) => {
                                self.rpc_error = Some(e.to_string());
                            }
                        }
                        Task::none()
                    }
                    RpcResponse::GetTransactions {
                        wallet_name,
                        count,
                        skip,
                        result,
                    } => {
                        match result {
                            Ok(transactions) => {
                                if let Some(wallet) = self.wallet.as_mut() {
                                    if wallet.name == wallet_name {
                                        if transactions.len() == 0 {
                                            return Task::none();
                                        }
                                        if wallet.transactions.is_empty() {
                                            wallet.transactions = transactions;
                                            return Task::none();
                                        }
                                        if skip == 0 {
                                            if transactions.len() < count {
                                                wallet.transactions = transactions;
                                                return Task::none();
                                            }
                                            let first_confirmed_txid = wallet
                                                .transactions
                                                .iter()
                                                .find(|tx| tx.confirmed)
                                                .map(|tx| tx.txid);
                                            if let Some(idx) = transactions.iter().position(|tx| {
                                                Some(tx.txid) == first_confirmed_txid
                                            }) {
                                                wallet.transactions.splice(
                                                    0..0,
                                                    transactions[0..idx].iter().cloned(),
                                                );
                                                return Task::none();
                                            }
                                            return Task::done(Message::RpcRequest(
                                                RpcRequest::GetTransactions {
                                                    count: count * 2,
                                                    skip: 0,
                                                },
                                            ));
                                        }
                                        // TODO
                                    }
                                }
                                Task::none()
                            }
                            Err(e) => {
                                self.rpc_error = Some(e.to_string());
                                Task::none()
                            }
                        }
                    }
                    RpcResponse::GetAddress {
                        wallet_name,
                        address_kind,
                        result,
                    } => {
                        match result {
                            Ok(address) => {
                                if let Some(wallet) = self.wallet.as_mut() {
                                    if wallet.name == wallet_name {
                                        let address = Some(AddressState::new(address));
                                        match address_kind {
                                            AddressKind::Coin => wallet.coin_address = address,
                                            AddressKind::Space => wallet.space_address = address,
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                self.rpc_error = Some(e.to_string());
                            }
                        }
                        Task::none()
                    }
                    RpcResponse::SendCoins { result } => match result {
                        Ok(_) => Task::done(Message::NavigateTo(Route::Home)),
                        Err(RpcError::Call { code, message }) => {
                            if code == -1 {
                                self.send_screen.set_error(message);
                            } else {
                                self.rpc_error = Some(message);
                            }
                            Task::none()
                        }
                        Err(e) => {
                            self.rpc_error = Some(e.to_string());
                            Task::none()
                        }
                    },
                    RpcResponse::OpenSpace { result }
                    | RpcResponse::BidSpace { result }
                    | RpcResponse::RegisterSpace { result }
                    | RpcResponse::TransferSpace { result } => match result {
                        Ok(_) => {
                            self.spaces_screen.reset_inputs();
                            Task::done(Message::NavigateTo(Route::Home))
                        }
                        Err(RpcError::Call { code, message }) => {
                            if code == -1 {
                                self.spaces_screen.set_error(message);
                            } else {
                                self.rpc_error = Some(message);
                            }
                            Task::none()
                        }
                        Err(e) => {
                            self.rpc_error = Some(e.to_string());
                            Task::none()
                        }
                    },
                }
            }
            Message::NavigateTo(route) => match route {
                Route::Home => {
                    self.screen = Screen::Home;
                    Task::batch([
                        Task::done(Message::RpcRequest(RpcRequest::GetBalance)),
                        Task::done(Message::RpcRequest(RpcRequest::GetWalletSpaces)),
                        Task::done(Message::RpcRequest(RpcRequest::GetTransactions {
                            count: 10,
                            skip: 0,
                        })),
                    ])
                }
                Route::Send => {
                    self.screen = Screen::Send;
                    Task::none()
                }
                Route::Receive => {
                    if self.screen == Screen::Receive {
                        self.receive_screen.reset();
                    } else {
                        self.screen = Screen::Receive;
                    }
                    Task::batch([
                        Task::done(Message::RpcRequest(RpcRequest::GetAddress {
                            address_kind: AddressKind::Coin,
                        })),
                        Task::done(Message::RpcRequest(RpcRequest::GetAddress {
                            address_kind: AddressKind::Space,
                        })),
                    ])
                }
                Route::Spaces => {
                    if self.screen == Screen::Spaces {
                        self.spaces_screen.reset_space();
                    } else {
                        self.screen = Screen::Spaces;
                    }
                    if let Some(slabel) = self.spaces_screen.get_slabel() {
                        Task::done(Message::RpcRequest(RpcRequest::GetSpaceInfo { slabel }))
                    } else {
                        Task::done(Message::RpcRequest(RpcRequest::GetWalletSpaces))
                    }
                }
                Route::Space(slabel) => {
                    self.screen = Screen::Spaces;
                    self.spaces_screen.set_slabel(&slabel);
                    Task::done(Message::RpcRequest(RpcRequest::GetSpaceInfo { slabel }))
                }
            },
            Message::HomeScreen(message) => match message {
                screen::home::Message::TxidCopyPress { txid } => clipboard::write(txid),
                screen::home::Message::SpaceClicked { slabel } => {
                    Task::done(Message::NavigateTo(Route::Space(slabel)))
                }
                screen::home::Message::TransactionsScrolled { percentage } => {
                    println!("{}", percentage);
                    if let Some(wallet) = self.wallet.as_ref() {
                        if percentage > 0.8 {
                            let wallet_name = wallet.name.clone();
                        }
                    }
                    println!("{}", percentage);
                    Task::none()
                }
            },
            Message::SendScreen(message) => match self.send_screen.update(message) {
                screen::send::Task::SendCoins {
                    recipient,
                    amount,
                    fee_rate,
                } => Task::done(Message::RpcRequest(RpcRequest::SendCoins {
                    recipient,
                    amount,
                    fee_rate,
                })),
                screen::send::Task::None => Task::none(),
            },
            Message::ReceiveScreen(message) => match self.receive_screen.update(message) {
                screen::receive::Task::WriteClipboard(s) => clipboard::write(s),
                screen::receive::Task::None => Task::none(),
            },
            Message::SpacesScreen(message) => match self.spaces_screen.update(message) {
                screen::spaces::Task::GetSpaceInfo { slabel } => {
                    Task::done(Message::RpcRequest(RpcRequest::GetSpaceInfo { slabel }))
                }
                screen::spaces::Task::OpenSpace {
                    slabel,
                    amount,
                    fee_rate,
                } => Task::done(Message::RpcRequest(RpcRequest::OpenSpace {
                    slabel,
                    amount,
                    fee_rate,
                })),
                screen::spaces::Task::BidSpace {
                    slabel,
                    amount,
                    fee_rate,
                } => Task::done(Message::RpcRequest(RpcRequest::BidSpace {
                    slabel,
                    amount,
                    fee_rate,
                })),
                screen::spaces::Task::ClaimSpace { slabel, fee_rate } => {
                    Task::done(Message::RpcRequest(RpcRequest::RegisterSpace {
                        slabel,
                        fee_rate,
                    }))
                }
                screen::spaces::Task::TransferSpace {
                    slabel,
                    recipient,
                    fee_rate,
                } => Task::done(Message::RpcRequest(RpcRequest::TransferSpace {
                    slabel,
                    recipient,
                    fee_rate,
                })),
                screen::spaces::Task::None => Task::none(),
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

        let main: Element<Message> = if let Some(wallet) = self.wallet.as_ref() {
            row![
                column![
                    navbar_button("Home", Icon::Artboard, Route::Home, Screen::Home,),
                    navbar_button("Send", Icon::ArrowDownFromArc, Route::Send, Screen::Send,),
                    navbar_button(
                        "Receive",
                        Icon::ArrowDownToArc,
                        Route::Receive,
                        Screen::Receive,
                    ),
                    navbar_button("Spaces", Icon::At, Route::Spaces, Screen::Spaces,),
                ]
                .padding(10)
                .spacing(5)
                .width(200),
                vertical_rule(3),
                container(match &self.screen {
                    Screen::Home => screen::home::view(wallet.balance, &wallet.transactions)
                        .map(Message::HomeScreen),
                    Screen::Send => self.send_screen.view().map(Message::SendScreen),
                    Screen::Receive => self
                        .receive_screen
                        .view(wallet.coin_address.as_ref(), wallet.space_address.as_ref(),)
                        .map(Message::ReceiveScreen),
                    Screen::Spaces => self
                        .spaces_screen
                        .view(self.tip_height, &self.spaces, &wallet.spaces)
                        .map(Message::SpacesScreen),
                })
                .padding(20)
            ]
            .into()
        } else {
            center(text("Loading").align_x(Center)).into()
        };
        column![error_block(self.rpc_error.as_ref()), main].into()
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.wallet.is_some() && self.rpc_error.is_none() {
            let mut subscriptions = vec![time::every(time::Duration::from_secs(30))
                .map(|_| Message::RpcRequest(RpcRequest::GetServerInfo))];
            match self.screen {
                Screen::Home => {
                    subscriptions.push(
                        time::every(time::Duration::from_secs(30))
                            .map(|_| Message::RpcRequest(RpcRequest::GetBalance)),
                    );
                    subscriptions.push(time::every(time::Duration::from_secs(30)).map(|_| {
                        Message::RpcRequest(RpcRequest::GetTransactions { count: 10, skip: 0 })
                    }));
                }
                Screen::Spaces => {
                    subscriptions.push(
                        time::every(time::Duration::from_secs(30))
                            .map(|_| Message::RpcRequest(RpcRequest::GetWalletSpaces)),
                    );
                    if let Some(slabel) = self.spaces_screen.get_slabel() {
                        subscriptions.push(
                            time::every(time::Duration::from_secs(30)).with(slabel).map(
                                |(slabel, _)| {
                                    Message::RpcRequest(RpcRequest::GetSpaceInfo { slabel })
                                },
                            ),
                        );
                    }
                }
                _ => {}
            }
            Subscription::batch(subscriptions)
        } else {
            Subscription::none()
        }
    }
}
