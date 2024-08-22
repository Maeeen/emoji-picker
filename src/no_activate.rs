use windows::Win32::{
    System::Threading::{AttachThreadInput, GetCurrentThreadId},
    UI::{
        Input::KeyboardAndMouse::SetFocus,
        WindowsAndMessaging::{GetForegroundWindow, GetGUIThreadInfo, GetWindowThreadProcessId, ShowCaret, GUITHREADINFO},
    },
};

#[cfg(windows)]
pub fn setup(s: &slint::Window) -> Option<()> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::Win32::{Foundation::{GetLastError, HWND}, UI::WindowsAndMessaging::{GetWindowLongPtrA, SetWindowLongPtrA, ShowWindow, GWL_EXSTYLE, SW_SHOWNOACTIVATE, WS_EX_NOACTIVATE, WS_EX_TOPMOST}};

    let b = s.window_handle();
    let hwnd = b.window_handle();
    if hwnd.is_err() {
        eprintln!("Invalid HWND.");
        return None;
    }
    let hwnd = hwnd.as_ref().map(|a| a.as_raw());
    match hwnd {
        Ok(RawWindowHandle::Win32(handle)) => {
            let hwnd: isize = handle.hwnd.get() as isize;
            unsafe {
                let hwnd = HWND(hwnd as *mut _);
                let mut long: isize = (GetWindowLongPtrA(hwnd, GWL_EXSTYLE) as isize);
                long |= WS_EX_NOACTIVATE.0 as isize;
                let r = SetWindowLongPtrA(hwnd, GWL_EXSTYLE, long);
                if r != 0 {
                } else {
                    eprintln!("Could not set WS_EX_NOACTIVATE.");
                    eprintln!("{:?}", GetLastError());
                }
            }
        }
        _ => {
            eprintln!("Unsupported platform.");
            return None;
        }
    }


    None
    /*let mut info: GUITHREADINFO = { unsafe { std::mem::zeroed() } };
    info.cbSize = std::mem::size_of::<GUITHREADINFO>() as u32;

    // Get thread of current active window
    unsafe { 
        let hwnd = GetForegroundWindow();
        let target_thread_id = if hwnd.is_invalid() {
            0
        } else {
            GetWindowThreadProcessId(hwnd, None)
        };
        GetGUIThreadInfo(target_thread_id, &mut info as *mut _).ok()?;
        let hwnd = info.hwndCaret;
        if let Err(d) = ShowCaret(hwnd) {
            eprintln!("Not successful {:?}!", d);
        }
    }
    Some(())*/


    /* let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_invalid() {
        return None;
    }

    let target_thread_id = unsafe { GetWindowThreadProcessId(hwnd, None) };

    if target_thread_id == 0 {
        return None;
    }

    let current_thread_id = unsafe { GetCurrentThreadId() };

    if current_thread_id == 0 { 
        return None;
    }
    if current_thread_id == target_thread_id {
        eprintln!("Can not get caret position in the same thread.");
        return None;
    }

    unsafe {
        if !AttachThreadInput(current_thread_id, target_thread_id, true).as_bool() {
            return None;
        }
        if let Err(e) = SetFocus(hwnd) {
            eprintln!("Error setting focus. {e}");
            return None;
        }
        eprintln!("Successful until here");
        if let Err(d) = ShowCaret(hwnd) {
            eprintln!("Not successful {:?}!", d);
        }
        eprintln!("Successful!");
        Some(())
    } */
}
