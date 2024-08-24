use std::{collections::BTreeMap, sync::{atomic::{AtomicBool, Ordering}, Arc}};
use slint::{ ModelRc, SharedString, VecModel };

mod emoji;
mod handlers;
mod poller;
slint::include_modules!();

fn open_window(app: &EmojiPickerWindow) {

}

fn main() {
    use handlers::*;

    let (tx, rx) = std::sync::mpsc::sync_channel::<()>(10);
    
    let app = EmojiPickerWindow::new().expect("Failed to create window.");
    let mut on_close_handlers: Vec<Handler<EmojiPickerWindow>> = vec![(Handler::new(|app: &EmojiPickerWindow| { println!("Closed"); }))];
    let mut on_open_handlers: Vec<Handler<EmojiPickerWindow>> = vec![(Handler::new(|app: &EmojiPickerWindow| { println!("Opened"); }))];
    let mut openers: Vec<Box<dyn Notifier<()> + Send + Sync>> = vec![];

    init_emojis(&app);
    
    // Setup close handlers
    app.window().on_close_requested({
        let app = app.as_weak();
        move || {
            let app = app.upgrade().unwrap();
            for handler in on_close_handlers.iter_mut() {
                handler.call(&app);
            }
            slint::CloseRequestResponse::HideWindow
        }
    });

    let open_window = {
        let app = app.as_weak();
        move || {
            let app = app.upgrade_in_event_loop(move |app| {
                for handler in on_open_handlers {
                    handler.call(&app);
                }
                app.window().show();
            });
        }
    };

    // Setup openers
    let open_poller = poller::Poller::new({
        let app = app.as_weak();
        move || {
            for handler in openers.iter_mut() {
                if let Some(_) = handler.has_notified() {
                    open_window()
                    // slint::invoke_from_event_loop(open_window);
                }
            }
        }
    });

    
    open_window();
    slint::run_event_loop_until_quit();
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