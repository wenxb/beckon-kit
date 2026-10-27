mod support;

use beckon_kit::{
    components::Surface,
    material::{Backdrop, BackdropImage, BackdropKind, ImagePlane, Material, PreparedImage},
    theme::{Appearance, Preferences, Theme, ThemeScope},
};
use gpui::{
    App, Bounds, Context, Corners, IntoElement, Pixels, Render, Window, div, prelude::*, px,
};
use image::{Rgba, RgbaImage};
use std::{cell::Cell, rc::Rc, sync::Arc};

struct FailingBackdrop {
    calls: Rc<Cell<usize>>,
}
impl Backdrop for FailingBackdrop {
    fn kind(&self) -> BackdropKind {
        BackdropKind::PreparedImage
    }
    fn paint(
        &self,
        _: Bounds<Pixels>,
        _: Corners<Pixels>,
        _: &mut Window,
        _: &mut App,
    ) -> gpui::Result<bool> {
        self.calls.set(self.calls.get() + 1);
        Ok(false)
    }
}

struct MaterialHost {
    theme: Theme,
    backdrop: Rc<dyn Backdrop>,
}

impl Render for MaterialHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        ThemeScope::new(
            self.theme.clone(),
            div().flex().p(px(20.0)).child(
                Surface::new("surface")
                    .w(px(160.0))
                    .h(px(100.0))
                    .rounded(px(17.0))
                    .material(Material::Frosted)
                    .shared_backdrop(self.backdrop.clone())
                    .child(
                        div()
                            .id("foreground")
                            .debug_selector(|| "foreground".into())
                            .size(px(20.0))
                            .bg(gpui::rgb(0xff0011)),
                    ),
            ),
        )
    }
}

#[gpui::test]
fn unavailable_region_paints_an_opaque_surface_and_foreground_is_in_a_later_layer(
    cx: &mut gpui::TestAppContext,
) {
    let calls = Rc::new(Cell::new(0));
    let handle = support::open(cx, |_, _| MaterialHost {
        theme: Theme::default(),
        backdrop: Rc::new(FailingBackdrop {
            calls: calls.clone(),
        }),
    });
    assert!(calls.get() > 0);
    let bounds = support::bounds(cx, handle.into(), "surface");
    let quads = support::with_window(cx, handle.into(), |window, _| window.painted_quads());
    let fills: Vec<_> = quads
        .iter()
        .filter(|quad| quad.bounds.size.width.as_f32() == f32::from(bounds.size.width))
        .collect();
    assert!(
        fills
            .iter()
            .all(|quad| quad.background.as_solid().unwrap().a == 1.0)
    );
    assert!(
        fills
            .iter()
            .all(|quad| quad.corner_radii.top_left.as_f32() == 17.0)
    );
    let foreground = quads
        .iter()
        .find(|quad| quad.background.as_solid().unwrap() == gpui::Hsla::from(gpui::rgb(0xff0011)))
        .unwrap();
    assert!(fills.iter().all(|quad| foreground.order > quad.order));
}

#[gpui::test]
fn reduced_transparency_does_not_even_invoke_the_backdrop(cx: &mut gpui::TestAppContext) {
    let calls = Rc::new(Cell::new(0));
    support::open(cx, |_, _| MaterialHost {
        theme: Theme::new(Appearance::Light).preferences(Preferences {
            reduced_transparency: true,
            increased_contrast: false,
        }),
        backdrop: Rc::new(FailingBackdrop {
            calls: calls.clone(),
        }),
    });
    assert_eq!(calls.get(), 0);
}

#[gpui::test]
fn actual_image_backdrop_is_drawn_and_tint_alpha_is_not_a_fake_blur(cx: &mut gpui::TestAppContext) {
    let prepared = Arc::new(
        PreparedImage::new(
            RgbaImage::from_pixel(32, 32, Rgba([160, 120, 70, 255])),
            3.0,
        )
        .unwrap(),
    );
    let handle = support::open(cx, |_, _| MaterialHost {
        theme: Theme::default(),
        backdrop: Rc::new(BackdropImage::new(prepared, ImagePlane::Local)),
    });
    let quads = support::with_window(cx, handle.into(), |window, _| window.painted_quads());
    let alpha = Material::Frosted
        .resolve(&Theme::default(), BackdropKind::PreparedImage)
        .fill
        .a;
    assert!(
        quads
            .iter()
            .any(|quad| (quad.background.as_solid().unwrap().a - alpha).abs() < 0.001)
    );
}

struct FadeHost;
impl Render for FadeHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().flex().child(
            Surface::new("faded")
                .w(px(100.0))
                .h(px(80.0))
                .opacity(0.5)
                .child(div().size(px(12.0)).bg(gpui::rgb(0xff0011))),
        )
    }
}

#[gpui::test]
fn surface_opacity_applies_to_material_and_foreground_once(cx: &mut gpui::TestAppContext) {
    let handle = support::open(cx, |_, _| FadeHost);
    let quads = support::with_window(cx, handle.into(), |window, _| window.painted_quads());
    let surface = Theme::default().colors.surface.opacity(0.5);
    let foreground = gpui::Hsla::from(gpui::rgb(0xff0011)).opacity(0.5);
    assert!(
        quads
            .iter()
            .any(|quad| quad.background.as_solid() == Some(surface))
    );
    assert!(
        quads
            .iter()
            .any(|quad| quad.background.as_solid() == Some(foreground))
    );
}
