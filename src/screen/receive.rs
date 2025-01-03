use iced::{
    widget::{button, center, column, container, qr_code, row, text},
    Border, Center, Element, Fill, Font, Theme,
};

use crate::{
    types::*,
    widget::icon::{button_icon, Icon},
};

#[derive(Debug, Clone, Default)]
pub enum State {
    #[default]
    Home,
    QrCode(AddressKind),
}

#[derive(Debug, Clone)]
pub enum Message {
    QrCodePress(AddressKind),
    ClosePress,
    CopyPress(String),
}

#[derive(Debug, Clone)]
pub enum Task {
    None,
    WriteClipboard(String),
}

impl State {
    pub fn reset(&mut self) {
        *self = Self::Home;
    }

    pub fn update(&mut self, message: Message) -> Task {
        match message {
            Message::ClosePress => {
                *self = State::Home;
                Task::None
            }
            Message::QrCodePress(kind) => {
                *self = Self::QrCode(kind);
                Task::None
            }
            Message::CopyPress(s) => Task::WriteClipboard(s),
        }
    }

    pub fn view<'a>(
        &self,
        coin_address: Option<&'a AddressState>,
        space_address: Option<&'a AddressState>,
    ) -> Element<'a, Message> {
        match (self, coin_address, space_address) {
            (Self::Home, Some(coin_address), Some(space_address)) => {
                let address_block = |title: &'a str, address: &'a str, kind: AddressKind| {
                    column![
                        text(title),
                        container(
                            row![
                                text(address).font(Font::MONOSPACE).width(Fill),
                                button_icon(Icon::Copy)
                                    .style(button::secondary)
                                    .on_press(Message::CopyPress(address.to_string())),
                                button_icon(Icon::Qrcode)
                                    .style(button::secondary)
                                    .on_press(Message::QrCodePress(kind)),
                            ]
                            .align_y(Center)
                            .spacing(5)
                        )
                        .padding(10)
                        .style(|theme: &Theme| {
                            let palette = theme.extended_palette();
                            container::Style::default()
                                .background(palette.background.base.color)
                                .border(Border {
                                    radius: 2.0.into(),
                                    width: 1.0,
                                    color: palette.background.strong.color,
                                })
                        })
                    ]
                };
                column![
                    address_block(
                        "Coins only address",
                        coin_address.as_str(),
                        AddressKind::Coin
                    ),
                    address_block("Spaces address", space_address.as_str(), AddressKind::Space),
                ]
            }
            .into(),
            (Self::QrCode(AddressKind::Coin), Some(address), _)
            | (Self::QrCode(AddressKind::Space), _, Some(address)) => center(
                column![
                    qr_code(address.as_qr_code()).cell_size(7),
                    text(address.as_str())
                        .font(Font::MONOSPACE)
                        .align_x(Center)
                        .width(Fill),
                    button("Close")
                        .style(button::secondary)
                        .on_press(Message::ClosePress)
                ]
                .spacing(10)
                .align_x(Center),
            )
            .into(),
            _ => center(text("Loading")).into(),
        }
    }
}
