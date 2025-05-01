pub const FONT: iced::Font = iced::Font::with_name("icons");
pub enum Icon {
    Settings,
    CurrencyBitcoin,
    Copy,
    BuildingBank,
    ArrowBigUpLines,
    FolderDown,
    ArrowDownToArc,
    ArrowDownFromArc,
    Signature,
    ChevronLeft,
    At,
    NewSection,
}
impl Icon {
    pub fn as_char(&self) -> char {
        match self {
            Icon::Settings => '\u{E000}',
            Icon::CurrencyBitcoin => '\u{E001}',
            Icon::Copy => '\u{E002}',
            Icon::BuildingBank => '\u{E003}',
            Icon::ArrowBigUpLines => '\u{E004}',
            Icon::FolderDown => '\u{E006}',
            Icon::ArrowDownToArc => '\u{E007}',
            Icon::ArrowDownFromArc => '\u{E008}',
            Icon::Signature => '\u{E009}',
            Icon::ChevronLeft => '\u{E00A}',
            Icon::At => '\u{E00B}',
            Icon::NewSection => '\u{E00C}',
        }
    }
}
