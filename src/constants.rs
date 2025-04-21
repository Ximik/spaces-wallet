use once_cell::sync::Lazy;

pub static WINDOW_ICON: Lazy<iced::window::Icon> = Lazy::new(|| {
    iced::window::icon::from_rgba(include_bytes!("../assets/spaces.rgba").to_vec(), 64, 64)
        .expect("Failed to load icon")
});

pub static ICONS_FONT: Lazy<&[u8]> = Lazy::new(|| include_bytes!("../assets/icons.ttf").as_slice());

pub static BITCOIN_THEME: Lazy<iced::theme::Theme> = Lazy::new(|| {
    iced::Theme::custom_with_fn(
        "Bitcoin".into(),
        iced::theme::Palette {
            text: iced::Color::from_rgb8(77, 77, 77),
            primary: iced::Color::from_rgb8(247, 147, 26),
            ..iced::theme::Palette::LIGHT
        },
        |pallete| {
            let mut pallete = iced::theme::palette::Extended::generate(pallete);
            pallete.primary.base.text = iced::Color::WHITE;
            pallete.primary.strong.text = iced::Color::WHITE;
            pallete.primary.weak.text = iced::Color::WHITE;
            pallete
        },
    )
});
