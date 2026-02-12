use gpui::{
    point, size, AnyElement, App, AvailableSpace, Element, GlobalElementId, Hitbox,
    InspectorElementId, Interactivity, IntoElement, IsZero, LayoutId, Pixels, Point, Size, Window,
};

pub struct DynamicGrid {
    delegate: Box<dyn GridDelegate>,
    render_item: Box<dyn Fn(usize, &mut Window, &mut App) -> AnyElement>,
    interactivity: Interactivity,
}

pub trait GridDelegate {
    fn len(&self) -> usize;
}

impl DynamicGrid {
    pub fn new(
        delegate: impl 'static + GridDelegate,
        render_item: impl 'static + Fn(usize, &mut Window, &mut App) -> AnyElement,
    ) -> DynamicGrid {
        DynamicGrid {
            delegate: Box::new(delegate),
            render_item: Box::new(render_item),
            interactivity: Interactivity::default(),
        }
    }

    fn measure_item_size(&self, window: &mut Window, cx: &mut App) -> Size<Pixels> {
        let count = self.delegate.len();

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
        let items = self.delegate.len();
        let item_size = self.measure_item_size(window, cx);
        let layout_id = self.interactivity.request_layout(
            global_id,
            inspector_id,
            window,
            cx,
            |style, window, cx| {
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

                        // Compute desired height, if constrained, then clamp (it's alright
                        // vertically)
                        let desired_height = match available_space.height {
                            AvailableSpace::Definite(h) => (item_size.height * rows).min(h),
                            AvailableSpace::MinContent | AvailableSpace::MaxContent => {
                                item_size.height * rows
                            }
                        };
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
        let n = self.delegate.len() as u32;
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

        // TODO: we don't care for now!
        self.interactivity.prepaint(
            id,
            inspector_id,
            bounds,
            content_size,
            window,
            cx,
            |_style, _, hitbox, window, cx| {
                if self.delegate.len() > 0 {
                    let mut items: Vec<AnyElement> = (0..self.delegate.len())
                        .map(|x| (self.render_item)(x, window, cx))
                        .collect();
                    for (i, item) in items.iter_mut().enumerate() {
                        let available_space = size(
                            AvailableSpace::Definite(item_size.width),
                            AvailableSpace::Definite(item_size.height),
                        );
                        item.layout_as_root(available_space, window, cx);
                        let row_index = i / (cols as usize);
                        let col_index = i % (cols as usize);
                        let x = item_size.width * col_index;
                        let y = item_size.height * row_index;
                        item.prepaint_at(point(x, y), window, cx);
                    }
                    request_layout.items = items;
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
