use crate::handler::{MpscNotifier, Notifier};
use crate::EmojiPickerWindow;

pub fn get_close_shortcut_notifier(app: &EmojiPickerWindow) -> Box<dyn Notifier<()>> {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    app.on_close_requested(move || {
        tx.send(()).unwrap();
    });
    Box::new(MpscNotifier::new(rx))
}
