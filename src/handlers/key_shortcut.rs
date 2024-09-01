use std::sync::{
    mpsc::{Receiver, SyncSender},
    Mutex,
};

use windows::Win32::{
    Foundation::{HMODULE, LPARAM, LRESULT, WPARAM},
    UI::{
        Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LWIN, VK_OEM_PERIOD, VK_RWIN},
        WindowsAndMessaging::{
            CallNextHookEx, SetWindowsHookExA, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT,
            WH_KEYBOARD_LL, WM_KEYDOWN,
        },
    },
};

use crate::handler::Notifier;

use super::{NotifierReason, NotifiersArgs};

static mut HOOK_CHANNEL: Option<SyncSender<NotifiersArgs>> = None;

unsafe extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let kbd: KBDLLHOOKSTRUCT = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };

    let windows_key_pressed = (GetAsyncKeyState(VK_LWIN.0.into()) >> 15) & 1 == 1
        || (GetAsyncKeyState(VK_RWIN.0.into()) >> 15) & 1 == 1;

    if windows_key_pressed
        && wparam.0 == WM_KEYDOWN as usize
        && kbd.vkCode == VK_OEM_PERIOD.0.into()
    {
        let tx = unsafe { HOOK_CHANNEL.as_ref() };
        if let Some(tx) = tx {
            tx.send(NotifierReason::Shortcut).unwrap();
        }
        return LRESULT(1);
    }

    // The first argument is “ignored”
    CallNextHookEx(HHOOK::default(), code, wparam, lparam)
}

#[derive(Debug, thiserror::Error)]
pub enum KeyShortcutError {
    #[error("Failed to create a hook. Reason: {0}")]
    HookError(#[from] windows::core::Error),
}

pub struct KeyShortcut(usize, Mutex<Receiver<NotifiersArgs>>);

impl KeyShortcut {
    pub fn create() -> Result<Self, KeyShortcutError> {
        let (tx, rx) = std::sync::mpsc::sync_channel::<NotifiersArgs>(1);
        unsafe {
            HOOK_CHANNEL = Some(tx);
        };
        let hook = unsafe {
            SetWindowsHookExA(
                WH_KEYBOARD_LL,
                Some(keyboard_hook),
                HMODULE::default(),
                Default::default(),
            )
        }
        .map_err(KeyShortcutError::HookError)?;
        Ok(Self(hook.0 as usize, Mutex::new(rx)))
    }
}

impl Notifier<NotifiersArgs> for KeyShortcut {
    fn has_notified(&self) -> Option<NotifiersArgs> {
        self.1.lock().unwrap().try_recv().ok()
    }
}

impl Drop for KeyShortcut {
    fn drop(&mut self) {
        unsafe {
            if let Err(e) = UnhookWindowsHookEx(HHOOK(self.0 as *mut _)) {
                eprintln!("Failed to unhook the keyboard hook. Reason: {:?}", e);
            }
        }
    }
}
