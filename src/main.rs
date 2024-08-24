use std::{collections::BTreeMap, sync::{atomic::{AtomicBool, Ordering}, Arc}};
use slint::{ ModelRc, SharedString, VecModel };

mod emoji;
mod handlers;
mod poller;
slint::include_modules!();

fn main() {
    use handlers::*;

    let (tx, rx) = std::sync::mpsc::sync_channel::<()>(10);
    
    let app = EmojiPickerWindow::new().expect("Failed to create window.");
    let mut on_close_handler = vec![(OnCloseHandler::new(on_close_request))];
    let mut on_open_handler = vec![(OnOpenHandler::new(|app: EmojiPickerWindow| { println!("Opened"); }))];
    let mut openers = vec![BasicOpener::new(rx)];

    init_emojis(&app);
    
    // Setup close handlers
    app.window().on_close_requested({
        let app = app.as_weak();
        move || {
            /*
            let app = app.clone();
            on_close_handler.iter_mut().for_each({
                let app2 = app.clone().upgrade().unwrap();
                |f| f.call(&app2)
            }); */
            let app = app.upgrade().unwrap();
            for handler in on_close_handler.iter_mut() {
                handler.call(&app);
            }
            slint::CloseRequestResponse::HideWindow
        }
    });

    let stop_threads_flag = Arc::new(AtomicBool::new(false));
    // Setup openers
    let opener_listener = std::thread::spawn({
        let stop = stop_threads_flag.clone();
        move || loop {
            if stop.load(Ordering::SeqCst) { break; };
            for i in openers.iter() {
                if let Some(_) = i.has_requested_open() {
                    println!("Open request.");
                }
            }
        }
    });

    

    slint::run_event_loop_until_quit();
}

fn on_close_request(app: &EmojiPickerWindow)  {
    
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