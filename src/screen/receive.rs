use iced::widget::{button, center, column, container, qr_code, row, text, toggler};
use iced::{Border, Center, Element, Fill, Font, Padding, Theme};

use crate::types::*;
use crate::widget::icon::{button_icon, Icon};

#[derive(Debug, Clone, Default)]
pub struct State {
    coin_address: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddressKindToggle(bool),
    CopyPress(String),
}

#[derive(Debug, Clone)]
pub enum Task {
    None,
    WriteClipboard(String),
}

impl State {
    pub fn update(&mut self, message: Message) -> Task {
        match message {
            Message::AddressKindToggle(coin_address) => {
                self.coin_address = coin_address;
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
        let address_block: Element<'a, Message> = match if self.coin_address {
            coin_address
        } else {
            space_address
        } {
            Some(address) => column![
                container(
                    row![
                        text(address.as_str()).font(Font::MONOSPACE),
                        button_icon(Icon::Copy)
                            .style(button::secondary)
                            .on_press(Message::CopyPress(address.as_str().to_string())),
                    ]
                    .align_y(Center)
                    .spacing(5),
                )
                .padding(Padding {
                    top: 5.0,
                    right: 5.0,
                    bottom: 5.0,
                    left: 15.0
                })
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    container::Style::default().border(Border {
                        color: palette.secondary.base.text,
                        width: 1.0,
                        radius: 0.into(),
                    })
                }),
                center(qr_code(address.as_qr_code()).cell_size(7))
                    .style(|theme: &Theme| {
                        let palette = theme.palette();
                        container::Style::default()
                            .border(Border {
                                color: palette.text,
                                width: 2.0,
                                radius: 0.into(),
                            })
                            .background(palette.background)
                    })
                    .width(300)
                    .height(300)
            ]
            .width(Fill)
            .align_x(Center)
            .spacing(10)
            .into(),
            None => center(text("Loading")).into(),
        };

        center(
            column![
                address_block,
                container(
                    toggler(self.coin_address)
                        .size(25)
                        .label("Coins only address")
                        .on_toggle(Message::AddressKindToggle),
                )
                .align_x(Center)
                .width(Fill),
            ]
            .spacing(20),
        )
        .into()
    }
}
