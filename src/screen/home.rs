use iced::alignment::Horizontal::Right;
use iced::widget::{button, column, container, row, scrollable, text, Column};
use iced::Alignment::Center;
use iced::{Element, Fill, FillPortion};

use crate::helpers::height_to_est;
use crate::types::*;

#[derive(Debug, Clone)]
pub enum Message {
    SpaceClicked { slabel: SLabel },
}

pub fn view<'a>(
    balance: Amount,
    tip_height: u32,
    spaces: impl Iterator<Item = (&'a SLabel, &'a Covenant)>,
) -> Element<'a, Message> {
    column![text("Balance (SAT)"), text(balance.to_sat()),]
        .spacing(5)
        .height(Fill)
        .width(Fill)
        .into()
}
