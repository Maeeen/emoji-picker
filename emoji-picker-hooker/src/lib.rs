#![no_std]
use core::{ffi::c_void, panic::PanicInfo};
use libc::printf;
use windows_sys::{w, Win32::{Foundation::HINSTANCE, System::Diagnostics::Debug::OutputDebugStringW, UI::{Input::KeyboardAndMouse::VK_ESCAPE, WindowsAndMessaging::{CallNextHookEx, PostMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KF_UP, WH_KEYBOARD, WM_KEYDOWN, WM_KEYUP}}}};

static mut H_INSTANCE: HINSTANCE = core::ptr::null_mut();

const KF_UP_MASK: u32 = KF_UP << 16;

#[link_section = ".shared"]
static mut HOOK: HHOOK = core::ptr::null_mut();
#[link_section = ".shared"]
static mut WINDOW: usize = 0;

// Maybe everything can be "system"
pub unsafe extern "system" fn keyboard_hook(ncode: i32, wparam: usize, lparam: isize) -> isize {
    if ncode >= 0 {
        printf("Keyboard hook called: vk: %d, lparam: %d\n\0".as_ptr() as *const i8, wparam as i32, lparam as i32);
        if wparam == VK_ESCAPE.into() {
            if !HOOK.is_null() {
                UnhookWindowsHookEx(HOOK);
                HOOK = core::ptr::null_mut();
                return 1;
            }
        }
    
        if WINDOW != 0 {
            let message = if lparam as u32 & KF_UP_MASK == KF_UP_MASK { WM_KEYUP } else { WM_KEYDOWN };
            PostMessageW(WINDOW as *mut _, message, wparam, lparam);
            return 1;
        };   
    }
    return CallNextHookEx(HOOK, ncode, wparam, lparam);
}

#[no_mangle]
pub unsafe extern "stdcall" fn DllMain(hInstDll: HINSTANCE, _: u32, _: *const c_void) -> bool {
    OutputDebugStringW(w!("dsqdqs"));
    // printf("DllMain called\n\0".as_ptr() as *const i8);
    // printf("Window: %d\n\0".as_ptr() as *const i8, WINDOW as i32);
    H_INSTANCE = hInstDll;
    true
}

#[no_mangle]
pub unsafe extern "C" fn test(a: usize) {
    unsafe { printf("Test called with %zu\n\0".as_ptr() as *const i8, a) };
}

#[no_mangle]
pub unsafe extern "C" fn install_hook(window: usize) {
    unsafe { 
        WINDOW = window;
        printf("Trying".as_ptr() as *const i8);
        if HOOK.is_null() {
            SetWindowsHookExW(WH_KEYBOARD, Some(keyboard_hook), H_INSTANCE, 0);
            printf("Installing hook\n\0".as_ptr() as *const i8);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn uninstall_hook() {
    unsafe {
        WINDOW = 0;
        if !HOOK.is_null() {
            UnhookWindowsHookEx(HOOK);
            HOOK = core::ptr::null_mut();
            printf("Uninstalling hook\n\0".as_ptr() as *const i8);
        }
    }
}

// Not really the best way to handle panics, let's not rely on any crate
#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    unsafe { printf("Panic occurred\n\0".as_ptr() as *const i8) };
    unreachable!()
}