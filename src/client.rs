use jsonrpsee::{
    core::ClientError as JsonClientError,
    http_client::{HttpClient, HttpClientBuilder},
};
use spaces_client::rpc::{
    BidParams, OpenParams, RegisterParams, RpcClient, RpcWalletRequest, RpcWalletTxBuilder,
    SendCoinsParams, ServerInfo, TransferSpacesParams,
};
use spaces_client::wallets::{AddressKind, ListSpacesResponse, TxInfo};
use spaces_protocol::{FullSpaceOut, bitcoin::Txid, slabel::SLabel};
use spaces_wallet::{
    Balance, Listing,
    bitcoin::{Amount, FeeRate},
    nostr::NostrEvent,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Client {
    client: Arc<HttpClient>,
}

#[derive(Debug, Clone)]
pub enum ClientError {
    Call(String),
    System(String),
}
fn convert_result<T>(result: Result<T, JsonClientError>) -> Result<T, ClientError> {
    result.map_err(|e| match e {
        JsonClientError::Call(e) => ClientError::Call (e.message().to_string()),
        _ => ClientError::System(e.to_string()),
    })
}
fn convert_result_error<T>(result: Result<T, JsonClientError>) -> Result<T, String> {
    result.map_err(|e| e.to_string())
}
fn convert_empty_result<T>(result: Result<T, JsonClientError>) -> Result<(), ClientError> {
    convert_result(result).map(|_| ())
}

impl Client {
    pub fn new(rpc_url: &str) -> Self {
        let client = Arc::new(HttpClientBuilder::default().build(rpc_url).unwrap());
        Self { client }
    }

    pub async fn get_server_info(&self) -> Result<ServerInfo, String> {
        let result = self.client.get_server_info().await;
        convert_result_error(result)
    }

    pub async fn get_space_info(&self, slabel: &SLabel) -> Result<Option<FullSpaceOut>, String> {
        use spaces_client::store::Sha256;
        use spaces_protocol::hasher::KeyHasher;
        let hash = hex::encode(Sha256::hash(slabel.as_ref()));
        let result = self.client.get_space(&hash).await;
        convert_result_error(result)
    }

    pub async fn load_wallet(&self, wallet_name: &str) -> Result<(), String> {
        let result = self.client.wallet_load(wallet_name).await;
        convert_result_error(result)
    }

    pub async fn create_wallet(&self, wallet_name: &str) -> Result<(), String> {
        let result = self.client.wallet_create(wallet_name).await;
        convert_result_error(result)
    }

    pub async fn get_wallet_balance(&self, wallet_name: &str) -> Result<Balance, String> {
        let result = self.client.wallet_get_balance(wallet_name).await;
        convert_result_error(result)
    }

    pub async fn get_wallet_spaces(&self, wallet_name: &str) -> Result<ListSpacesResponse, String> {
        let result = self.client.wallet_list_spaces(wallet_name).await;
        convert_result_error(result)
    }

    pub async fn get_wallet_transactions(
        &self,
        wallet_name: &str,
        count: usize,
    ) -> Result<Vec<TxInfo>, String> {
        let result = self
            .client
            .wallet_list_transactions(wallet_name, count, 0)
            .await;
        convert_result_error(result)
    }

    pub async fn get_wallet_address(
        &self,
        wallet_name: &str,
        address_kind: AddressKind,
    ) -> Result<String, String> {
        let result = self
            .client
            .wallet_get_new_address(wallet_name, address_kind)
            .await;
        convert_result_error(result)
    }

    pub async fn send_coins(
        &self,
        wallet_name: &str,
        recipient: String,
        amount: Amount,
        fee_rate: Option<FeeRate>,
    ) -> Result<(), ClientError> {
        let result = self
            .client
            .wallet_send_request(
                wallet_name,
                RpcWalletTxBuilder {
                    bidouts: None,
                    requests: vec![RpcWalletRequest::SendCoins(SendCoinsParams {
                        amount,
                        to: recipient,
                    })],
                    fee_rate,
                    dust: None,
                    force: false,
                    confirmed_only: false,
                    skip_tx_check: false,
                },
            )
            .await;
        convert_empty_result(result)
    }

    pub async fn open_space(
        &self,
        wallet_name: &str,
        slabel: SLabel,
        amount: Amount,
        fee_rate: Option<FeeRate>,
    ) -> Result<(), ClientError> {
        let name = slabel.to_string();
        let amount = amount.to_sat();
        let result = self
            .client
            .wallet_send_request(
                wallet_name,
                RpcWalletTxBuilder {
                    bidouts: None,
                    requests: vec![RpcWalletRequest::Open(OpenParams { name, amount })],
                    fee_rate,
                    dust: None,
                    force: false,
                    confirmed_only: false,
                    skip_tx_check: false,
                },
            )
            .await;
        convert_empty_result(result)
    }

    pub async fn bid_space(
        &self,
        wallet_name: &str,
        slabel: SLabel,
        amount: Amount,
        fee_rate: Option<FeeRate>,
    ) -> Result<(), ClientError> {
        let name = slabel.to_string();
        let amount = amount.to_sat();
        let result = self
            .client
            .wallet_send_request(
                wallet_name,
                RpcWalletTxBuilder {
                    bidouts: None,
                    requests: vec![RpcWalletRequest::Bid(BidParams { name, amount })],
                    fee_rate,
                    dust: None,
                    force: false,
                    confirmed_only: false,
                    skip_tx_check: false,
                },
            )
            .await;
        convert_empty_result(result)
    }

    pub async fn register_space(
        &self,
        wallet_name: &str,
        slabel: SLabel,
        fee_rate: Option<FeeRate>,
    ) -> Result<(), ClientError> {
        let result = self
            .client
            .wallet_send_request(
                wallet_name,
                RpcWalletTxBuilder {
                    bidouts: None,
                    requests: vec![RpcWalletRequest::Register(RegisterParams {
                        name: slabel.to_string(),
                        to: None,
                    })],
                    fee_rate,
                    dust: None,
                    force: false,
                    confirmed_only: false,
                    skip_tx_check: false,
                },
            )
            .await;
        convert_empty_result(result)
    }

    pub async fn renew_space(
        &self,
        wallet_name: &str,
        slabel: SLabel,
        fee_rate: Option<FeeRate>,
    ) -> Result<(), ClientError> {
        let result = self
            .client
            .wallet_send_request(
                wallet_name,
                RpcWalletTxBuilder {
                    bidouts: None,
                    requests: vec![RpcWalletRequest::Transfer(TransferSpacesParams {
                        spaces: vec![slabel.to_string()],
                        to: None,
                    })],
                    fee_rate,
                    dust: None,
                    force: false,
                    confirmed_only: false,
                    skip_tx_check: false,
                },
            )
            .await;
        convert_empty_result(result)
    }

    pub async fn send_space(
        &self,
        wallet_name: &str,
        recipient: String,
        slabel: SLabel,
        fee_rate: Option<FeeRate>,
    ) -> Result<(), ClientError> {
        let result = self
            .client
            .wallet_send_request(
                wallet_name,
                RpcWalletTxBuilder {
                    bidouts: None,
                    requests: vec![RpcWalletRequest::Transfer(TransferSpacesParams {
                        spaces: vec![slabel.to_string()],
                        to: Some(recipient),
                    })],
                    fee_rate,
                    dust: None,
                    force: false,
                    confirmed_only: false,
                    skip_tx_check: false,
                },
            )
            .await;
        convert_empty_result(result)
    }

    pub async fn bump_fee(
        &self,
        wallet_name: &str,
        txid: Txid,
        fee_rate: FeeRate,
    ) -> Result<(), ClientError> {
        let result = self
            .client
            .wallet_bump_fee(wallet_name, txid, fee_rate, false)
            .await;
        convert_empty_result(result)
    }

    pub async fn buy_space(
        &self,
        wallet_name: &str,
        listing: Listing,
        fee_rate: Option<FeeRate>,
    ) -> Result<(), ClientError> {
        let result = self
            .client
            .wallet_buy(wallet_name, listing, fee_rate, false)
            .await;
        convert_empty_result(result)
    }

    pub async fn sell_space(
        &self,
        wallet_name: &str,
        slabel: SLabel,
        price: Amount,
    ) -> Result<Listing, ClientError> {
        let result = self
            .client
            .wallet_sell(wallet_name, slabel.to_string(), price.to_sat())
            .await;
        convert_result(result)
    }

    pub async fn sign_event(
        &self,
        wallet_name: &str,
        slabel: SLabel,
        event: NostrEvent,
    ) -> Result<NostrEvent, ClientError> {
        let result = self
            .client
            .wallet_sign_event(wallet_name, &slabel.to_string(), event)
            .await;
        convert_result(result)
    }
}
