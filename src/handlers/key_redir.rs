use emoji_picker_hooker::{install_hook, uninstall_hook};

use crate::{backend_link::BackendLink, handler::Handler};


mod emoji_picker_hooker {
    // Linking DLLs on Windows with Rust is a damn shame.
    #[link(name = "emoji_picker_hooker.dll")]
    extern "C" {
        pub fn install_hook(window: usize) -> u32;
        pub fn uninstall_hook();
    }
}

pub fn get_open_handler<'a>() -> Handler<'a, BackendLink> {
    Handler::new(|app: &BackendLink| {
        if let Some(hwnd) = app.get_main_window_hwnd() {
            let r = unsafe { install_hook(hwnd.0 as usize) };
            if r != 0 {
                eprintln!("[key_redir] Failed to set hook.");
            }
        }
    })
}

pub fn get_close_handler<'a>() -> Handler<'a, BackendLink> {
    Handler::new(|_: &BackendLink| unsafe { uninstall_hook() })
}
