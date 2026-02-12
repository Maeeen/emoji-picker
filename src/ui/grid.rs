use gpui::{
    div, fill, point, px, relative, rgb, size, solid_background, AnyElement, App, AvailableSpace,
    Background, BorderStyle, Bounds, ContentMask, Corners, DispatchPhase, Element, GlobalElementId,
    Hitbox, HitboxBehavior, Hsla, InspectorElementId, Interactivity, IntoElement, IsZero, LayoutId,
    MouseDownEvent, Pixels, Position, Render, Rgba, Size, Style, Styled, Window,
};
use gpui::{Overflow, StyleRefinement};
use std::{cell::RefCell, rc::Rc};

type ItemRender = dyn Fn(usize, &mut Window, &mut App) -> AnyElement;

pub struct DynamicGrid {
    model: Box<dyn DynamicGridView>,
    render_item: Box<ItemRender>,
    interactivity: Interactivity,
}

struct DynamicGridScrollState {
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

            state.offset += delta.y;
            state.clamp_on_known_bounds();
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
                    let content_mask = ContentMask { bounds };

                    window.with_content_mask(Some(content_mask), |window| {
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
                    })
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

pub struct GridScrollbar {
    scroll_handle: DynamicGridScrollHandle,
}

impl GridScrollbar {
    pub fn new(view: impl 'static + DynamicGridView) -> GridScrollbar {
        GridScrollbar {
            scroll_handle: view.scroll_handle(),
        }
    }
}

impl IntoElement for GridScrollbar {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

pub struct GridScrollbarPrepaintState {
    hitbox: Option<Hitbox>,
}

impl Element for GridScrollbar {
    type RequestLayoutState = ();

    type PrepaintState = GridScrollbarPrepaintState;

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let style = Style {
            position: Position::Absolute,
            flex_grow: 1.0,
            flex_shrink: 1.0,
            size: size(relative(1.).into(), relative(1.0).into()),
            ..Style::default()
        };

        (window.request_layout(style, None, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        _: &mut App,
    ) -> Self::PrepaintState {
        let scroll_handle = self.scroll_handle.0.borrow();
        let vp = scroll_handle.viewport.size;
        let cs = scroll_handle.content_size;

        if vp.height >= cs.height {
            return GridScrollbarPrepaintState { hitbox: None };
        }

        let scroll_ratio = (-scroll_handle.offset) / (cs.height - vp.height);

        let sb_width: Pixels = px(8.);
        let sb_height: Pixels = px(vp.height.pow(2.) / cs.height);

        let x = bounds.size.width - sb_width;
        let y = scroll_ratio * (bounds.size.height - sb_height);

        let bar_hitbox = window.with_content_mask(Some(ContentMask { bounds }), |window| {
            window.insert_hitbox(
                Bounds::new(point(x, y), size(sb_width, sb_height)),
                HitboxBehavior::Normal,
            )
        });

        GridScrollbarPrepaintState {
            hitbox: Some(bar_hitbox),
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        _: &mut App,
    ) {
        if let Some(hitbox) = prepaint.hitbox.as_ref() {
            let thumb_bounds = hitbox.bounds;
            let sb_bounds = Bounds {
                origin: point(thumb_bounds.origin.x, bounds.origin.y),
                size: size(thumb_bounds.size.width, bounds.size.height),
            };

            window.paint_quad(gpui::PaintQuad {
                bounds: thumb_bounds.inset(px(1.)),
                corner_radii: Corners::all(px(2.)),
                background: solid_background(Hsla::from(Rgba {
                    r: 255.,
                    g: 255.,
                    b: 255.,
                    a: 0.5,
                })),
                border_widths: gpui::Edges::default(),
                border_color: Default::default(),
                border_style: BorderStyle::Solid,
            });

            window.on_mouse_event::<MouseDownEvent>({
                let scroll_handle = self.scroll_handle.clone();
                move |event, phase, window, cx| {
                    if phase.bubble() && sb_bounds.contains(&event.position) {
                        cx.stop_propagation();

                        if !thumb_bounds.contains(&event.position) {
                            // Outside of thumb, need to scroll exactly
                            let mut scroll_handle = scroll_handle.0.borrow_mut();
                            let clicked_percentage = (event.position.y
                                - thumb_bounds.size.height / 2.)
                                / bounds.size.height;
                            let new_offset = clicked_percentage * scroll_handle.content_size.height;

                            scroll_handle.offset = -new_offset;
                            scroll_handle.clamp_on_known_bounds();
                            window.refresh();
                        } else {
                            // need to do something annoying…
                        }
                    }
                }
            });
        }
    }
}
