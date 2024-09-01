use std::sync::{Arc, RwLock};

use crate::handler::{Handler, Notifier, OnceNotifier};
use crate::poller::Poller;
use crate::{EmojiPickerWindow, SharedApp};

mod on_close_slint;
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

/// Defines the reason for the notifier to be called
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotifierReason {
    /// If there is nothing specific to signal
    None,
    /// If the notifier is a shortcut
    /// This is useful for open notifiers that can signal
    /// whether the window was opened by a shortcut (e.g. not showing
    /// the back-click for a tray icon activation.)
    Shortcut,
    /// Used by opening the window from the tray icon
    TrayIcon,
    /// If the close notifier is a back-click
    Backclick,
}

impl NotifierReason {
    pub fn should_type_emoji(&self) -> bool {
        matches!(self, NotifierReason::Shortcut)
    }
}

type NotifiersArgs = NotifierReason;
type HandlerArgs<Data = NotifierReason> = (SharedApp, Data);

type EmojiSelectedHandler<'a> = Handler<'a, HandlerArgs<String>>;
type GeneralAppHandler<'a> = Handler<'a, HandlerArgs>;
type OnCloseHandler<'a> = GeneralAppHandler<'a>;
type BeforeOpenHandler<'a> = GeneralAppHandler<'a>;
type OnOpenHandler<'a> = GeneralAppHandler<'a>;
type OpenerNotifier = Box<dyn Notifier<NotifiersArgs>>;
type CloserNotifier = Box<dyn Notifier<NotifiersArgs>>;

/// Represents all the handlers that can be triggered.
pub struct Handlers<'a> {
    pub openers: Arc<Vec<Box<dyn Notifier<NotifiersArgs>>>>,
    pub closers: Arc<Vec<Box<dyn Notifier<NotifiersArgs>>>>,
    pub emoji_selected: Vec<EmojiSelectedHandler<'a>>,
    pub on_close_handlers: Vec<OnCloseHandler<'a>>,
    pub before_open_handlers: Vec<BeforeOpenHandler<'a>>,
    pub on_open_handlers: Vec<OnOpenHandler<'a>>,
}

/// Represents the different events that can be triggered from the UI.
pub enum HandlerEvent<'a> {
    Open(&'a HandlerArgs),
    BeforeOpen(&'a HandlerArgs),
    Close(&'a HandlerArgs),
    EmojiSelected(&'a HandlerArgs<String>),
}

/// Represents the different events that can be triggered by an external source.
pub enum HandlerNotifyEvent {
    Open,
    Close,
}

impl<'a> Handlers<'a> {
    /// Generates a default set of handlers.
    pub fn new(ui: &EmojiPickerWindow) -> Self {
        get_handlers(ui)
    }

    /// Triggers the event.
    pub fn trigger(&self, event: HandlerEvent) {
        // TODO: Debate, should handlers have all the same type?
        if let HandlerEvent::EmojiSelected(code) = event {
            for handler in self.emoji_selected.iter() {
                handler.call(code);
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

    /// Setups a poller that will call the callback when the event is triggered.
    /// The callback will be called with the arguments of the event.
    pub fn setup_poller(
        &self,
        event: HandlerNotifyEvent,
        callback: RwLock<impl Fn(NotifiersArgs) + Send + 'static>,
    ) -> Poller {
        let notifiers = match event {
            HandlerNotifyEvent::Open => &self.openers,
            HandlerNotifyEvent::Close => &self.closers,
        };

        let notifiers = notifiers.clone();
        Poller::new(move || {
            let callback = callback.read().unwrap();
            for notifier in notifiers.iter() {
                if let Some(t) = notifier.has_notified() {
                    callback(t);
                }
            }
        })
    }
}

/// Generates the handlers for the UI.
fn get_handlers<'a>(ui: &EmojiPickerWindow) -> Handlers<'a> {
    let mut emoji_selected: Vec<EmojiSelectedHandler> = vec![];
    let mut openers: Vec<OpenerNotifier> = vec![];
    let mut closers: Vec<CloserNotifier> = vec![];
    let mut on_close_handlers: Vec<OnCloseHandler> = vec![];
    let mut on_open_handlers: Vec<OnOpenHandler> = vec![];
    let mut before_open_handlers: Vec<BeforeOpenHandler> = vec![];

    // Open the window on startup
    openers.push(Box::new(OnceNotifier::new(NotifierReason::None)));

    // Slint dependant
    closers.push(on_close_slint::get_close_shortcut_notifier(ui));
    before_open_handlers.push(on_open_slint::get_handler());

    emoji_selected.push(emoji_selected::get_handler());

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

    #[cfg(feature = "tray-icon")]
    #[cfg(target_os = "windows")]
    {
        openers.push(tray_icon::initialize());
    };

    #[cfg(feature = "back-click")]
    #[cfg(target_os = "windows")]
    if let Some(outside_click_handlers) = back_click::generate_handlers(ui) {
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
