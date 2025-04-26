use iced::{
    Alignment::Center,
    Element, Fill,
    widget::{button, column, text, vertical_space},
};

use crate::widget::form::pick_list;

#[derive(Debug, Default)]
pub struct State;

#[derive(Debug, Clone)]
pub enum Message {
    WalletSelect(String),
    ResetBackendPress,
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    SetCurrentWallet(String),
    ResetBackend,
}

impl State {
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::WalletSelect(s) => Action::SetCurrentWallet(s),
            Message::ResetBackendPress => Action::ResetBackend,
        }
    }

    pub fn view<'a>(
        &self,
        wallets_names: Vec<&'a String>,
        wallet_name: Option<&'a String>,
    ) -> Element<'a, Message> {
        column![
            column![pick_list(wallets_names, wallet_name, |name| {
                Message::WalletSelect(name.clone())
            })],
            vertical_space(),
            button(text("Reset backend settings").align_x(Center).width(Fill))
                .on_press(Message::ResetBackendPress)
                .style(button::danger)
                .padding(10)
                .width(Fill),
        ]
        .into()
    }
}
