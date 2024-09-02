#![windows_subsystem = "windows"]

use handlers::{HandlerEvent, HandlerNotifyEvent, Handlers, NotifierReason};
use slint::{Model, ModelRc, Weak};
use std::sync::{Arc, Mutex, RwLock};

mod emoji;
mod emoji_model;
mod handler;
mod handlers;
mod poller;

slint::include_modules!();

type SharedApp = Arc<App>;
struct App {
    ui: Arc<Mutex<Weak<EmojiPickerWindow>>>,
    open_source: RwLock<NotifierReason>,
}

impl App {
    pub fn new(ui: Weak<EmojiPickerWindow>) -> Self {
        Self {
            ui: Arc::new(Mutex::new(ui)),
            open_source: RwLock::new(NotifierReason::None),
        }
    }

    pub fn weak_ui(&self) -> Weak<EmojiPickerWindow> {
        self.ui.lock().unwrap().clone()
    }

    pub fn set_reason(&self, reason: NotifierReason) {
        *self.open_source.write().unwrap() = reason;
    }

    pub fn get_reason(&self) -> NotifierReason {
        *self.open_source.read().unwrap()
    }
}

fn main() {
    let ui = EmojiPickerWindow::new().expect("Failed to create window.");
    let app = Arc::new(App::new(ui.as_weak()));
    let handlers = Arc::new(Handlers::new(&ui));

    init_emojis(&ui);

    // Setup emoji selected
    ui.on_emoji_selected({
        let (app, handlers) = (app.clone(), handlers.clone());
        move |emoji| {
            handlers.trigger(HandlerEvent::EmojiSelected(&(app.clone(), emoji.into())));
        }
    });

    // Setup close handlers
    ui.window().on_close_requested({
        let (app, handlers) = (app.clone(), handlers.clone());
        move || {
            handlers.trigger(HandlerEvent::Close(&(app.clone(), NotifierReason::None)));
            slint::CloseRequestResponse::HideWindow
        }
    });

    // Caller to open a window and call the open handlers
    let open_window = {
        let (app, ui, handlers) = (app.clone(), ui.as_weak(), handlers.clone());
        move |reason: NotifierReason| {
            app.set_reason(reason);
            handlers.trigger(HandlerEvent::BeforeOpen(&(app.clone(), reason)));
            ui.upgrade_in_event_loop({
                let (handlers, app) = (handlers.clone(), app.clone());
                move |ui| {
                    ui.window().show().expect("Failed to show window.");
                    handlers.trigger(HandlerEvent::Open(&(app, reason)));
                }
            })
            .unwrap();
        }
    };

    let close_window = {
        let (ui, handlers) = (ui.as_weak(), handlers.clone());
        move |reason| {
            let handlers = handlers.clone();
            handlers.trigger(HandlerEvent::Close(&(app.clone(), reason)));
            ui.upgrade_in_event_loop(move |app| {
                app.window().hide().expect("Failed to hide window.");
            })
            .unwrap();
        }
    };

    let open_window_shared = RwLock::new(open_window);
    let poller_for_open = handlers.setup_poller(HandlerNotifyEvent::Open, open_window_shared);

    let close_window_shared = RwLock::new(close_window);
    let poller_for_close = handlers.setup_poller(HandlerNotifyEvent::Close, close_window_shared);

    slint::run_event_loop_until_quit().expect("Failed to run event loop.");

    // This is not really necessary.
    poller_for_open.signal_stop();
    poller_for_close.signal_stop();
}

/// This function initializes the emoji buttons in the app.
/// It also sets up the filter function to filter the emojis
fn init_emojis(app: &EmojiPickerWindow) {
    let model = emoji_model::VecEmojiGroupModel::new();
    let model = ModelRc::new(model);

    app.set_emoji_groups(model.clone());

    app.on_filter(move |s| {
        model
            .as_any()
            .downcast_ref::<emoji_model::VecEmojiGroupModel>()
            .unwrap()
            .filter(s.into());
    });
}
