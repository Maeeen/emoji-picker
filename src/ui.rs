use std::collections::{HashMap, HashSet};

use gpui::{
    div, rgb, App, AppContext, Context, Div, Entity, InteractiveElement, IntoElement,
    ParentElement, Render, Stateful, StatefulInteractiveElement, Styled, Window,
};

use crate::{
    provider::{Emoji, EmojiCategory, EmojiProvider},
    ui::grid::{DynamicGrid, DynamicGridScrollHandle, DynamicGridView, GridScrollbar},
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
    grid_state: DummyDelegate, // provider: Entity<EmojiProvider>,
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
            grid_state: DummyDelegate {
                scroll_handle: DynamicGridScrollHandle::default(),
            }, // grid, // provider,
               // emoji_list,
        }
    }

    pub fn enable_acrylic_effect(&self, window: &mut Window, cx: &mut Context<Self>) {
        let _ = dwm::apply_acrilic(window, cx)
            .inspect_err(|e| println!("Failed to set Acrylic effect: {e}"));
    }
}

#[derive(Clone)]
struct DummyDelegate {
    scroll_handle: grid::DynamicGridScrollHandle,
}

impl DynamicGridView for DummyDelegate {
    fn len(&self) -> usize {
        92 * 100
    }

    fn scroll_handle(&self) -> grid::DynamicGridScrollHandle {
        self.scroll_handle.clone()
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let grid = DynamicGrid::new(self.grid_state.clone(), |i, w, cx| {
            let ip = i;
            let i = i % 92;
            if i == 0 {
                return div().child("merde").into_any_element();
            }
            div()
                .id(format!("{:?}", ip))
                .child(format!("{}", char::from_u32((i + 33) as u32).unwrap()))
                .on_click(move |e, w, a| println!("Pressed on {}", ip))
                .into_any_element()
        });

        div()
            .flex()
            .flex_col()
            .size_full()
            .relative()
            .child(div().child("text"))
            .child(
                div()
                    .border_2()
                    .border_color(rgb(0xff0000))
                    .size_full()
                    .relative()
                    .child(GridScrollbar::new(self.grid_state.clone()))
                    .child(grid.border_2().border_color(rgb(0xffff00)).size_full()),
            )
            .child(div().child("text"))
    }
}
