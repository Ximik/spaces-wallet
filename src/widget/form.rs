use iced::{
    widget::{Button, Column, Container, Text, TextInput},
    Center, Element, Fill, Font, Shrink,
};

pub fn text_input<'a, Message: 'a>(placeholder: &'a str, value: &'a str) -> TextInput<'a, Message>
where
    Message: Clone,
{
    TextInput::new(placeholder, value).font(Font::MONOSPACE)
}

pub fn labeled_input<'a, Message: 'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
    on_submit: Option<Message>,
) -> Element<'a, Message>
where
    Message: Clone,
{
    Column::new()
        .push(Text::new(label).size(14))
        .push(
            text_input(placeholder, value)
                .on_input(on_input)
                .on_submit_maybe(on_submit)
                .padding(10),
        )
        .spacing(5)
        .into()
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
    labeled_inputs: Vec<Element<'a, Message>>,
}

impl<'a, Message: Clone + 'a> Form<'a, Message> {
    pub fn new(submit_label: &'a str, submit_message: Option<Message>) -> Self {
        Self {
            submit_label,
            submit_message,
            labeled_inputs: Vec::new(),
        }
    }

    pub fn add_labeled_input(
        mut self,
        label: &'a str,
        placeholder: &'a str,
        value: &'a str,
        on_input: impl Fn(String) -> Message + 'a,
    ) -> Self {
        self.labeled_inputs.push(labeled_input(
            label,
            placeholder,
            value,
            on_input,
            self.submit_message.clone(),
        ));
        self
    }
}

impl<'a, Message: 'a + Clone> From<Form<'a, Message>> for Element<'a, Message> {
    fn from(form: Form<'a, Message>) -> Self {
        Column::from_vec(form.labeled_inputs)
            .push(submit_button(form.submit_label, form.submit_message))
            .spacing(10)
            .width(Fill)
            .into()
    }
}
