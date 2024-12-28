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
    let mut spaces = spaces.collect::<Vec<_>>();
    spaces.sort_unstable_by_key(|s| s.0.as_str_unprefixed().unwrap());

    let transfer_spaces = spaces
        .iter()
        .filter_map(|(slabel, covenant)| match covenant {
            Covenant::Transfer { expire_height, .. } => Some((slabel, expire_height)),
            _ => None,
        });
    let bid_spaces = spaces
        .iter()
        .filter_map(|(slabel, covenant)| match covenant {
            Covenant::Bid {
                total_burned,
                claim_height,
                ..
            } => Some((slabel, total_burned, claim_height)),
            _ => None,
        });

    column![
        text("Balance (SAT)"),
        text(balance.to_sat()),
        // text("Your spaces"),
        // scrollable(column![
        //     column![
        //         text("Registered"),
        //         row![
        //             text("Space").width(FillPortion(1)),
        //             text("Expires").width(FillPortion(2)),
        //         ],
        //         Column::with_children(transfer_spaces.map(|(slabel, expire_height)| {
        //             row![
        //                 text(slabel.to_string()).width(FillPortion(1)),
        //                 text(height_to_est(*expire_height, tip_height)).width(FillPortion(1)),
        //                 container(button("View").on_press(Message::SpaceClicked {
        //                     slabel: *slabel.clone(),
        //                 }))
        //                 .width(FillPortion(1))
        //                 .align_x(Right),
        //             ]
        //             .align_y(Center)
        //             .into()
        //         })),
        //     ],
        //     column![
        //         text("Bid"),
        //         row![
        //             text("Space").width(FillPortion(1)),
        //             text("Highest Bid").width(FillPortion(1)),
        //             text("Claim").width(FillPortion(2)),
        //         ],
        //         Column::with_children(bid_spaces.map(|(slabel, total_burned, claim_height)| {
        //             row![
        //                 text(slabel.to_string()).width(FillPortion(1)),
        //                 text(total_burned.to_string_with_denomination(Denomination::Satoshi))
        //                     .width(FillPortion(1)),
        //                 text(
        //                     claim_height
        //                         .map(|h| height_to_est(h, tip_height))
        //                         .unwrap_or("pre-auction".to_string())
        //                 )
        //                 .width(FillPortion(1)),
        //                 container(button("View").on_press(Message::SpaceClicked {
        //                     slabel: *slabel.clone()
        //                 }))
        //                 .width(FillPortion(1))
        //                 .align_x(Right),
        //             ]
        //             .align_y(Center)
        //             .into()
        //         })),
        //     ]
        // ])
        // .spacing(10)
    ]
    .spacing(5)
    .height(Fill)
    .width(Fill)
    .into()
}
