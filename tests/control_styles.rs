#![cfg(feature = "test-support")]
mod support;

use beckon_kit::{
    Root,
    components::{Button, ButtonVariant, Segment, SegmentedControl, TabVariant, Tabs},
    theme::{Appearance, Preferences, Theme},
};
use gpui::{
    AnyWindowHandle, Context, Entity, IntoElement, Quad, Render, SharedString, TestAppContext,
    VisualTestContext, Window, div, prelude::*, px,
};

struct Styles {
    button: ButtonVariant,
    tabs: TabVariant,
    disabled: bool,
    busy: bool,
    selected: SharedString,
    clicks: usize,
    changes: usize,
}

impl Render for Styles {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let options = || {
            [
                Segment::new("one", "One"),
                Segment::new("two", "Two").disabled(true),
                Segment::new("three", "Three"),
            ]
        };
        Root::new("styles").child(
            div()
                .flex()
                .flex_col()
                .w(px(340.0))
                .p(px(20.0))
                .gap(px(16.0))
                .child(
                    Button::new("button", "Action")
                        .variant(self.button)
                        .disabled(self.disabled)
                        .busy(self.busy)
                        .on_click(cx.listener(|view, _, _, cx| {
                            view.clicks += 1;
                            cx.notify();
                        })),
                )
                .child(
                    Tabs::new("tabs", "Pages", options(), self.selected.clone())
                        .variant(self.tabs)
                        .disabled(self.disabled)
                        .on_change(cx.listener(|view, selected: &SharedString, _, cx| {
                            view.selected = selected.clone();
                            view.changes += 1;
                            cx.notify();
                        })),
                )
                .child(
                    SegmentedControl::new("choices", "Choices", options(), self.selected.clone())
                        .variant(self.tabs)
                        .disabled(self.disabled)
                        .on_change(cx.listener(|view, selected: &SharedString, _, cx| {
                            view.selected = selected.clone();
                            view.changes += 1;
                            cx.notify();
                        })),
                ),
        )
    }
}

fn fixture(cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<Styles>) {
    let handle = support::open(cx, |_, _| Styles {
        button: ButtonVariant::Primary,
        tabs: TabVariant::Segmented,
        disabled: false,
        busy: false,
        selected: "one".into(),
        clicks: 0,
        changes: 0,
    });
    (handle.into(), handle.root(cx).unwrap())
}

fn quad(cx: &mut TestAppContext, window: AnyWindowHandle, id: &'static str) -> Option<Quad> {
    let bounds = support::bounds(cx, window, id);
    support::with_window(cx, window, |window, _| {
        let bounds = bounds.scale(window.scale_factor());
        // GPUI paints fill first and the visible border last at the same bounds.
        window
            .painted_quads()
            .into_iter()
            .rev()
            .find(|q| q.bounds == bounds)
    })
}

fn scene(cx: &mut TestAppContext, window: AnyWindowHandle) -> Vec<String> {
    support::with_window(cx, window, |window, _| {
        window
            .painted_quads()
            .iter()
            .map(|q| format!("{q:?}"))
            .collect()
    })
}

#[gpui::test]
fn button_variants_keep_the_input_contract_and_geometry(cx: &mut TestAppContext) {
    let (window, entity) = fixture(cx);
    for variant in [
        ButtonVariant::Primary,
        ButtonVariant::Secondary,
        ButtonVariant::Outline,
        ButtonVariant::Soft,
        ButtonVariant::Quiet,
        ButtonVariant::Link,
        ButtonVariant::Danger,
    ] {
        entity.update(cx, |view, cx| {
            view.button = variant;
            cx.notify();
        });
        support::frame(cx, window);
        let bounds = support::bounds(cx, window, "button");
        let before = entity.read_with(cx, |view, _| view.clicks);
        support::click(cx, window, "button");
        support::press(cx, window, "enter");
        assert_eq!(entity.read_with(cx, |view, _| view.clicks), before + 2);
        assert_eq!(support::bounds(cx, window, "button"), bounds);
        for busy in [false, true] {
            entity.update(cx, |view, cx| {
                view.disabled = !busy;
                view.busy = busy;
                cx.notify();
            });
            support::frame(cx, window);
            support::click(cx, window, "button");
            support::press(cx, window, "enter");
            assert_eq!(entity.read_with(cx, |view, _| view.clicks), before + 2);
        }
        entity.update(cx, |view, cx| {
            view.disabled = false;
            view.busy = false;
            cx.notify();
        });
        support::frame(cx, window);
    }
}

#[gpui::test]
fn tabs_variants_are_keyboard_accessible_without_hover_or_active_surfaces(cx: &mut TestAppContext) {
    let (window, entity) = fixture(cx);
    for appearance in [Appearance::Dark, Appearance::Light] {
        cx.update(|cx| Theme::new(appearance).install(cx));
        for variant in [
            TabVariant::Segmented,
            TabVariant::Pills,
            TabVariant::Underline,
        ] {
            entity.update(cx, |view, cx| {
                view.tabs = variant;
                view.selected = "one".into();
                cx.notify();
            });
            support::frame(cx, window);
            let selected_quad = quad(cx, window, "tabs-one").expect("selected tab indicator");
            let colors = Theme::new(appearance).colors;
            if variant == TabVariant::Underline {
                assert_eq!(selected_quad.border_color, colors.info);
                assert_eq!(selected_quad.background.as_solid().unwrap().a, 0.0);
            } else {
                assert_eq!(selected_quad.background.as_solid(), Some(colors.selection));
            }
            let before = scene(cx, window);
            let bounds = support::bounds(cx, window, "tabs-three");
            VisualTestContext::from_window(window, cx).simulate_mouse_move(
                bounds.center(),
                None,
                Default::default(),
            );
            support::frame(cx, window);
            assert_eq!(
                scene(cx, window),
                before,
                "hover changes text, never a tab surface"
            );
            let hovered = scene(cx, window);
            support::mouse_down(cx, window, bounds.center());
            assert_eq!(
                scene(cx, window),
                hovered,
                "active must not add a tab surface"
            );
            assert_eq!(entity.read_with(cx, |v, _| v.selected.clone()), "one");
            support::mouse_up(cx, window, bounds.center());
            assert_eq!(entity.read_with(cx, |v, _| v.selected.clone()), "three");
            support::press(cx, window, "right");
            assert_eq!(entity.read_with(cx, |v, _| v.selected.clone()), "one");
            support::press(cx, window, "end");
            assert_eq!(entity.read_with(cx, |v, _| v.selected.clone()), "three");
            support::click(cx, window, "tabs-two");
            assert_eq!(entity.read_with(cx, |v, _| v.selected.clone()), "three");
            support::press(cx, window, "tab");
            support::press(cx, window, "home");
            assert_eq!(
                entity.read_with(cx, |v, _| v.selected.clone()),
                "one",
                "one tab stop per group"
            );
            let changes = entity.read_with(cx, |v, _| v.changes);
            entity.update(cx, |v, cx| {
                v.disabled = true;
                cx.notify();
            });
            support::frame(cx, window);
            support::click(cx, window, "tabs-three");
            support::press(cx, window, "end");
            assert_eq!(entity.read_with(cx, |v, _| v.changes), changes);
            entity.update(cx, |v, cx| {
                v.disabled = false;
                cx.notify();
            });
        }
    }
}

#[gpui::test]
fn keyboard_focus_is_visible_without_reserving_a_thick_border(cx: &mut TestAppContext) {
    let (window, _) = fixture(cx);
    let bounds = support::bounds(cx, window, "button");
    support::press(cx, window, "tab");
    let ring = quad(cx, window, "button").expect("focused button outline");
    assert_eq!(ring.border_color, Theme::default().colors.focus);
    assert_eq!(support::bounds(cx, window, "button"), bounds);
    cx.update(|cx| {
        Theme::default()
            .preferences(Preferences {
                increased_contrast: true,
                ..Default::default()
            })
            .install(cx)
    });
    support::frame(cx, window);
    support::press(cx, window, "tab");
    support::press(cx, window, "tab");
    // Moving focus away restores the high-contrast hairline, not a permanent focus ring.
    assert_eq!(
        quad(cx, window, "button").unwrap().border_color,
        Theme::default().colors.secondary
    );
}
