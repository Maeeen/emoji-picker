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

// use gpui::{
//     div, prelude::*, px, rgb, size, uniform_list, App, Application, Bounds, Context, Window,
//     WindowBounds, WindowOptions,
// };
//
// struct UniformListExample {}
//
// impl Render for UniformListExample {
//     fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//         div().size_full().bg(rgb(0xffffff)).child(
//             uniform_list(
//                 "entries",
//                 50,
//                 cx.processor(|_this, range, _window, _cx| {
//                     let mut items = Vec::new();
//                     for ix in range {
//                         let item = ix + 1;
//
//                         items.push(
//                             div()
//                                 .id(ix)
//                                 .px_2()
//                                 .cursor_pointer()
//                                 .on_click(move |_event, _window, _cx| {
//                                     println!("clicked Item {item:?}");
//                                 })
//                                 .child(format!("Item {item}")),
//                         );
//                     }
//                     items
//                 }),
//             )
//             .h_full(),
//         )
//     }
// }
//
// fn main() {
//     Application::new().run(|cx: &mut App| {
//         let bounds = Bounds::centered(None, size(px(300.0), px(300.0)), cx);
//         cx.open_window(
//             WindowOptions {
//                 window_bounds: Some(WindowBounds::Windowed(bounds)),
//                 ..Default::default()
//             },
//             |_, cx| cx.new(|_| UniformListExample {}),
//         )
//         .unwrap();
//     });
// }
