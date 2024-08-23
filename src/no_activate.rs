use windows::Win32::{Foundation::GetLastError, UI::WindowsAndMessaging::{GetWindowLongPtrA, SetWindowLongPtrA, GWL_EXSTYLE, WS_EX_NOACTIVATE}};

use crate::to_hwnd::ToHWND;

pub fn setup(s: &slint::Window) -> Option<()> {
    let hwnd = s.to_hwnd()?;
    unsafe {
        let mut long: isize = GetWindowLongPtrA(hwnd, GWL_EXSTYLE) as isize;
        long |= WS_EX_NOACTIVATE.0 as isize;
        let r = SetWindowLongPtrA(hwnd, GWL_EXSTYLE, long);
        if r != 0 {
            Some(())
        } else {
            eprintln!("Could not set WS_EX_NOACTIVATE.");
            eprintln!("{:?}", GetLastError());
            None
        }
    }
}
