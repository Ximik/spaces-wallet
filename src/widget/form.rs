use iced::{
    Background, Border, Center, Element, Fill, Font, Shrink, Theme,
    widget::{Button, Column, Container, Text, TextInput, button, column, pick_list, text_editor},
};
use std::borrow::Borrow;

pub fn text_input<'a, Message>(placeholder: &'a str, value: &'a str) -> TextInput<'a, Message>
where
    Message: Clone + 'a,
{
    TextInput::new(placeholder, value).font(Font::MONOSPACE)
}

pub fn text_label(text: &str) -> Text<'_> {
    Text::new(text).size(14)
}

pub fn submit_button<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    on_submit: Option<Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    Container::new(
        Button::new(content)
            .on_press_maybe(on_submit)
            .padding([10, 20])
            .width(Shrink),
    )
    .align_x(Center)
    .width(Fill)
    .into()
}

pub struct Form<'a, Message> {
    submit_label: &'a str,
    submit_message: Option<Message>,
    elements: Vec<Element<'a, Message>>,
}

impl<'a, Message: Clone + 'a> Form<'a, Message> {
    pub fn new(submit_label: &'a str, submit_message: Option<Message>) -> Self {
        Self {
            submit_label,
            submit_message,
            elements: Vec::new(),
        }
    }

    pub fn add_text_input(
        mut self,
        label: &'a str,
        placeholder: &'a str,
        value: &'a str,
        on_input: impl Fn(String) -> Message + 'a,
    ) -> Self {
        self.elements.push(
            column![
                text_label(label),
                text_input(placeholder, value)
                    .on_input(on_input)
                    .on_submit_maybe(self.submit_message.clone())
                    .padding(10),
            ]
            .spacing(5)
            .into(),
        );
        self
    }

    pub fn add_text_editor(
        mut self,
        label: &'a str,
        placeholder: &'a str,
        content: &'a text_editor::Content,
        on_action: impl Fn(text_editor::Action) -> Message + 'a,
    ) -> Self {
        self.elements.push(
            column![
                text_label(label),
                text_editor(content)
                    .placeholder(placeholder)
                    .on_action(on_action)
                    .font(Font::MONOSPACE)
                    .padding(10),
            ]
            .spacing(5)
            .into(),
        );
        self
    }

    pub fn add_pick_list<
        T: ToString + PartialEq + Clone + 'a,
        L: Borrow<[T]> + 'a,
        V: Borrow<T> + 'a,
    >(
        mut self,
        label: &'a str,
        options: L,
        selected: Option<V>,
        on_select: impl Fn(T) -> Message + 'a,
    ) -> Self {
        self.elements.push(
            column![
                text_label(label),
                pick_list(options, selected, on_select)
                    .style(|theme: &Theme, status: pick_list::Status| {
                        let palette = theme.extended_palette();
                        pick_list::Style {
                            background: Background::Color(palette.background.base.color),
                            border: Border {
                                radius: 2.0.into(),
                                width: 1.0,
                                color: if status == pick_list::Status::Hovered {
                                    palette.background.base.text
                                } else {
                                    palette.background.strong.color
                                },
                            },
                            ..pick_list::default(theme, status)
                        }
                    })
                    .font(Font::MONOSPACE)
                    .width(Fill)
                    .padding(10),
            ]
            .spacing(5)
            .into(),
        );
        self
    }

    pub fn add_text_button(
        mut self,
        label: &'a str,
        placeholder: &'a str,
        value: &'a str,
        on_press: Message,
    ) -> Self {
        self.elements.push(
            column![
                text_label(label),
                button(Text::new(if value.is_empty() {
                    placeholder
                } else {
                    value
                }))
                .style(move |theme: &Theme, status: button::Status| {
                    let palette = theme.extended_palette();
                    button::Style {
                        border: Border {
                            radius: 2.0.into(),
                            width: 1.0,
                            color: if status == button::Status::Hovered {
                                palette.background.base.text
                            } else {
                                palette.background.strong.color
                            },
                        },
                        text_color: if value.is_empty() {
                            palette.background.strong.color
                        } else {
                            palette.background.base.text
                        },
                        background: Some(palette.background.base.color.into()),
                        ..Default::default()
                    }
                })
                .on_press(on_press)
                .width(Fill)
                .padding(10),
            ]
            .spacing(5)
            .into(),
        );
        self
    }
}

impl<'a, Message: 'a + Clone> From<Form<'a, Message>> for Element<'a, Message> {
    fn from(form: Form<'a, Message>) -> Self {
        Column::from_vec(form.elements)
            .push(submit_button(form.submit_label, form.submit_message))
            .spacing(10)
            .width(Fill)
            .into()
    }
}
