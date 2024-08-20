use caret_locator::get_caret_pos;
use keyhandler::KeyHandler;
use slint::{ModelRc, VecModel};
use slint_generatedAppWindow::Emoji as EmojiModel;
use std::{path::Path, sync::mpsc};
use thiserror::Error;

mod caret_locator;
mod no_activate;
mod keyhandler;

slint::include_modules!();

struct EmojiWrapper<'a>(&'a emojis::Emoji);

impl<'a> EmojiWrapper<'a> {
    fn get_filename(&self) -> String {
        let chars: Vec<u32> = self.0.to_string().chars().map(u32::from).collect();
        let codes: Vec<u32> = if !chars.contains(&0x200D) {
            chars.into_iter().filter(|c| *c != 0xFE0F).collect()
        } else {
            chars
        };
        let codes: Vec<String> = codes
            .into_iter()
            .map(|e| format!("{:0>4x}", e))
            .collect();
        codes.join("-")
    }
}

#[derive(Error, Debug)]
enum EmojiError {
    #[error("Could not load emoji image.")]
    LoadEmojiImageError(#[from] slint::LoadImageError),
}

impl<'a> TryInto<EmojiModel> for &EmojiWrapper<'a> {
    type Error = EmojiError;

    fn try_into(self) -> Result<EmojiModel, Self::Error> {
        // TODO: eye in speech bubble 1f441-fe0f-200d-1f5e8-fe0f does not work. Issue with (un)qualified sequences.
        let path = format!("./twemoji/assets/svg/{}.svg", self.get_filename());
        let image: Result<slint::Image, slint::LoadImageError> =
            slint::Image::load_from_path(Path::new(&path));
        image
            .map(|i| EmojiModel {
                code: self.0.to_string().into(),
                image: i,
                name: self.0.name().into(),
            })
            .map_err(EmojiError::LoadEmojiImageError)
    }
}

fn main() {
    let ui = AppWindow::new().expect("Could not create window.");

    let mut emojis_vec: Vec<EmojiWrapper> = Vec::with_capacity(1024);
    for emoji in emojis::iter() {
        emojis_vec.push(EmojiWrapper(emoji));
    }
    let emojis_vec = emojis_vec;

    let emojis: Vec<EmojiModel> = emojis_vec.iter().flat_map(|e| e.try_into()).collect();
    let emojis_model = VecModel::from(emojis.clone());

    ui.set_emojis(ModelRc::new(emojis_model));

    ui.on_filter({
        let ui = ui.as_weak().unwrap();
        move |s| {
            let filtered: Vec<EmojiModel> = emojis
                .clone()
                .into_iter()
                .filter(|e| e.name.contains(s.as_str()))
                .collect();
            ui.set_emojis(ModelRc::new(VecModel::from(filtered)));
        }
    });

    // Setup-ing hook.
    let hook_result = KeyHandler::hook();

    if hook_result.is_err() {
        eprintln!("Could not add key handler.");
    }

    // Run the UI.
    ui.show().expect("Could not show app.");

    // Setuping listener of messages from hook.
    no_activate::setup(ui.window());

    let (stop_tx, stop_rx) = mpsc::sync_channel::<()>(1);
    let (hook, rx) = hook_result.ok().unzip();
    // Setup thread to listen for hook messages.
    let listener = rx.map(|rx| {
        std::thread::spawn({
            let ui = ui.as_weak();
            move || loop {
                if rx.try_recv().is_ok() {
                    println!("Received message.");
                    let _ = ui.upgrade_in_event_loop(|a| {
                        if let Some(position) = get_caret_pos() {
                            a.window().set_position(position);
                        }
                        // a.window().show().expect("OK");
                        a.show().expect("OK2");
                        no_activate::setup(a.window());
                    });
                }
                if stop_rx.try_recv().is_ok() {
                    break;
                }
            }
        })
    });

    slint::run_event_loop_until_quit().expect("Could not run event loop.");

    // Unhook the key handler.
    if let Some(hook) = hook {
        if let Err(e) = hook.unhook() {
            eprintln!("Could not unhook key handler: {:?}", e);
        }
    }
    // Stop the listener thread
    if let Some(listener) = listener {
        if let Err(e) = stop_tx.send(()) {
            eprintln!("Could not send stop signal to listener thread: {:?}", e);
        }
        if let Err(e) = listener.join() {
            eprintln!("Could not join listener thread: {:?}", e);
        }
    }
}
