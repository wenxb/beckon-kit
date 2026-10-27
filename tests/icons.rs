#![cfg(feature = "test-support")]
mod support;

use beckon_kit::{
    Root,
    components::{Button, Icon, IconName},
};
use gpui::{
    Context, IntoElement, Render, SvgRenderer, TestAppContext, Window, div, prelude::*, px,
};
use std::{collections::HashSet, sync::Arc};

#[test]
fn embedded_lucide_catalog_is_unique_and_every_svg_renders() {
    let renderer = SvgRenderer::new(Arc::new(()));
    let mut names = HashSet::new();
    for name in IconName::ALL {
        assert!(names.insert(name.as_str()));
        for scale in [1.0, 2.0] {
            let image = renderer
                .render_single_frame(name.svg(), scale)
                .unwrap_or_else(|error| panic!("{} at {scale}x: {error}", name.as_str()));
            assert!(
                image
                    .as_bytes(0)
                    .unwrap()
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .any(|pixel| pixel[3] > 0),
                "{} must contain visible strokes",
                name.as_str(),
            );
        }
    }
    assert_eq!(names.len(), 40);
}

struct IconActions {
    clicks: usize,
}
impl Render for IconActions {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        Root::new("icons-root").child(
            div()
                .flex()
                .gap(px(10.0))
                .child(
                    div()
                        .id("default-icon")
                        .debug_selector(|| "default-icon".into())
                        .child(Icon::new(IconName::Search)),
                )
                .child(
                    div()
                        .id("custom-icon")
                        .debug_selector(|| "custom-icon".into())
                        .child(Icon::data(IconName::Copy.svg()).size(px(24.0))),
                )
                .child(
                    Button::new("icon-action", "添加")
                        .leading(Icon::new(IconName::Plus))
                        .on_click(cx.listener(|view, _, _, cx| {
                            view.clicks += 1;
                            cx.notify();
                        })),
                ),
        )
    }
}

#[gpui::test]
fn icons_have_stable_sizes_and_do_not_steal_button_keyboard_or_mouse_input(
    cx: &mut TestAppContext,
) {
    let handle = support::open(cx, |_, _| IconActions { clicks: 0 });
    let window = handle.into();
    assert_eq!(
        support::bounds(cx, window, "default-icon").size.width,
        px(16.0)
    );
    assert_eq!(
        support::bounds(cx, window, "custom-icon").size.width,
        px(24.0)
    );
    support::click(cx, window, "icon-action");
    support::press(cx, window, "space");
    assert_eq!(
        handle
            .root(cx)
            .unwrap()
            .read_with(cx, |view, _| view.clicks),
        2
    );
}
