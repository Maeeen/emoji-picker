use windows::Win32::{
    Foundation::GetLastError,
    UI::WindowsAndMessaging::{
        GetWindowLongPtrA, SetWindowLongPtrA, GWL_EXSTYLE, WS_EX_NOACTIVATE,
    },
};

use crate::{backend_link::BackendLink, handler::Handler};

fn setup(s: &BackendLink) -> Option<()> {
    let hwnd = s.get_main_window_hwnd()?;
    unsafe {
        let mut long = GetWindowLongPtrA(hwnd, GWL_EXSTYLE);
        long |= WS_EX_NOACTIVATE.0 as isize;
        let r = SetWindowLongPtrA(hwnd, GWL_EXSTYLE, long);
        if r != 0 {
            Some(())
        } else {
            eprintln!(
                "Could not set WS_EX_NOACTIVATE. Reason: {:?}",
                GetLastError()
            );
            None
        }
    }
}

pub fn get_handler<'a>() -> Handler<'a, BackendLink> {
    Handler::new(|app: &BackendLink| {
        setup(app);
    })
}
