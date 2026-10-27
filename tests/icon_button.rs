#![cfg(feature = "test-support")]
mod support;

use beckon_kit::{
    Root,
    components::{Button, ButtonVariant, ControlSize, Icon, IconName},
    theme::{Appearance, Density, Theme, ThemeScope},
};
use gpui::{
    App, Context, Element, IntoElement, Render, RenderOnce, Role, TestAppContext, Window,
    accesskit::Node, div, prelude::*, px,
};
use std::{cell::RefCell, rc::Rc, time::Duration};

#[derive(IntoElement)]
struct InspectButton {
    button: Button,
    semantics: Rc<RefCell<Node>>,
}

impl RenderOnce for InspectButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let element = self.button.render(window, cx).into_element();
        let mut node = Node::new(element.a11y_role().unwrap());
        element.write_a11y_info(&mut node);
        *self.semantics.borrow_mut() = node;
        element
    }
}

struct Actions {
    theme: Theme,
    size: ControlSize,
    disabled: bool,
    busy: bool,
    tooltip: bool,
    clicks: usize,
    semantics: Rc<RefCell<Node>>,
}

impl Render for Actions {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        ThemeScope::new(
            self.theme.clone(),
            Root::new("actions").child(
                div().p(px(20.0)).child(InspectButton {
                    button: Button::icon("icon-action", "紧凑布局", Icon::new(IconName::Settings2))
                        .size(self.size)
                        .variant(ButtonVariant::Quiet)
                        .toggled(self.clicks % 2 == 1)
                        .disabled(self.disabled)
                        .busy(self.busy)
                        .tooltip(self.tooltip)
                        .on_click(cx.listener(|view, _, _, cx| {
                            view.clicks += 1;
                            cx.notify();
                        })),
                    semantics: self.semantics.clone(),
                }),
            ),
        )
    }
}

fn fixture(cx: &mut TestAppContext) -> (gpui::AnyWindowHandle, gpui::Entity<Actions>) {
    let handle = support::open(cx, |_, _| Actions {
        theme: Theme::default(),
        size: ControlSize::Small,
        disabled: false,
        busy: false,
        tooltip: true,
        clicks: 0,
        semantics: Rc::new(RefCell::new(Node::new(Role::Button))),
    });
    (handle.into(), handle.root(cx).unwrap())
}

#[gpui::test]
fn icon_button_is_square_and_keeps_labels_toggle_state_and_keyboard_input(cx: &mut TestAppContext) {
    let (window, view) = fixture(cx);
    let semantics = view.read_with(cx, |view, _| view.semantics.clone());
    assert_eq!(semantics.borrow().role(), Role::Button);
    assert_eq!(semantics.borrow().label(), Some("紧凑布局"));
    assert_eq!(semantics.borrow().toggled(), Some(gpui::Toggled::False));
    for density in [Density::Comfortable, Density::Compact] {
        for size in [ControlSize::Small, ControlSize::Medium, ControlSize::Large] {
            view.update(cx, |view, cx| {
                view.theme = Theme::default().density(density);
                view.size = size;
                cx.notify();
            });
            support::frame(cx, window);
            let bounds = support::bounds(cx, window, "icon-action");
            assert_eq!(bounds.size.width, bounds.size.height);
            // The 27.5px typography minimum is rounded to the device-pixel grid.
            assert!(bounds.size.width >= px(27.0));
        }
    }
    support::press(cx, window, "tab");
    support::press(cx, window, "space");
    assert_eq!(view.read_with(cx, |v, _| v.clicks), 1);
    assert_eq!(semantics.borrow().toggled(), Some(gpui::Toggled::True));
    support::click(cx, window, "icon-action");
    support::press(cx, window, "enter");
    assert_eq!(view.read_with(cx, |v, _| v.clicks), 3);
    for busy in [false, true] {
        view.update(cx, |v, cx| {
            v.disabled = !busy;
            v.busy = busy;
            cx.notify();
        });
        support::frame(cx, window);
        assert_eq!(semantics.borrow().is_disabled(), !busy);
        assert_eq!(semantics.borrow().is_busy(), busy);
        let bounds = support::bounds(cx, window, "icon-action");
        assert_eq!(bounds.size.width, bounds.size.height);
        support::click(cx, window, "icon-action");
        support::press(cx, window, "space");
        assert_eq!(view.read_with(cx, |v, _| v.clicks), 3);
    }
}

#[gpui::test]
fn icon_hint_uses_the_scoped_theme_and_does_not_shift_the_button(cx: &mut TestAppContext) {
    let (window, view) = fixture(cx);
    for appearance in [Appearance::Light, Appearance::Dark] {
        view.update(cx, |view, cx| {
            view.theme = Theme::new(appearance);
            cx.notify();
        });
        support::with_window(cx, window, |_, cx| {
            Theme::new(if appearance == Appearance::Light {
                Appearance::Dark
            } else {
                Appearance::Light
            })
            .install(cx);
        });
        support::frame(cx, window);
        let button = support::bounds(cx, window, "icon-action");
        gpui::VisualTestContext::from_window(window, cx).simulate_mouse_move(
            button.center(),
            None,
            Default::default(),
        );
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(600));
        support::frame(cx, window);
        let hint = support::bounds(cx, window, "button-tooltip");
        assert!(hint.size.width > px(0.0));
        assert_eq!(support::bounds(cx, window, "icon-action"), button);
        let colors = Theme::new(appearance).colors;
        support::with_window(cx, window, |window, _| {
            let hint = hint.scale(window.scale_factor());
            assert!(window.painted_quads().iter().any(|quad| {
                quad.bounds == hint && quad.background.as_solid() == Some(colors.elevated)
            }));
        });
        gpui::VisualTestContext::from_window(window, cx).simulate_mouse_move(
            gpui::point(px(600.0), px(400.0)),
            None,
            Default::default(),
        );
        support::frame(cx, window);
        assert!(
            gpui::VisualTestContext::from_window(window, cx)
                .debug_bounds("button-tooltip")
                .is_none()
        );
    }
}

#[gpui::test]
fn disabling_the_hint_keeps_the_accessible_name_and_keyboard_action(cx: &mut TestAppContext) {
    let (window, view) = fixture(cx);
    view.update(cx, |view, cx| {
        view.tooltip = false;
        cx.notify();
    });
    support::frame(cx, window);
    let bounds = support::bounds(cx, window, "icon-action");
    gpui::VisualTestContext::from_window(window, cx).simulate_mouse_move(
        bounds.center(),
        None,
        Default::default(),
    );
    cx.run_until_parked();
    cx.executor().advance_clock(Duration::from_millis(1000));
    support::frame(cx, window);
    assert!(
        gpui::VisualTestContext::from_window(window, cx)
            .debug_bounds("button-tooltip")
            .is_none()
    );
    let semantics = view.read_with(cx, |view, _| view.semantics.clone());
    assert_eq!(semantics.borrow().label(), Some("紧凑布局"));
    support::click(cx, window, "icon-action");
    support::press(cx, window, "enter");
    assert_eq!(view.read_with(cx, |view, _| view.clicks), 2);
}
