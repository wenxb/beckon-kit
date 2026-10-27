mod support;

use beckon_kit::{
    components::{Button, Surface},
    material::SceneBackdrop,
    theme::{Preferences, Theme, ThemeScope},
};
use gpui::{App, Context, IntoElement, Render, TestAppContext, Window, div, prelude::*, px, size};

struct Host {
    theme: Theme,
    sigma: Option<f32>,
    opacity: f32,
    clicks: usize,
}

impl Default for Host {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            sigma: None,
            opacity: 1.0,
            clicks: 0,
        }
    }
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut surface = Surface::new("scene-surface")
            .w(px(170.0))
            .h(px(120.0))
            .rounded(px(17.0))
            .opacity(self.opacity)
            .frosted();
        if let Some(sigma) = self.sigma {
            surface = surface.backdrop(SceneBackdrop::new(px(sigma)));
        }
        ThemeScope::new(
            self.theme.clone(),
            div()
                .relative()
                .size_full()
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .bg(gpui::rgb(0x3366cc))
                        .child("Actual background text"),
                )
                .child(div().relative().p(px(20.0)).child(surface.child(
                    Button::new("scene-button", "Sharp action").on_click(cx.listener(
                        |host, _, _, cx| {
                            host.clicks += 1;
                            cx.notify();
                        },
                    )),
                ))),
        )
    }
}

#[gpui::test]
fn default_frost_samples_the_scene_before_tint_and_keeps_controls_working(cx: &mut TestAppContext) {
    let handle = support::open(cx, |_, _| Host::default());
    cx.simulate_backdrop_blur_support(handle.into(), true);
    support::frame(cx, handle.into());
    let (blurs, quads) = support::with_window(cx, handle.into(), |window, _| {
        (window.painted_backdrop_blurs(), window.painted_quads())
    });
    assert_eq!(blurs.len(), 1, "frost needs no image source");
    let blur = &blurs[0];
    let bounds = support::bounds(cx, handle.into(), "scene-surface");
    let scale = support::with_window(cx, handle.into(), |window, _| window.scale_factor());
    assert_eq!(blur.bounds, bounds.scale(scale));
    assert_eq!(blur.corner_radii.top_left.0, 17.0 * scale);
    assert_eq!(blur.sigma.0, 16.0 * scale);
    assert_eq!(blur.opacity, 1.0);
    assert!(quads.iter().any(|quad| quad.order < blur.order));
    let fills: Vec<_> = quads
        .iter()
        .filter(|quad| quad.bounds == blur.bounds)
        .collect();
    assert_eq!(fills.len(), 1, "an opaque base would erase the backdrop");
    assert!(fills[0].order > blur.order);
    assert!(fills[0].background.as_solid().unwrap().a < 1.0);
    support::click(cx, handle.into(), "scene-button");
    assert_eq!(
        handle
            .root(cx)
            .unwrap()
            .read_with(cx, |host, _| host.clicks),
        1
    );
}

#[gpui::test]
fn unsupported_renderer_and_invalid_sigma_fall_back_to_opaque(cx: &mut TestAppContext) {
    let handle = support::open(cx, |_, _| Host::default());
    assert!(
        support::with_window(cx, handle.into(), |window, _| window
            .painted_backdrop_blurs())
        .is_empty()
    );
    cx.simulate_backdrop_blur_support(handle.into(), true);
    let host = handle.root(cx).unwrap();
    for sigma in [0.0, -1.0, f32::NAN, f32::INFINITY, 65.0] {
        host.update(cx, |host, cx| {
            host.sigma = Some(sigma);
            cx.notify();
        });
        support::frame(cx, handle.into());
        assert_opaque(cx, handle.into());
    }
    host.update(cx, |host, cx| {
        host.sigma = Some(12.0);
        cx.notify();
    });
    support::frame(cx, handle.into());
    assert_eq!(
        support::with_window(cx, handle.into(), |window, _| window
            .painted_backdrop_blurs())
        .len(),
        1
    );
    cx.simulate_backdrop_blur_support(handle.into(), false);
    support::frame(cx, handle.into());
    assert_opaque(cx, handle.into());
}

#[gpui::test]
fn accessibility_preferences_remove_sampling_instead_of_only_changing_alpha(
    cx: &mut TestAppContext,
) {
    let handle = support::open(cx, |_, _| Host::default());
    cx.simulate_backdrop_blur_support(handle.into(), true);
    let host = handle.root(cx).unwrap();
    for preferences in [
        Preferences {
            reduced_transparency: true,
            increased_contrast: false,
        },
        Preferences {
            reduced_transparency: false,
            increased_contrast: true,
        },
    ] {
        host.update(cx, |host, cx| {
            host.theme.preferences = preferences;
            cx.notify();
        });
        support::frame(cx, handle.into());
        assert_opaque(cx, handle.into());
    }
}

#[gpui::test]
fn custom_theme_alpha_cannot_make_an_unsupported_surface_transparent(cx: &mut TestAppContext) {
    let handle = support::open(cx, |_, _| {
        let mut theme = Theme::default();
        theme.colors.surface.a = 0.1;
        Host {
            theme,
            ..Host::default()
        }
    });
    assert_opaque(cx, handle.into());
    cx.simulate_backdrop_blur_support(handle.into(), true);
    handle.root(cx).unwrap().update(cx, |host, cx| {
        host.sigma = Some(0.0);
        cx.notify();
    });
    support::frame(cx, handle.into());
    assert_opaque(cx, handle.into());
}

#[gpui::test]
fn blur_tracks_dpi_and_ancestor_opacity_without_double_application(cx: &mut TestAppContext) {
    let handle = support::open(cx, |_, _| Host {
        opacity: 0.5,
        ..Host::default()
    });
    cx.simulate_backdrop_blur_support(handle.into(), true);
    for scale in [1.0, 1.25, 2.0] {
        cx.simulate_window_scale_factor_change(handle.into(), scale);
        cx.simulate_window_resize(handle.into(), size(px(400.0), px(300.0)));
        support::frame(cx, handle.into());
        let blurs = support::with_window(cx, handle.into(), |window, _| {
            window.painted_backdrop_blurs()
        });
        assert_eq!(blurs[0].sigma.0, 16.0 * scale);
        assert_eq!(blurs[0].opacity, 0.5);
    }
}

fn assert_opaque(cx: &mut TestAppContext, handle: gpui::AnyWindowHandle) {
    let bounds = support::bounds(cx, handle, "scene-surface");
    support::with_window(cx, handle, |window: &mut Window, _: &mut App| {
        assert!(window.painted_backdrop_blurs().is_empty());
        let bounds = bounds.scale(window.scale_factor());
        let quads = window.painted_quads();
        let fill = quads.iter().find(|quad| quad.bounds == bounds).unwrap();
        assert_eq!(fill.background.as_solid().unwrap().a, 1.0);
    });
}
