use iced::{
    widget::{text_input, Button, Text},
    Font, Pixels,
};

pub const FONT: Font = Font::with_name("icons");
pub const FONT_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/icons.ttf"));
include!(concat!(env!("OUT_DIR"), "/icons.rs"));

pub fn text_icon<'a>(icon: Icon) -> Text<'a> {
    Text::new(icon.as_char()).font(FONT)
}

pub fn button_icon<'a, Message>(icon: Icon) -> Button<'a, Message> {
    Button::new(text_icon(icon))
}

pub fn text_input_icon(icon: Icon, size: Option<Pixels>, spacing: f32) -> text_input::Icon<Font> {
    text_input::Icon {
        font: FONT,
        code_point: icon.as_char(),
        size,
        spacing,
        side: text_input::Side::Left,
    }
}
