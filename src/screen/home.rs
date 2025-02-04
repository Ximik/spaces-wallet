use std::str::FromStr;

use crate::{
    types::*,
    widget::{
        icon::{button_icon, Icon},
        text::{text_header, text_monospace},
    },
};
use iced::{
    widget::{button, center, column, container, horizontal_space, row, scrollable, text, Column},
    Border, Center, Element, Fill, Theme,
};
use wallet::bdk_wallet::serde::Deserialize;

#[derive(Debug, Clone)]
pub enum Message {
    TxidCopyPress { txid: String },
    SpaceViewPress { slabel: SLabel },
    TxsListScrolled { percentage: f32 },
}

pub fn view<'a>(balance: Amount, transactions: &'a Vec<TxInfo>) -> Element<'a, Message> {
    let transactions: Element<'a, Message> = if transactions.is_empty() {
        center(text("No transactions yet")).into()
    } else {
        let button_style = |theme: &Theme, status: button::Status| button::Style {
            background: match status {
                button::Status::Hovered => {
                    Some(theme.extended_palette().secondary.strong.color.into())
                }
                _ => None,
            },
            ..Default::default()
        };
        scrollable(
            Column::with_children(transactions.into_iter().map(|transaction| {
                let txid = transaction.txid.to_string();
                let diff = transaction.received.to_sat() as i64 - transaction.sent.to_sat() as i64;
                container(
                    Column::new()
                        .push(
                            row![
                                text_monospace(format!("{} .. {}", &txid[..8], &txid[54..])),
                                button_icon(Icon::Copy)
                                    .style(button_style)
                                    .on_press(Message::TxidCopyPress { txid }),
                                horizontal_space(),
                                text(format!("{:+} satoshi", diff)).style(move |theme: &Theme| {
                                    let palette = theme.extended_palette();
                                    text::Style {
                                        color: Some(if diff > 0 {
                                            palette.success.strong.color
                                        } else {
                                            palette.danger.strong.color
                                        }),
                                    }
                                }),
                            ]
                            .spacing(10)
                            .align_y(Center),
                        )
                        .extend(transaction.events.iter().filter_map(move |event| {
                            let event_row =
                                |label: &'a str, space: &'a String, amount: Option<Amount>| {
                                    let slabel = SLabel::from_str(space).unwrap();
                                    row![
                                        text(label),
                                        text_monospace(space),
                                        button_icon(Icon::Eye)
                                            .style(button_style)
                                            .on_press(Message::SpaceViewPress { slabel }),
                                        horizontal_space(),
                                    ]
                                    .push_maybe(
                                        amount.map(|amount| {
                                            text(format!("{} satoshi", amount.to_sat()))
                                        }),
                                    )
                                    .spacing(10)
                                    .align_y(Center)
                                };
                            match event.kind {
                                TxEventKind::Open => Some(
                                    event_row(
                                        "Open",
                                        event.space.as_ref().unwrap(),
                                        Some(
                                            OpenEventDetails::deserialize(
                                                event.details.as_ref().unwrap(),
                                            )
                                            .unwrap()
                                            .initial_bid,
                                        ),
                                    )
                                    .into(),
                                ),
                                TxEventKind::Bid => Some(
                                    event_row(
                                        "Bid",
                                        event.space.as_ref().unwrap(),
                                        Some(
                                            BidEventDetails::deserialize(
                                                event.details.as_ref().unwrap(),
                                            )
                                            .unwrap()
                                            .current_bid,
                                        ),
                                    )
                                    .into(),
                                ),
                                TxEventKind::Transfer => Some(
                                    event_row("Transfer", event.space.as_ref().unwrap(), None)
                                        .into(),
                                ),
                                TxEventKind::Renew => Some(
                                    event_row("Renew", event.space.as_ref().unwrap(), None).into(),
                                ),
                                _ => None,
                            }
                        }))
                        .spacing(5),
                )
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    container::Style {
                        border: Border {
                            width: 1.0,
                            color: palette.background.strong.color.into(),
                            ..Default::default()
                        },
                        background: if transaction.confirmed {
                            Some(palette.background.weak.color.into())
                        } else {
                            None
                        },
                        ..Default::default()
                    }
                })
                .padding(10)
                .width(Fill)
                .into()
            }))
            .spacing(5),
        )
        .on_scroll(|viewport| Message::TxsListScrolled {
            percentage: viewport.relative_offset().y,
        })
        .spacing(2)
        .height(Fill)
        .into()
    };

    column![
        text_header("Balance"),
        text(format!("{} satoshi", balance.to_sat())),
        text_header("Transactions"),
        transactions
    ]
    .spacing(10)
    .height(Fill)
    .width(Fill)
    .into()
}
