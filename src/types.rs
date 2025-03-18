pub use spaces_client::wallets::{AddressKind, ListSpacesResponse, TxInfo};
pub use spaces_protocol::{Covenant, FullSpaceOut, bitcoin::Txid, slabel::SLabel};
use spaces_wallet::bdk_wallet::serde_json;
pub use spaces_wallet::{
    Balance, Listing,
    bdk_wallet::serde::Deserialize,
    bitcoin::{Amount, Denomination, FeeRate, OutPoint},
    nostr::NostrEvent,
    tx_event::{
        BidEventDetails, BidoutEventDetails, OpenEventDetails, SendEventDetails, TxEvent,
        TxEventKind,
    },
};
pub use std::str::FromStr;

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

pub fn listing_from_str(s: &str) -> Option<Listing> {
    serde_json::from_str(s).ok()
}

#[derive(Debug)]
pub struct SpaceState {
    outpoint: OutPoint,
    covenant: Covenant,
}
#[derive(Debug, Default)]
pub struct SpacesState(rustc_hash::FxHashMap<SLabel, Option<SpaceState>>);
impl SpacesState {
    pub fn insert(&mut self, slabel: SLabel, out: Option<FullSpaceOut>) {
        self.0.insert(
            slabel,
            out.map(|out| SpaceState {
                outpoint: out.outpoint(),
                covenant: out.spaceout.space.unwrap().covenant,
            }),
        );
    }

    pub fn get_outpoint(&self, slabel: &SLabel) -> Option<&OutPoint> {
        self.0
            .get(slabel)
            .and_then(|o| o.as_ref().map(|s| &s.outpoint))
    }

    pub fn get_covenant(&self, slabel: &SLabel) -> Option<Option<&Covenant>> {
        self.0.get(slabel).map(|o| o.as_ref().map(|s| &s.covenant))
    }
}

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
