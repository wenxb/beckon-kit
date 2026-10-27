#![cfg(feature = "test-support")]
mod support;

use beckon_kit::{
    Root,
    components::{Segment, SegmentedControl, Toggle},
    theme::{Appearance, Theme},
};
use gpui::{
    Context, IntoElement, Render, StyleRefinement, TestAppContext, Window, div, prelude::*, px,
    size,
};

struct Defaults;
impl Render for Defaults {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Root::new("defaults").child(
            div()
                .flex()
                .flex_col()
                .items_start()
                .child(Toggle::new("default-toggle", "Default", false))
                .child(SegmentedControl::new(
                    "default-tabs",
                    "Default choices",
                    [Segment::new("one", "One")],
                    "one",
                )),
        )
    }
}

#[gpui::test]
fn uncustomized_controls_keep_their_default_geometry_and_tokens(cx: &mut TestAppContext) {
    let handle = support::open(cx, |_, _| Defaults);
    let window = handle.into();
    let theme = Theme::default();
    let tabs = support::bounds(cx, window, "default-tabs");
    let item = support::bounds(cx, window, "default-tabs-one");
    assert_eq!(item.size.height, px(theme.metrics.control_height));
    assert_eq!(tabs.size.height, item.size.height + px(6.0));
    support::with_window(cx, window, |window, _| {
        let scale = window.scale_factor();
        let quads = window.painted_quads();
        assert!(
            quads
                .iter()
                .any(|q| q.bounds.size == size(px(34.0), px(20.0)).scale(scale)
                    && q.corner_radii.top_left.0 == 10.0 * scale
                    && q.background.as_solid() == Some(theme.colors.elevated))
        );
        assert!(quads.iter().any(|q| q.bounds == item.scale(scale)
            && q.background.as_solid() == Some(theme.colors.selection)));
    });
}

struct Host {
    on: bool,
    selected: gpui::SharedString,
    disabled: bool,
}
impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        Root::new("root").child(
            div()
                .flex()
                .flex_col()
                .items_start()
                .p(px(20.0))
                .gap(px(10.0))
                .child(
                    Toggle::new("custom-toggle", "Custom switch", self.on)
                        .track_size(size(px(40.0), px(24.0)))
                        .track_colors(
                            gpui::rgb(0x777777).into(),
                            gpui::rgb(0x34c759).into(),
                            gpui::rgb(0xffffff).into(),
                        )
                        .label_visible(false)
                        .border_0()
                        .px(px(0.0))
                        .min_h(px(24.0))
                        .disabled(self.disabled)
                        .on_change(cx.listener(|host, value: &bool, _, cx| {
                            host.on = *value;
                            cx.notify();
                        })),
                )
                .child(
                    SegmentedControl::new(
                        "custom-tabs",
                        "Custom choices",
                        [Segment::new("one", "One"), Segment::new("two", "Two")],
                        self.selected.clone(),
                    )
                    .p(px(4.0))
                    .gap(px(0.0))
                    .rounded(px(12.0))
                    .bg(gpui::rgb(0xdddddd))
                    .item_style(
                        StyleRefinement::default()
                            .flex_none()
                            .min_h(px(28.0))
                            .px(px(12.0))
                            .py(px(6.0))
                            .text_size(px(12.0))
                            .line_height(px(16.0))
                            .rounded(px(8.0)),
                    )
                    .selected_style(
                        StyleRefinement::default()
                            .bg(gpui::rgb(0xffffff))
                            .text_color(gpui::rgb(0x171717)),
                    )
                    .disabled(self.disabled)
                    .on_change(cx.listener(
                        |host, value: &gpui::SharedString, _, cx| {
                            host.selected = value.clone();
                            cx.notify();
                        },
                    )),
                ),
        )
    }
}

#[gpui::test]
fn custom_control_geometry_and_fills_preserve_clicks_keyboard_and_disabled_state(
    cx: &mut TestAppContext,
) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let handle = support::open(cx, |_, _| Host {
        on: false,
        selected: "one".into(),
        disabled: false,
    });
    let window = handle.into();
    let host = handle.root(cx).unwrap();
    for appearance in [Appearance::Light, Appearance::Dark] {
        cx.update(|cx| Theme::new(appearance).install(cx));
        support::frame(cx, window);
        assert_eq!(
            support::bounds(cx, window, "custom-toggle").size,
            size(px(40.0), px(24.0))
        );
        assert_eq!(
            support::bounds(cx, window, "custom-tabs").size.height,
            px(36.0)
        );
        let selected = support::bounds(cx, window, "custom-tabs-one");
        assert_eq!(selected.size.height, px(28.0));
        support::with_window(cx, window, |window, _| {
            let scale = window.scale_factor();
            let quads = window.painted_quads();
            assert!(
                quads
                    .iter()
                    .any(|q| q.bounds.size == size(px(40.0), px(24.0)).scale(scale)
                        && q.corner_radii.top_left.0 == 12.0 * scale)
            );
            assert!(quads.iter().any(|q| q.bounds == selected.scale(scale)
                && q.corner_radii.top_left.0 == 8.0 * scale
                && q.background.as_solid() == Some(gpui::rgb(0xffffff).into())));
        });
    }
    support::click(cx, window, "custom-toggle");
    assert!(host.read_with(cx, |host, _| host.on));
    support::press(cx, window, "space");
    assert!(!host.read_with(cx, |host, _| host.on));
    support::click(cx, window, "custom-tabs-two");
    assert_eq!(host.read_with(cx, |host, _| host.selected.clone()), "two");
    support::press(cx, window, "left");
    assert_eq!(host.read_with(cx, |host, _| host.selected.clone()), "one");
    host.update(cx, |host, cx| {
        host.disabled = true;
        cx.notify();
    });
    support::frame(cx, window);
    support::click(cx, window, "custom-toggle");
    support::click(cx, window, "custom-tabs-two");
    support::press(cx, window, "right");
    assert!(!host.read_with(cx, |host, _| host.on));
    assert_eq!(host.read_with(cx, |host, _| host.selected.clone()), "one");
}
