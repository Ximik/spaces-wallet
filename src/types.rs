pub use protocol::{bitcoin::Txid, slabel::SLabel, Covenant, FullSpaceOut};
pub use spaced::wallets::{AddressKind, ListSpacesResponse, TxInfo};
pub use std::str::FromStr;
pub use wallet::{
    bdk_wallet::serde::Deserialize,
    bitcoin::{Amount, Denomination, FeeRate},
    tx_event::{BidEventDetails, BidoutEventDetails, OpenEventDetails, TxEvent, TxEventKind},
    Balance,
};

pub fn is_slabel_input(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase() || c == '-')
}

pub fn slabel_from_str(s: &str) -> Option<SLabel> {
    SLabel::from_str_unprefixed(s)
        .ok()
        .filter(|slabel| !slabel.is_reserved())
}

pub fn is_recipient_input(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase() || c == '-' || c == '@')
}

pub fn recipient_from_str(s: &str) -> Option<String> {
    // TODO: check
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

pub fn is_amount_input(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit())
}

pub fn amount_from_str(s: &str) -> Option<Amount> {
    Amount::from_str_in(s, Denomination::Satoshi).ok()
}

pub fn is_fee_rate_input(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit())
}

pub fn fee_rate_from_str(s: &str) -> Option<Option<FeeRate>> {
    if s.is_empty() {
        Some(None)
    } else {
        s.parse().ok().map(|n| FeeRate::from_sat_per_vb(n))
    }
}

pub type SpacesState = rustc_hash::FxHashMap<SLabel, Option<Covenant>>;

use iced::widget::qr_code::Data as QrCode;
#[derive(Debug)]
pub struct AddressState {
    text: String,
    qr_code: QrCode,
}
impl AddressState {
    pub fn new(text: String) -> Self {
        let qr_code = QrCode::new(&text).unwrap();
        Self { text, qr_code }
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn as_qr_code(&self) -> &QrCode {
        &self.qr_code
    }
}

#[derive(Debug, Default)]
pub struct WalletState {
    pub name: String,
    pub balance: Amount,
    pub coin_address: Option<AddressState>,
    pub space_address: Option<AddressState>,
    pub winning_spaces: Vec<SLabel>,
    pub outbid_spaces: Vec<SLabel>,
    pub owned_spaces: Vec<SLabel>,
    pub transactions: Vec<TxInfo>,
}
impl WalletState {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}
