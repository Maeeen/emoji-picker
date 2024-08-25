use windows::Win32::UI::{
    Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
        VIRTUAL_KEY,
    },
    WindowsAndMessaging::GetMessageExtraInfo,
};

use crate::handler::Handler;

// On a Windows system, the emoji picker will the requested String to the active window.
pub fn get_handler<'a>() -> Handler<'a, String> {
    Handler::new(|code: &String| {
        let encoded = str::encode_utf16(code);
        let extra_info = unsafe { GetMessageExtraInfo() };
        let input_struct_kd = encoded.into_iter().map(|c| INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    dwExtraInfo: extra_info.0 as usize,
                    wVk: VIRTUAL_KEY(0),
                    wScan: c,
                    dwFlags: KEYEVENTF_UNICODE,
                    time: 0,
                },
            },
        });
        let input_struct_kf = input_struct_kd.clone().into_iter().map(|mut k| {
            unsafe {
                k.Anonymous.ki.dwFlags |= KEYEVENTF_KEYUP;
            }
            k
        });
        let input_struct = input_struct_kd.chain(input_struct_kf).collect::<Vec<_>>();
        // println!("Sending {:?}", input_struct);
        unsafe {
            SendInput(input_struct.as_slice(), std::mem::size_of::<INPUT>() as i32);
        }
    })
}
