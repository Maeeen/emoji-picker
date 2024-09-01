use slint::ComponentHandle;
use windows::Win32::{
    Foundation::GetLastError,
    UI::WindowsAndMessaging::{
        GetWindowLongPtrA, SetWindowLongPtrA, GWL_EXSTYLE, WS_EX_NOACTIVATE,
    },
};

use super::{utils::ToHWND, NotifierReason, OnOpenHandler};
use crate::EmojiPickerWindow;
use crate::{handler::Handler, SharedApp};

fn setup(s: EmojiPickerWindow, no_activate: bool) {
    if let Some(hwnd) = s.window().to_hwnd() {
        unsafe {
            let mut long = GetWindowLongPtrA(hwnd, GWL_EXSTYLE);
            if no_activate {
                long |= WS_EX_NOACTIVATE.0 as isize;
            } else {
                long &= !WS_EX_NOACTIVATE.0 as isize;
            }
            let r = SetWindowLongPtrA(hwnd, GWL_EXSTYLE, long);
            if r == 0 {
                eprintln!(
                    "Could not set WS_EX_NOACTIVATE. Reason: {:?}",
                    GetLastError()
                );
            }
        }
    }
}

pub fn get_handler<'a>() -> OnOpenHandler<'a> {
    Handler::new(|args: &(SharedApp, _)| {
        let (app, reason) = args;

        let enable_noactivate_if = *reason == NotifierReason::Shortcut;

        let _ = app
            .weak_ui()
            .upgrade_in_event_loop(move |ui| setup(ui, enable_noactivate_if));
    })
}
