mod support;

use beckon_kit::{Root, components::*};
use gpui::{Context, IntoElement, Render, SharedString, Window, div, prelude::*, px};

struct Selection {
    state: CheckState,
    toggle: bool,
    selected: SharedString,
    value: f64,
    events: usize,
}

impl Render for Selection {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        Root::new("root").child(
            div()
                .flex()
                .flex_col()
                .w(px(300.0))
                .p(px(20.0))
                .gap(px(10.0))
                .child(
                    Checkbox::new("check", "Check", self.state).on_change(cx.listener(
                        |view, state: &CheckState, _, cx| {
                            view.state = *state;
                            cx.notify();
                        },
                    )),
                )
                .child(
                    Toggle::new("toggle", "Toggle", self.toggle).on_change(cx.listener(
                        |view, state: &bool, _, cx| {
                            view.toggle = *state;
                            cx.notify();
                        },
                    )),
                )
                .child(
                    SegmentedControl::new(
                        "segments",
                        "Options",
                        [
                            Segment::new("one", "One"),
                            Segment::new("two", "Two").disabled(true),
                            Segment::new("three", "Three"),
                        ],
                        self.selected.clone(),
                    )
                    .on_change(cx.listener(
                        |view, selected: &SharedString, _, cx| {
                            view.selected = selected.clone();
                            cx.notify();
                        },
                    )),
                )
                .child(
                    Slider::new(
                        "slider",
                        "Value",
                        SliderScale::new(0.0, 10.0, 0.5).unwrap(),
                        self.value,
                    )
                    .on_change(cx.listener(|view, value: &f64, _, cx| {
                        view.value = *value;
                        view.events += 1;
                        cx.notify();
                    })),
                ),
        )
    }
}

fn fixture(cx: &mut gpui::TestAppContext) -> (gpui::AnyWindowHandle, gpui::Entity<Selection>) {
    let window = support::open(cx, |_, _| Selection {
        state: CheckState::Mixed,
        toggle: false,
        selected: "one".into(),
        value: 5.0,
        events: 0,
    });
    let entity = window.root(cx).unwrap();
    (window.into(), entity)
}

#[gpui::test]
fn mixed_check_and_toggle_use_the_same_keyboard_and_mouse_contract(cx: &mut gpui::TestAppContext) {
    let (window, entity) = fixture(cx);
    support::click(cx, window, "check");
    assert_eq!(entity.read_with(cx, |view, _| view.state), CheckState::On);
    support::press(cx, window, "space");
    assert_eq!(entity.read_with(cx, |view, _| view.state), CheckState::Off);
    support::click(cx, window, "toggle");
    assert!(entity.read_with(cx, |view, _| view.toggle));
    support::press(cx, window, "enter");
    assert!(!entity.read_with(cx, |view, _| view.toggle));
}

#[gpui::test]
fn segmented_arrows_skip_disabled_items_and_wrap(cx: &mut gpui::TestAppContext) {
    let (window, entity) = fixture(cx);
    support::click(cx, window, "segments-one");
    support::press(cx, window, "right");
    assert_eq!(
        entity.read_with(cx, |view, _| view.selected.clone()),
        "three"
    );
    support::press(cx, window, "right");
    assert_eq!(entity.read_with(cx, |view, _| view.selected.clone()), "one");
    support::click(cx, window, "segments-two");
    assert_eq!(entity.read_with(cx, |view, _| view.selected.clone()), "one");
    support::press(cx, window, "end");
    assert_eq!(
        entity.read_with(cx, |view, _| view.selected.clone()),
        "three"
    );
}

#[gpui::test]
fn slider_keyboard_snaps_and_reaches_both_endpoints(cx: &mut gpui::TestAppContext) {
    let (window, entity) = fixture(cx);
    support::click(cx, window, "slider");
    support::press(cx, window, "right");
    assert_eq!(entity.read_with(cx, |view, _| view.value), 5.5);
    support::press(cx, window, "shift-right");
    assert_eq!(entity.read_with(cx, |view, _| view.value), 10.0);
    support::press(cx, window, "home");
    assert_eq!(entity.read_with(cx, |view, _| view.value), 0.0);
    support::press(cx, window, "end");
    assert_eq!(entity.read_with(cx, |view, _| view.value), 10.0);
}

#[gpui::test]
fn slider_drag_continues_outside_the_hitbox_and_stops_on_release(cx: &mut gpui::TestAppContext) {
    let (window, entity) = fixture(cx);
    let bounds = support::bounds(cx, window, "slider");
    support::mouse_down(cx, window, bounds.center());
    let beyond = gpui::point(bounds.right() + px(150.0), bounds.bottom() + px(60.0));
    gpui::VisualTestContext::from_window(window, cx).simulate_mouse_move(
        beyond,
        Some(gpui::MouseButton::Left),
        Default::default(),
    );
    support::frame(cx, window);
    assert_eq!(entity.read_with(cx, |view, _| view.value), 10.0);
    let events = entity.read_with(cx, |view, _| view.events);
    gpui::VisualTestContext::from_window(window, cx).simulate_mouse_move(
        beyond,
        Some(gpui::MouseButton::Left),
        Default::default(),
    );
    support::frame(cx, window);
    assert_eq!(
        entity.read_with(cx, |view, _| view.events),
        events,
        "same snapped value is not re-emitted"
    );
    support::mouse_up(cx, window, beyond);
    gpui::VisualTestContext::from_window(window, cx).simulate_mouse_move(
        bounds.origin,
        None,
        Default::default(),
    );
    support::frame(cx, window);
    assert_eq!(entity.read_with(cx, |view, _| view.value), 10.0);
}
