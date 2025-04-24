use iced::{
    Alignment::Center,
    Element, Fill,
    widget::{button, column, text, vertical_space},
};

#[derive(Debug, Default)]
pub struct State;

#[derive(Debug, Clone)]
pub enum Message {
    ResetBackendPress,
}

#[derive(Debug, Clone)]
pub enum Action {
    None,
    ResetBackend,
}

impl State {
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ResetBackendPress => Action::ResetBackend,
        }
    }

    pub fn view<'a>(&self) -> Element<'a, Message> {
        column![
            vertical_space(),
            button(text("Reset backend settings").align_x(Center).width(Fill))
                .on_press(Message::ResetBackendPress)
                .style(button::danger)
                .padding(10)
                .width(Fill),
        ]
        .into()
    }
}
