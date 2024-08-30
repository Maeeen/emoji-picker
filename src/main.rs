use handlers::{HandlerEvent, HandlerNotifyEvent, Handlers};
use slint::{Model, ModelRc};
use std::sync::{Arc, RwLock};

mod emoji;
mod emoji_model;
mod handler;
mod handlers;
mod poller;

slint::include_modules!();

fn main() {
    let app = EmojiPickerWindow::new().expect("Failed to create window.");

    let handlers = Arc::new(Handlers::new(&app));

    init_emojis(&app);

    // Setup emoji selected
    app.on_emoji_selected({
        let handlers = handlers.clone();
        move |emoji| {
            handlers.trigger(HandlerEvent::EmojiSelected(emoji.code.into()));
        }
    });

    // Setup close handlers
    app.window().on_close_requested({
        let app = app.as_weak();
        let handlers = handlers.clone();
        move || {
            let app = app.upgrade().unwrap();
            handlers.trigger(HandlerEvent::Close(&app));
            slint::CloseRequestResponse::HideWindow
        }
    });

    // Caller to open a window and call the open handlers
    let open_window = {
        let app = app.as_weak();
        let handlers = handlers.clone();
        move || {
            let handlers = handlers.clone();
            app.upgrade_in_event_loop(move |app| {
                handlers.trigger(HandlerEvent::BeforeOpen(&app));
                app.window().show().expect("Failed to show window.");
                handlers.trigger(HandlerEvent::Open(&app));
            })
            .unwrap();
        }
    };

    let close_window = {
        let app = app.as_weak();
        let handlers = handlers.clone();
        move || {
            let handlers = handlers.clone();
            app.upgrade_in_event_loop(move |app| {
                handlers.trigger(HandlerEvent::Close(&app));
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
