use gpui::{
    div, rgb, App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement,
    Render, StatefulInteractiveElement, Styled, Window,
};

use crate::{
    provider::EmojiProvider,
    ui::grid::{DynamicGrid, DynamicGridScrollHandle, DynamicGridView, GridScrollbar},
};
mod dwm;
mod emoji_button;
mod grid;

#[derive(Clone)]
struct EmojiGridView {
    provider: Entity<EmojiProvider>,
    scroll_handle: grid::DynamicGridScrollHandle,
}

impl EmojiGridView {
    fn new(provider: Entity<EmojiProvider>) -> EmojiGridView {
        EmojiGridView {
            provider,
            scroll_handle: Default::default(),
        }
    }
}

impl DynamicGridView for EmojiGridView {
    fn len(&self, section: usize, cx: &App) -> usize {
        let provider = self.provider.read(cx);
        let cat_name = provider.categories_order.get(section);
        if let Some(cat_name) = cat_name {
            provider.categories.get(cat_name).unwrap().len()
        } else {
            0
        }
    }

    fn number_sections(&self, cx: &App) -> usize {
        let provider = self.provider.read(cx);
        provider.categories_order.len()
    }

    fn scroll_handle(&self) -> Option<DynamicGridScrollHandle> {
        Some(self.scroll_handle.clone())
    }
}

pub struct MainWindow {
    provider: Entity<EmojiProvider>,
    grid_view: EmojiGridView,
}

impl MainWindow {
    pub fn new(cx: &mut Context<Self>, window: &mut Window, provider: EmojiProvider) -> MainWindow {
        let provider = cx.new(move |_| provider);

        let grid_view = EmojiGridView::new(provider.clone());
        MainWindow {
            provider,
            grid_view,
        }
    }

    pub fn enable_acrylic_effect(&self, window: &mut Window, cx: &mut Context<Self>) {
        let _ = dwm::apply_acrilic(window, cx)
            .inspect_err(|e| println!("Failed to set Acrylic effect: {e}"));
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let grid = DynamicGrid::new(
            self.grid_view.clone(),
            {
                let provider = self.provider.clone();

                move |i, s, _w, cx| {
                    let provider = provider.read(cx);

                    // TODO: add checks!
                    let category = provider.get_category_from_index(s).unwrap();
                    let emoji = category.get(i).unwrap();

                    div()
                        .id(format!("emoji-{:?}-{:?}", s, i))
                        .child(format!("{:?}", emoji))
                        .into_any_element()
                }
            },
            {
                let provider = self.provider.clone();

                Some(move |s, _w: &mut Window, cx: &mut App| {
                    let provider = provider.read(cx);
                    let category = provider.get_category_name_from_index(s).unwrap();
                    div().child(category.clone()).into_any_element()
                })
            },
        );

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
                    .child(GridScrollbar::new(self.grid_view.clone()))
                    .child(grid.border_2().border_color(rgb(0xffff00)).size_full()),
            )
            .child(div().child("text"))
    }
}
