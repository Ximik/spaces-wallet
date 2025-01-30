use std::sync::OnceLock;
use verglas::{build_icon_map, IconMap};
use iced::{
    widget::{text_input, Button, Text},
    Font, Pixels,
};

pub const FONT: Font = Font::with_name("icons");
pub const FONT_BYTES: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons.ttf"));

static ICON_MAP: OnceLock<IconMap> = OnceLock::new();
fn get_char(name: &str) -> char {
    ICON_MAP
        .get_or_init(|| {
            build_icon_map(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/icons.ttf"
            ))
            .unwrap_or_else(|e| {
                panic!("icon map creation failed: {:?}", e);
            })
        })
        .get(name)
        .copied()
        .unwrap_or_else(|| {
            panic!("icon '{}' not found", name);
        })
}

macro_rules! define_icons {
    (
        $(
            $variant:ident => $value:expr
        ),* $(,)?
    ) => {
        pub enum Icon {
            $(
                $variant,
            )*
        }

        impl Icon {
            pub fn as_char(&self) -> char {
                match self {
                    $(
                        Icon::$variant => get_char($value),
                    )*
                }
            }
        }
    };
}

define_icons! {
    Artboard => "artboard",
    Copy => "copy",
    ArrowDownToArc => "arrow-down-to-arc",
    ArrowDownFromArc => "arrow-down-from-arc",
    Qrcode => "qrcode",
    At => "at",
}

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
