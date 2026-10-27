use gpui::{
    App, BoxShadow, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, MouseButton, Role,
    SharedString, Stateful, StatefulInteractiveElement, Styled, Window, div, px,
};
use std::rc::Rc;

use crate::theme::Theme;

pub(crate) type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

pub(crate) struct Pressable {
    pub id: ElementId,
    pub label: SharedString,
    pub role: Role,
    pub enabled: bool,
    pub focus: Option<FocusHandle>,
    pub callback: Option<ClickHandler>,
}

impl Pressable {
    pub fn render(self, window: &mut Window, cx: &mut App) -> (Stateful<Div>, bool) {
        let selector = self.id.to_string();
        let mut element = div()
            .id(self.id.clone())
            .role(self.role)
            .aria_label(self.label)
            .debug_selector(move || selector);
        let focus = control_focus(self.id.clone(), self.focus, window, cx).tab_stop(self.enabled);
        element = element.track_focus(&focus);
        if !self.enabled {
            return (element.block_mouse_except_scroll(), false);
        }

        let theme = Theme::current(cx);
        let keyboard = window.use_keyed_state((self.id, "keyboard-press"), cx, |_, _| false);
        if !focus.is_focused(window) {
            keyboard.update(cx, |pressed, _| *pressed = false);
        }
        let pressed = *keyboard.read(cx);
        let down_keyboard = keyboard.clone();
        element = element
            .key_context("BeckonKitPressable")
            .cursor_pointer()
            .focus_visible(move |style| {
                style.border_color(theme.colors.focus).shadow(vec![
                    BoxShadow::new(px(0.0), px(0.0), theme.colors.focus).spread_radius(px(1.0)),
                ])
            })
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                focus.focus(window, cx);
                cx.stop_propagation();
            })
            .on_key_down(move |event, _, cx| {
                let stroke = &event.keystroke;
                let active = !stroke.modifiers.modified()
                    && matches!(stroke.key.as_str(), "enter" | "space");
                down_keyboard.update(cx, |pressed, cx| {
                    if *pressed != active {
                        *pressed = active;
                        cx.notify();
                    }
                });
                if active {
                    cx.stop_propagation();
                }
            })
            .capture_key_up(move |_, _, cx| {
                keyboard.update(cx, |pressed, cx| {
                    if *pressed {
                        *pressed = false;
                        cx.notify();
                    }
                });
            });
        if let Some(callback) = self.callback {
            element = element.on_click(move |event, window, cx| {
                callback(event, window, cx);
                cx.stop_propagation();
            });
        }
        (element, pressed)
    }
}

pub(crate) fn control_focus(
    id: ElementId,
    supplied: Option<FocusHandle>,
    window: &mut Window,
    cx: &mut App,
) -> FocusHandle {
    let focus = supplied.unwrap_or_else(|| {
        window
            .use_keyed_state((id, "focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone()
    });
    // Div's tab_index builder only updates automatically allocated focus handles.
    focus.tab_stop(true)
}
