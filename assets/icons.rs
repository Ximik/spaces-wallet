pub const FONT: iced::Font = iced::Font::with_name("icons");
pub enum Icon {
    Artboard,
    Copy,
    ArrowDownToArc,
    Eye,
    ArrowDownFromArc,
    Qrcode,
    At,
}
impl Icon {
    pub fn as_char(&self) -> char {
        match self {
            Icon::Artboard => '\u{E000}',
            Icon::Copy => '\u{E001}',
            Icon::ArrowDownToArc => '\u{E003}',
            Icon::Eye => '\u{E004}',
            Icon::ArrowDownFromArc => '\u{E005}',
            Icon::Qrcode => '\u{E006}',
            Icon::At => '\u{E007}',
        }
    }
}
