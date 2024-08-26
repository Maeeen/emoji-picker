// TODO: Position should be on screen and not be out of bounds.

use std::ffi::c_void;

use slint::{ComponentHandle as _, WindowPosition};
use windows::{
    core::{Interface, VARIANT},
    Win32::{
        Foundation::{POINT, RECT},
        Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST},
        UI::{
            Accessibility::{AccessibleObjectFromWindow, IAccessible},
            WindowsAndMessaging::{
                GetForegroundWindow, GetGUIThreadInfo, GetWindowRect, GetWindowThreadProcessId,
                CHILDID_SELF, GUITHREADINFO, OBJID_CARET,
            },
        },
    },
};

use crate::handler::Handler;
use crate::EmojiPickerWindow;

use super::utils::ToHWND;

struct Position {
    x: i32,
    y: i32,
}

struct CaretPosition {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl From<Position> for WindowPosition {
    fn from(val: Position) -> Self {
        WindowPosition::Physical(slint::PhysicalPosition { x: val.x, y: val.y })
    }
}

/// Returns a position for the emoji picker window to be placed near the caret
/// in the “work”/“safe” area.
fn get_window_position(window: &slint::Window, cp: CaretPosition) -> Position {
    let caret_height = cp.h;
    let default_position = Position {
        x: cp.x,
        y: cp.y + caret_height,
    };

    unsafe fn inner(window: &slint::Window, cp: CaretPosition) -> Option<Position> {
        let point = POINT { x: cp.x, y: cp.y };
        let window_size = {
            let hwnd = window.to_hwnd()?;

            let mut lprect: RECT = { std::mem::zeroed() };
            GetWindowRect(hwnd, &mut lprect as *mut _).ok()?;
            (lprect.right - lprect.left, lprect.bottom - lprect.top)
        };

        // Retrieve the monitor information
        let mut monitor_info: MONITORINFO = { std::mem::zeroed() };
        monitor_info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;

        let monitor = MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST);
        if monitor.is_invalid() {
            return None;
        }
        if !GetMonitorInfoW(monitor, &mut monitor_info as *mut _).as_bool() {
            return None;
        }

        // Adjust the position of the window
        let mut final_position = Position { x: cp.x, y: cp.y };
        let work_area = monitor_info.rcWork;
        // Adjusting on the x coordinates
        if cp.x < work_area.left {
            final_position.x = work_area.left;
        } else if cp.x + cp.w + window_size.0 > work_area.right {
            final_position.x = work_area.right - cp.w - window_size.0;
        };
        // Adjusting on the y coordinates
        if cp.y < work_area.top {
            final_position.y = work_area.top;
        } else if cp.y + cp.h + window_size.1 > work_area.bottom {
            // Put the window above the caret
            final_position.y = cp.y - window_size.1 as i32;
        } else {
            // Put the window below the caret
            final_position.y = cp.y + cp.h;
        };
        Some(final_position)
    }

    unsafe { inner(window, cp) }.unwrap_or(default_position)
}

fn get_caret_position() -> Option<CaretPosition> {
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
        )
        .ok()?;
        let acc_if: IAccessible = IAccessible::from_raw(if_ptr);

        let (mut x, mut y, mut w, mut h) = (0, 0, 0, 0);
        let variant = VARIANT::from(CHILDID_SELF as i32);
        acc_if
            .accLocation(
                &mut x as *mut _,
                &mut y as *mut _,
                &mut w as *mut _,
                &mut h as *mut _,
                &variant,
            )
            .ok()?;

        Some(CaretPosition { x, y, w, h })
    }
}

pub fn get_handler<'a>() -> Handler<'a, EmojiPickerWindow> {
    Handler::new(|app: &EmojiPickerWindow| {
        if let Some(caret_location) = get_caret_position() {
            let window_pos = get_window_position(app.window(), caret_location);
            app.window().set_position(window_pos)
        }
    })
}
