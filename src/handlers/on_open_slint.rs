use crate::{handler::Handler, EmojiPickerWindow};

pub fn get_handler<'a>() -> Handler<'a, EmojiPickerWindow> {
    Handler::new(|app: &EmojiPickerWindow| {
        app.invoke_on_open();
    })
}
