mod support;

use beckon_kit::{
    Root,
    components::{Select, SelectOption},
};
use gpui::{Context, Entity, IntoElement, Render, SharedString, Window, div, prelude::*, px};

struct Host {
    selected: SharedString,
    changes: usize,
    disabled: bool,
}
impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let host = cx.entity();
        Root::new("root")
            .child(
                div().p(px(20.0)).w(px(260.0)).child(
                    Select::new(
                        "select",
                        "Mode",
                        [
                            SelectOption::new("a", "Alpha"),
                            SelectOption::new("disabled", "Unavailable").disabled(true),
                            SelectOption::new("b", "Beta"),
                        ],
                        Some(self.selected.clone()),
                    )
                    .disabled(self.disabled)
                    .on_change(move |value, _, cx| {
                        host.update(cx, |host, cx| {
                            host.selected = value.clone();
                            host.changes += 1;
                            cx.notify();
                        })
                    }),
                ),
            )
            .child(
                div()
                    .id("outside")
                    .debug_selector(|| "outside".into())
                    .size(px(40.0)),
            )
    }
}
fn fixture(cx: &mut gpui::TestAppContext) -> (gpui::AnyWindowHandle, Entity<Host>) {
    let window = support::open(cx, |_, _| Host {
        selected: "a".into(),
        changes: 0,
        disabled: false,
    });
    let host = window.root(cx).unwrap();
    (window.into(), host)
}
fn option_bounds(
    cx: &mut gpui::TestAppContext,
    window: gpui::AnyWindowHandle,
    index: usize,
) -> gpui::Bounds<gpui::Pixels> {
    cx.update_window(window, |_, window, _| {
        gpui::test::observation::find(
            window,
            &[],
            &(
                gpui::ElementId::from("select"),
                SharedString::from(format!("option-{index}")),
            )
                .into(),
        )
        .unwrap_or_else(|| {
            panic!(
                "missing option {index}: {}",
                gpui::test::observation::registered_paths(window)
            )
        })
        .bounds()
    })
    .unwrap()
}

#[gpui::test]
fn mouse_selection_is_controlled_and_disabled_rows_are_inert(cx: &mut gpui::TestAppContext) {
    let (window, host) = fixture(cx);
    support::click(cx, window, "select");
    let position = option_bounds(cx, window, 1).center();
    support::mouse_down(cx, window, position);
    support::mouse_up(cx, window, position);
    assert_eq!(host.read_with(cx, |host, _| host.changes), 0);
    let position = option_bounds(cx, window, 2).center();
    support::mouse_down(cx, window, position);
    support::mouse_up(cx, window, position);
    assert_eq!(
        host.read_with(cx, |host, _| (host.selected.clone(), host.changes)),
        ("b".into(), 1)
    );
}

#[gpui::test]
fn outside_click_closes_without_changing_the_controlled_value(cx: &mut gpui::TestAppContext) {
    let (window, host) = fixture(cx);
    support::click(cx, window, "select");
    let expanded = |cx: &mut gpui::TestAppContext| {
        cx.update_window(window, |_, window, _| {
            gpui::test::observation::find(window, &[], &"select".into())
                .unwrap()
                .expanded()
        })
        .unwrap()
    };
    assert_eq!(expanded(cx), Some(true));
    support::click(cx, window, "outside");
    assert_eq!(expanded(cx), Some(false));
    assert_eq!(
        host.read_with(cx, |host, _| (host.selected.clone(), host.changes)),
        ("a".into(), 0)
    );
}

#[gpui::test]
fn keyboard_skips_disabled_options_and_escape_cancels(cx: &mut gpui::TestAppContext) {
    let (window, host) = fixture(cx);
    support::click(cx, window, "select");
    support::press(cx, window, "down");
    support::press(cx, window, "enter");
    assert_eq!(
        host.read_with(cx, |host, _| (host.selected.clone(), host.changes)),
        ("b".into(), 1)
    );
    support::press(cx, window, "space");
    support::press(cx, window, "home");
    support::press(cx, window, "escape");
    assert_eq!(host.read_with(cx, |host, _| host.changes), 1);
    host.update(cx, |host, cx| {
        host.disabled = true;
        cx.notify();
    });
    support::frame(cx, window);
    support::click(cx, window, "select");
    support::press(cx, window, "enter");
    assert_eq!(host.read_with(cx, |host, _| host.changes), 1);
}
