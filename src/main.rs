use handlers::Handlers;
use slint::{ModelRc, SharedString, VecModel};
use std::{
    collections::{BTreeMap, HashMap}, iter::Map, sync::{Arc, RwLock}
};

mod emoji;
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

// TODO: dissociate all below into a separate file

/// An equivalent of EmojiGroupModel's from Slint in Rust
struct EmojiGroupModelR {
    title: String,
    emojis: Vec<EmojiModel>,
}

impl EmojiGroupModelR {
    fn new(title: String, emojis: Vec<EmojiModel>) -> Self {
        Self { title, emojis }
    }
}

impl Default for EmojiGroupModelR {
    fn default() -> Self {
        Self {
            title: String::new(),
            emojis: Vec::new(),
        }
    }
}

impl From<EmojiGroupModelR> for EmojiGroupModel {
    fn from(model: EmojiGroupModelR) -> Self {
        Self {
            title: model.title.into(),
            emojis: ModelRc::new(VecModel::from(model.emojis)),
        }
    }
}

/// This function initializes the emoji buttons in the app.
/// It also sets up the filter function to filter the emojis
fn init_emojis(app: &EmojiPickerWindow) {
    use emoji::*;

    let emojis_groupped: HashMap<EmojiGroupWrapper, Vec<EmojiModel>> = {
        list_emojis()
            .into_iter()
            .fold(HashMap::new(), |mut acc, emoji| {
                let filename = emoji.get_filename_path();
                let image = slint::Image::load_from_path(&filename);

                if let Some(image) = image.ok() {
                    let model = EmojiModel {
                        name: emoji.name().into(),
                        code: emoji.code().into(),
                        image,
                    };

                    acc.entry(emoji.group()).or_insert_with(Vec::new).push(model);
                }

                acc
            })
    };

    let mut almost_model: Vec<EmojiGroupModel> = emojis_groupped.clone()
        .into_iter()
        .map(|(group, emojis)| EmojiGroupModelR::new(group.group_name().into(), emojis).into())
        .collect::<Vec<_>>();

    let nb_groups = almost_model.len();
    let model = ModelRc::new(VecModel::from(almost_model));
    app.set_emoji_groups(model);
    
    let scroll_model = ModelRc::new(VecModel::from(0..nb_groups));
    app.window().

    let weak_app = app.as_weak();

    // Probably not the best way to redefine a new vec each time and re-updating
    // the VecModel but it may be a good thing to…
    // TODO: Use a FilterModel (it may be even better to implement a custom one.)
    let filter = move |search: SharedString| {
        // let mut emoji_buttons = Vec::new();
        // let search = search.to_lowercase();
        // let filtered_emojis = emojis
        //     .iter()
        //     .filter(|(_, emoji)| emoji.name.contains(&search));
        // for (_, emoji) in filtered_emojis {
        //     emoji_buttons.push(emoji.clone());
        // }
        // weak_app
        //     .upgrade()
        //     .unwrap()
        //     .set_emojis(ModelRc::new(VecModel::from(emoji_buttons)));
    };

    // app.set_emoji_groups(ModelRc::new(VecModel::from()))
}
