use protocol::bitcoin;
use protocol::{
    hasher::{KeyHasher, SpaceHash},
    sname::{NameLike, SName},
};
use spaced::{
    config::{default_spaces_rpc_port, ExtendedNetwork},
    store::Sha256,
};
use std::str::FromStr;

fn default_spaced_rpc_url(chain: &ExtendedNetwork) -> String {
    format!("http://127.0.0.1:{}", default_spaces_rpc_port(chain))
}

pub fn default_testnet4_spaced_rpc_url() -> String {
    default_spaced_rpc_url(&spaced::config::ExtendedNetwork::Testnet4)
}

pub fn space_sname(spaceish: &str) -> Option<SName> {
    let mut space = spaceish.to_ascii_lowercase();
    if !space.starts_with('@') {
        space.insert_str(0, "@");
    }
    SName::from_str(&space).ok()
}

pub fn space_hash(spaceish: &str) -> Option<String> {
    space_sname(spaceish).map(|sname| {
        let spacehash = SpaceHash::from(Sha256::hash(sname.to_bytes()));
        hex::encode(spacehash.as_slice())
    })
}

pub fn coin_address(
    addr: &str,
) -> Option<bitcoin::address::Address<bitcoin::address::NetworkUnchecked>> {
    bitcoin::address::Address::from_str(addr).ok()
}
