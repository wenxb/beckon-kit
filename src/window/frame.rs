use super::{ClientFrame, FrameGeometry};
use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, Hitbox, HitboxBehavior,
    InspectorElementId, IntoElement, LayoutId, MouseButton, MouseDownEvent, Pixels, ResizeEdge,
    RoundedClip, Style, Window, point, px, size,
};

/// Single paint owner: shadow outside the group clip, content inside, border last.
pub(crate) struct Frame {
    geometry: FrameGeometry,
    style: ClientFrame,
    client: bool,
    clip: RoundedClip,
    child: AnyElement,
}
impl Frame {
    pub fn new(
        geometry: FrameGeometry,
        style: ClientFrame,
        client: bool,
        clip: RoundedClip,
        child: impl IntoElement,
    ) -> Self {
        Self {
            geometry,
            style,
            client,
            clip,
            child: child.into_any_element(),
        }
    }
}
impl IntoElement for Frame {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl Element for Frame {
    type RequestLayoutState = (LayoutId, RoundedClip);
    type PrepaintState = Vec<(Hitbox, ResizeEdge)>;
    fn id(&self) -> Option<ElementId> {
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
        let child = self.child.request_layout(window, cx);
        let mut style = Style::default();
        style.size.width = gpui::relative(1.0).into();
        style.size.height = gpui::relative(1.0).into();
        let g = &self.geometry;
        style.padding.left = g.insets.left.into();
        style.padding.right = g.insets.right.into();
        style.padding.top = g.insets.top.into();
        style.padding.bottom = g.insets.bottom.into();
        let layout = window.request_layout(style, [child], cx);
        (layout, (child, self.clip.clone()))
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        state: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        if self.client && window.is_rounded_clip_supported() {
            window.with_rounded_clip(&state.1, |w| self.child.prepaint(w, cx));
        } else {
            self.child.prepaint(window, cx);
        }
        let mut hitboxes = Vec::new();
        let g = &self.geometry;
        let w = g.resize_width;
        let b = g.visible;
        if w <= px(0.0) {
            return hitboxes;
        }
        let e = &g.insets;
        for (edge, bounds, enabled) in [
            (
                ResizeEdge::Top,
                Bounds::new(
                    point(b.left() + w, b.top() - w),
                    size((b.size.width - w * 2.0).max(px(0.0)), w),
                ),
                e.top > px(0.0),
            ),
            (
                ResizeEdge::Bottom,
                Bounds::new(
                    point(b.left() + w, b.bottom()),
                    size((b.size.width - w * 2.0).max(px(0.0)), w),
                ),
                e.bottom > px(0.0),
            ),
            (
                ResizeEdge::Left,
                Bounds::new(
                    point(b.left() - w, b.top() + w),
                    size(w, (b.size.height - w * 2.0).max(px(0.0))),
                ),
                e.left > px(0.0),
            ),
            (
                ResizeEdge::Right,
                Bounds::new(
                    point(b.right(), b.top() + w),
                    size(w, (b.size.height - w * 2.0).max(px(0.0))),
                ),
                e.right > px(0.0),
            ),
            (
                ResizeEdge::TopLeft,
                Bounds::new(point(b.left() - w, b.top() - w), size(w * 2.0, w * 2.0)),
                e.left > px(0.0) && e.top > px(0.0),
            ),
            (
                ResizeEdge::TopRight,
                Bounds::new(point(b.right() - w, b.top() - w), size(w * 2.0, w * 2.0)),
                e.right > px(0.0) && e.top > px(0.0),
            ),
            (
                ResizeEdge::BottomLeft,
                Bounds::new(point(b.left() - w, b.bottom() - w), size(w * 2.0, w * 2.0)),
                e.left > px(0.0) && e.bottom > px(0.0),
            ),
            (
                ResizeEdge::BottomRight,
                Bounds::new(point(b.right() - w, b.bottom() - w), size(w * 2.0, w * 2.0)),
                e.right > px(0.0) && e.bottom > px(0.0),
            ),
        ] {
            if enabled {
                hitboxes.push((
                    window.insert_hitbox(bounds, HitboxBehavior::BlockMouse),
                    edge,
                ));
            }
        }
        hitboxes
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        state: &mut Self::RequestLayoutState,
        hits: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let g = &self.geometry;
        if self.client {
            if g.insets.top > px(0.0)
                || g.insets.bottom > px(0.0)
                || g.insets.left > px(0.0)
                || g.insets.right > px(0.0)
            {
                window.paint_drop_shadows(g.visible, g.corner_radii, &self.style.shadows);
            }
            if window.is_rounded_clip_supported() {
                window.with_rounded_clip(&state.1, |w| self.child.paint(w, cx));
            } else {
                self.child.paint(window, cx);
            }
            window.paint_quad(
                gpui::outline(g.visible, self.style.border_color, Default::default())
                    .corner_radii(g.corner_radii)
                    .border_widths(g.border_widths),
            );
        } else {
            self.child.paint(window, cx);
        }
        for (hitbox, edge) in hits.iter() {
            let cursor = match edge {
                ResizeEdge::Top | ResizeEdge::Bottom => gpui::CursorStyle::ResizeUpDown,
                ResizeEdge::Left | ResizeEdge::Right => gpui::CursorStyle::ResizeLeftRight,
                ResizeEdge::TopLeft | ResizeEdge::BottomRight => {
                    gpui::CursorStyle::ResizeUpLeftDownRight
                }
                ResizeEdge::TopRight | ResizeEdge::BottomLeft => {
                    gpui::CursorStyle::ResizeUpRightDownLeft
                }
            };
            window.set_cursor_style(cursor, hitbox);
            let (hitbox, edge) = (hitbox.clone(), *edge);
            window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                if phase.bubble() && event.button == MouseButton::Left && hitbox.is_hovered(window)
                {
                    window.start_window_resize(edge);
                    cx.stop_propagation();
                }
            });
        }
    }
}
