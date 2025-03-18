pub const FONT: iced::Font = iced::Font::with_name("icons");
pub enum Icon {
    CurrencyBitcoin,
    Copy,
    BuildingBank,
    ArrowBigUpLines,
    ArrowDownToArc,
    ArrowDownFromArc,
    Signature,
    ChevronLeft,
    At,
}
impl Icon {
    pub fn as_char(&self) -> char {
        match self {
            Icon::CurrencyBitcoin => '\u{E000}',
            Icon::Copy => '\u{E001}',
            Icon::BuildingBank => '\u{E002}',
            Icon::ArrowBigUpLines => '\u{E003}',
            Icon::ArrowDownToArc => '\u{E005}',
            Icon::ArrowDownFromArc => '\u{E006}',
            Icon::Signature => '\u{E007}',
            Icon::ChevronLeft => '\u{E008}',
            Icon::At => '\u{E009}',
        }
    }
}
