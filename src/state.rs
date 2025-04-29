use iced::widget::qr_code::Data as QrCode;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use spaces_client::{
    config::ExtendedNetwork,
    wallets::{AddressKind, ListSpacesResponse, TxInfo},
};
use spaces_protocol::{Covenant, FullSpaceOut, bitcoin::Txid, slabel::SLabel};
use spaces_wallet::{
    Balance, Listing,
    bitcoin::{Amount, Denomination, FeeRate, OutPoint},
    nostr::NostrEvent,
    tx_event::{
        BidEventDetails, BidoutEventDetails, OpenEventDetails, SendEventDetails, TxEvent,
        TxEventKind,
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    path: PathBuf,
    pub spaced_rpc_url: Option<String>,
    pub network: ExtendedNetwork,
    pub wallet: Option<String>,
}

impl Config {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            spaced_rpc_url: None,
            network: ExtendedNetwork::Mainnet,
            wallet: None,
        }
    }

    pub fn load(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let config = fs::read_to_string(&path)?;
        let config = Self {
            path,
            ..serde_json::from_str(&config)?
        };
        Ok(config)
    }

    pub fn save(&self) {
        let config = serde_json::to_string_pretty(&self).unwrap();
        fs::write(&self.path, config).unwrap();
    }

    pub fn remove(&self) {
        fs::remove_file(&self.path).unwrap();
    }
}

#[derive(Debug)]
pub struct SpaceData {
    outpoint: OutPoint,
    covenant: Covenant,
}
#[derive(Debug, Default)]
pub struct SpacesCollection(rustc_hash::FxHashMap<SLabel, Option<SpaceData>>);
impl SpacesCollection {
    pub fn set(&mut self, slabel: SLabel, out: Option<FullSpaceOut>) {
        self.0.insert(
            slabel,
            out.map(|out| SpaceData {
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

#[derive(Debug)]
pub struct AddressData {
    text: String,
    qr_code: QrCode,
}
impl AddressData {
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
pub struct WalletData {
    pub tip: u32,
    pub balance: Amount,
    pub coin_address: Option<AddressData>,
    pub space_address: Option<AddressData>,
    pub winning_spaces: Vec<SLabel>,
    pub outbid_spaces: Vec<SLabel>,
    pub owned_spaces: Vec<SLabel>,
    pub transactions: Vec<TxInfo>,
}

pub struct WalletEntry<'a> {
    pub label: &'a String,
    pub state: &'a WalletData,
}

#[derive(Debug, Default)]
pub struct WalletsCollection {
    current: Option<String>,
    wallets: rustc_hash::FxHashMap<String, Option<WalletData>>,
}
impl WalletsCollection {
    pub fn set_wallets(&mut self, names: &[String]) {
        for name in names {
            self.wallets.retain(|key, _| names.contains(key));
            if !self.wallets.contains_key(name) {
                self.wallets.insert(name.clone(), None);
            }
        }
        if let Some(current) = self.current.take() {
            if self.wallets.contains_key(&current) {
                self.current = Some(current);
            }
        }
    }

    pub fn get_wallets(&self) -> Vec<&String> {
        self.wallets.keys().collect()
    }

    pub fn set_current(&mut self, label: &str) -> bool {
        if let Some(wallet_state) = self.wallets.get_mut(label) {
            self.current = Some(label.to_string());
            if wallet_state.is_none() {
                *wallet_state = Some(WalletData::default());
            }
            true
        } else {
            false
        }
    }

    pub fn unset_current(&mut self) {
        self.current = None;
    }

    pub fn get_current(&self) -> Option<WalletEntry<'_>> {
        self.current.as_ref().and_then(|label| {
            self.wallets
                .get_key_value(label)
                .and_then(|(name, wallet_state)| {
                    wallet_state
                        .as_ref()
                        .map(|state| WalletEntry { label: name, state })
                })
        })
    }

    pub fn get_data_mut(&mut self, label: &str) -> Option<&mut WalletData> {
        self.wallets.get_mut(label).and_then(|state| state.as_mut())
    }
}
