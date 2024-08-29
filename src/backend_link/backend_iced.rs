use std::{collections::HashMap, sync::RwLock};

use iced::{widget::{container, text, text_input, Column}, Application, Length, Theme};

use super::*;

struct FrozenEmojiPicker { 
    shown_message: String
}

impl FrozenEmojiPicker {
    pub fn new() -> Self {
        Self {
            shown_message: "Hello, world!".into()
        }
    }
}

#[derive(Debug, Clone)]
enum EmojiPickerMessage {
    SetMessage(String)
}

impl Application for FrozenEmojiPicker {
    type Executor = iced::executor::Default;
    type Message = EmojiPickerMessage;
    type Flags = ();
    type Theme = Theme;

    fn new(_: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (FrozenEmojiPicker::new(), iced::Command::none())
    }

    fn title(&self) -> String {
        "Emoji Picker".into()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            EmojiPickerMessage::SetMessage(message) => {
                self.shown_message = message;
            }
        }
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        Column::new()
            .push(
                text(&self.shown_message)
            )
            .push(
                text_input("Filter".into(), "")
                .on_input(|m| EmojiPickerMessage::SetMessage(m))
            )
            .into()
    }
}

pub struct BackendLinkInner {
    listeners: RwLock<HashMap<BackendEventKind, Listener>>
}

impl BackendLinkInner {
    pub fn new() -> Self {
        Self {
            listeners: RwLock::new(HashMap::new())
        }
    }

    pub fn run_event_loop(&self) {
        loop {
            FrozenEmojiPicker::run(iced::Settings::default());
        }
    }

    pub fn hide(&self) {

    }

    pub fn show(&self) {

    }

    pub fn on(&self, event: BackendEventKind, callback: Listener) {
        // self.inner.lock().unwrap().on(event, callback);
    }

    pub fn dispatch(&self, parent: &BackendLink, event: BackendEvents) {
        // self.inner.dispatch(event);
    }
}