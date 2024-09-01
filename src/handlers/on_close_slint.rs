use crate::handler::MpscNotifier;
use crate::EmojiPickerWindow;

use super::{CloserNotifier, NotifiersArgs};

/// This is to pass the message that the window is closing to the handlers.
pub fn get_close_shortcut_notifier(ui: &EmojiPickerWindow) -> CloserNotifier {
    let (tx, rx) = std::sync::mpsc::sync_channel::<NotifiersArgs>(1);
    ui.on_close_requested(move || {
        tx.send(super::NotifierReason::None).unwrap();
    });
    Box::new(MpscNotifier::new(rx))
}
