// This works by creating a transparent window under the window. If a click is registered on the transparent window, the window is closed. This is useful for closing the window when the user clicks outside of it.

use std::sync::mpsc;

use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{GetLastError, COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, CreateSolidBrush, DeleteObject, EndPaint, FillRect, GetStockObject, HBRUSH,
            NULL_BRUSH, PAINTSTRUCT,
        },
        UI::WindowsAndMessaging::{
            CloseWindow, CreateWindowExW, DefWindowProcW, GetSystemMetrics, GetWindowLongPtrW, PostQuitMessage, RegisterClassW, SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos, ShowWindow, CS_HREDRAW, CS_VREDRAW, GWL_HINSTANCE, GWL_HWNDPARENT, HMENU, LWA_ALPHA, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SWP_HIDEWINDOW, SWP_NOACTIVATE, SW_HIDE, SW_NORMAL, SW_SHOWNA, SW_SHOWNOACTIVATE, WM_DESTROY, WM_LBUTTONUP, WM_MBUTTONDBLCLK, WM_PAINT, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOPMOST, WS_POPUP
        },
    },
};

use crate::{backend_link::BackendLink, handler::{Handler, MpscNotifier, Notifier}};

pub struct OutsideClickHandlers<'a> {
    pub on_open_handler: Handler<'a, BackendLink>,
    pub on_close_handler: Handler<'a, BackendLink>,
    pub closer: Box<dyn Notifier<()>>,
}

unsafe extern "system" fn transparent_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            let brush = CreateSolidBrush(COLORREF(0));
            FillRect(hdc, &ps.rcPaint, brush);
            if !DeleteObject(brush).as_bool() {
                eprintln!("Failed to delete brush: {:?}", GetLastError());
            }
            if !EndPaint(hwnd, &ps).as_bool() {
                eprintln!("Failed to end paint: {:?}", GetLastError());
            }
        }
        WM_LBUTTONUP..=WM_MBUTTONDBLCLK => {
            // Any mouse event
            if let Some(tx) = TX.as_ref() {
                let _ = tx.send(());
            }
            // Hide window (despite the name, it does not destroy the window.)
            CloseWindow(hwnd).expect("Closing window failed");
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            return LRESULT(0);
        }
        _ => {}
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}

// Setups the transparent window to cover the whole screen.
// Mainly a function because it has to be called after the window is created
// and shown. In the rare case one will change their monitors while the app is
// running, this function should be called again.
// TODO: proper error handling
unsafe fn setup_transp_window_dimensions(hwnd: HWND) -> Option<()> {
    let virtual_screen_dim = (
        GetSystemMetrics(SM_CXVIRTUALSCREEN),
        GetSystemMetrics(SM_CYVIRTUALSCREEN),
    );
    if virtual_screen_dim.0 == 0 || virtual_screen_dim.1 == 0 {
        eprintln!(
            "Failed to get virtual screen dimensions {:?}",
            GetLastError()
        );
        return None;
    }
    // Position at the top left corner of the main window
    SetWindowPos(
        hwnd,
        HWND::default(),
        0,
        0,
        virtual_screen_dim.0,
        virtual_screen_dim.1,
        SWP_HIDEWINDOW | SWP_NOACTIVATE,
    )
    .expect("Failed to set window position");

    Some(())
}

// Yeah, this is not pretty. But it's a way to send a message to the event loop.
static mut TX: Option<mpsc::SyncSender<()>> = None;

fn generate_transparent_window(app: &BackendLink, tx: mpsc::SyncSender<()>) -> Option<HWND> {
    let hwnd = app.get_main_window_hwnd()?;
    let hinstance: HINSTANCE =
        unsafe { HINSTANCE(GetWindowLongPtrW(hwnd, GWL_HINSTANCE) as *mut _) };
    const CLASS_NAME: PCWSTR = w!("EmojiPickerTransparentWindow");

    let wndclass: WNDCLASSW = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(transparent_window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: Default::default(),
        hCursor: Default::default(),
        hbrBackground: HBRUSH(unsafe { GetStockObject(NULL_BRUSH) }.0 as *mut _),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: CLASS_NAME,
    };

    return unsafe {
        if RegisterClassW(&wndclass as *const _) == 0 {
            return None;
        }
        let transp_win = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
            CLASS_NAME,
            w!("Emoji picker transparent window"),
            WS_POPUP,
            100,
            100,
            500,
            500,
            HWND::default(),
            HMENU::default(),
            hinstance,
            None,
        )
        .ok()?;

        // Not the best flag but I want this to slightly appear only on debug builds
        let opacity: u8 = if cfg!(debug_assertions) { 128 } else { 0 };

        SetLayeredWindowAttributes(transp_win, COLORREF(0), opacity, LWA_ALPHA)
            .expect("Failed to set the window to be transparent");

        setup_transp_window_dimensions(transp_win)?;

        TX = Some(tx);

        Some(transp_win)
    };
}

pub fn generate_handlers<'a>(app: &BackendLink) -> Option<OutsideClickHandlers<'a>> {
    let (tx, rx) = mpsc::sync_channel::<()>(1);
    // The cast to isize is to send the HWND.
    // TODO: wrap it and mark it as Send.
    let transp_win = generate_transparent_window(app, tx)?.0 as isize;

    let on_open_handler = Handler::new(move |app: &BackendLink| unsafe {
        if let Some(win) = app.get_main_window_hwnd() {
            let transp_win = HWND(transp_win as *mut _);
            let _ = setup_transp_window_dimensions(transp_win);
            // This is a bit of a hack, but we set the main window of the emoji
            // picker to be the child window of the transparent window. This way,
            // the emoji picker will be above the transparent window.
            let _ = SetWindowLongPtrW(win, GWL_HWNDPARENT, transp_win.0 as isize);

            let _ = ShowWindow(transp_win, SW_NORMAL);
        }
    });
    let on_close_handler = Handler::new(move |_| unsafe {
        let _ = ShowWindow(HWND(transp_win as *mut _), SW_HIDE);
    });
    let closer = Box::new(MpscNotifier::new(rx));

    Some(OutsideClickHandlers {
        on_open_handler,
        on_close_handler,
        closer,
    })
}
