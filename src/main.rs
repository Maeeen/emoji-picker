use backend_link::{BackendEventKind, BackendEvents, BackendLink};
use handlers::Handlers;
use std::{
    collections::{BTreeMap, HashMap}, iter::Map, sync::{Arc, RwLock}
};

mod emoji;
mod handler;
mod handlers;
mod poller;
mod backend_link;

fn main() {
    use handler::*;

    let app = BackendLink::new().expect("Failed to initialize back end.");

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

    // Setup emoji selected
    app.on(BackendEventKind::EmojiSelected, Arc::new(move |_, e| {
        match e {
            BackendEvents::EmojiSelected(code) => {
                for handler in emoji_selected.iter() {
                    handler.call(&code);
                }
            }
            _ => {}
        }
    }));

    // Setup close handlers
    app.on(BackendEventKind::CloseRequested, {
        let on_close_handlers = on_close_handlers.clone();
        Arc::new(move |app, _| {
            for handler in on_close_handlers.iter() {
                handler.call(&app);
            }
        })
    });

    // Caller to open a window and call the open handlers
    let open_window = move |app: &BackendLink| {
        let on_open_handlers = on_open_handlers.clone();
        let before_open_handlers = before_open_handlers.clone();
        for handler in before_open_handlers.iter() {
            handler.call(&app);
        }
        &app.show();
        for handler in on_open_handlers.iter() {
            handler.call(&app);
        }
    };

    let close_window = move |app: &BackendLink| {
        let arc = on_close_handlers.clone();
        for handler in arc.iter() {
            handler.call(&app);
        }
        &app.hide();
    };

    // Setup window openers
    let poller_for_open = {
        let app = app.clone();
        poller::Poller::new(move || {
            // let open_window = open_window_shared.read().unwrap();
            for handler in openers.iter() {
                if handler.has_notified().is_some() {
                    open_window(&app);
                }
            }
        })
    };

    let close_window_shared = RwLock::new(close_window);

    let poller_for_close = {
        let app = app.clone();
        poller::Poller::new(move || {
            let close_window = close_window_shared.read().unwrap();
            for handler in closers.iter() {
                if handler.has_notified().is_some() {
                    close_window(&app);
                }
            }
        })
    };

    app.run_event_loop();

    // This is not really necessary.
    poller_for_open.signal_stop();
    poller_for_close.signal_stop();
}
