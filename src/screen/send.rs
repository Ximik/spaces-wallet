use iced::widget::column;
use iced::Element;

use crate::{
    types::*,
    widget::{
        form::Form,
        text::{error_block, text_big},
    },
};

#[derive(Debug, Clone, Default)]
pub struct State {
    recipient: String,
    amount: String,
    fee_rate: String,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    RecipientInput(String),
    AmountInput(String),
    FeeRateInput(String),
    SendSubmit,
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    SendCoins {
        recipient: String,
        amount: Amount,
        fee_rate: Option<FeeRate>,
    },
}

impl State {
    pub fn set_error(&mut self, message: String) {
        self.error = Some(message);
    }

    pub fn update(&mut self, message: Message) -> Action {
        self.error = None;
        match message {
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
            Message::FeeRateInput(fee_rate) => {
                if is_fee_rate_input(&fee_rate) {
                    self.fee_rate = fee_rate
                }
                Action::None
            }
            Message::SendSubmit => {
                self.error = None;
                Action::SendCoins {
                    recipient: recipient_from_str(&self.recipient).unwrap(),
                    amount: amount_from_str(&self.amount).unwrap(),
                    fee_rate: fee_rate_from_str(&self.fee_rate).unwrap(),
                }
            }
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        column![
            text_big("Send coins"),
            error_block(self.error.as_ref()),
            Form::new(
                "Send",
                (recipient_from_str(&self.recipient).is_some()
                    && amount_from_str(&self.amount).is_some()
                    && fee_rate_from_str(&self.fee_rate).is_some())
                .then_some(Message::SendSubmit),
            )
            .add_labeled_input("Amount", "sat", &self.amount, Message::AmountInput)
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
            ),
        ]
        .spacing(10)
        .into()
    }
}
