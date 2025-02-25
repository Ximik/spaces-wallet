use crate::{
    helpers::{format_amount, format_amount_number, height_to_past_est},
    types::*,
    widget::{
        form::Form,
        icon::{text_icon, Icon},
        text::{error_block, text_big, text_monospace, text_small},
    },
};
use iced::{
    widget::{
        button, center, column, container, horizontal_space, row, scrollable, text, Column, Row,
        Space,
    },
    Border, Center, Element, Fill, FillPortion, Theme,
};

#[derive(Debug, Clone)]
pub struct State {
    txid: Option<Txid>,
    transactions_limit: usize,
    fee_rate: String,
    error: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            txid: None,
            transactions_limit: 10,
            fee_rate: String::new(),
            error: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    TxidPress { txid: Txid },
    SpacePress { slabel: SLabel },
    TxsListScrolled { percentage: f32, count: usize },
    FeeRateInput(String),
    BumpFeeSubmit,
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    ShowSpace { slabel: SLabel },
    GetTransactions,
    BumpFee { txid: Txid, fee_rate: FeeRate },
}

impl State {
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error)
    }

    pub fn reset_inputs(&mut self) {
        self.fee_rate = String::new();
    }

    pub fn reset(&mut self) {
        self.txid = None;
        self.reset_inputs();
    }

    pub fn get_transactions_limit(&self) -> usize {
        self.transactions_limit
    }

    pub fn update(&mut self, message: Message) -> Action {
        self.error = None;
        match message {
            Message::TxidPress { txid } => {
                self.txid = Some(txid);
                Action::None
            }
            Message::SpacePress { slabel } => Action::ShowSpace { slabel },
            Message::TxsListScrolled { percentage, count } => {
                if percentage > 0.8 && count >= self.transactions_limit {
                    self.transactions_limit += (percentage * count as f32) as usize;
                    Action::GetTransactions
                } else {
                    Action::None
                }
            }
            Message::FeeRateInput(fee_rate) => {
                if is_fee_rate_input(&fee_rate) {
                    self.fee_rate = fee_rate
                }
                Action::None
            }
            Message::BumpFeeSubmit => Action::BumpFee {
                txid: self.txid.as_ref().unwrap().clone(),
                fee_rate: fee_rate_from_str(&self.fee_rate).unwrap().unwrap(),
            },
        }
    }

    pub fn view<'a>(
        &'a self,
        tip_height: u32,
        balance: Amount,
        transactions: &'a Vec<TxInfo>,
    ) -> Element<'a, Message> {
        column![
            column![text_big("Balance"), text_monospace(format_amount(balance)),]
                .spacing(10)
                .width(Fill)
                .align_x(Center),
            match self.txid {
                Some(txid) => column![
                    text_big("Bump fee"),
                    text_monospace(format!("TXID: {}", txid)),
                    error_block(self.error.as_ref()),
                    Form::new(
                        "Bump fee",
                        fee_rate_from_str(&self.fee_rate)
                            .flatten()
                            .map(|_| Message::BumpFeeSubmit),
                    )
                    .add_labeled_input(
                        "Fee rate",
                        "sat/vB",
                        &self.fee_rate,
                        Message::FeeRateInput,
                    ),
                ]
                .spacing(10),
                None => column![text_big("Transactions"), {
                    let element: Element<'a, Message> = if transactions.is_empty() {
                        center(text("No transactions yet")).into()
                    } else {
                        scrollable(
                            Column::from_iter(transactions.into_iter().map(|transaction| {
                                let block_height = transaction.block_height;
                                let txid = transaction.txid;
                                let txid_string = txid.to_string();
                                let event = transaction
                                    .events
                                    .iter()
                                    .find(|event| event.space.is_some());
                                let bumped = transaction
                                    .events
                                    .iter()
                                    .any(|event| event.kind == TxEventKind::FeeBump);

                                let tx_row_without_event =
                                    || -> Row<'a, Message> {
                                        let diff = transaction.received.to_sat() as i64
                                            - transaction.sent.to_sat() as i64;
                                        row![
                                            horizontal_space(),
                                            if diff >= 0 {
                                                text_monospace(format!(
                                                    "+{}",
                                                    format_amount_number(diff as u64)
                                                ))
                                                .style(move |theme: &Theme| text::Style {
                                                    color: Some(
                                                        theme
                                                            .extended_palette()
                                                            .success
                                                            .strong
                                                            .color,
                                                    ),
                                                })
                                            } else {
                                                text_monospace(format!(
                                                    "-{}",
                                                    format_amount_number(-diff as u64)
                                                ))
                                                .style(move |theme: &Theme| text::Style {
                                                    color: Some(
                                                        theme
                                                            .extended_palette()
                                                            .danger
                                                            .strong
                                                            .color,
                                                    ),
                                                })
                                            }
                                        ]
                                    };

                                let tx_row_with_event =
                                    |action: &'static str,
                                     space: &'a str,
                                     amount: Option<Amount>|
                                     -> Row<'a, Message> {
                                        let slabel = SLabel::from_str(space).unwrap();
                                        row![
                                            text(action),
                                            Space::with_width(5),
                                            button(text_monospace(space))
                                                .on_press(Message::SpacePress { slabel })
                                                .style(button::text)
                                                .padding(0),
                                            Space::with_width(Fill),
                                        ]
                                        .push_maybe(
                                            amount.map(|amount| {
                                                text_monospace(format_amount(amount))
                                            }),
                                        )
                                        .align_y(Center)
                                    };

                                container(
                                    column![
                                        row![
                                            container(
                                                button(
                                                    Row::new()
                                                        .push_maybe(if bumped {
                                                            Some(text_icon(Icon::ArrowBigUpLines))
                                                        } else {
                                                            None
                                                        })
                                                        .push(text_monospace(format!(
                                                            "{} .. {}",
                                                            &txid_string[..8],
                                                            &txid_string[54..]
                                                        ))),
                                                )
                                                .style(button::text)
                                                .padding(0)
                                                .on_press(Message::TxidPress { txid })
                                            )
                                            .width(FillPortion(3)),
                                            match event {
                                                Some(TxEvent {
                                                    kind: TxEventKind::Commit,
                                                    space,
                                                    ..
                                                }) => tx_row_with_event(
                                                    "Commit",
                                                    space.as_ref().unwrap(),
                                                    None,
                                                ),
                                                Some(TxEvent {
                                                    kind: TxEventKind::Open,
                                                    space,
                                                    details,
                                                    ..
                                                }) => tx_row_with_event(
                                                    "Open",
                                                    space.as_ref().unwrap(),
                                                    Some(
                                                        OpenEventDetails::deserialize(
                                                            details.as_ref().unwrap(),
                                                        )
                                                        .unwrap()
                                                        .initial_bid,
                                                    ),
                                                ),
                                                Some(TxEvent {
                                                    kind: TxEventKind::Bid,
                                                    space,
                                                    details,
                                                    ..
                                                }) => tx_row_with_event(
                                                    "Bid",
                                                    space.as_ref().unwrap(),
                                                    Some(
                                                        BidEventDetails::deserialize(
                                                            details.as_ref().unwrap(),
                                                        )
                                                        .unwrap()
                                                        .current_bid,
                                                    ),
                                                ),
                                                Some(TxEvent {
                                                    kind: TxEventKind::Transfer,
                                                    space,
                                                    ..
                                                }) => tx_row_with_event(
                                                    "Transfer",
                                                    space.as_ref().unwrap(),
                                                    None
                                                ),
                                                Some(TxEvent {
                                                    kind: TxEventKind::Renew,
                                                    space,
                                                    ..
                                                }) => tx_row_with_event(
                                                    "Renew",
                                                    space.as_ref().unwrap(),
                                                    None
                                                ),
                                                _ => tx_row_without_event(),
                                            }
                                            .width(FillPortion(4)),
                                        ],
                                        match block_height {
                                            Some(block_height) => text_small(height_to_past_est(
                                                block_height,
                                                tip_height
                                            ),),
                                            None => text_small("Unconfirmed"),
                                        }
                                    ]
                                    .spacing(5)
                                    .padding(10),
                                )
                                .style(move |theme: &Theme| {
                                    let palette = theme.extended_palette();
                                    container::Style {
                                        border: Border {
                                            width: 1.0,
                                            color: palette.background.strong.color.into(),
                                            ..Default::default()
                                        },
                                        background: block_height
                                            .map(|_| palette.background.weak.color.into()),
                                        ..Default::default()
                                    }
                                })
                                .padding(10)
                                .width(Fill)
                                .into()
                            }))
                            .push(Space::with_height(5))
                            .spacing(5),
                        )
                        .on_scroll(|viewport| Message::TxsListScrolled {
                            percentage: viewport.relative_offset().y,
                            count: transactions.len(),
                        })
                        .spacing(20)
                        .height(Fill)
                        .into()
                    };
                    element
                }]
                .spacing(10)
                .height(Fill)
                .width(Fill),
            }
        ]
        .spacing(20)
        .height(Fill)
        .width(Fill)
        .into()
    }
}
