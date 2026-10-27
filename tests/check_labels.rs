mod support;
use beckon_kit::{
    Root,
    components::{CheckState, Checkbox, Toggle},
};
use gpui::{Context, IntoElement, Render, Window, div, prelude::*, px};

struct Labels {
    on: bool,
    checked: CheckState,
}
impl Render for Labels {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        Root::new("labels").child(
            div()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(8.0))
                .child(Toggle::new("default", "Notifications", self.on))
                .child(
                    Toggle::new("hidden", "Notifications", self.on)
                        .label_visible(false)
                        .on_change(cx.listener(|view, on: &bool, _, cx| {
                            view.on = *on;
                            cx.notify();
                        })),
                )
                .child(
                    Checkbox::new("check", "Notifications", self.checked)
                        .label_visible(false)
                        .on_change(cx.listener(|view, state: &CheckState, _, cx| {
                            view.checked = *state;
                            cx.notify();
                        })),
                ),
        )
    }
}
#[gpui::test]
fn hidden_labels_keep_accessible_names_and_mouse_keyboard_behavior(cx: &mut gpui::TestAppContext) {
    let handle = support::open(cx, |_, _| Labels {
        on: false,
        checked: CheckState::Off,
    });
    let view = handle.root(cx).unwrap();
    let default = support::bounds(cx, handle.into(), "default");
    let hidden = support::bounds(cx, handle.into(), "hidden");
    assert!(default.size.width > hidden.size.width + px(50.0));
    assert!(hidden.size.width < px(55.0), "indicator only: {hidden:?}");
    cx.update_window(handle.into(), |_, window, _| {
        for id in ["hidden", "check"] {
            let snapshot = gpui::test::observation::find(window, &[], &id.into()).unwrap();
            assert_eq!(snapshot.label(), Some("Notifications"));
            assert_eq!(snapshot.checked(), Some(false));
        }
    })
    .unwrap();
    support::click(cx, handle.into(), "hidden");
    assert!(view.read_with(cx, |view, _| view.on));
    support::press(cx, handle.into(), "space");
    assert!(!view.read_with(cx, |view, _| view.on));
    support::click(cx, handle.into(), "check");
    assert_eq!(view.read_with(cx, |view, _| view.checked), CheckState::On);
    support::press(cx, handle.into(), "enter");
    assert_eq!(view.read_with(cx, |view, _| view.checked), CheckState::Off);
}
