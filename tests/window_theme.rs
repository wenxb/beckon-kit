#![cfg(feature = "test-support")]
use beckon_kit::{
    WindowConfig,
    components::{Button, ButtonVariant, Segment, SegmentedControl, Surface, Toggle},
    test_support,
    theme::{Appearance, Theme, ThemeScope},
    window::TitlebarMode,
};
use gpui::{
    AppContext, Context, Entity, IntoElement, Render, StyleRefinement, Window, div, prelude::*, px,
};

struct Controls;
impl Render for Controls {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .items_start()
            .child(Button::new("primary", "Run").variant(ButtonVariant::Primary))
            .child(Toggle::new("toggle", "Enabled", true))
            .child(SegmentedControl::new(
                "tabs",
                "Mode",
                [Segment::new("one", "One"), Segment::new("two", "Two")],
                "one",
            ))
            .child(Button::new("frosted-action", "Frosted action").frosted())
            .child(
                Surface::new("frosted-surface")
                    .frosted()
                    .w(px(100.0))
                    .h(px(60.0)),
            )
    }
}
struct Content(Entity<Controls>);
impl Render for Content {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.0
            .clone()
            .cached(StyleRefinement::default().size_full())
    }
}
fn open(
    cx: &mut gpui::TestAppContext,
    config: WindowConfig,
) -> gpui::WindowHandle<beckon_kit::WindowRoot<Content>> {
    cx.update(|cx| {
        config
            .titlebar(TitlebarMode::None)
            .open(cx, |_, cx| {
                let controls = cx.new(|_| Controls);
                cx.new(|_| Content(controls))
            })
            .unwrap()
    })
}
fn assert_theme(cx: &mut gpui::TestAppContext, handle: gpui::AnyWindowHandle, theme: &Theme) {
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        // Do not force refresh: theme installation must invalidate cached children itself.
        window.draw(cx).clear(cx);
        let quads = window.painted_quads();
        for color in [
            theme.colors.canvas,
            theme.colors.primary,
            theme.colors.selection,
            theme.colors.success,
        ] {
            assert!(
                quads
                    .iter()
                    .filter_map(|quad| quad.background.as_solid())
                    .any(|fill| {
                        let (a, b) = (fill.to_rgb(), color.to_rgb());
                        (a.r - b.r)
                            .abs()
                            .max((a.g - b.g).abs())
                            .max((a.b - b.b).abs())
                            .max((a.a - b.a).abs())
                            < 1e-5
                    }),
                "missing themed fill {color:?}"
            );
        }
    })
    .unwrap();
}

#[gpui::test]
fn one_install_updates_existing_and_new_windows_including_cached_controls(
    cx: &mut gpui::TestAppContext,
) {
    cx.update(|cx| Theme::new(Appearance::Light).install(cx));
    let first = open(cx, WindowConfig::new("Global"));
    assert_theme(cx, first.into(), &Theme::new(Appearance::Light));
    for appearance in [Appearance::Dark, Appearance::Light] {
        cx.update(|cx| Theme::new(appearance).install(cx));
        assert_theme(cx, first.into(), &Theme::new(appearance));
        first
            .read_with(cx, |root, _| {
                assert_eq!(
                    root.client_style().border_color,
                    Theme::new(appearance).colors.window_border
                )
            })
            .unwrap();
    }
    cx.update(|cx| Theme::new(Appearance::Dark).install(cx));
    let second = open(cx, WindowConfig::new("New"));
    assert_theme(cx, second.into(), &Theme::new(Appearance::Dark));
    cx.update(|cx| Theme::new(Appearance::Light).install(cx));
    for handle in [first, second] {
        assert_theme(cx, handle.into(), &Theme::new(Appearance::Light));
    }
}

fn assert_frosted(cx: &mut gpui::TestAppContext, handle: gpui::AnyWindowHandle, theme: &Theme) {
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        // Installation, not a manual refresh, must invalidate cached materials.
        window.draw(cx).clear(cx);
        let blurs = window.painted_backdrop_blurs();
        assert_eq!(
            blurs.len(),
            2,
            "both Button and Surface must use the shared material"
        );
        let quads = window.painted_quads();
        for blur in blurs {
            assert_eq!(
                blur.sigma.0,
                theme.frosted.blur_sigma.as_f32() * window.scale_factor()
            );
            assert!(quads.iter().any(|quad| {
                quad.bounds == blur.bounds
                    && quad.order > blur.order
                    && quad.background.as_solid()
                        == Some(theme.colors.surface.alpha(theme.frosted.tint_opacity))
            }));
        }
    })
    .unwrap();
}

#[gpui::test]
fn global_frosted_tokens_update_cached_components_and_new_windows(cx: &mut gpui::TestAppContext) {
    let first = open(cx, WindowConfig::new("Shared material"));
    cx.simulate_backdrop_blur_support(first.into(), true);
    assert_frosted(cx, first.into(), &Theme::default());
    for appearance in [Appearance::Light, Appearance::Dark] {
        let mut custom = Theme::new(appearance);
        custom.frosted.blur_sigma = px(23.0);
        custom.frosted.tint_opacity = 0.83;
        cx.update(|cx| custom.clone().install(cx));
        assert_frosted(cx, first.into(), &custom);
        let second = open(cx, WindowConfig::new("New shared material"));
        cx.simulate_backdrop_blur_support(second.into(), true);
        assert_frosted(cx, second.into(), &custom);
        let reset = Theme::new(appearance);
        cx.update(|cx| reset.clone().install(cx));
        for handle in [first, second] {
            assert_frosted(cx, handle.into(), &reset);
        }
    }
}

#[gpui::test]
fn installing_custom_tokens_updates_frame_and_cached_control_layout(cx: &mut gpui::TestAppContext) {
    let handle = open(cx, WindowConfig::new("Custom tokens"));
    assert_theme(cx, handle.into(), &Theme::default());
    let mut custom = Theme::new(Appearance::Light);
    custom.colors.primary = gpui::rgb(0x224466).into();
    custom.colors.window_border = gpui::rgb(0x445566).into();
    custom.metrics.surface_radius = 22.0;
    custom.metrics.control_height = 52.0;
    cx.update(|cx| custom.clone().install(cx));
    assert_theme(cx, handle.into(), &custom);
    handle
        .read_with(cx, |root, _| {
            assert_eq!(
                root.client_style().border_color,
                custom.colors.window_border
            );
            assert_eq!(root.client_style().corner_radii.top_left, px(22.0));
        })
        .unwrap();
    cx.update_window(handle.into(), |_, window, _| {
        let button = gpui::test::observation::find(window, &[], &"primary".into()).unwrap();
        assert!(button.bounds().size.height >= px(52.0));
    })
    .unwrap();
}

#[gpui::test]
fn explicit_window_theme_and_frame_overrides_remain_local(cx: &mut gpui::TestAppContext) {
    let border = gpui::rgb(0x123456).into();
    let fixed = open(
        cx,
        WindowConfig::new("Fixed")
            .theme(Theme::new(Appearance::Light))
            .client_style(|frame| {
                frame
                    .border(px(2.0), border)
                    .radius(px(24.0))
                    .shadows(Vec::new())
            }),
    );
    cx.simulate_backdrop_blur_support(fixed.into(), true);
    let mut global = Theme::new(Appearance::Dark);
    global.frosted.blur_sigma = px(29.0);
    global.frosted.tint_opacity = 0.78;
    cx.update(|cx| global.install(cx));
    assert_theme(cx, fixed.into(), &Theme::new(Appearance::Light));
    assert_frosted(cx, fixed.into(), &Theme::new(Appearance::Light));
    fixed
        .update(cx, |root, _, cx| {
            root.set_theme(Theme::new(Appearance::Dark), cx)
        })
        .unwrap();
    assert_theme(cx, fixed.into(), &Theme::new(Appearance::Dark));
    fixed
        .read_with(cx, |root, _| {
            assert_eq!(root.client_style().border_color, border);
            assert_eq!(root.client_style().corner_radii.top_left, px(24.0));
            assert!(root.client_style().shadows.is_empty());
        })
        .unwrap();
}

#[gpui::test]
fn explicit_frame_values_equal_to_defaults_are_not_lost_on_theme_changes(
    cx: &mut gpui::TestAppContext,
) {
    let light = Theme::new(Appearance::Light);
    cx.update(|cx| light.clone().install(cx));
    let handle = open(
        cx,
        WindowConfig::new("Pinned border")
            .client_style(|frame| frame.border(px(1.0), light.colors.window_border)),
    );
    let dark = Theme::new(Appearance::Dark);
    cx.update(|cx| dark.clone().install(cx));
    assert_theme(cx, handle.into(), &dark);
    handle
        .read_with(cx, |root, _| {
            assert_eq!(root.client_style().border_color, light.colors.window_border)
        })
        .unwrap();
}

struct Local;
impl Render for Local {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        ThemeScope::new(
            Theme::new(Appearance::Light),
            Button::new("local", "Local").variant(ButtonVariant::Primary),
        )
    }
}
#[gpui::test]
fn subtree_override_still_wins_over_global_theme(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        WindowConfig::new("Scoped")
            .open(cx, |_, cx| cx.new(|_| Local))
            .unwrap()
    });
    cx.update(|cx| Theme::new(Appearance::Dark).install(cx));
    test_support::render(cx, handle.into());
    cx.update_window(handle.into(), |_, window, _| {
        assert!(
            window
                .painted_quads()
                .iter()
                .any(|quad| quad.background.as_solid()
                    == Some(Theme::new(Appearance::Light).colors.primary))
        );
    })
    .unwrap();
}
