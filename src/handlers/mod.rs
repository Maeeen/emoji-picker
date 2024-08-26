use crate::handler::{Handler, Notifier};
use crate::EmojiPickerWindow;

mod close_shortcut;

#[cfg(feature = "caret")]
#[cfg(target_os = "windows")]
mod caret_locator;

#[cfg(feature = "no-activate")]
#[cfg(target_os = "windows")]
mod no_activate;

#[cfg(feature = "key-shortcut")]
#[cfg(target_os = "windows")]
mod key_shortcut;

#[cfg(feature = "key-redir")]
#[cfg(target_os = "windows")]
mod key_redir;

#[cfg(target_os = "windows")]
mod emoji_selected;

#[cfg(feature = "tray-icon")]
mod tray_icon;

mod utils;

pub struct Handlers<'a> {
    pub emoji_selected: Vec<Handler<'a, String>>,
    pub openers: Vec<Box<dyn Notifier<()>>>,
    pub closers: Vec<Box<dyn Notifier<()>>>,
    pub on_close_handlers: Vec<Handler<'a, EmojiPickerWindow>>,
    pub before_open_handlers: Vec<Handler<'a, EmojiPickerWindow>>,
    pub on_open_handlers: Vec<Handler<'a, EmojiPickerWindow>>,
}

pub fn get_handlers<'a>(app: &EmojiPickerWindow) -> Handlers<'a> {
    let mut handlers = Handlers {
        emoji_selected: vec![],
        openers: vec![],
        closers: vec![],
        on_close_handlers: vec![],
        on_open_handlers: vec![],
        before_open_handlers: vec![],
    };

    #[cfg(feature = "caret")]
    #[cfg(target_os = "windows")]
    {
        handlers
            .before_open_handlers
            .push(caret_locator::get_handler());
    };

    #[cfg(feature = "no-activate")]
    #[cfg(target_os = "windows")]
    {
        handlers.on_open_handlers.push(no_activate::get_handler());
    };

    #[cfg(feature = "key-shortcut")]
    #[cfg(target_os = "windows")]
    {
        let key_shortcut = key_shortcut::KeyShortcut::create();
        if let Err(e) = key_shortcut {
            eprintln!("Failed to create a key shortcut. Reason: {}", e);
        } else {
            handlers.openers.push(Box::new(key_shortcut.unwrap()));
        }
    };

    #[cfg(feature = "key-redir")]
    #[cfg(target_os = "windows")]
    {
        handlers
            .on_open_handlers
            .push(key_redir::get_open_handler());
        handlers
            .on_close_handlers
            .push(key_redir::get_close_handler());
    };

    handlers
        .closers
        .push(close_shortcut::get_close_shortcut_notifier(app));

    #[cfg(target_os = "windows")]
    {
        handlers.emoji_selected.push(emoji_selected::get_handler());
    };

    #[cfg(feature = "tray-icon")]
    {
        handlers.openers.push(tray_icon::initialize());
    };

    handlers
}
