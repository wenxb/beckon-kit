use gpui::{
    AnyElement, App, Bounds, BoxShadow, Corners, Element, ElementId, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, ParentElement, Pixels, RenderOnce, Style,
    StyleRefinement, Styled, Window, div, fill, point, prelude::*, px, size,
};
use std::rc::Rc;

use crate::{
    material::{Backdrop, BackdropKind, Material, MaterialPlan, SceneBackdrop},
    theme::Theme,
};

#[derive(IntoElement)]
pub struct Surface {
    id: ElementId,
    material: Material,
    backdrop: Option<Rc<dyn Backdrop>>,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl Surface {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            material: Material::default(),
            backdrop: None,
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    pub fn material(mut self, material: Material) -> Self {
        self.material = material;
        self
    }

    /// Enable real scene blur using the shared theme's frosted material.
    pub fn frosted(self) -> Self {
        self.material(Material::Frosted)
    }

    pub fn backdrop(mut self, backdrop: impl Backdrop) -> Self {
        self.backdrop = Some(Rc::new(backdrop));
        self
    }

    pub fn shared_backdrop(mut self, backdrop: Rc<dyn Backdrop>) -> Self {
        self.backdrop = Some(backdrop);
        self
    }
}

impl Styled for Surface {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for Surface {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Surface {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let selector = self.id.to_string();
        let mut body = div()
            .id(self.id)
            .debug_selector(move || selector)
            .block_mouse_except_scroll()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .p(px(theme.metrics.surface_padding))
            .rounded(px(theme.metrics.surface_radius));
        body.style().refine(&self.style);
        let mut resolved_style = Style::default();
        resolved_style.refine(body.style());
        let shadows = if let Some(shadows) = &body.style().box_shadow {
            shadows.clone()
        } else {
            vec![BoxShadow::new(px(0.0), px(6.0), theme.colors.shadow).blur_radius(px(24.0))]
        };
        let material = material_background(
            self.material,
            self.backdrop,
            &resolved_style,
            shadows,
            &theme,
            window,
        );
        // Material is painted by the background child, not duplicated by Div.
        body.style().background = None;
        body.style().box_shadow = None;
        // Each child has a later draw order without adding a layout node.
        body.child(material)
            .children(self.children.into_iter().map(Layered::new))
    }
}

pub(super) fn material_background(
    material: Material,
    mut backdrop: Option<Rc<dyn Backdrop>>,
    style: &Style,
    shadows: Vec<BoxShadow>,
    theme: &Theme,
    window: &Window,
) -> SurfaceElement {
    if backdrop.is_none() && material == Material::Frosted {
        backdrop = Some(Rc::new(SceneBackdrop::new(theme.frosted.blur_sigma)));
    }
    let kind = backdrop
        .as_ref()
        .map_or(BackdropKind::Unavailable, |backdrop| {
            if backdrop.kind() == BackdropKind::Scene && !window.is_backdrop_blur_supported() {
                BackdropKind::Unavailable
            } else {
                backdrop.kind()
            }
        });
    let edge = style.border_widths.to_pixels(window.rem_size());
    let mut background = div()
        .absolute()
        .top(-edge.top)
        .right(-edge.right)
        .bottom(-edge.bottom)
        .left(-edge.left);
    let mut background_style = Style::default();
    background_style.refine(background.style());
    SurfaceElement {
        style: background_style,
        corners: style.corner_radii,
        plan: material.resolve(theme, kind),
        backdrop,
        fallback: theme.colors.surface.alpha(1.0),
        shadows,
    }
}

pub(super) struct SurfaceElement {
    style: Style,
    corners: Corners<gpui::AbsoluteLength>,
    pub(super) plan: MaterialPlan,
    backdrop: Option<Rc<dyn Backdrop>>,
    fallback: gpui::Hsla,
    shadows: Vec<BoxShadow>,
}

impl IntoElement for SurfaceElement {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for SurfaceElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

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
    ) -> (LayoutId, ()) {
        (window.request_layout(self.style.clone(), [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut Window,
        _: &mut App,
    ) {
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let corners: Corners<Pixels> = self
            .corners
            .to_pixels(window.rem_size())
            .clamp_radii_for_quad_size(bounds.size);
        window.paint_drop_shadows(bounds, corners, &self.shadows);
        window.paint_layer(bounds, |window| {
            let painted = self.plan.uses_backdrop
                && self.backdrop.as_ref().is_some_and(|backdrop| {
                    backdrop.paint(bounds, corners, window, cx).unwrap_or(false)
                });
            // Never paint an opaque fill before scene sampling: it would erase
            // the very background the blur needs to read.
            window.paint_layer(bounds, |window| {
                let fill_color = if self.plan.uses_backdrop && !painted {
                    self.fallback
                } else {
                    self.plan.fill
                };
                window.paint_quad(
                    fill(bounds, fill_color)
                        .corner_radii(corners)
                        .border_widths(px(1.0))
                        .border_color(self.plan.border),
                );
                let inset = corners.top_left.max(corners.top_right).max(px(1.0));
                let highlight = Bounds::new(
                    bounds.origin + point(inset, px(1.0)),
                    size((bounds.size.width - inset * 2.0).max(px(0.0)), px(1.0)),
                );
                window.paint_quad(fill(highlight, self.plan.highlight));
                window.paint_inset_shadows(bounds, corners, &self.shadows);
            });
        });
    }
}

pub(super) struct Layered {
    child: AnyElement,
}

impl Layered {
    pub(super) fn new(child: impl IntoElement) -> Self {
        Self {
            child: child.into_any_element(),
        }
    }
}

impl IntoElement for Layered {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for Layered {
    type RequestLayoutState = ();
    type PrepaintState = ();
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
    ) -> (LayoutId, ()) {
        (self.child.request_layout(window, cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.prepaint(window, cx);
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        window.paint_layer(bounds, |window| self.child.paint(window, cx));
    }
}
