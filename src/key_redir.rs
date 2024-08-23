// This is linked to key-hooker/emoji-key-hooker.c

#[cfg(windows)]
#[link(name = "emoji-key-hooker")]
extern "C" {
    fn install_hook_dll(hwnd: usize) -> bool;
    fn uninstall_hook_dll();
    fn test_dll() -> u32;
}

#[cfg(windows)]
pub fn install_hook(hwnd: usize) -> bool {
    unsafe { install_hook_dll(hwnd) }
}

pub fn test() -> u32 {
    unsafe { test_dll() }
}

#[cfg(windows)]
pub fn uninstall_hook() {
    unsafe { uninstall_hook_dll() }
}

#[cfg(not(windows))]
pub fn install_hook(_hwnd: usize) -> bool {
    false
}

#[cfg(not(windows))]
pub fn uninstall_hook() {}