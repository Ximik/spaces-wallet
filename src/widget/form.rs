use iced::{
    Background, Center, Element, Fill, Font, Shrink, Theme,
    widget::{
        Button, Column, Container, Text, TextInput, column, pick_list, text_editor,
    },
};
use std::borrow::Borrow;

pub fn text_input<'a, Message: 'a>(placeholder: &'a str, value: &'a str) -> TextInput<'a, Message>
where
    Message: Clone,
{
    TextInput::new(placeholder, value).font(Font::MONOSPACE)
}

pub fn text_label<'a>(text: &'a str) -> Text<'a> {
    Text::new(text).size(14)
}

pub fn submit_button<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    on_submit: Option<Message>,
) -> Element<'a, Message>
where
    Message: Clone,
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
