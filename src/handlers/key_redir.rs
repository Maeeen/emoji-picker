use emoji_picker_hooker::{install_hook, uninstall_hook};
use slint::ComponentHandle as _;

use crate::handler::Handler;
use crate::EmojiPickerWindow;

use super::utils::ToHWND;

mod emoji_picker_hooker {
    // Linking DLLs on Windows with Rust is a damn shame.
    #[link(name = "emoji_picker_hooker.dll")]
    extern "C" {
        // #[link_name = "install_hook"]
        pub fn install_hook(window: usize);
        // #[link_name = "test"]
        pub fn test(a: usize);
        // #[link_name = "uninstall_hook"]
        pub fn uninstall_hook();
    }
}

pub fn get_open_handler<'a>() -> Handler<'a, EmojiPickerWindow> {
    unsafe { emoji_picker_hooker::test(5); }
    Handler::new(|app: &EmojiPickerWindow| {
        unsafe { install_hook(app.window().to_hwnd().unwrap().0 as usize) }
    })
}


pub fn get_close_handler<'a>() -> Handler<'a, EmojiPickerWindow> {
    Handler::new(|app: &EmojiPickerWindow| {
        unsafe { uninstall_hook() }
    })
}