use std::rc::Rc;

use gpui::{
    App, Bounds, ElementId, Entity, FocusHandle, IntoElement, MouseButton, ParentElement, Pixels,
    Refineable, RenderOnce, Role, SharedString, StyleRefinement, Styled, Window, anchored, canvas,
    deferred, div, point, prelude::*, px,
};

use crate::{
    components::{Button, ButtonVariant, Icon, IconName},
    element::Semantic,
    theme::Theme,
};

type ChangeHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectOption {
    pub value: SharedString,
    pub label: SharedString,
    pub disabled: bool,
}

impl SelectOption {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Default)]
struct Popup {
    open: bool,
    active: usize,
    bounds: Bounds<Pixels>,
}

/// Controlled single selection. Popup state never becomes a second selected value.
#[derive(IntoElement)]
pub struct Select {
    id: ElementId,
    label: SharedString,
    options: Vec<SelectOption>,
    selected: Option<SharedString>,
    placeholder: SharedString,
    disabled: bool,
    focus: Option<FocusHandle>,
    callback: Option<ChangeHandler>,
    style: StyleRefinement,
}

impl Select {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        options: impl IntoIterator<Item = SelectOption>,
        selected: Option<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            options: options.into_iter().collect(),
            selected,
            placeholder: "Select…".into(),
            disabled: false,
            focus: None,
            callback: None,
            style: StyleRefinement::default(),
        }
    }
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn focus_handle(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }
    pub fn on_change(
        mut self,
        callback: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callback = Some(Rc::new(callback));
        self
    }
}

impl Styled for Select {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

fn close(popup: &Entity<Popup>, window: &mut Window, cx: &mut App) {
    popup.update(cx, |popup, _| popup.open = false);
    window.refresh();
}

impl RenderOnce for Select {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let selector = self.id.to_string();
        let focus = crate::interaction::control_focus(self.id.clone(), self.focus, window, cx);
        let popup = window.use_keyed_state((self.id.clone(), "select-popup"), cx, |_, _| {
            Popup::default()
        });
        let enabled: Vec<usize> = self
            .options
            .iter()
            .enumerate()
            .filter_map(|(i, item)| (!item.disabled).then_some(i))
            .collect();
        let disabled = self.disabled || enabled.is_empty();
        let selected_index = self
            .options
            .iter()
            .position(|item| Some(&item.value) == self.selected.as_ref());
        if disabled || (popup.read(cx).open && !focus.is_focused(window)) {
            popup.update(cx, |state, _| state.open = false);
        }
        let open = popup.read(cx).open;
        let label = selected_index
            .map(|i| self.options[i].label.clone())
            .unwrap_or(self.placeholder);
        let mut root = div()
            .id(self.id.clone())
            .key_context(if open {
                "BeckonKitSelectOpen"
            } else {
                "BeckonKitSelect"
            })
            .role(Role::ComboBox)
            .aria_label(self.label)
            .aria_value(label.clone())
            .aria_expanded(open)
            .debug_selector(move || selector)
            .relative()
            .w_full();
        if !disabled {
            let key_popup = popup.clone();
            let options = self.options.clone();
            let callback = self.callback.clone();
            root = root
                .track_focus(&focus)
                .tab_index(0)
                .capture_key_down(move |event, window, cx| {
                    if event.keystroke.modifiers.modified() {
                        return;
                    }
                    let key = event.keystroke.key.as_str();
                    let state = key_popup.read(cx);
                    if key == "escape" && state.open {
                        close(&key_popup, window, cx);
                    } else if matches!(key, "enter" | "space") && state.open {
                        let index = state.active;
                        close(&key_popup, window, cx);
                        if let Some(item) = options.get(index).filter(|item| !item.disabled)
                            && let Some(callback) = &callback
                        {
                            callback(&item.value, window, cx);
                        }
                    } else if matches!(key, "enter" | "space" | "up" | "down" | "home" | "end") {
                        let is_open = state.open;
                        let position = enabled.iter().position(|i| *i == state.active).unwrap_or(0);
                        let next = if !is_open {
                            selected_index
                                .filter(|i| enabled.contains(i))
                                .unwrap_or(enabled[0])
                        } else {
                            enabled[match key {
                                "up" => (position + enabled.len() - 1) % enabled.len(),
                                "down" => (position + 1) % enabled.len(),
                                "end" => enabled.len() - 1,
                                "home" => 0,
                                _ => position,
                            }]
                        };
                        key_popup.update(cx, |state, _| {
                            state.active = next;
                            state.open = true;
                        });
                        window.refresh();
                    } else {
                        return;
                    }
                    window.prevent_default();
                    cx.stop_propagation();
                })
                .capture_key_up(|event, window, cx| {
                    if !event.keystroke.modifiers.modified()
                        && matches!(event.keystroke.key.as_str(), "enter" | "space")
                    {
                        window.prevent_default();
                        cx.stop_propagation();
                    }
                });
        }
        let toggle = popup.clone();
        let first = selected_index
            .filter(|index| !self.options[*index].disabled)
            .or_else(|| self.options.iter().position(|item| !item.disabled))
            .unwrap_or(0);
        let trigger_focus = focus.clone();
        root = root.child(
            Button::new((self.id.clone(), "trigger"), label)
                .variant(ButtonVariant::Outline)
                .disabled(disabled)
                .focus_handle(&focus)
                .w_full()
                .justify_between()
                .leading(Icon::new(if open {
                    IconName::ChevronUp
                } else {
                    IconName::ChevronDown
                }))
                .on_click(move |event, window, cx| {
                    if matches!(event, gpui::ClickEvent::Keyboard(_)) {
                        return;
                    }
                    trigger_focus.focus(window, cx);
                    toggle.update(cx, |state, _| {
                        state.open = !state.open;
                        state.active = first;
                    });
                    window.refresh();
                }),
        );
        let geometry = popup.clone();
        root = root.child(
            canvas(
                move |bounds, _, cx| geometry.update(cx, |state, _| state.bounds = bounds),
                |_, _, _, _| {},
            )
            .absolute()
            .size_full(),
        );
        if open {
            let bounds = popup.read(cx).bounds;
            let outside = popup.clone();
            let active = popup.read(cx).active;
            let mut list = div()
                .id((self.id.clone(), "list"))
                .role(Role::ListBox)
                .min_w(bounds.size.width.max(px(200.0)))
                .max_h(px(240.0))
                .overflow_y_scroll()
                .p(px(4.0))
                .rounded(px(theme.metrics.control_radius))
                .border_1()
                .border_color(theme.colors.border)
                .bg(theme.colors.surface)
                .shadow_md()
                .on_mouse_down(MouseButton::Left, |_, window, cx| {
                    // Deferred rows must not transfer focus to the surrounding root.
                    window.prevent_default();
                    cx.stop_propagation();
                })
                .on_mouse_down_out(move |event, window, cx| {
                    if !bounds.contains(&event.position) {
                        close(&outside, window, cx);
                    }
                });
            for (index, item) in self.options.into_iter().enumerate() {
                let state = popup.clone();
                let callback = self.callback.clone();
                let focus = focus.clone();
                let selected = Some(&item.value) == self.selected.as_ref();
                let mut row = div()
                    .id((
                        self.id.clone(),
                        SharedString::from(format!("option-{index}")),
                    ))
                    .role(Role::ListBoxOption)
                    .aria_label(item.label.clone())
                    .aria_selected(selected)
                    .min_h(px(theme.metrics.control_height))
                    .px(px(10.0))
                    .flex()
                    .items_center()
                    .rounded(px(4.0))
                    .text_color(if item.disabled {
                        theme.colors.disabled
                    } else {
                        theme.colors.text
                    })
                    .when(index == active, |row| row.bg(theme.colors.selection))
                    .when(index == active, |row| row.aria_active_descendant())
                    .child(item.label);
                if !item.disabled {
                    row = row
                        .cursor_pointer()
                        .hover({
                            let color = theme.colors.hover;
                            move |style| style.bg(color)
                        })
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .on_click(move |_, window, cx| {
                            close(&state, window, cx);
                            focus.focus(window, cx);
                            if let Some(callback) = &callback {
                                callback(&item.value, window, cx);
                            }
                            cx.stop_propagation();
                        });
                }
                list = list.child(Semantic::new(row).disabled(item.disabled));
            }
            root = root.child(
                deferred(
                    anchored()
                        .position(bounds.bottom_left() + point(px(0.0), px(4.0)))
                        .snap_to_window()
                        .child(list),
                )
                .with_priority(1),
            );
        }
        root.style().refine(&self.style);
        Semantic::new(root).disabled(disabled)
    }
}
