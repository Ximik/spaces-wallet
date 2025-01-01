use iced::{
    widget::{container, text, Space},
    Element, Fill, Theme,
};

pub fn error<'a, Message: 'a>(
    message: Option<impl text::IntoFragment<'a>>,
) -> Element<'a, Message> {
    match message {
        Some(message) => container(
            text(message)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().danger.base.text),
                })
                .center()
                .width(Fill),
        )
        .style(|theme: &Theme| {
            container::Style::default().background(theme.extended_palette().danger.base.color)
        })
        .width(Fill)
        .padding(10)
        .into(),
        None => Space::new(0, 0).into(),
    }
}
