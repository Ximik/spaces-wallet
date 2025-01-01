pub const FONT: iced::Font = iced::Font::with_name("icons");
pub enum Icon {
    Artboard,
    Copy,
    ArrowDownToArc,
    ArrowDownFromArc,
    ArrowsExchange,
    At,
}
impl Icon {
    pub fn as_char(&self) -> char {
        match self {
            Icon::Artboard => '\u{E000}',
            Icon::Copy => '\u{E001}',
            Icon::ArrowDownToArc => '\u{E003}',
            Icon::ArrowDownFromArc => '\u{E004}',
            Icon::ArrowsExchange => '\u{E005}',
            Icon::At => '\u{E006}',
        }
    }
}
