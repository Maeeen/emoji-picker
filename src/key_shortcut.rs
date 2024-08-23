use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::{atomic::AtomicBool, mpsc::SyncSender};
use windows::core::Free;
use windows::Win32::Foundation::{HMODULE, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_LWIN, VK_OEM_PERIOD, VK_RWIN};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExA, HHOOK, KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN,
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyHandlerError {
    #[error("Could not hook key handler.")]
    HookError, // TODO add source depending on OS
    #[error("Unsupported OS")]
    UnsupportedOS,
}

pub struct KeyHandler {
    #[cfg(not(target_os = "windows"))]
    inner: Option<()>,
    #[cfg(target_os = "windows")]
    inner: Option<HHOOK>,
}

// Windows: This is the entry point for the low-level keyboard hook.
#[cfg(target_os = "windows")]
unsafe extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let kbd: KBDLLHOOKSTRUCT = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };

    if kbd.vkCode == VK_LWIN.0.into() || kbd.vkCode == VK_RWIN.0.into() {
        WINDOWS_KEY_PRESSED.store(wparam.0 == WM_KEYDOWN as usize, Ordering::Relaxed);
    }

    if WINDOWS_KEY_PRESSED.load(Ordering::Relaxed) && wparam.0 == WM_KEYDOWN as usize && kbd.vkCode == VK_OEM_PERIOD.0.into() {
        let tx = unsafe { HOOK_CHANNEL.as_ref() };
        if let Some(tx) = tx {
            tx.send(()).unwrap();
        }
        return LRESULT(1);
    }

    CallNextHookEx(HHOOK::default(), code, wparam, lparam)
}

#[cfg(target_os = "windows")]
static WINDOWS_KEY_PRESSED: AtomicBool = AtomicBool::new(false);
#[cfg(target_os = "windows")]
static mut HOOK_CHANNEL: Option<SyncSender<()>> = None; // This is only in use by the hook.

#[cfg(target_os = "windows")]
impl KeyHandler {
    // TODO: There is only a single hook allowed per thread. This should be a singleton.
    pub fn hook() -> Result<(KeyHandler, mpsc::Receiver<()>), KeyHandlerError> {
        // Setup the hook.
        let h = unsafe {
            SetWindowsHookExA(
                WH_KEYBOARD_LL,
                Some(keyboard_hook),
                HMODULE::default(),
                Default::default(),
            )
        };

        let h = h.map_err(|_| KeyHandlerError::HookError)?;

        // Setup MPSC channel between caller and hook.
        let (tx, rx) = mpsc::sync_channel::<()>(1);
        unsafe {
            HOOK_CHANNEL = Some(tx);
        };

        Ok((KeyHandler { inner: Some(h) }, rx))
    }

    pub fn unhook(mut self) -> Result<(), KeyHandlerError> {
        // If hook is still active, free it.
        if let Some(mut h) = self.inner {
            unsafe {
                h.free();
            }
        }
        // Clear
        self.inner = None;
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
impl KeyHandler {
    pub fn hook() -> Result<(KeyHandler, mpsc::Receiver<()>), KeyHandlerError> {
        Err(KeyHandlerError::UnsupportedOS)
    }

    pub fn unhook(self) -> Result<(), KeyHandlerError> {
        Err(KeyHandlerError::UnsupportedOS)
    }
}
