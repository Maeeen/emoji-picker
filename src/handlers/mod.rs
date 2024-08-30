use std::sync::{Arc, RwLock};

use crate::handler::{Handler, Notifier, OnceNotifier};
use crate::poller::Poller;
use crate::EmojiPickerWindow;

mod close_shortcut;
mod on_open_slint;

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

#[cfg(feature = "back-click")]
#[cfg(target_os = "windows")]
mod back_click;

mod utils;

type NotifiersArgs = ();
type HandlerArgs = EmojiPickerWindow;

pub struct Handlers<'a> {
    pub emoji_selected: Vec<Handler<'a, String>>,
    pub openers: Arc<Vec<Box<dyn Notifier<NotifiersArgs>>>>,
    pub closers: Arc<Vec<Box<dyn Notifier<NotifiersArgs>>>>,
    pub on_close_handlers: Vec<Handler<'a, HandlerArgs>>,
    pub before_open_handlers: Vec<Handler<'a, HandlerArgs>>,
    pub on_open_handlers: Vec<Handler<'a, HandlerArgs>>,
}

pub enum HandlerEvent<'a> {
    Open(&'a EmojiPickerWindow),
    BeforeOpen(&'a EmojiPickerWindow),
    Close(&'a EmojiPickerWindow),
    EmojiSelected(String),
}

pub enum HandlerNotifyEvent {
    Open,
    Close,
}

impl<'a> Handlers<'a> {
    pub fn new(app: &EmojiPickerWindow) -> Self {
        get_handlers(app)
    }

    pub fn trigger(&self, event: HandlerEvent) {
        // TODO: Debate, should handlers have all the same type?
        if let HandlerEvent::EmojiSelected(code) = event {
            for handler in self.emoji_selected.iter() {
                handler.call(&code);
            }
            return;
        }

        let (app, to_call) = match event {
            HandlerEvent::Open(app) => (app, &self.on_open_handlers),
            HandlerEvent::BeforeOpen(app) => (app, &self.before_open_handlers),
            HandlerEvent::Close(app) => (app, &self.on_close_handlers),
            HandlerEvent::EmojiSelected(_) => unreachable!(),
        };

        for handler in to_call.iter() {
            handler.call(app);
        }
    }

    pub fn setup_poller(
        &self,
        event: HandlerNotifyEvent,
        callback: RwLock<impl Fn() + Send + 'static>,
    ) -> Poller {
        let notifiers = match event {
            HandlerNotifyEvent::Open => &self.openers,
            HandlerNotifyEvent::Close => &self.closers,
        };

        let notifiers = notifiers.clone();
        Poller::new(move || {
            let callback = callback.read().unwrap();
            for notifier in notifiers.iter() {
                if notifier.has_notified().is_some() {
                    callback();
                }
            }
        })
    }
}

fn get_handlers<'a>(app: &EmojiPickerWindow) -> Handlers<'a> {
    let mut emoji_selected = vec![];
    let mut openers: Vec<Box<dyn Notifier<NotifiersArgs>>> = vec![];
    let mut closers: Vec<Box<dyn Notifier<NotifiersArgs>>> = vec![];
    let mut on_close_handlers = vec![];
    let mut on_open_handlers = vec![];
    let mut before_open_handlers = vec![];

    // Open the window on startup
    openers.push(Box::new(OnceNotifier::new(())));

    // Slint dependant
    closers.push(close_shortcut::get_close_shortcut_notifier(app));
    before_open_handlers.push(on_open_slint::get_handler());

    #[cfg(feature = "caret")]
    #[cfg(target_os = "windows")]
    {
        before_open_handlers.push(caret_locator::get_handler());
    };

    #[cfg(feature = "no-activate")]
    #[cfg(target_os = "windows")]
    {
        on_open_handlers.push(no_activate::get_handler());
    };

    #[cfg(feature = "key-shortcut")]
    #[cfg(target_os = "windows")]
    {
        let key_shortcut = key_shortcut::KeyShortcut::create();
        if let Err(e) = key_shortcut {
            eprintln!("Failed to create a key shortcut. Reason: {}", e);
        } else {
            openers.push(Box::new(key_shortcut.unwrap()));
        }
    };

    #[cfg(feature = "key-redir")]
    #[cfg(target_os = "windows")]
    {
        on_open_handlers.push(key_redir::get_open_handler());
        on_close_handlers.push(key_redir::get_close_handler());
    };

    #[cfg(target_os = "windows")]
    {
        emoji_selected.push(emoji_selected::get_handler());
    };

    #[cfg(feature = "tray-icon")]
    #[cfg(target_os = "windows")]
    {
        openers.push(tray_icon::initialize());
    };

    #[cfg(feature = "back-click")]
    #[cfg(target_os = "windows")]
    if let Some(outside_click_handlers) = back_click::generate_handlers(app) {
        before_open_handlers.push(outside_click_handlers.on_open_handler);
        on_close_handlers.push(outside_click_handlers.on_close_handler);
        closers.push(outside_click_handlers.closer);
    } else {
        eprintln!("Failed to generate outside click handlers.");
    }

    Handlers {
        emoji_selected,
        openers: Arc::new(openers),
        closers: Arc::new(closers),
        on_close_handlers,
        before_open_handlers,
        on_open_handlers,
    }
}
