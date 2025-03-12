use iced::Element;
use iced::widget::column;

use crate::{
    types::*,
    widget::{
        form::Form,
        tabs::TabsRow,
        text::{error_block, text_big},
    },
};

#[derive(Debug, Clone)]
pub struct State {
    asset_kind: AddressKind,
    recipient: String,
    amount: String,
    slabel: Option<SLabel>,
    fee_rate: String,
    error: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            asset_kind: AddressKind::Coin,
            recipient: Default::default(),
            amount: Default::default(),
            slabel: Default::default(),
            fee_rate: Default::default(),
            error: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    TabPress(AddressKind),
    RecipientInput(String),
    AmountInput(String),
    SLabelSelect(SLabel),
    FeeRateInput(String),
    SendCoinsSubmit,
    SendSpaceSubmit,
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    SendCoins {
        recipient: String,
        amount: Amount,
        fee_rate: Option<FeeRate>,
    },
    SendSpace {
        recipient: String,
        slabel: SLabel,
        fee_rate: Option<FeeRate>,
    },
}

impl State {
    pub fn set_error(&mut self, message: String) {
        self.error = Some(message);
    }

    pub fn reset_inputs(&mut self) {
        self.recipient = Default::default();
        self.amount = Default::default();
        self.slabel = Default::default();
        self.fee_rate = Default::default();
    }

    pub fn update(&mut self, message: Message) -> Action {
        self.error = None;
        match message {
            Message::TabPress(asset_kind) => {
                self.asset_kind = asset_kind;
                self.amount = Default::default();
                self.slabel = Default::default();
                Action::None
            }
            Message::RecipientInput(recipient) => {
                if is_recipient_input(&recipient) {
                    self.recipient = recipient;
                }
                Action::None
            }
            Message::AmountInput(amount) => {
                if is_amount_input(&amount) {
                    self.amount = amount
                }
                Action::None
            }
            Message::SLabelSelect(slabel) => {
                self.slabel = Some(slabel);
                Action::None
            }
            Message::FeeRateInput(fee_rate) => {
                if is_fee_rate_input(&fee_rate) {
                    self.fee_rate = fee_rate
                }
                Action::None
            }
            Message::SendCoinsSubmit => {
                self.error = None;
                Action::SendCoins {
                    recipient: recipient_from_str(&self.recipient).unwrap(),
                    amount: amount_from_str(&self.amount).unwrap(),
                    fee_rate: fee_rate_from_str(&self.fee_rate).unwrap(),
                }
            }
            Message::SendSpaceSubmit => {
                self.error = None;
                Action::SendSpace {
                    slabel: self.slabel.clone().unwrap(),
                    recipient: recipient_from_str(&self.recipient).unwrap(),
                    fee_rate: fee_rate_from_str(&self.fee_rate).unwrap(),
                }
            }
        }
    }

    pub fn view<'a>(&'a self, owned_spaces: &'a Vec<SLabel>) -> Element<'a, Message> {
        column![
            TabsRow::new()
                .add_tab(
                    "Coins",
                    matches!(self.asset_kind, AddressKind::Coin),
                    Message::TabPress(AddressKind::Coin)
                )
                .add_tab(
                    "Spaces",
                    matches!(self.asset_kind, AddressKind::Space),
                    Message::TabPress(AddressKind::Space)
                ),
            text_big(match self.asset_kind {
                AddressKind::Coin => "Send coins",
                AddressKind::Space => "Send space",
            }),
            error_block(self.error.as_ref()),
            match self.asset_kind {
                AddressKind::Coin => Form::new(
                    "Send",
                    (recipient_from_str(&self.recipient).is_some()
                        && amount_from_str(&self.amount).is_some()
                        && fee_rate_from_str(&self.fee_rate).is_some())
                    .then_some(Message::SendCoinsSubmit),
                )
                .add_input("Amount", "sat", &self.amount, Message::AmountInput)
                .add_input(
                    "To",
                    "bitcoin address or @space",
                    &self.recipient,
                    Message::RecipientInput,
                )
                .add_input(
                    "Fee rate",
                    "sat/vB (auto if empty)",
                    &self.fee_rate,
                    Message::FeeRateInput,
                ),
                AddressKind::Space => Form::new(
                    "Send",
                    (recipient_from_str(&self.recipient).is_some()
                        && self.slabel.is_some()
                        && fee_rate_from_str(&self.fee_rate).is_some())
                    .then_some(Message::SendSpaceSubmit),
                )
                .add_pick_list(
                    "Space",
                    owned_spaces.as_slice(),
                    self.slabel.as_ref(),
                    Message::SLabelSelect
                )
                .add_input(
                    "To",
                    "bitcoin address or @space",
                    &self.recipient,
                    Message::RecipientInput,
                )
                .add_input(
                    "Fee rate",
                    "sat/vB (auto if empty)",
                    &self.fee_rate,
                    Message::FeeRateInput,
                ),
            }
        ]
        .spacing(10)
        .into()
    }
}
