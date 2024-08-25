use slint::ComponentHandle;
use windows::Win32::{
    Foundation::GetLastError,
    UI::WindowsAndMessaging::{
        GetWindowLongPtrA, SetWindowLongPtrA, GWL_EXSTYLE, WS_EX_NOACTIVATE,
    },
};

use super::utils::ToHWND;
use crate::handler::Handler;
use crate::EmojiPickerWindow;

fn setup(s: &slint::Window) -> Option<()> {
    let hwnd = s.to_hwnd()?;
    unsafe {
        let mut long = GetWindowLongPtrA(hwnd, GWL_EXSTYLE);
        long |= WS_EX_NOACTIVATE.0 as isize;
        let r = SetWindowLongPtrA(hwnd, GWL_EXSTYLE, long);
        if r != 0 {
            Some(())
        } else {
            // TODO: remove this
            eprintln!("Could not set WS_EX_NOACTIVATE.");
            eprintln!("{:?}", GetLastError());
            None
        }
    }
}

pub fn get_handler<'a>() -> Handler<'a, EmojiPickerWindow> {
    Handler::new(|app: &EmojiPickerWindow| {
        setup(app.window());
    })
}
