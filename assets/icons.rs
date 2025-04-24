pub const FONT: iced::Font = iced::Font::with_name("icons");
pub enum Icon {
    Settings,
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
            Icon::Settings => '\u{E000}',
            Icon::CurrencyBitcoin => '\u{E001}',
            Icon::Copy => '\u{E002}',
            Icon::BuildingBank => '\u{E003}',
            Icon::ArrowBigUpLines => '\u{E004}',
            Icon::ArrowDownToArc => '\u{E006}',
            Icon::ArrowDownFromArc => '\u{E007}',
            Icon::Signature => '\u{E008}',
            Icon::ChevronLeft => '\u{E009}',
            Icon::At => '\u{E00A}',
        }
    }
}
