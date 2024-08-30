use handlers::Handlers;
use slint::{Model, ModelRc};
use std::sync::{Arc, RwLock};

mod emoji;
mod emoji_model;
mod handler;
mod handlers;
mod poller;

slint::include_modules!();

fn main() {
    use handler::*;

    let app = EmojiPickerWindow::new().expect("Failed to create window.");

    let Handlers {
        emoji_selected,
        mut openers,
        closers,
        on_close_handlers,
        before_open_handlers,
        on_open_handlers,
    } = handlers::get_handlers(&app);

    // Open the window on start
    if true {
        // This is a bit of a hack. We need to open the window on start.
        // The borrow checker is not happy when we try to open the window using
        // the RwLock in the poller. With an Arc, since a slint's weak reference
        // is not Sync, and making a Mutex for ONLY that is not really interesting.
        // Little hack ftw.
        openers.push(Box::new(OnceNotifier::new(())))
    }

    let before_open_handlers = Arc::new(before_open_handlers);
    let on_close_handlers = Arc::new(on_close_handlers);
    let on_open_handlers = Arc::new(on_open_handlers);
    let openers = Arc::new(openers);

    init_emojis(&app);

    // Setup emoji selected
    app.on_emoji_selected(move |emoji| {
        let code: String = emoji.code.into();
        for handler in emoji_selected.iter() {
            handler.call(&code);
        }
    });

    // Setup close handlers
    app.window().on_close_requested({
        let app = app.as_weak();
        let on_close_handlers = on_close_handlers.clone();
        move || {
            let app = app.upgrade().unwrap();
            for handler in on_close_handlers.iter() {
                handler.call(&app);
            }
            slint::CloseRequestResponse::HideWindow
        }
    });

    // Caller to open a window and call the open handlers
    let open_window = {
        let app = app.as_weak();
        move || {
            let on_open_handlers = on_open_handlers.clone();
            let before_open_handlers = before_open_handlers.clone();
            app.upgrade_in_event_loop(move |app| {
                for handler in before_open_handlers.iter() {
                    handler.call(&app);
                }
                app.window().show().expect("Failed to show window.");
                for handler in on_open_handlers.iter() {
                    handler.call(&app);
                }
            })
            .unwrap();
        }
    };

    let close_window = {
        let app = app.as_weak();
        move || {
            let arc = on_close_handlers.clone();
            app.upgrade_in_event_loop(move |app| {
                for handler in arc.iter() {
                    handler.call(&app);
                }
                app.window().hide().expect("Failed to hide window.");
            })
            .unwrap();
        }
    };

    let open_window_shared = RwLock::new(open_window);

    // Setup window openers
    let poller_for_open = poller::Poller::new(move || {
        let open_window = open_window_shared.read().unwrap();
        for handler in openers.iter() {
            if handler.has_notified().is_some() {
                open_window();
            }
        }
    });

    let close_window_shared = RwLock::new(close_window);

    let poller_for_close = poller::Poller::new(move || {
        let close_window = close_window_shared.read().unwrap();
        for handler in closers.iter() {
            if handler.has_notified().is_some() {
                close_window();
            }
        }
    });

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
