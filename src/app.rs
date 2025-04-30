use iced::{Color, Element, Subscription, Task, application, theme, window};
use once_cell::sync::Lazy;

use crate::{Config, pages::*};

pub static WINDOW_TITLE: &str = "Akron";

pub static WINDOW_ICON: Lazy<window::Icon> = Lazy::new(|| {
    window::icon::from_rgba(include_bytes!("../assets/akron.rgba").to_vec(), 64, 64)
        .expect("Failed to load icon")
});

pub static ICONS_FONT: Lazy<&[u8]> = Lazy::new(|| include_bytes!("../assets/icons.ttf").as_slice());

pub static BITCOIN_THEME: Lazy<theme::Theme> = Lazy::new(|| {
    theme::Theme::custom_with_fn(
        "Bitcoin".into(),
        theme::Palette {
            text: Color::from_rgb8(77, 77, 77),
            primary: Color::from_rgb8(247, 147, 26),
            ..theme::Palette::LIGHT
        },
        |pallete| {
            let mut pallete = theme::palette::Extended::generate(pallete);
            pallete.primary.base.text = Color::WHITE;
            pallete.primary.strong.text = Color::WHITE;
            pallete.primary.weak.text = Color::WHITE;
            pallete
        },
    )
});

#[derive(Debug)]
pub enum State {
    Setup(setup::State),
    Main(main::State),
}

#[derive(Debug)]
enum Message {
    Setup(setup::Message),
    Main(main::Message),
}

impl State {
    pub fn run(config: Config, config_existing: bool) -> iced::Result {
        let (state, task) = setup::State::run(config, config_existing);
        let state = Self::Setup(state);
        let task = task.map(Message::Setup);
        application(WINDOW_TITLE, Self::update, Self::view)
            .font(ICONS_FONT.clone())
            .subscription(Self::subscription)
            .window(window::Settings {
                min_size: Some((1300.0, 500.0).into()),
                icon: Some(WINDOW_ICON.clone()),
                ..Default::default()
            })
            .theme(|_| BITCOIN_THEME.clone())
            .run_with(move || (state, task))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match (&mut *self, message) {
            (Self::Setup(state), Message::Setup(message)) => match state.update(message) {
                setup::Action::Exit(config, client) => {
                    let (state, task) = main::State::run(config, client);
                    let task = task.map(Message::Main);
                    *self = Self::Main(state);
                    task
                }
                setup::Action::Task(task) => task.map(Message::Setup),
            },
            (Self::Main(state), Message::Main(message)) => match state.update(message) {
                main::Action::Exit(config) => {
                    let (state, task) = setup::State::run(config, false);
                    let task = task.map(Message::Setup);
                    *self = Self::Setup(state);
                    task
                }
                main::Action::Task(task) => task.map(Message::Main),
            },
            _ => unreachable!(),
        }
    }

    fn view(&self) -> Element<Message> {
        match self {
            Self::Setup(state) => state.view().map(Message::Setup),
            Self::Main(state) => state.view().map(Message::Main),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        if let Self::Main(state) = self {
            state.subscription().map(Message::Main)
        } else {
            Subscription::none()
        }
    }
}
