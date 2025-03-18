use std::fmt;
use std::sync::Arc;

use iced::time;
use iced::widget::{button, center, column, container, row, text, vertical_rule};
use iced::{Center, Element, Fill, Subscription, Task, clipboard};

use jsonrpsee::core::ClientError;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use spaces_client::rpc::{
    BidParams, OpenParams, RegisterParams, RpcClient, RpcWalletRequest, RpcWalletTxBuilder,
    SendCoinsParams, ServerInfo, TransferSpacesParams,
};
use spaces_wallet::bdk_wallet::serde_json;

use crate::screen;
use crate::types::*;
use crate::widget::{
    icon::{Icon, text_icon},
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
    CreateWallet {
        wallet_name: String,
    },
    GetBalance,
    GetWalletSpaces,
    GetTransactions,
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
    RenewSpace {
        slabel: SLabel,
        fee_rate: Option<FeeRate>,
    },
    TransferSpace {
        slabel: SLabel,
        recipient: String,
        fee_rate: Option<FeeRate>,
    },
    BumpFee {
        txid: Txid,
        fee_rate: FeeRate,
    },
    BuySpace {
        listing: Listing,
        fee_rate: Option<FeeRate>,
    },
    SellSpace {
        slabel: SLabel,
        price: Amount,
    },
    SignEvent {
        slabel: SLabel,
        event: NostrEvent,
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
    CreateWallet {
        wallet_name: String,
        result: RpcResult<()>,
    },
    GetBalance {
        wallet_name: String,
        result: RpcResult<Balance>,
    },
    GetTransactions {
        wallet_name: String,
        result: RpcResult<Vec<TxInfo>>,
    },
    GetWalletSpaces {
        wallet_name: String,
        result: RpcResult<ListSpacesResponse>,
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
    RenewSpace {
        result: RpcResult<()>,
    },
    TransferSpace {
        result: RpcResult<()>,
    },
    BumpFee {
        result: RpcResult<()>,
    },
    BuySpace {
        result: RpcResult<()>,
    },
    SellSpace {
        wallet_name: String,
        result: RpcResult<Listing>,
    },
    SignEvent {
        result: RpcResult<NostrEvent>,
    },
}

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
    RpcRequest(RpcRequest),
    RpcResponse(RpcResponse),
    NavigateTo(Route),
    HomeScreen(screen::home::Message),
    SendScreen(screen::send::Message),
    ReceiveScreen(screen::receive::Message),
    SpacesScreen(screen::spaces::Message),
    MarketScreen(screen::market::Message),
    SignScreen(screen::sign::Message),
    EventFileLoaded(Result<Option<(String, NostrEvent)>, String>),
    EventFileSaved(Result<(), String>),
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
    rpc_client: Arc<HttpClient>,
    rpc_error: Option<String>,
    screen: Screen,
    tip_height: u32,
    wallet: Option<WalletState>,
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
                home_screen: Default::default(),
                send_screen: Default::default(),
                receive_screen: Default::default(),
                spaces_screen: Default::default(),
                market_screen: Default::default(),
                sign_screen: Default::default(),
            },
            Task::done(Message::RpcRequest(RpcRequest::LoadWallet {
                wallet_name: args.wallet.into(),
            })),
            // Task::batch([
            //     Task::done(Message::RpcRequest(RpcRequest::LoadWallet {
            //         wallet_name: args.wallet.into(),
            //     })),
            //     Task::done(Message::RpcRequest(RpcRequest::GetServerInfo)),
            // ]),
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
                            use spaces_client::store::Sha256;
                            use spaces_protocol::hasher::KeyHasher;

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
                    RpcRequest::CreateWallet { wallet_name } => Task::perform(
                        async move {
                            let result = client
                                .wallet_create(&wallet_name)
                                .await
                                .map_err(RpcError::from);
                            RpcResponse::CreateWallet {
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
                    RpcRequest::GetTransactions => {
                        if let Some(wallet) = self.wallet.as_ref() {
                            let wallet_name = wallet.name.clone();
                            let count = self.home_screen.get_transactions_limit();
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_list_transactions(&wallet_name, count, 0)
                                        .await
                                        .map_err(RpcError::from);
                                    RpcResponse::GetTransactions {
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
                    RpcRequest::RenewSpace { slabel, fee_rate } => {
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
                                    RpcResponse::RenewSpace { result }
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
                                                        to: Some(recipient),
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
                    RpcRequest::BumpFee { txid, fee_rate } => {
                        if let Some(wallet) = self.wallet.as_ref() {
                            let wallet_name = wallet.name.clone();
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_bump_fee(&wallet_name, txid, fee_rate, false)
                                        .await
                                        .map(|_| ())
                                        .map_err(RpcError::from);
                                    RpcResponse::BumpFee { result }
                                },
                                Message::RpcResponse,
                            )
                        } else {
                            Task::none()
                        }
                    }
                    RpcRequest::BuySpace { listing, fee_rate } => {
                        if let Some(wallet) = self.wallet.as_ref() {
                            let wallet_name = wallet.name.clone();
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_buy(&wallet_name, listing, fee_rate, false)
                                        .await
                                        .map(|_| ())
                                        .map_err(RpcError::from);
                                    RpcResponse::BuySpace { result }
                                },
                                Message::RpcResponse,
                            )
                        } else {
                            Task::none()
                        }
                    }
                    RpcRequest::SellSpace { slabel, price } => {
                        if let Some(wallet) = self.wallet.as_ref() {
                            let wallet_name = wallet.name.clone();
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_sell(
                                            &wallet_name,
                                            slabel.to_string(),
                                            price.to_sat(),
                                        )
                                        .await
                                        .map_err(RpcError::from);
                                    RpcResponse::SellSpace {
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
                    RpcRequest::SignEvent { slabel, event } => {
                        if let Some(wallet) = self.wallet.as_ref() {
                            let wallet_name = wallet.name.clone();
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_sign_event(&wallet_name, &slabel.to_string(), event)
                                        .await
                                        .map_err(RpcError::from);
                                    RpcResponse::SignEvent { result }
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
                                self.spaces.insert(slabel, out);
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
                            Task::batch([
                                Task::done(Message::NavigateTo(Route::Home)),
                                Task::done(Message::RpcRequest(RpcRequest::GetServerInfo)),
                            ])
                        }
                        Err(e) => {
                            if let RpcError::Call { code, message: _ } = e {
                                if code == -18 {
                                    return Task::done(Message::RpcRequest(
                                        RpcRequest::CreateWallet { wallet_name },
                                    ));
                                }
                            }
                            self.rpc_error = Some(e.to_string());
                            Task::none()
                        }
                    },
                    RpcResponse::CreateWallet {
                        wallet_name,
                        result,
                    } => match result {
                        Ok(_) => {
                            self.wallet = Some(WalletState::new(wallet_name));
                            Task::batch([
                                Task::done(Message::NavigateTo(Route::Home)),
                                Task::done(Message::RpcRequest(RpcRequest::GetServerInfo)),
                            ])
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
                                        let mut collect =
                                            |spaces: Vec<FullSpaceOut>| -> Vec<SLabel> {
                                                spaces
                                                    .into_iter()
                                                    .map(|out| {
                                                        let name = out
                                                            .spaceout
                                                            .space
                                                            .as_ref()
                                                            .unwrap()
                                                            .name
                                                            .clone();
                                                        self.spaces.insert(name.clone(), Some(out));
                                                        name
                                                    })
                                                    .collect()
                                            };
                                        wallet.winning_spaces = collect(spaces.winning);
                                        wallet.outbid_spaces = collect(spaces.outbid);
                                        wallet.owned_spaces = collect(spaces.owned);
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
                        result,
                    } => {
                        match result {
                            Ok(transactions) => {
                                if let Some(wallet) = self.wallet.as_mut() {
                                    if wallet.name == wallet_name {
                                        wallet.transactions = transactions;
                                    }
                                }
                            }
                            Err(e) => {
                                self.rpc_error = Some(e.to_string());
                            }
                        }
                        Task::none()
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
                    | RpcResponse::RenewSpace { result } => match result {
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
                    RpcResponse::TransferSpace { result } => match result {
                        Ok(_) => {
                            self.send_screen.reset_inputs();
                            Task::done(Message::NavigateTo(Route::Home))
                        }
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
                    RpcResponse::BumpFee { result } => match result {
                        Ok(_) => {
                            self.home_screen.reset_inputs();
                            Task::done(Message::NavigateTo(Route::Home))
                        }
                        Err(RpcError::Call { code, message }) => {
                            if code == -1 {
                                self.home_screen.set_error(message);
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
                    RpcResponse::BuySpace { result } => match result {
                        Ok(_) => Task::done(Message::NavigateTo(Route::Home)),
                        Err(RpcError::Call { code, message }) => {
                            if code == -1 {
                                self.market_screen.set_buy_error(message);
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
                    RpcResponse::SellSpace {
                        wallet_name,
                        result,
                    } => {
                        match result {
                            Ok(listing) => {
                                if let Some(wallet) = self.wallet.as_ref() {
                                    if wallet.name == wallet_name {
                                        self.market_screen.set_sell_listing(listing);
                                    }
                                }
                            }
                            Err(RpcError::Call { code, message }) => {
                                if code == -1 {
                                    self.market_screen.set_sell_error(message);
                                } else {
                                    self.rpc_error = Some(message);
                                }
                            }
                            Err(e) => {
                                self.rpc_error = Some(e.to_string());
                            }
                        }
                        Task::none()
                    }
                    RpcResponse::SignEvent { result } => match result {
                        Ok(event) => Task::future(async move {
                            let file_path = rfd::AsyncFileDialog::new()
                                .add_filter("JSON event", &["json"])
                                .add_filter("All files", &["*"])
                                .save_file()
                                .await
                                .map(|file| file.path().to_path_buf());

                            if let Some(file_path) = file_path {
                                let contents = serde_json::to_vec(&event).unwrap();
                                Message::EventFileSaved(
                                    tokio::fs::write(&file_path, contents)
                                        .await
                                        .map_err(|e| e.to_string()),
                                )
                            } else {
                                Message::EventFileSaved(Ok(()))
                            }
                        }),
                        Err(RpcError::Call { code, message }) => {
                            if code == -1 {
                                self.sign_screen.set_error(message);
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
                    if self.screen == Screen::Home {
                        self.home_screen.reset();
                    } else {
                        self.screen = Screen::Home;
                    }
                    Task::batch([
                        Task::done(Message::RpcRequest(RpcRequest::GetBalance)),
                        Task::done(Message::RpcRequest(RpcRequest::GetWalletSpaces)),
                        Task::done(Message::RpcRequest(RpcRequest::GetTransactions)),
                    ])
                }
                Route::Send => {
                    self.screen = Screen::Send;
                    Task::done(Message::RpcRequest(RpcRequest::GetWalletSpaces))
                }
                Route::Receive => {
                    self.screen = Screen::Receive;
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
                        self.spaces_screen.reset();
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
                Route::Market => {
                    self.screen = Screen::Market;
                    Task::done(Message::RpcRequest(RpcRequest::GetWalletSpaces))
                }
                Route::Sign => {
                    self.screen = Screen::Sign;
                    Task::done(Message::RpcRequest(RpcRequest::GetWalletSpaces))
                }
            },
            Message::HomeScreen(message) => match self.home_screen.update(message) {
                screen::home::Action::WriteClipboard(s) => clipboard::write(s),
                screen::home::Action::ShowSpace { slabel } => {
                    Task::done(Message::NavigateTo(Route::Space(slabel)))
                }
                screen::home::Action::GetTransactions => {
                    Task::done(Message::RpcRequest(RpcRequest::GetTransactions))
                }
                screen::home::Action::BumpFee { txid, fee_rate } => {
                    Task::done(Message::RpcRequest(RpcRequest::BumpFee { txid, fee_rate }))
                }
                screen::home::Action::None => Task::none(),
            },
            Message::SendScreen(message) => match self.send_screen.update(message) {
                screen::send::Action::SendCoins {
                    recipient,
                    amount,
                    fee_rate,
                } => Task::done(Message::RpcRequest(RpcRequest::SendCoins {
                    recipient,
                    amount,
                    fee_rate,
                })),
                screen::send::Action::SendSpace {
                    recipient,
                    slabel,
                    fee_rate,
                } => Task::done(Message::RpcRequest(RpcRequest::TransferSpace {
                    slabel,
                    recipient,
                    fee_rate,
                })),
                screen::send::Action::None => Task::none(),
            },
            Message::ReceiveScreen(message) => match self.receive_screen.update(message) {
                screen::receive::Action::WriteClipboard(s) => clipboard::write(s),
                screen::receive::Action::None => Task::none(),
            },
            Message::SpacesScreen(message) => match self.spaces_screen.update(message) {
                screen::spaces::Action::WriteClipboard(s) => clipboard::write(s),
                screen::spaces::Action::GetSpaceInfo { slabel } => {
                    Task::done(Message::RpcRequest(RpcRequest::GetSpaceInfo { slabel }))
                }
                screen::spaces::Action::OpenSpace {
                    slabel,
                    amount,
                    fee_rate,
                } => Task::done(Message::RpcRequest(RpcRequest::OpenSpace {
                    slabel,
                    amount,
                    fee_rate,
                })),
                screen::spaces::Action::BidSpace {
                    slabel,
                    amount,
                    fee_rate,
                } => Task::done(Message::RpcRequest(RpcRequest::BidSpace {
                    slabel,
                    amount,
                    fee_rate,
                })),
                screen::spaces::Action::RegisterSpace { slabel, fee_rate } => {
                    Task::done(Message::RpcRequest(RpcRequest::RegisterSpace {
                        slabel,
                        fee_rate,
                    }))
                }
                screen::spaces::Action::RenewSpace { slabel, fee_rate } => {
                    Task::done(Message::RpcRequest(RpcRequest::RenewSpace {
                        slabel,
                        fee_rate,
                    }))
                }
                screen::spaces::Action::None => Task::none(),
            },
            Message::MarketScreen(message) => match self.market_screen.update(message) {
                screen::market::Action::None => Task::none(),
                screen::market::Action::Buy { listing, fee_rate } => {
                    Task::done(Message::RpcRequest(RpcRequest::BuySpace {
                        listing,
                        fee_rate,
                    }))
                }
                screen::market::Action::Sell { slabel, price } => {
                    Task::done(Message::RpcRequest(RpcRequest::SellSpace { slabel, price }))
                }
                screen::market::Action::WriteClipboard(s) => clipboard::write(s),
            },
            Message::SignScreen(message) => match self.sign_screen.update(message) {
                screen::sign::Action::None => Task::none(),
                screen::sign::Action::FilePick => Task::future(async move {
                    let path = rfd::AsyncFileDialog::new()
                        .add_filter("JSON event", &["json"])
                        .pick_file()
                        .await
                        .map(|file| file.path().to_path_buf());

                    Message::EventFileLoaded(if let Some(path) = path {
                        match tokio::fs::read_to_string(&path).await {
                            Ok(content) => match serde_json::from_str::<NostrEvent>(&content) {
                                Ok(event) => Ok(Some((path.to_string_lossy().to_string(), event))),
                                Err(err) => Err(format!("Failed to parse JSON: {}", err)),
                            },
                            Err(err) => Err(format!("Failed to read file: {}", err)),
                        }
                    } else {
                        Ok(None)
                    })
                }),
                screen::sign::Action::Sign(slabel, event) => {
                    Task::done(Message::RpcRequest(RpcRequest::SignEvent { slabel, event }))
                }
            },
            Message::EventFileLoaded(result) => {
                match result {
                    Ok(Some(event_file)) => {
                        self.sign_screen.set_event_file(event_file);
                    }
                    Ok(None) => {}
                    Err(err) => self.sign_screen.set_error(err),
                }
                Task::none()
            }
            Message::EventFileSaved(result) => {
                if let Err(err) = result {
                    self.sign_screen.set_error(err);
                }
                Task::none()
            }
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
        column![error_block(self.rpc_error.as_ref()), main].into()
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.wallet.is_some() && self.rpc_error.is_none() {
            let mut subscriptions = vec![
                time::every(time::Duration::from_secs(30))
                    .map(|_| Message::RpcRequest(RpcRequest::GetServerInfo)),
            ];
            match self.screen {
                Screen::Home => {
                    subscriptions.push(
                        time::every(time::Duration::from_secs(30))
                            .map(|_| Message::RpcRequest(RpcRequest::GetBalance)),
                    );
                    subscriptions.push(
                        time::every(time::Duration::from_secs(30))
                            .map(|_| Message::RpcRequest(RpcRequest::GetTransactions)),
                    );
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
