use crate::{
    types::*,
    widget::icon::{button_icon, text_icon, Icon},
};
use iced::{
    widget::{button, center, column, container, horizontal_space, row, scrollable, text, Column},
    Border, Center, Element, Fill, Theme,
};

#[derive(Debug, Clone)]
pub enum Message {
    TxidCopyPress { txid: String },
    SpaceClicked { slabel: SLabel },
}

pub fn view<'a>(balance: Amount, transactions: &'a Vec<TxInfo>) -> Element<'a, Message> {
    let transactions: Element<'a, Message> = if transactions.is_empty() {
        center(text("No transactions yet")).into()
    } else {
        scrollable(container(
            Column::with_children(transactions.into_iter().map(|transaction| {
                let txid = transaction.txid.to_string();
                container(column![
                    row![
                        text_icon(if transaction.sent >= transaction.received {
                            Icon::ArrowDownFromArc
                        } else {
                            Icon::ArrowDownToArc
                        }),
                        text(txid.clone()),
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
        ))
        .spacing(2)
        .height(Fill)
        .into()
    };

    column![
        text("Balance (SAT)"),
        text(balance.to_sat()),
        text("Transactions"),
        transactions
    ]
    .spacing(5)
    .height(Fill)
    .width(Fill)
    .into()
}
