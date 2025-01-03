use crate::{
    helpers::height_to_est,
    types::*,
    widget::{
        block::error,
        form::Form,
        icon::{text_input_icon, Icon},
    },
};
use iced::{
    font,
    widget::{
        button, column, container, horizontal_rule, row, scrollable, text, text_input, Column,
        Space,
    },
    Center, Element, Fill, FillPortion, Right,
};

#[derive(Debug, Clone, Default)]
pub struct State {
    space: String,
    amount: String,
    recipient: String,
    fee_rate: String,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SLabelSet(SLabel),
    SpaceInput(String),
    AmountInput(String),
    RecipientInput(String),
    FeeRateInput(String),
    OpenSubmit,
    BidSubmit,
    ClaimSubmit,
    TransferSubmit,
}

#[derive(Debug, Clone)]
pub enum Task {
    None,
    GetSpaceInfo {
        slabel: SLabel,
    },
    OpenSpace {
        slabel: SLabel,
        amount: Amount,
        fee_rate: Option<FeeRate>,
    },
    BidSpace {
        slabel: SLabel,
        amount: Amount,
        fee_rate: Option<FeeRate>,
    },
    ClaimSpace {
        slabel: SLabel,
        fee_rate: Option<FeeRate>,
    },
    TransferSpace {
        slabel: SLabel,
        recipient: String,
        fee_rate: Option<FeeRate>,
    },
}

impl State {
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error)
    }

    pub fn reset_inputs(&mut self) {
        self.amount = Default::default();
        self.recipient = Default::default();
        self.fee_rate = Default::default();
    }

    pub fn reset_space(&mut self) {
        self.reset_inputs();
        self.space = Default::default();
    }

    pub fn set_slabel(&mut self, slabel: &SLabel) {
        self.reset_inputs();
        self.space = slabel.as_str_unprefixed().unwrap().to_string()
    }

    pub fn get_slabel(&self) -> Option<SLabel> {
        slabel_from_str(&self.space)
    }

    pub fn update(&mut self, message: Message) -> Task {
        self.error = None;
        match message {
            Message::SLabelSet(slabel) => {
                self.space = slabel.to_string_unprefixed().unwrap();
                Task::GetSpaceInfo { slabel }
            }
            Message::SpaceInput(space) => {
                if is_slabel_input(&space) {
                    self.space = space;
                    if let Some(slabel) = self.get_slabel() {
                        Task::GetSpaceInfo { slabel }
                    } else {
                        Task::None
                    }
                } else {
                    Task::None
                }
            }
            Message::AmountInput(amount) => {
                if is_amount_input(&amount) {
                    self.amount = amount
                }
                Task::None
            }
            Message::RecipientInput(recipient) => {
                if is_recipient_input(&recipient) {
                    self.recipient = recipient
                }
                Task::None
            }
            Message::FeeRateInput(fee_rate) => {
                if is_fee_rate_input(&fee_rate) {
                    self.fee_rate = fee_rate
                }
                Task::None
            }
            Message::OpenSubmit => Task::OpenSpace {
                slabel: self.get_slabel().unwrap(),
                amount: amount_from_str(&self.amount).unwrap(),
                fee_rate: fee_rate_from_str(&self.fee_rate).unwrap(),
            },
            Message::BidSubmit => Task::BidSpace {
                slabel: self.get_slabel().unwrap(),
                amount: amount_from_str(&self.amount).unwrap(),
                fee_rate: fee_rate_from_str(&self.fee_rate).unwrap(),
            },
            Message::ClaimSubmit => Task::ClaimSpace {
                slabel: self.get_slabel().unwrap(),
                fee_rate: fee_rate_from_str(&self.fee_rate).unwrap(),
            },
            Message::TransferSubmit => Task::TransferSpace {
                slabel: self.get_slabel().unwrap(),
                recipient: recipient_from_str(&self.recipient).unwrap(),
                fee_rate: fee_rate_from_str(&self.fee_rate).unwrap(),
            },
        }
    }

    fn open_form<'a>(&'a self) -> Element<'a, Message> {
        Form::new(
            "Open",
            (amount_from_str(&self.amount).is_some()
                && fee_rate_from_str(&self.fee_rate).is_some())
            .then_some(Message::OpenSubmit),
        )
        .add_labeled_input("Amount", "sat", &self.amount, Message::AmountInput)
        .add_labeled_input(
            "Fee rate",
            "sat/vB (auto if empty)",
            &self.fee_rate,
            Message::FeeRateInput,
        )
        .into()
    }

    fn bid_form<'a>(&'a self, current_bid: Amount, is_highest: bool) -> Element<'a, Message> {
        column![
            text(format!("Current bid: {} satoshi", current_bid.to_sat())),
            text(format!(
                "Highest bid is {}",
                if is_highest { "yours" } else { "not yours" }
            )),
            Form::new(
                "Bid",
                (amount_from_str(&self.amount).map_or(false, |amount| amount > current_bid)
                    && fee_rate_from_str(&self.fee_rate).is_some())
                .then_some(Message::BidSubmit),
            )
            .add_labeled_input("Amount", "sat", &self.amount, Message::AmountInput)
            .add_labeled_input(
                "Fee rate",
                "sat/vB (auto if empty)",
                &self.fee_rate,
                Message::FeeRateInput,
            )
        ]
        .spacing(5)
        .into()
    }

    fn claim_form<'a>(&'a self) -> Element<'a, Message> {
        Form::new(
            "Claim",
            fee_rate_from_str(&self.fee_rate).map(|_| Message::ClaimSubmit),
        )
        .add_labeled_input(
            "Fee rate",
            "sat/vB (auto if empty)",
            &self.fee_rate,
            Message::FeeRateInput,
        )
        .into()
    }

    fn transfer_form<'a>(&'a self) -> Element<'a, Message> {
        Form::new(
            "Send",
            (recipient_from_str(&self.recipient).is_some()
                && fee_rate_from_str(&self.fee_rate).is_some())
            .then_some(Message::TransferSubmit),
        )
        .add_labeled_input(
            "To",
            "bitcoin address or @space",
            &self.recipient,
            Message::RecipientInput,
        )
        .add_labeled_input(
            "Fee rate",
            "sat/vB (auto if empty)",
            &self.fee_rate,
            Message::FeeRateInput,
        )
        .into()
    }

    fn open_view<'a>(&'a self) -> Element<'a, Message> {
        row![
            timeline::view(0, "Make an open to propose the space for auction"),
            self.open_form(),
        ]
        .into()
    }

    fn bid_view<'a>(
        &'a self,
        tip_height: u32,
        claim_height: Option<u32>,
        current_bid: Amount,
        is_owned: bool,
    ) -> Element<'a, Message> {
        row![
            timeline::view(
                if claim_height.is_none() { 1 } else { 2 },
                claim_height.map_or(
                    "Make a bid to improve the chance of moving the space to auction".to_string(),
                    |height| format!("Auction ends {}", height_to_est(height, tip_height))
                )
            ),
            column![
                error(self.error.as_ref()),
                self.bid_form(current_bid, is_owned),
            ]
        ]
        .into()
    }

    fn claim_view<'a>(&'a self, current_bid: Amount, is_owned: bool) -> Element<'a, Message> {
        row![
            timeline::view(
                3,
                if is_owned {
                    "You can claim the space"
                } else {
                    "The auction is ended, but you still can outbid"
                }
            ),
            if is_owned {
                column![error(self.error.as_ref()), self.claim_form()]
            } else {
                column![
                    text(format!("Current bid: {} satoshi", current_bid.to_sat())),
                    error(self.error.as_ref()),
                    self.bid_form(current_bid, is_owned),
                ]
                .spacing(5)
            }
        ]
        .into()
    }

    fn registered_view<'a>(
        &'a self,
        tip_height: u32,
        expire_height: u32,
        is_owned: bool,
    ) -> Element<'a, Message> {
        row![
            timeline::view(
                4,
                format!(
                    "The space registration expires {}",
                    height_to_est(expire_height, tip_height)
                )
            ),
            if is_owned {
                column![error(self.error.as_ref()), self.transfer_form()]
            } else {
                column![Space::new(Fill, Fill)]
            }
        ]
        .into()
    }

    pub fn view<'a>(
        &'a self,
        tip_height: u32,
        spaces: &'a SpacesState,
        wallet_spaces: &'a Vec<SLabel>,
    ) -> Element<'a, Message> {
        let main: Element<'a, Message> = if self.space.is_empty() {
            let mut spaces = wallet_spaces
                .into_iter()
                .map(|slabel| (slabel, spaces.get(slabel)))
                .collect::<Vec<_>>();
            spaces.sort_unstable_by_key(|s| s.0.as_str_unprefixed().unwrap());

            let transfer_spaces = spaces
                .iter()
                .filter_map(|(slabel, covenant)| match covenant {
                    Some(Some(Covenant::Transfer { expire_height, .. })) => {
                        Some((*slabel, expire_height))
                    }
                    _ => None,
                });
            let bid_spaces = spaces
                .iter()
                .filter_map(|(slabel, covenant)| match covenant {
                    Some(Some(Covenant::Bid {
                        total_burned,
                        claim_height,
                        ..
                    })) => Some((*slabel, total_burned, claim_height)),
                    _ => None,
                });

            scrollable(
                column![
                    column![
                        text("Registered")
                            .font(font::Font {
                                weight: font::Weight::Bold,
                                ..font::Font::DEFAULT
                            })
                            .size(18),
                        Space::with_height(5),
                        horizontal_rule(1),
                        row![
                            text("Space")
                                .font(font::Font {
                                    weight: font::Weight::Bold,
                                    ..font::Font::DEFAULT
                                })
                                .width(FillPortion(1)),
                            text("Expires")
                                .font(font::Font {
                                    weight: font::Weight::Bold,
                                    ..font::Font::DEFAULT
                                })
                                .width(FillPortion(2)),
                        ]
                        .padding([10, 0]),
                        horizontal_rule(1),
                        Space::with_height(5),
                        Column::with_children(transfer_spaces.map(|(slabel, expire_height)| {
                            row![
                                text(slabel.to_string()).width(FillPortion(1)),
                                text(height_to_est(*expire_height, tip_height))
                                    .width(FillPortion(1)),
                                container(
                                    button("View")
                                        .style(button::secondary)
                                        .on_press(Message::SLabelSet(slabel.clone()))
                                )
                                .width(FillPortion(1))
                                .align_x(Right),
                            ]
                            .align_y(Center)
                            .into()
                        }))
                        .spacing(3),
                    ],
                    column![
                        text("Bid")
                            .font(font::Font {
                                weight: font::Weight::Bold,
                                ..font::Font::DEFAULT
                            })
                            .size(18),
                        Space::with_height(5),
                        horizontal_rule(1),
                        row![
                            text("Space")
                                .font(font::Font {
                                    weight: font::Weight::Bold,
                                    ..font::Font::DEFAULT
                                })
                                .width(FillPortion(1)),
                            text("Highest Bid")
                                .font(font::Font {
                                    weight: font::Weight::Bold,
                                    ..font::Font::DEFAULT
                                })
                                .width(FillPortion(1)),
                            text("Claim")
                                .font(font::Font {
                                    weight: font::Weight::Bold,
                                    ..font::Font::DEFAULT
                                })
                                .width(FillPortion(2)),
                        ]
                        .padding([10, 0]),
                        horizontal_rule(1),
                        Space::with_height(5),
                        Column::with_children(bid_spaces.map(
                            |(slabel, total_burned, claim_height)| {
                                row![
                                    text(slabel.to_string()).width(FillPortion(1)),
                                    text(
                                        total_burned
                                            .to_string_with_denomination(Denomination::Satoshi)
                                    )
                                    .width(FillPortion(1)),
                                    text(
                                        claim_height
                                            .map(|h| height_to_est(h, tip_height))
                                            .unwrap_or("pre-auction".to_string())
                                    )
                                    .width(FillPortion(1)),
                                    container(
                                        button("View")
                                            .style(button::secondary)
                                            .on_press(Message::SLabelSet(slabel.clone()))
                                    )
                                    .width(FillPortion(1))
                                    .align_x(Right),
                                ]
                                .align_y(Center)
                                .into()
                            }
                        ))
                        .spacing(3),
                    ]
                ]
                .spacing(30),
            )
            .spacing(10)
            .into()
        } else if let Some(slabel) = self.get_slabel() {
            let is_owned = wallet_spaces.contains(&slabel);
            let covenant = spaces.get(&slabel);
            match covenant {
                None => text("loading").into(),
                Some(None) => self.open_view(),
                Some(Some(Covenant::Bid {
                    claim_height,
                    total_burned,
                    ..
                })) => {
                    if claim_height.map_or(false, |height| height <= tip_height) {
                        self.claim_view(*total_burned, is_owned)
                    } else {
                        self.bid_view(tip_height, *claim_height, *total_burned, is_owned)
                    }
                }
                Some(Some(Covenant::Transfer { expire_height, .. })) => {
                    self.registered_view(tip_height, *expire_height, is_owned)
                }
                Some(Some(Covenant::Reserved)) => unreachable!("reserved"),
            }
        } else {
            text("Invalid space name").into()
        };

        column![
            container(
                text_input("space", &self.space)
                    .icon(text_input_icon(
                        Icon::At,
                        None,
                        10.0,
                        text_input::Side::Left
                    ))
                    .on_input(Message::SpaceInput)
                    .font(font::Font::MONOSPACE)
                    .padding(10)
            ),
            main,
        ]
        .spacing(20)
        .into()
    }
}

mod timeline {
    use crate::widget::rect::*;
    use iced::{
        widget::{text, Column, Row},
        Border, Center, Element, Fill, Theme,
    };

    const CIRCLE_RADIUS: f32 = 20.0;
    const LINE_WIDTH: f32 = 3.0;
    const LINE_HEIGHT: f32 = 50.0;
    const ROW_SPACING: f32 = 10.0;

    fn circle<'a>(filled: bool, border: bool, inner: bool) -> Rect<'a> {
        Rect::new(CIRCLE_RADIUS * 2.0, CIRCLE_RADIUS * 2.0).style(move |theme: &Theme| {
            let palette = theme.palette();
            Style {
                border: Border {
                    color: if border {
                        palette.primary
                    } else {
                        palette.text
                    },
                    width: LINE_WIDTH,
                    radius: CIRCLE_RADIUS.into(),
                },
                background: if filled {
                    Some(palette.primary.into())
                } else {
                    None
                },
                inner: if inner {
                    Some(Inner {
                        border: Border {
                            radius: CIRCLE_RADIUS.into(),
                            ..Border::default()
                        },
                        background: Some(palette.primary.into()),
                        padding: (CIRCLE_RADIUS / 2.0).into(),
                    })
                } else {
                    None
                },
            }
        })
    }

    fn line<'a>(filled: bool) -> Rect<'a> {
        Rect::new(CIRCLE_RADIUS * 2.0, LINE_HEIGHT).style(move |theme: &Theme| {
            let palette = theme.palette();
            Style {
                inner: Some(Inner {
                    background: Some(
                        if filled {
                            palette.primary
                        } else {
                            palette.text
                        }
                        .into(),
                    ),
                    padding: [0.0, CIRCLE_RADIUS - LINE_WIDTH / 2.0].into(),
                    ..Inner::default()
                }),
                ..Style::default()
            }
        })
    }

    fn space<'a>() -> Rect<'a> {
        Rect::new(CIRCLE_RADIUS * 2.0, LINE_HEIGHT)
    }

    pub fn view<'a, Message: 'a>(
        state: u8,
        label: impl text::IntoFragment<'a> + Clone,
    ) -> Element<'a, Message> {
        const LABELS: [&str; 4] = ["Open", "Pre-auction", "Auction", "Claim"];
        if state > LABELS.len() as u8 {
            panic!("state is out of range");
        }
        Column::from_iter((0..(LABELS.len() as u8) * 2).map(|i| {
            let c = i % 2 == 0;
            let n = i / 2;
            let o = n.cmp(&state);
            let row = Row::new()
                .push(if c {
                    circle(o.is_lt(), o.is_le(), o.is_eq())
                } else if n == LABELS.len() as u8 - 1 {
                    space()
                } else {
                    line(o.is_lt())
                })
                .push_maybe(if c {
                    Some(text(LABELS[n as usize]))
                } else if (state == LABELS.len() as u8 && state - n == 1) || o.is_eq() {
                    Some(text(label.clone()))
                } else {
                    None
                })
                .spacing(ROW_SPACING);
            if c { row.align_y(Center) } else { row }.into()
        }))
        .width(Fill)
        .into()
    }
}
