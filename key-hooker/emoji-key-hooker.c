#pragma comment(lib, "user32")
#pragma comment(linker, "/section:.shared,RWS")

#include <windows.h>
#include <stdio.h>

#define API_EXPORT __declspec(dllexport)

static HMODULE hInstance = 0;

#pragma data_seg (".shared") 
// Volatile is enough since the races are not bothering (only the
// emoji-picker will write, and reading an invalid value a few times is okay.)
static volatile HWND TARGET_WINDOW = 0;
#pragma data_seg()

static HHOOK HOOK = 0;
const int KF_UP_MASK = KF_UP << 16;

API_EXPORT int test_dll() {
    return 1;
}

// Dll main
BOOL WINAPI DllMain(HINSTANCE hinstDLL, DWORD fdwReason, LPVOID lpvReserved)
{
    hInstance = hinstDLL;
    return TRUE;
}

LRESULT CALLBACK KeyboardProc(int nCode, WPARAM vk_code, LPARAM flags) {
    if (nCode >= 0) {
        if (vk_code == VK_ESCAPE) {
            if (HOOK) {
                UnhookWindowsHookEx(HOOK);
                HOOK = 0;
            }
            return 1;
        }

        if (TARGET_WINDOW) {
            int message = flags & KF_UP_MASK ? WM_KEYUP : WM_KEYDOWN;
            PostMessageW(TARGET_WINDOW, message, vk_code, flags);
            return 1;
        }
    }
    
    return CallNextHookEx(HOOK, nCode, vk_code, flags);
}

API_EXPORT int install_hook_dll(size_t window) {
    TARGET_WINDOW = (HWND) window;
    if (!HOOK)
        HOOK = SetWindowsHookExW(WH_KEYBOARD, (HOOKPROC) KeyboardProc, hInstance, 0);
    return HOOK ? 1 : 0;
}

API_EXPORT void uninstall_hook_dll() {
    if (HOOK) {
        UnhookWindowsHookEx(HOOK);
        HOOK = 0;
    }
}