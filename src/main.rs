use std::collections::BTreeMap;
use slint::{ ModelRc, SharedString, VecModel };

mod emoji;
slint::include_modules!();

fn main() {
    let app = EmojiPickerWindow::new().expect("Failed to create window.");

    init_emojis(&app);

    app.run().expect("Failed to run the app.");
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