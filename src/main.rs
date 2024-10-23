// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

use jsonrpsee::http_client::HttpClientBuilder;
use protocol::bitcoin::Amount;
use spaced::{
    rpc::{
        BidParams, ExecuteParams, OpenParams, RegisterParams, RpcClient, RpcWalletRequest,
        RpcWalletTxBuilder, SendCoinsParams, TransferSpacesParams,
    },
    wallets::AddressKind,
};

use tokio::runtime::Runtime;
use tokio::sync::mpsc;

mod util;

slint::include_modules!();

#[derive(Debug)]
enum Command {
    GenerateAddress(AddressKind),
    LoadSpace(String),
    SendCoins(u64, String),
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    let (tx, mut rx) = mpsc::unbounded_channel::<Command>();

    let ui_handle = ui.as_weak();

    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move {
            let spaced_rpc_url = util::default_testnet4_spaced_rpc_url();
            let rpc = HttpClientBuilder::default().build(spaced_rpc_url).unwrap();
            rpc.wallet_load("default").await.unwrap();

            while let Some(command) = rx.recv().await {
                match command {
                    Command::GenerateAddress(kind) => {
                        let address = rpc.wallet_get_new_address("default", kind).await.unwrap();
                        let ui_handle = ui_handle.clone();
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_handle.upgrade() {
                                match kind {
                                    AddressKind::Coin => ui.set_coin_address(address.into()),
                                    AddressKind::Space => ui.set_space_address(address.into()),
                                };
                            }
                        })
                        .unwrap_or_else(|e| {
                            eprintln!("Failed to invoke UI update: {}", e);
                        });
                    }
                    Command::LoadSpace(space) => {
                        if let Some(space_hash) = util::space_hash(&space) {
                            let space = rpc
                                .get_space(&space_hash)
                                .await
                                .unwrap_or_default()
                                .and_then(|out| out.spaceout.space);
                            if let Some(space) = space {
                                let ui_handle = ui_handle.clone();
                                slint::invoke_from_event_loop(move || {
                                    if let Some(ui) = ui_handle.upgrade() {
                                        ui.set_current_space(Space {
                                            name: space.name.to_string().into(),
                                        })
                                    }
                                })
                                .unwrap_or_else(|e| {
                                    eprintln!("Failed to invoke UI update: {}", e);
                                });
                            }
                        }
                    }
                    Command::SendCoins(amount, address) => {
                        let result = rpc
                            .wallet_send_request(
                                "default",
                                RpcWalletTxBuilder {
                                    auction_outputs: None,
                                    requests: vec![RpcWalletRequest::SendCoins(SendCoinsParams {
                                        amount: Amount::from_sat(amount),
                                        to: address,
                                    })],
                                    fee_rate: None,
                                    dust: None,
                                    force: false,
                                },
                            )
                            .await;
                        println!("{:?}", result);
                    }
                }
            }
        });
    });

    let txc = tx.clone();
    ui.on_generate_coin_address(move || {
        txc.send(Command::GenerateAddress(AddressKind::Coin))
            .unwrap_or_else(|e| {
                eprintln!("Failed to send command: {}", e);
            });
    });
    let txc = tx.clone();
    ui.on_generate_space_address(move || {
        txc.send(Command::GenerateAddress(AddressKind::Space))
            .unwrap_or_else(|e| {
                eprintln!("Failed to send command: {}", e);
            });
    });
    let txc = tx.clone();
    ui.on_load_space(move |space| {
        txc.send(Command::LoadSpace(space.into()))
            .unwrap_or_else(|e| {
                eprintln!("Failed to send command: {}", e);
            });
    });
    let txc = tx.clone();
    ui.on_send_coins(move |address, amount| {
        txc.send(Command::SendCoins(amount as u64, address.into()))
            .unwrap_or_else(|e| {
                eprintln!("Failed to send command: {}", e);
            });
    });

    ui.global::<QrCodeAdapter>().on_qr_code(|s| {
        let qr = qrcode::QrCode::new(s).unwrap();
        let image = qr
            .render()
            .dark_color(qrcode::render::svg::Color("#FF8400"))
            .light_color(qrcode::render::svg::Color("rgba(0,0,0,0)"))
            .build();
        slint::Image::load_from_svg_data(image.as_bytes()).unwrap()
    });

    ui.global::<Validators>().on_space_name(|s| {
        if !s.chars().all(|c| c.is_alphanumeric() || c == '@') {
            ValidatorResult::Reject
        } else if util::space_hash(&s.to_string()).is_some() {
            ValidatorResult::Valid
        } else {
            ValidatorResult::Invalid
        }
    });
    ui.global::<Validators>().on_coin_address(|s| {
        if !s.chars().all(|c| c.is_alphanumeric() || c == '@') {
            ValidatorResult::Reject
        } else if util::coin_address(&s.to_string()).is_some() {
            ValidatorResult::Valid
        } else {
            ValidatorResult::Invalid
        }
    });

    ui.run()?;

    Ok(())
}
