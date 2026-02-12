use gpui::{App, AppContext, Application, WindowOptions};

mod provider;
mod ui;

use ui::MainWindow;

fn main() {
    Application::new().run(|cx: &mut App| {
        // let provider = EmojiProvider::from_default().unwrap();

        let handle = cx
            .open_window(
                WindowOptions {
                    window_background: gpui::WindowBackgroundAppearance::MicaBackdrop,
                    ..Default::default()
                },
                |window, cx| {
                    let r = cx.new(|cx| MainWindow::new(cx, window));
                    r.update(cx, |main_window, cx| {
                        main_window.enable_acrylic_effect(window, cx)
                    });
                    r
                },
            )
            .expect("Did not create the window successfully.");
    });
}
