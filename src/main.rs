use std::{ collections::BTreeMap, sync::{ Arc, RwLock } };
use slint::{ ModelRc, SharedString, VecModel };

mod emoji;
mod handlers;
mod poller;
slint::include_modules!();

fn main() {
    use handlers::*;

    let app = EmojiPickerWindow::new().expect("Failed to create window.");
    let on_close_handlers: Vec<Handler<EmojiPickerWindow>> = vec![(Handler::new(|app: &EmojiPickerWindow| { println!("Closed"); }))];
    let on_open_handlers: Vec<Handler<EmojiPickerWindow>> = vec![(Handler::new(|app: &EmojiPickerWindow| { println!("Opened"); }))];
    let mut openers: Vec<Box<dyn Notifier<()> + Send + Sync>> = vec![];

    // Open the window on start
    if true {
        // A slint's weak reference is not Sync, and making a Mutex
        // for ONLY that is not really interesting. 
        openers.push(Box::new(OnceNotifier::new(())))
    }

    let on_close_handlers = Arc::new(on_close_handlers);
    let on_open_handlers = Arc::new(on_open_handlers);
    let openers = Arc::new(openers);

    init_emojis(&app);
    
    // Setup close handlers
    app.window().on_close_requested({
        let app = app.as_weak();
        move || {
            let app = app.upgrade().unwrap();
            for handler in on_close_handlers.iter() {
                handler.call(&app);
            }
            slint::CloseRequestResponse::HideWindow
        }
    });

    let open_window = {
        let app = app.as_weak();
        move || {
            let arc = on_open_handlers.clone();
            app.upgrade_in_event_loop(move |app| {
                for handler in arc.iter() {
                    handler.call(&app);
                }
                app.window().show().expect("Failed to show window.");
            }).unwrap();
        }
    };

    let open_window_shared = RwLock::new(Box::new(open_window));

    // Setup window openers
    let poller_for_open = poller::Poller::new(move || {
        let open_window = open_window_shared.read().unwrap();
        for handler in openers.iter() {
            if let Some(_) = handler.has_notified() {
                open_window();
            }
        }
    });
    
    slint::run_event_loop_until_quit().expect("Failed to run event loop.");

    poller_for_open.join();
}

/// This function initializes the emoji buttons in the app.
/// It also sets up the filter function to filter the emojis
fn init_emojis(app: &EmojiPickerWindow) {
    use emoji::*;

    let emojis: BTreeMap<_, EmojiModel> = {
        list_emojis().into_iter().flat_map(|(key, emoji)| {
            let filename = emoji.get_filename_path();
            let image = slint::Image::load_from_path(&filename);

            image.ok().map(|image| (key, EmojiModel {
                name: emoji.name().into(),
                code: emoji.code().into(),
                image
            }))
        }).collect()
    };
    let weak_app = app.as_weak();

    // Probably not the best way to redefine a new vec each time and re-updating
    // the VecModel but it may be a good thing to…
    // TODO: Use a FilterModel
    let filter = move |search: SharedString| {
        let mut emoji_buttons = Vec::new();
        let search = search.to_lowercase();
        let filtered_emojis = emojis.iter().filter(|(_, emoji)| emoji.name.contains(&search));
        for (_, emoji) in filtered_emojis {
            emoji_buttons.push(emoji.clone());
        };
        weak_app.upgrade().unwrap().set_emojis(ModelRc::new(VecModel::from(emoji_buttons)));
    };
    
    filter("".into());
    app.on_filter(filter);
}