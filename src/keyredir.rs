use std::{ffi::CString, sync::mpsc::{ sync_channel, Receiver, SyncSender }};
        
use raw_window_handle::HasWindowHandle;
use slint::ComponentHandle;

use windows::Win32::{Foundation::{HWND, LPARAM, LRESULT, WPARAM}, UI::{Input::KeyboardAndMouse::VK_ESCAPE, WindowsAndMessaging::{DispatchMessageW, GetMessageTime, PostMessageW, SendMessageW, TranslateMessage, HHOOK, MSG}}};

slint::include_modules!();

unsafe extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    use windows::Win32::UI::WindowsAndMessaging::{CallNextHookEx, KBDLLHOOKSTRUCT, WM_KEYDOWN, WM_KEYUP};

    let kbd: KBDLLHOOKSTRUCT = *(lparam.0 as *const KBDLLHOOKSTRUCT);

    if kbd.vkCode == VK_ESCAPE.0.into() {
        println!("Trying to escape.");
        if let Some(h) = HOOK {
            println!("Unhooking");
            // windows::Win32::UI::WindowsAndMessaging::UnhookWindowsHookEx(h);
        }
        return LRESULT(1);
    }

    if let Some(hwnd) = WINDOW {
        let msg = MSG {
            hwnd: hwnd,
            message: wparam.0 as u32,
            wParam: WPARAM(kbd.vkCode as usize),
            lParam: LPARAM(kbd.flags.0 as isize),
            time: kbd.time,
            pt: Default::default()
        };
        // TranslateMessage(&msg);
        // DispatchMessageW(&msg);
        println!("PostMessageW({:?}, {:?}, {:?}, {:?})", hwnd, wparam.0, WPARAM(kbd.vkCode as usize), kbd.flags);
        println!("PostMessageW({:?}, {:?}, {:?}, {:?})", hwnd, wparam.0 as u32, WPARAM(kbd.vkCode as usize), LPARAM(kbd.flags.0 as isize));
        PostMessageW(hwnd, wparam.0 as u32, WPARAM(kbd.vkCode as usize), LPARAM(kbd.flags.0 as isize));
        // println!("PosMessageW({:?}, {:?}, {:?}, {:?})", hwnd, wparam.0 as u32, WPARAM(kbd.vkCode as usize), lparam);
        return LRESULT(1);
    } else {
        println!("could not get hwnd.");
    }


    /*
    if let Some(window) = WINDOW.as_ref() {
        window.upgrade_in_event_loop(|w| {  
            /* let mut key = KeyPress::KeyDown(kbd.vkCode);
            if wparam.0 == WM_KEYUP as usize {
                key = KeyPress::KeyUp(kbd.vkCode);
            } */

            // w.window().dispatch_event(slint::platform::WindowEvent::KeyPressed { text: "".into(), key: kbd.vkCode.into() });

            // let mut key: CString = CString::

            // HWND of the window
            /* let hwnd: raw_window_handle::WindowHandle = w.window().window_handle().window_handle().ok().unwrap();
            let hwnd = match hwnd.as_raw() {
                raw_window_handle::RawWindowHandle::Win32(handle) => HWND(handle.hwnd.get() as isize as *mut _),
                _ => panic!("Unsupported platform.")
            }; */
            // SendMessageW(hwnd, WM_KEY, wparam, lparam)

        });
        return LRESULT(1);
    }*/



    

    CallNextHookEx(HHOOK::default(), code, wparam, lparam)
}

#[cfg(target_os = "windows")]
static mut WINDOW: Option<HWND> = None;
#[cfg(target_os = "windows")]
static mut HOOK: Option<HHOOK> = None;

pub struct KeyRedirection {
    #[cfg(windows)]
    inner: Option<HHOOK>
}

impl KeyRedirection {
    #[cfg(not(windows))]
    fn new(_: HWND) -> Result<KeyRedirection, ()> {
        Err(())
    }

    pub fn new() -> KeyRedirection {
        KeyRedirection { inner: None }
    }

    #[cfg(windows)]
    // TODO: Better error
    pub fn set_target(&mut self, window: /*slint::Weak<AppWindow>*/ &slint::Window) -> Result<(), ()> {
        use windows::Win32::{Foundation::HMODULE, UI::WindowsAndMessaging::{SetWindowsHookExA, WH_CALLWNDPROC, WH_KEYBOARD_LL}};
        unsafe {
            let window = window.window_handle();
            let hwnd: raw_window_handle::WindowHandle = window.window_handle().ok().unwrap();
            let hwnd = match hwnd.as_raw() {
                raw_window_handle::RawWindowHandle::Win32(handle) => HWND(handle.hwnd.get() as isize as *mut _),
                _ => panic!("Unsupported platform.")
            };
            WINDOW = Some(hwnd)
        }

        let hook = if self.inner.is_none() {
            unsafe { SetWindowsHookExA(WH_KEYBOARD_LL, Some(keyboard_hook), HMODULE::default(), 0) }
        } else {
            Ok(self.inner.unwrap())
        };

        match hook {
            Ok(h) => {
                unsafe { HOOK = Some(h); }
                Ok(())
            },
            Err(_) => Err(())
        }
    }

    #[cfg(windows)]
    pub fn stop(&mut self) {
        unsafe {
            if let Some(h) = self.inner.take() {
                windows::Win32::UI::WindowsAndMessaging::UnhookWindowsHookEx(h);
            }
        }
    }
}