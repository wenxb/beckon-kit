mod support;

use beckon_kit::{
    Root,
    components::{Button, ButtonVariant},
};
use gpui::{Context, FocusHandle, IntoElement, Render, Window, div, prelude::*, px};

struct Actions {
    a: FocusHandle,
    b: FocusHandle,
    disabled: FocusHandle,
    clicks: usize,
    pending: bool,
}

impl Render for Actions {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        Root::new("root").child(
            div()
                .flex()
                .gap(px(20.0))
                .p(px(20.0))
                .child(
                    Button::new("a", "First")
                        .content(
                            div()
                                .flex()
                                .gap(px(6.0))
                                .child("Ctrl")
                                .child("+")
                                .child("K"),
                        )
                        .variant(ButtonVariant::Primary)
                        .disabled(self.pending)
                        .focus_handle(&self.a)
                        .on_click(cx.listener(|view, _, _, cx| {
                            view.clicks += 1;
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("disabled", "Disabled")
                        .content(div().child("Unavailable"))
                        .disabled(true)
                        .focus_handle(&self.disabled)
                        .on_click(cx.listener(|view, _, _, cx| {
                            view.clicks += 100;
                            cx.notify();
                        })),
                )
                .child(Button::new("busy", "Busy").busy(true).on_click(cx.listener(
                    |view, _, _, cx| {
                        view.clicks += 1000;
                        cx.notify();
                    },
                )))
                .child(
                    Button::new("b", "Second")
                        .focus_handle(&self.b)
                        .on_click(cx.listener(|view, _, _, cx| {
                            view.clicks += 10;
                            cx.notify();
                        })),
                ),
        )
    }
}

fn fixture(cx: &mut gpui::TestAppContext) -> (gpui::AnyWindowHandle, gpui::Entity<Actions>) {
    let window = support::open(cx, |_, cx| Actions {
        a: cx.focus_handle(),
        b: cx.focus_handle(),
        disabled: cx.focus_handle(),
        clicks: 0,
        pending: false,
    });
    let entity = window.root(cx).unwrap();
    (window.into(), entity)
}

#[gpui::test]
fn mouse_press_only_commits_on_release_and_can_cancel(cx: &mut gpui::TestAppContext) {
    let (window, entity) = fixture(cx);
    let button = support::bounds(cx, window, "a");
    support::with_window(cx, window, |window, _| {
        let snapshot = gpui::test::observation::find(window, &[], &"a".into()).unwrap();
        assert_eq!(snapshot.label(), Some("First"));
    });
    support::mouse_down(cx, window, button.center());
    assert_eq!(entity.read_with(cx, |view, _| view.clicks), 0);
    support::mouse_up(cx, window, gpui::point(px(680.0), px(480.0)));
    assert_eq!(entity.read_with(cx, |view, _| view.clicks), 0);
    support::click(cx, window, "a");
    assert_eq!(entity.read_with(cx, |view, _| view.clicks), 1);
}

#[gpui::test]
fn disabled_and_busy_controls_do_not_activate(cx: &mut gpui::TestAppContext) {
    let (window, entity) = fixture(cx);
    support::click(cx, window, "disabled");
    support::click(cx, window, "busy");
    assert_eq!(entity.read_with(cx, |view, _| view.clicks), 0);
}

#[gpui::test]
fn keyboard_activation_does_not_cross_a_focus_change(cx: &mut gpui::TestAppContext) {
    let (window, entity) = fixture(cx);
    let a = entity.read_with(cx, |view, _| view.a.clone());
    let b = entity.read_with(cx, |view, _| view.b.clone());
    support::with_window(cx, window, |window, cx| a.focus(window, cx));
    support::frame(cx, window);
    support::key_down(cx, window, "space");
    assert_eq!(entity.read_with(cx, |view, _| view.clicks), 0);
    support::with_window(cx, window, |window, cx| b.focus(window, cx));
    support::frame(cx, window);
    support::key_up(cx, window, "space");
    assert_eq!(entity.read_with(cx, |view, _| view.clicks), 0);
    support::press(cx, window, "enter");
    assert_eq!(entity.read_with(cx, |view, _| view.clicks), 10);
}

#[gpui::test]
fn tab_navigation_skips_noninteractive_controls(cx: &mut gpui::TestAppContext) {
    let (window, entity) = fixture(cx);
    let a = entity.read_with(cx, |view, _| view.a.clone());
    let b = entity.read_with(cx, |view, _| view.b.clone());
    support::with_window(cx, window, |window, cx| a.focus(window, cx));
    support::frame(cx, window);
    support::press(cx, window, "tab");
    assert!(support::with_window(cx, window, |window, _| b.is_focused(window)));
    support::press(cx, window, "shift-tab");
    assert!(support::with_window(cx, window, |window, _| a.is_focused(window)));
}

#[gpui::test]
fn initial_tab_can_enter_the_component_tree_without_a_prior_click(cx: &mut gpui::TestAppContext) {
    let (window, entity) = fixture(cx);
    support::press(cx, window, "tab");
    let a = entity.read_with(cx, |view, _| view.a.clone());
    assert!(support::with_window(cx, window, |window, _| a.is_focused(window)));
}

#[gpui::test]
fn repaint_keeps_explicit_blur_until_a_control_is_clicked(cx: &mut gpui::TestAppContext) {
    let (window, entity) = fixture(cx);
    support::with_window(cx, window, |window, cx| window.blur(cx));
    support::frame(cx, window);
    assert!(support::with_window(cx, window, |window, cx| window
        .focused(cx)
        .is_none()));
    support::click(cx, window, "a");
    let focus = entity.read_with(cx, |view, _| view.a.clone());
    assert!(support::with_window(cx, window, |window, _| focus.is_focused(window)));
}

#[gpui::test]
fn pressed_feedback_is_visible_before_activation(cx: &mut gpui::TestAppContext) {
    let (window, entity) = fixture(cx);
    let idle = support::with_window(cx, window, |window, _| window.painted_quads());
    let position = support::bounds(cx, window, "a").center();
    support::mouse_down(cx, window, position);
    let pressed = support::with_window(cx, window, |window, _| window.painted_quads());
    assert_eq!(entity.read_with(cx, |view, _| view.clicks), 0);
    let idle_colors: Vec<_> = idle.iter().map(|quad| quad.background).collect();
    let pressed_colors: Vec<_> = pressed.iter().map(|quad| quad.background).collect();
    assert_ne!(idle_colors, pressed_colors);
}

#[gpui::test]
fn keyboard_press_feedback_is_immediate_and_does_not_change_layout(cx: &mut gpui::TestAppContext) {
    let (window, entity) = fixture(cx);
    let focus = entity.read_with(cx, |view, _| view.a.clone());
    support::with_window(cx, window, |window, cx| focus.focus(window, cx));
    support::frame(cx, window);
    let bounds = support::bounds(cx, window, "a");
    let colors = beckon_kit::theme::Theme::default().colors;
    let contains = |cx: &mut gpui::TestAppContext, color: gpui::Hsla| {
        support::with_window(cx, window, |window, _| {
            window
                .painted_quads()
                .iter()
                .any(|quad| quad.background.as_solid() == Some(color))
        })
    };
    assert!(contains(cx, colors.primary));
    support::key_down(cx, window, "space");
    assert!(contains(cx, colors.primary_pressed));
    assert_eq!(support::bounds(cx, window, "a"), bounds);
    assert_eq!(entity.read_with(cx, |view, _| view.clicks), 0);
    support::key_up(cx, window, "space");
    assert!(contains(cx, colors.primary));
    assert_eq!(entity.read_with(cx, |view, _| view.clicks), 1);
}

#[gpui::test]
fn temporary_disable_keeps_focus_without_allowing_activation(cx: &mut gpui::TestAppContext) {
    let (window, entity) = fixture(cx);
    let focus = entity.read_with(cx, |view, _| view.a.clone());
    support::with_window(cx, window, |window, cx| focus.focus(window, cx));
    entity.update(cx, |view, cx| {
        view.pending = true;
        cx.notify();
    });
    support::frame(cx, window);
    support::press(cx, window, "enter");
    assert_eq!(entity.read_with(cx, |view, _| view.clicks), 0);
    entity.update(cx, |view, cx| {
        view.pending = false;
        cx.notify();
    });
    support::frame(cx, window);
    assert!(support::with_window(cx, window, |window, _| focus.is_focused(window)));
    support::press(cx, window, "enter");
    assert_eq!(entity.read_with(cx, |view, _| view.clicks), 1);
}
