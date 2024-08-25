use emoji_picker_hooker::{install_hook, uninstall_hook};
use slint::ComponentHandle as _;

use crate::handler::Handler;
use crate::EmojiPickerWindow;

use super::utils::ToHWND;

mod emoji_picker_hooker {
    // Linking DLLs on Windows with Rust is a damn shame.
    #[link(name = "emoji_picker_hooker.dll")]
    extern "C" {
        pub fn install_hook(window: usize) -> u32;
        pub fn uninstall_hook();
    }
}

pub fn get_open_handler<'a>() -> Handler<'a, EmojiPickerWindow> {
    Handler::new(|app: &EmojiPickerWindow| {
        let r = unsafe { install_hook(app.window().to_hwnd().unwrap().0 as usize) };
        if r != 0 {
            eprintln!("[key_redir] Failed to set hook.");
        }
    })
}

pub fn get_close_handler<'a>() -> Handler<'a, EmojiPickerWindow> {
    Handler::new(|_: &EmojiPickerWindow| unsafe { uninstall_hook() })
}
