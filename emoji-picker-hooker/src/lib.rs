#![no_std]
#![no_main]

use libc::printf;
use core::{ffi::c_void, panic::PanicInfo};
use windows_sys::Win32::{Foundation::{GetLastError, HINSTANCE}, UI::{Input::KeyboardAndMouse::VK_PACKET, WindowsAndMessaging::{CallNextHookEx, PostMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KF_UP, WH_KEYBOARD, WM_KEYDOWN, WM_KEYUP}}};

mod volatile;
use volatile::Volatile;

static mut H_INSTANCE: HINSTANCE = core::ptr::null_mut();

const KF_UP_MASK: u32 = KF_UP << 16;

#[link_section = ".shared"]
static mut HOOK: Volatile<HHOOK> = Volatile::new(core::ptr::null_mut());
#[link_section = ".shared"]
static mut WINDOW: Volatile<usize> = Volatile::new(0);

// Maybe everything can be "system"
pub unsafe extern "system" fn keyboard_hook(ncode: i32, wparam: usize, lparam: isize) -> isize {
    // VK_PACKET is a VK-code that is used to send Unicode characters. There is no requirement
    // (yet) to redirect it, so we can ignore it.
    // We ignore them because the emoji picker will create them, and so, we don't want to
    // redirect them.
    if ncode >= 0 && wparam != VK_PACKET.into() {
        let window = WINDOW.get();
        if window != 0 {
            let message = if lparam as u32 & KF_UP_MASK == KF_UP_MASK { WM_KEYUP } else { WM_KEYDOWN };
            PostMessageW(window as *mut _, message, wparam, lparam);
            return 1;
        } else {
            // If there is no window, no hook should be installed.
            let hook = HOOK.get();
            if !hook.is_null() {
                uninstall_hook();
                return 1;
            }
        }
    }
    return CallNextHookEx(HOOK.get(), ncode, wparam, lparam);
}

#[no_mangle]
pub unsafe extern "stdcall" fn DllMain(h_instance: HINSTANCE, _: u32, _: *const c_void) -> bool {
    H_INSTANCE = h_instance;
    true
}

#[no_mangle]
pub unsafe extern "C" fn install_hook(window: usize) -> u32 {
    unsafe { 
        WINDOW.set(window);
        let hook = HOOK.get();
        if hook.is_null() {
            let hook = SetWindowsHookExW(WH_KEYBOARD, Some(keyboard_hook), H_INSTANCE, 0);
            HOOK.set(hook);
            if !hook.is_null() {
                return GetLastError()
            }
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn uninstall_hook() {
    unsafe {
        WINDOW.set(0);
        let hook = HOOK.get();
        if !hook.is_null(){
            UnhookWindowsHookEx(hook);
            HOOK.set(core::ptr::null_mut())
        }
    }
}


// Not really the best way to handle panics, let's not rely on any crate
#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    unsafe { printf("Panic occurred\n\0".as_ptr() as *const i8) };
    unreachable!()
}