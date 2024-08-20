use slint::WindowPosition;
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::{
    Foundation::POINT,
    UI::WindowsAndMessaging::{GetGUIThreadInfo, GUITHREADINFO},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Position {
    x: i32,
    y: i32,
}

impl From<POINT> for Position {
    fn from(value: POINT) -> Self {
        Position {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<Position> for WindowPosition {
    fn from(val: Position) -> Self {
        WindowPosition::Physical(slint::PhysicalPosition {
            x: val.x,
            y: val.y,
        })
    }
}

#[cfg(not(windows))]
pub fn get_caret_pos() -> Option<Position> {
    None
}

#[cfg(windows)]
pub fn get_caret_pos() -> Option<Position> {
    use std::ffi::c_void;

    use windows::{
        core::{Interface, VARIANT},
        Win32::{
            System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED},
            UI::{
                Accessibility::{
                    AccessibleObjectFromWindow, IAccessible
                },
                WindowsAndMessaging::{
                    GetForegroundWindow, GetWindowThreadProcessId, CHILDID_SELF, OBJID_CARET
                },
            },
        },
    };

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

        let mut if_ptr: *mut c_void = std::ptr::null_mut();
        let guid = IAccessible::IID;
        let r = AccessibleObjectFromWindow(
            info.hwndFocus,
            OBJID_CARET.0 as u32,
            &guid as *const _,
            &mut if_ptr as *mut _,
        );

        if r.is_err() {
            // Is it that relevant to implement a legacy way of getting the caret position?
            eprintln!(
                "Error getting IAccessibleEx. Fallbacking to legacy. {:?}",
                r
            );
            // Coordinate relative to the HWND
            let (left, top) = (info.rcCaret.left, info.rcCaret.top);

            // Get the HWND of the caret
            let hwnd = info.hwndFocus;
            if hwnd.is_invalid() {
                return None;
            }

            // Get the screen coordinates of the caret
            let mut p = POINT { x: left, y: top };
            if !ClientToScreen(hwnd, &mut p as *mut _).as_bool() {
                return None;
            }

            return Some(p.into());
        }

        let acc_if: IAccessible = IAccessible::from_raw(if_ptr);
        let (mut x, mut y, mut w, mut h) = (0, 0, 0, 0);
        let variant = VARIANT::from(CHILDID_SELF as i32); // maybe u32
        acc_if
            .accLocation(
                &mut x as *mut _,
                &mut y as *mut _,
                &mut w as *mut _,
                &mut h as *mut _,
                &variant,
            )
            .ok()?;
        Some(Position { x, y: y + h })
    }
}

// // Gets the position of the caret in screen coordinates.
// // Expects the caret to be visible AND the target window to be focused.
// pub fn get_caret_pos_legacy() -> Option<Position> {
//     let hwnd = unsafe { GetForegroundWindow() };
//     if hwnd.is_invalid() {
//         return None
//     }

//     let target_thread_id = unsafe {
//         GetWindowThreadProcessId(hwnd, None)
//     };

//     if target_thread_id <= 0 { return None }

//     let current_thread_id = unsafe {
//         GetCurrentThreadId()
//     };

//     if current_thread_id <= 0 { return None }
//     if current_thread_id == target_thread_id {
//         eprintln!("Can not get caret position in the same thread.");
//         return None
//     }

//     let result = unsafe {
//         let mut p = POINT { x: 0, y: 0 };
//         if !AttachThreadInput(current_thread_id, target_thread_id, true).as_bool() { return None }
//         if let Err(e) = SetFocus(hwnd) {
//             eprintln!("Error setting focus. {e}");
//             return None
//         }
//         if let Err(e) = GetCaretPos(&mut p as *mut _) {
//             eprintln!("Error getting caret position. {e}");
//             return None
//         }
//         Some(p.into())
//     };
//     unsafe {
//         if !AttachThreadInput(current_thread_id, target_thread_id, false).as_bool() {
//             println!("Failed to detach threads.");
//         };
//     }
//     result
// }
