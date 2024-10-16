// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

use jsonrpsee::http_client::HttpClientBuilder;
use spaced::{
    config::{default_spaces_rpc_port, ExtendedNetwork},
    rpc::RpcClient,
    wallets::AddressKind,
};

use tokio::runtime::Runtime;
use tokio::sync::mpsc;

fn default_spaced_rpc_url(chain: &ExtendedNetwork) -> String {
    format!("http://127.0.0.1:{}", default_spaces_rpc_port(chain))
}

slint::include_modules!();

#[derive(Debug)]
enum Command {
    GenerateAddress(AddressKind),
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    let (tx, mut rx) = mpsc::unbounded_channel::<Command>();

    let ui_handle = ui.as_weak();
    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move {
            let spaced_rpc_url = default_spaced_rpc_url(&spaced::config::ExtendedNetwork::Testnet4);
            let rpc = HttpClientBuilder::default().build(spaced_rpc_url).unwrap();
            rpc.wallet_load("default").await.unwrap();

            while let Some(command) = rx.recv().await {
                match command {
                    Command::GenerateAddress(kind) => {
                        let address = rpc.wallet_get_new_address("default", kind).await.unwrap();
                        let ui_handle = ui_handle.clone();
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_handle.upgrade() {
                                let adapter = ui.global::<ReceiveAdapter>();
                                match kind {
                                    AddressKind::Coin => adapter.set_coin_address(address.into()),
                                    AddressKind::Space => adapter.set_space_address(address.into()),
                                };
                            }
                        })
                        .unwrap_or_else(|e| {
                            eprintln!("Failed to invoke UI update: {}", e);
                        });
                    }
                };
            }
        });
    });

    let receive_adapter = ui.global::<ReceiveAdapter>();
    receive_adapter.on_generate_address(move |is_space_address| {
        tx.send(Command::GenerateAddress(if is_space_address {
            AddressKind::Space
        } else {
            AddressKind::Coin
        }));
    });
    receive_adapter.on_qr_code(|s| {
        let qr = qrcode::QrCode::new(s).unwrap();
        let image = qr
            .render()
            .dark_color(qrcode::render::svg::Color("#FF8400"))
            .light_color(qrcode::render::svg::Color("rgba(0,0,0,0)"))
            .build();
        slint::Image::load_from_svg_data(image.as_bytes()).unwrap()
    });

    ui.run()?;

    Ok(())
}
