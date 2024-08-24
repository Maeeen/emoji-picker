use windows::Win32::Foundation::HWND;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

pub trait ToHWND {
    fn to_hwnd(&self) -> Option<HWND>;
}

impl ToHWND for slint::Window {
    fn to_hwnd(&self) -> Option<HWND> {
        let handle = self.window_handle();
        let handle = handle.window_handle();
        let handle = handle.ok()?;
        match handle.as_ref() {
            RawWindowHandle::Win32(handle) => Some(HWND(handle.hwnd.get() as *mut _)),
            _ => None,
        }
    }
}