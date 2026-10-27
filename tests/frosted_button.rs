#![cfg(feature = "test-support")]
mod support;

use beckon_kit::{
    Root,
    components::{Button, Icon, IconName},
    theme::{Appearance, Preferences, Theme, ThemeScope},
};
use gpui::{Context, IntoElement, Render, TestAppContext, Window, div, prelude::*, px};

struct Host {
    theme: Theme,
    disabled: bool,
    clicks: usize,
}
impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        ThemeScope::new(
            self.theme.clone(),
            Root::new("root").child(
                div().p(px(20.0)).bg(gpui::rgb(0x3366cc)).child(
                    Button::icon("glass", "Glass action", Icon::new(IconName::X))
                        .tooltip(false)
                        .frosted()
                        .w(px(40.0))
                        .h(px(40.0))
                        .min_w(px(40.0))
                        .min_h(px(40.0))
                        .rounded_full()
                        .disabled(self.disabled)
                        .on_click(cx.listener(|host, _, _, cx| {
                            host.clicks += 1;
                            cx.notify();
                        })),
                ),
            ),
        )
    }
}

#[gpui::test]
fn frosted_button_samples_before_fill_keeps_rounding_and_all_input_contracts(
    cx: &mut TestAppContext,
) {
    let handle = support::open(cx, |_, _| Host {
        theme: Theme::default(),
        disabled: false,
        clicks: 0,
    });
    let window = handle.into();
    let host = handle.root(cx).unwrap();
    cx.simulate_backdrop_blur_support(window, true);
    support::frame(cx, window);
    for appearance in [Appearance::Light, Appearance::Dark] {
        host.update(cx, |host, cx| {
            host.theme = Theme::new(appearance);
            cx.notify();
        });
        support::frame(cx, window);
        support::with_window(cx, window, |window, _| {
            assert_eq!(window.painted_backdrop_blurs().len(), 1);
            assert!(window.painted_quads().iter().any(|quad| {
                quad.background.as_solid()
                    == Some(
                        Theme::new(appearance)
                            .colors
                            .surface
                            .alpha(Theme::new(appearance).frosted.tint_opacity),
                    )
            }));
        });
    }
    let bounds = support::bounds(cx, window, "glass");
    support::with_window(cx, window, |window, _| {
        let blurs = window.painted_backdrop_blurs();
        assert_eq!(blurs.len(), 1);
        let blur = blurs[0];
        let scale = window.scale_factor();
        assert!((blur.bounds.size.width.0 / scale - 40.0).abs() <= 2.0);
        assert_eq!(blur.corner_radii.top_left.0, blur.bounds.size.width.0 / 2.0);
        let quads = window.painted_quads();
        let tint = quads
            .iter()
            .find(|q| q.bounds == blur.bounds && q.background.as_solid().is_some_and(|c| c.a > 0.0))
            .unwrap();
        assert!(tint.order > blur.order);
        assert!(tint.background.as_solid().unwrap().a < 1.0);
        assert!(!quads.iter().any(|q| q.bounds == blur.bounds
            && q.order < blur.order
            && q.background.as_solid().is_some_and(|c| c.a == 1.0)));
    });
    let feedback = |cx: &mut TestAppContext| {
        support::with_window(cx, window, |window, _| {
            window
                .painted_quads()
                .into_iter()
                .filter(|q| q.bounds == bounds.scale(window.scale_factor()))
                .filter_map(|q| q.background.as_solid())
                .rfind(|color| color.a > 0.0 && color.a < 0.2)
        })
    };
    support::mouse_down(cx, window, bounds.center());
    assert_eq!(feedback(cx), Some(Theme::default().colors.text.alpha(0.14)));
    support::mouse_up(cx, window, bounds.center());
    assert_eq!(feedback(cx), Some(Theme::default().colors.text.alpha(0.08)));
    support::press(cx, window, "enter");
    assert_eq!(host.read_with(cx, |host, _| host.clicks), 2);
    assert_eq!(support::bounds(cx, window, "glass"), bounds);
    host.update(cx, |host, cx| {
        host.disabled = true;
        cx.notify();
    });
    support::frame(cx, window);
    support::click(cx, window, "glass");
    support::press(cx, window, "space");
    assert_eq!(host.read_with(cx, |host, _| host.clicks), 2);
}

#[gpui::test]
fn frosted_button_uses_opaque_fallback_for_unsupported_or_inaccessible_backdrops(
    cx: &mut TestAppContext,
) {
    let handle = support::open(cx, |_, _| Host {
        theme: Theme::default(),
        disabled: false,
        clicks: 0,
    });
    let window = handle.into();
    let host = handle.root(cx).unwrap();
    for (supported, preferences) in [
        (false, Preferences::default()),
        (
            true,
            Preferences {
                reduced_transparency: true,
                increased_contrast: false,
            },
        ),
        (
            true,
            Preferences {
                reduced_transparency: false,
                increased_contrast: true,
            },
        ),
    ] {
        cx.simulate_backdrop_blur_support(window, supported);
        host.update(cx, |host, cx| {
            host.theme.preferences = preferences;
            host.theme.colors.surface.a = 0.1;
            cx.notify();
        });
        support::frame(cx, window);
        support::with_window(cx, window, |window, _| {
            assert!(window.painted_backdrop_blurs().is_empty());
            assert!(
                window
                    .painted_quads()
                    .iter()
                    .any(|q| q.bounds.size.width.0 > 30.0
                        && q.background.as_solid()
                            == Some(Theme::default().colors.surface.alpha(1.0)))
            );
        });
    }
}
