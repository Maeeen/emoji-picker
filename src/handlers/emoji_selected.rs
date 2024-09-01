use windows::Win32::UI::{
    Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
        VIRTUAL_KEY,
    },
    WindowsAndMessaging::GetMessageExtraInfo,
};

use crate::{handler::Handler, SharedApp};

use super::EmojiSelectedHandler;

// On a Windows system, the emoji picker will send the requested String to the active window.
pub fn get_handler<'a>() -> EmojiSelectedHandler<'a> {
    Handler::new(|args: &(SharedApp, String)| {
        let (app, code) = args;

        if cfg!(target_os = "windows") && app.get_reason().should_type_emoji() {
            type_emoji(code);
        } else {
            clipboard(code);
        }
    })
}

/// Types the given String as Unicode characters.
#[cfg(target_os = "windows")]
pub fn type_emoji(code: &str) {
    let encoded = str::encode_utf16(code);
    let extra_info = unsafe { GetMessageExtraInfo() };
    let input_struct_kd = encoded.into_iter().map(|c| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                dwExtraInfo: extra_info.0 as usize,
                wVk: VIRTUAL_KEY(0),
                wScan: c,
                dwFlags: KEYEVENTF_UNICODE,
                time: 0,
            },
        },
    });
    let input_struct_kf = input_struct_kd.clone().map(|mut k| {
        unsafe {
            k.Anonymous.ki.dwFlags |= KEYEVENTF_KEYUP;
        }
        k
    });
    let input_struct = input_struct_kd.chain(input_struct_kf).collect::<Vec<_>>();
    unsafe {
        SendInput(input_struct.as_slice(), std::mem::size_of::<INPUT>() as i32);
    }
}

/// Copies the string to the clipboard.
#[cfg(target_os = "windows")]
pub fn clipboard(code: &str) {
    use windows::Win32::{
        Foundation::{GlobalFree, HANDLE, NO_ERROR},
        System::{
            DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData},
            Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
            Ole::CF_UNICODETEXT,
        },
    };

    let utf16: Vec<u16> = str::encode_utf16(code)
        .chain(std::iter::once(0u16))
        .collect();
    let len = utf16.len();
    unsafe {
        if OpenClipboard(None).is_err() {
            eprintln!("Failed to open clipboard");
            return;
        };
        if EmptyClipboard().is_err() {
            eprint!("Failed to empty clipboard");
            let _ = CloseClipboard();
        };
        let global = GlobalAlloc(GMEM_MOVEABLE, len * std::mem::size_of::<u16>());

        if global.is_err() {
            eprintln!("Failed to allocate memory");
            let _ = CloseClipboard();
            return;
        }

        let global = global.unwrap();

        let addr: *mut _ = GlobalLock(global);
        if addr.is_null() {
            eprintln!("Failed to lock memory");
            let _ = GlobalFree(global);
            let _ = CloseClipboard();
            return;
        }

        let slice = std::slice::from_raw_parts_mut(addr as *mut u16, len);
        slice.copy_from_slice(&utf16);
        let unlock_result = GlobalUnlock(global); // This… when… completes… successfully… returns… an… Err…?
        if unlock_result.is_ok() || unlock_result.unwrap_err().code() != NO_ERROR.into() {
            eprintln!("Failed to unlock memory");
            let _ = GlobalFree(global);
            let _ = CloseClipboard();
            return;
        }
        if SetClipboardData(CF_UNICODETEXT.0.into(), HANDLE(global.0)).is_err() {
            eprint!("Failed to set clipboard data");
            let _ = GlobalFree(global);
            let _ = CloseClipboard();
            return;
        };
        let _ = CloseClipboard();
    };
}
