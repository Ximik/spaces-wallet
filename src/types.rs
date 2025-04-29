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
        s.parse().ok().map(FeeRate::from_sat_per_vb)
    }
}

pub fn listing_from_str(s: &str) -> Option<Listing> {
    serde_json::from_str(s).ok()
}
