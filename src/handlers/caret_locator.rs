// TODO: Position should be on screen and not be out of bounds.

use std::ffi::c_void;

use slint::{ComponentHandle as _, WindowPosition};
use windows::{
    core::{Interface, VARIANT},
    Win32::UI::{
        Accessibility::{
            AccessibleObjectFromWindow, IAccessible
        },
        WindowsAndMessaging::{
            GetGUIThreadInfo, GUITHREADINFO,
            GetForegroundWindow, GetWindowThreadProcessId, CHILDID_SELF, OBJID_CARET
        },
    },
};

use crate::handler::Handler;
use crate::EmojiPickerWindow;

pub struct Position { x: i32, y: i32 }

impl From<Position> for WindowPosition {
    fn from(val: Position) -> Self {
        WindowPosition::Physical(slint::PhysicalPosition {
            x: val.x,
            y: val.y,
        })
    }
}

fn get_caret_location() -> Option<Position> {
    unsafe {
        let mut info: GUITHREADINFO = { std::mem::zeroed() };
        info.cbSize = std::mem::size_of::<GUITHREADINFO>() as u32;

        // Get thread of current active window
        let hwnd = GetForegroundWindow();
        let target_thread_id = if hwnd.is_invalid() {
            0
        } else {
            GetWindowThreadProcessId(hwnd, None)
        };
        GetGUIThreadInfo(target_thread_id, &mut info as *mut _).ok()?;

        // Get IAccessible interface
        let mut if_ptr: *mut c_void = std::ptr::null_mut();
        let guid = IAccessible::IID;
        AccessibleObjectFromWindow(
            info.hwndFocus,
            OBJID_CARET.0 as u32,
            &guid as *const _,
            &mut if_ptr as *mut _,
        ).ok()?;
        let acc_if: IAccessible = IAccessible::from_raw(if_ptr);

        let (mut x, mut y, mut w, mut h) = (0, 0, 0, 0);
        let variant = VARIANT::from(CHILDID_SELF as i32);
        acc_if.accLocation(
            &mut x as *mut _,
            &mut y as *mut _,
            &mut w as *mut _,
            &mut h as *mut _,
            &variant,
        ).ok()?;
        Some(Position { x, y: y + h })
    }
}

pub fn get_handler<'a>() -> Handler<'a, EmojiPickerWindow> {
    Handler::new(|app: &EmojiPickerWindow| {
        match get_caret_location() {
            Some(p) => app.window().set_position(p),
            None => ()
        }
    })
}