use gpui::{Context, Render, Window};
use raw_window_handle::{
    HandleError, HasWindowHandle, RawWindowHandle, Win32WindowHandle, WindowHandle,
};
use thiserror::Error;
use windows_sys::{
    Wdk::System::SystemServices::RtlGetVersion,
    Win32::{
        Foundation::{HWND, S_OK},
        Graphics::Dwm::{
            DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DWMSBT_TRANSIENTWINDOW,
            DWMWA_SYSTEMBACKDROP_TYPE,
        },
        System::SystemInformation::OSVERSIONINFOW,
        UI::Controls::MARGINS,
    },
};

struct WinVersion {
    major: u32,
    #[allow(dead_code)]
    minor: u32,
    build_number: u32,
}

fn dwm_composition_toggled(hwnd: HWND) {
    let margins = MARGINS {
        cxLeftWidth: -1,
        cxRightWidth: -1,
        cyTopHeight: -1,
        cyBottomHeight: -1,
    };
    unsafe { DwmExtendFrameIntoClientArea(hwnd, &margins as *const _) };
}

fn set_backdrop_type(hwnd: HWND) {
    unsafe {
        let r = DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE as u32,
            &DWMSBT_TRANSIENTWINDOW as *const _ as _,
            4,
        );
        assert!(r == S_OK)
    };
}

#[derive(Error, Debug)]
pub enum DwmError {
    #[error("Got a non-Win32 handle")]
    InvalidPlatformHandle,
    #[error("Can not get Windows handle for the window")]
    UnvailableHandle,
    #[error("Windows version not supported")]
    UnsupportedWindows,
}

pub fn apply_acrilic(window: &mut Window, cx: &mut Context<impl Render>) -> Result<(), DwmError> {
    let ver = unsafe {
        let mut vi = OSVERSIONINFOW {
            dwOSVersionInfoSize: 0,
            dwMajorVersion: 0,
            dwMinorVersion: 0,
            dwBuildNumber: 0,
            dwPlatformId: 0,
            szCSDVersion: [0; 128],
        };
        RtlGetVersion(&mut vi as _);
        WinVersion {
            major: vi.dwMajorVersion,
            minor: vi.dwMinorVersion,
            build_number: vi.dwBuildNumber,
        }
    };

    let hwnd: Result<WindowHandle, HandleError> = window.window_handle();
    let hwnd = hwnd.map_err(|_| DwmError::UnvailableHandle)?;
    let hwnd = match hwnd.as_raw() {
        RawWindowHandle::Win32(hwnd) => hwnd.hwnd.get(),
        _ => return Err(DwmError::InvalidPlatformHandle),
    };
    let hwnd = HWND::from(hwnd as *mut _);

    // Win 10 after 21H2
    if ver.major == 10 && ver.build_number >= 22621 {
        dwm_composition_toggled(hwnd);
        set_backdrop_type(hwnd);
        Ok(())
    } else {
        Err(DwmError::UnsupportedWindows)
    }
}
