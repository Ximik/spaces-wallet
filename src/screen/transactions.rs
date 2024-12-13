use iced::widget::{
    button, center, column, container, horizontal_space, row, scrollable, text, Column,
};
use iced::Alignment::Center;
use iced::{Border, Element, Fill, Font, Theme};

use crate::store::TxInfo;
use crate::widget::icon::{button_icon, text_icon, Icon};

#[derive(Debug, Clone)]
pub enum Message {
    TxidCopyPress { txid: String },
}

pub fn view<'a>(transactions: &'a [TxInfo]) -> Element<'a, Message> {
    if transactions.is_empty() {
        center(text("No transactions yet")).into()
    } else {
        scrollable(
            container(
                Column::with_children(transactions.into_iter().map(|transaction| {
                    let txid = transaction.txid.to_string();
                    container(column![
                        row![
                            text_icon(if transaction.sent >= transaction.received {
                                Icon::ArrowDownFromArc
                            } else {
                                Icon::ArrowDownToArc
                            }),
                            text(txid.clone()).font(Font::MONOSPACE),
                            horizontal_space(),
                            button_icon(Icon::Copy)
                                .style(button::secondary)
                                .on_press(Message::TxidCopyPress { txid })
                        ]
                        .spacing(5)
                        .align_y(Center),
                        row![
                            text(format!("Sent: {} SAT", transaction.sent.to_sat())),
                            text(format!("Received: {} SAT", transaction.received.to_sat())),
                        ]
                        .push_maybe(
                            transaction
                                .fee
                                .map(|fee| { text(format!("Fee: {} SAT", fee.to_sat())) })
                        )
                        .spacing(5),
                    ])
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        container::Style::default()
                            .border(Border {
                                color: palette.secondary.base.text,
                                width: 1.0,
                                radius: 5.0.into(),
                            })
                            .background(if transaction.confirmed {
                                palette.background.strong.color
                            } else {
                                palette.background.weak.color
                            })
                    })
                    .padding(10)
                    .width(Fill)
                    .into()
                }))
                .spacing(5),
            )
            .padding(10),
        )
        .spacing(2)
        .height(Fill)
        .into()
    }
}
