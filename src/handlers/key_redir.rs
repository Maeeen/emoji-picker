use emoji_picker_hooker::{install_hook, uninstall_hook};
use slint::ComponentHandle as _;

use crate::handler::Handler;
use crate::SharedApp;

use super::utils::ToHWND;
use super::{NotifierReason, NotifiersArgs, OnCloseHandler, OnOpenHandler};

mod emoji_picker_hooker {
    // Linking DLLs on Windows with Rust is a damn shame.
    #[link(name = "emoji_picker_hooker.dll")]
    extern "C" {
        pub fn install_hook(window: usize) -> u32;
        pub fn uninstall_hook();
    }
}

pub fn get_open_handler<'a>() -> OnOpenHandler<'a> {
    Handler::new(|app: &(SharedApp, NotifiersArgs)| {
        let (app, reason) = app;

        // This is a shortcut related behavior.
        if *reason != NotifierReason::Shortcut {
            return;
        }

        let _ = app.weak_ui().upgrade_in_event_loop(move |ui| {
            if let Some(hwnd) = ui.window().to_hwnd() {
                let r = unsafe { install_hook(hwnd.0 as usize) };
                if r != 0 {
                    eprintln!("[key_redir] Failed to set hook.");
                }
            }
        });
    })
}

pub fn get_close_handler<'a>() -> OnCloseHandler<'a> {
    Handler::new(|_| unsafe { uninstall_hook() })
}
