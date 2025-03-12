pub const FONT: iced::Font = iced::Font::with_name("icons");
pub enum Icon {
    CurrencyBitcoin,
    Copy,
    ArrowBigUpLines,
    ArrowDownToArc,
    ArrowDownFromArc,
    ChevronLeft,
    At,
}
impl Icon {
    pub fn as_char(&self) -> char {
        match self {
            Icon::CurrencyBitcoin => '\u{E000}',
            Icon::Copy => '\u{E001}',
            Icon::ArrowBigUpLines => '\u{E002}',
            Icon::ArrowDownToArc => '\u{E004}',
            Icon::ArrowDownFromArc => '\u{E005}',
            Icon::ChevronLeft => '\u{E006}',
            Icon::At => '\u{E007}',
        }
    }
}
