use std::collections::{HashMap, HashSet};

use gpui::{
    div, App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
};

use crate::{
    provider::{Emoji, EmojiCategory, EmojiProvider},
    ui::grid::{DynamicGrid, GridDelegate},
};
mod dwm;
mod grid;

pub struct EmojiListDelegate {
    provider: Entity<EmojiProvider>,
}

impl EmojiListDelegate {
    fn new(provider: &Entity<EmojiProvider>) -> EmojiListDelegate {
        EmojiListDelegate {
            provider: provider.clone(),
        }
    }
}

// impl ListDelegate for EmojiListDelegate {
//     type Item = ListItem;
//
//     fn sections_count(&self, cx: &App) -> usize {
//         self.provider.read(cx).categories.len()
//     }
//
//     fn items_count(&self, section: usize, cx: &App) -> usize {
//         let provider = self.provider.read(cx);
//         let cat_name = provider.categories_order.get(section);
//         let cat = cat_name.and_then(|x| provider.categories.get(x));
//
//         match cat {
//             None => 0,
//             Some(x) => x.len(),
//         }
//     }
//
//     fn render_section_header(
//         &mut self,
//         section: usize,
//         _window: &mut Window,
//         cx: &mut Context<ListState<Self>>,
//     ) -> Option<impl IntoElement> {
//         let title = self.provider.read(cx).categories_order.get(section)?;
//
//         Some(h_flex().px().py_1().gap_2().text_sm().child(title.clone()))
//     }
//
//     fn render_item(
//         &mut self,
//         ix: IndexPath,
//         window: &mut Window,
//         cx: &mut Context<ListState<EmojiListDelegate>>,
//     ) -> Option<ListItem> {
//         Some(ListItem::new(ix).child(div().child(format!("{:?}", ix))))
//     }
//
//     fn set_selected_index(
//         &mut self,
//         ix: Option<IndexPath>,
//         window: &mut Window,
//         cx: &mut Context<ListState<Self>>,
//     ) {
//         println!("Setting set_selected_index at {ix:?}")
//     }
// }

pub struct MainWindow {
    // provider: Entity<EmojiProvider>,
    // emoji_list: Entity<ListState<EmojiListDelegate>>,
}

impl MainWindow {
    pub fn new(
        cx: &mut Context<Self>,
        window: &mut Window,
        // provider: EmojiProvider
    ) -> MainWindow {
        // let provider = cx.new(move |_| provider);
        // let emoji_list = cx.new(|cx| ListState::new(EmojiListDelegate::new(&provider), window, cx));
        MainWindow {
            // provider,
            // emoji_list,
        }
    }

    pub fn enable_acrylic_effect(&self, window: &mut Window, cx: &mut Context<Self>) {
        dwm::apply_acrilic(window, cx);
    }
}

struct DummyDelegate {}

impl GridDelegate for DummyDelegate {
    fn len(&self) -> usize {
        92
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let grid = DynamicGrid::new(DummyDelegate {}, |i, w, cx| {
            if i == 0 {
                return div().child("merde").into_any_element();
            }
            div()
                .child(format!("{}", char::from_u32((i + 33) as u32).unwrap()))
                .into_any_element()
        });
        // div().child(grid)
        div().child("Hello world!")
        // div()
        //     .v_flex()
        //     .gap_2()
        //     .size_full()
        //     .items_center()
        //     .justify_center()
        //     .child("Hello2, World!")
        //     .child(
        //         Button::new("ok")
        //             .primary()
        //             .label("Let's Go!")
        //             .on_click(|_, _, _| println!("Clicked!")),
        //     )
        //     .child(List::new(&self.emoji_list))
    }
}
