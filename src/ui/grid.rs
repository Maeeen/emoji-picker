use gpui::{
    point, size, AnyElement, App, AvailableSpace, Bounds, ContentMask, Element, GlobalElementId,
    Hitbox, InspectorElementId, Interactivity, IntoElement, IsZero, LayoutId, Pixels, Point,
    PointRefinement, ScrollHandle, Size, Styled, Window,
};
use gpui::{Overflow, StatefulInteractiveElement, StyleRefinement};
use std::{cell::RefCell, rc::Rc}; // provides .track_scroll()

pub struct DynamicGrid {
    model: Box<dyn DynamicGridView>,
    render_item: Box<dyn Fn(usize, &mut Window, &mut App) -> AnyElement>,
    interactivity: Interactivity,
}

struct DynamicGridScrollState {
    base_handle: ScrollHandle,
    offset: Pixels,
    viewport: Bounds<Pixels>,
    content_size: Size<Pixels>,
}

impl DynamicGridScrollState {
    fn clamp_on_known_bounds(&mut self) {
        self.clamp_on_bounds(self.viewport, self.content_size);
    }

    /// Update the scroll if we're out of bounds.
    fn clamp_on_bounds(&mut self, bounds: Bounds<Pixels>, content_size: Size<Pixels>) {
        let viewport_size = bounds.size;
        let min_scroll = (content_size - viewport_size).height;
        self.offset = self.offset.max(-min_scroll).min(Pixels::ZERO);
    }
}

#[derive(Clone)]
pub struct DynamicGridScrollHandle(Rc<RefCell<DynamicGridScrollState>>);

impl DynamicGridScrollHandle {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(DynamicGridScrollState {
            base_handle: ScrollHandle::new(),
            content_size: size(Pixels::ZERO, Pixels::ZERO),
            offset: Pixels::ZERO,
            viewport: Bounds::default(),
        })))
    }
}

pub trait DynamicGridView {
    fn len(&self) -> usize;

    fn scroll_handle(&self) -> DynamicGridScrollHandle;
}

impl DynamicGrid {
    pub fn new(
        view: impl 'static + DynamicGridView,
        render_item: impl 'static + Fn(usize, &mut Window, &mut App) -> AnyElement,
    ) -> DynamicGrid {
        let mut grid = DynamicGrid {
            model: Box::new(view),
            render_item: Box::new(render_item),
            interactivity: Interactivity::new(),
        };

        let scroll_handle = grid.model.scroll_handle().clone();

        grid.interactivity.on_scroll_wheel(move |e, window, _| {
            let mut state = scroll_handle.0.borrow_mut();

            let delta = e.delta.pixel_delta(Pixels::from(12.0));

            let max_scroll = state.content_size.height - state.viewport.size.height;
            let add = state.offset + delta.y;
            println!(
                "{} + {} = {} (min = {}, max = {}). difference is = {}",
                state.offset, delta.y, add, -max_scroll, 0, max_scroll
            );
            state.offset = (state.offset + delta.y);
            state.clamp_on_known_bounds();
            println!("set offset to {:?} ( + {:?}", state.offset, delta.y);
            window.refresh();
        });
        grid
    }

    fn measure_item_size(&self, window: &mut Window, cx: &mut App) -> Size<Pixels> {
        let count = self.model.len();

        if count == 0 {
            return Size::default();
        }

        let mut item = (self.render_item)(0, window, cx);
        let available_space = size(AvailableSpace::MinContent, AvailableSpace::MinContent);
        item.layout_as_root(available_space, window, cx)
    }
}

pub struct DynamicGridFrameState {
    // TODO: use SmallVec
    items: Vec<AnyElement>,
}

impl IntoElement for DynamicGrid {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Styled for DynamicGrid {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.interactivity.base_style
    }
}

impl Element for DynamicGrid {
    type RequestLayoutState = DynamicGridFrameState;
    type PrepaintState = Option<Hitbox>;

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let items = self.model.len();
        let item_size = self.measure_item_size(window, cx);
        let layout_id = self.interactivity.request_layout(
            global_id,
            inspector_id,
            window,
            cx,
            |mut style, window, cx| {
                style.overflow.y = Overflow::Hidden;
                window.request_measured_layout(
                    style,
                    move |known_dimensions, available_space, _window, _cx| {
                        // Infer the available width
                        let max_width = known_dimensions.width.unwrap_or_else(|| {
                            match available_space.width {
                                AvailableSpace::Definite(p) => p,
                                AvailableSpace::MinContent | AvailableSpace::MaxContent => {
                                    // no horizontal constraint
                                    item_size.width * items as f32
                                }
                            }
                        });

                        // Infer number of columns, do not divide by zero.
                        let cols = if item_size.width > Pixels::ZERO {
                            (max_width / item_size.width).floor().max(1.0) as usize
                        } else {
                            1
                        };

                        // Infer number of rows
                        // Round up the number of rows
                        let rows = items.div_ceil(cols);

                        let desired_width = match available_space.width {
                            // We have already taken the constraint on the width
                            AvailableSpace::Definite(_) => max_width,
                            AvailableSpace::MinContent | AvailableSpace::MaxContent => {
                                item_size.width * cols
                            }
                        };

                        // Compute desired height
                        let desired_height =
                            known_dimensions
                                .height
                                .unwrap_or(match available_space.height {
                                    AvailableSpace::Definite(h) => (item_size.height * rows).max(h),
                                    AvailableSpace::MinContent | AvailableSpace::MaxContent => {
                                        item_size.height * rows
                                    }
                                });

                        size(desired_width, desired_height)
                    },
                )
            },
        );

        (layout_id, DynamicGridFrameState { items: Vec::new() })
    }

    fn id(&self) -> Option<gpui::ElementId> {
        self.interactivity.element_id.clone()
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: gpui::Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let n = self.model.len() as u32;
        let item_size = self.measure_item_size(window, cx);

        let cols = if item_size.width.is_zero() {
            1
        } else {
            (bounds.size.width / item_size.width) as u32
        };
        let rows = n.div_ceil(cols);

        let content_size = size(
            item_size.width * (cols as usize),
            (rows as usize) * item_size.height,
        );

        let scroll_handle = self.model.scroll_handle();
        let mut scroll_handle = scroll_handle.0.borrow_mut();

        // Ensure that the scroll is valid when resized
        scroll_handle.content_size = content_size;
        scroll_handle.viewport = bounds;
        scroll_handle.clamp_on_known_bounds();

        let scroll_offset = scroll_handle.offset;

        // TODO: we don't care for now!
        self.interactivity.prepaint(
            id,
            inspector_id,
            bounds,
            content_size,
            window,
            cx,
            |_style, _, hitbox, window, cx| {
                if self.model.len() > 0 {
                    // Set a content mask to avoid setting click handlers where they should not be
                    // let content_mask = ContentMask { bounds };
                    //
                    // window.with_content_mask(Some(content_mask), |window| {
                    let top_row_index = (-scroll_offset / item_size.height) as u32;
                    // Substracting to show mid-visible elements
                    let first_visible_element_idx = top_row_index.saturating_sub(1u32) * cols;
                    let nb_visible_rows = (bounds.size.height / item_size.height) as u32 + 2;
                    let last_visible_element_idx = (first_visible_element_idx
                        + nb_visible_rows * cols)
                        .min(self.model.len() as u32);

                    let items_iterator = first_visible_element_idx..last_visible_element_idx;

                    let mut items: Vec<AnyElement> = items_iterator
                        .map(|x| (self.render_item)(x as usize, window, cx))
                        .collect();

                    for (i, item) in items.iter_mut().enumerate() {
                        let i = i + (first_visible_element_idx as usize);
                        let available_space = size(
                            AvailableSpace::Definite(item_size.width),
                            AvailableSpace::Definite(item_size.height),
                        );
                        item.layout_as_root(available_space, window, cx);
                        let row_index = i / (cols as usize);
                        let col_index = i % (cols as usize);
                        let x = bounds.origin.x + item_size.width * col_index;
                        let y = bounds.origin.y + item_size.height * row_index + scroll_offset;
                        item.prepaint_at(point(x, y), window, cx);
                    }
                    request_layout.items = items;
                    // })
                }
                hitbox
            },
        )
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: gpui::Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let hitbox = prepaint;
        self.interactivity.paint(
            id,
            inspector_id,
            bounds,
            hitbox.as_ref(),
            window,
            cx,
            |_, window, cx| {
                for item in request_layout.items.iter_mut() {
                    item.paint(window, cx)
                }
            },
        )
    }
}
