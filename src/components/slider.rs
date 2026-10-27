use gpui::{
    App, Bounds, DispatchPhase, ElementId, Entity, FocusHandle, IntoElement, MouseButton,
    MouseMoveEvent, MouseUpEvent, ParentElement, Pixels, RenderOnce, Role, SharedString,
    StyleRefinement, Styled, Window,
    accesskit::{Action, ActionData},
    canvas, div, fill, point,
    prelude::*,
    px, size,
};
use std::rc::Rc;

use crate::{
    element::Semantic,
    interaction::{SliderScale, control_focus},
    theme::{Colors, Theme, control_transition},
};

type ChangeHandler = Rc<dyn Fn(&f64, &mut Window, &mut App)>;

#[derive(Default)]
struct DragState {
    bounds: Bounds<Pixels>,
    active: bool,
    direct: bool,
    origin: gpui::Point<Pixels>,
    presentation: f32,
    grab_offset: Pixels,
    last_emitted: Option<f64>,
}

#[derive(IntoElement)]
pub struct Slider {
    id: ElementId,
    label: SharedString,
    scale: SliderScale,
    value: f64,
    disabled: bool,
    callback: Option<ChangeHandler>,
    commit: Option<ChangeHandler>,
    focus: Option<FocusHandle>,
    style: StyleRefinement,
}

impl Slider {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        scale: SliderScale,
        value: f64,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            scale,
            value: scale.normalize(value),
            disabled: false,
            callback: None,
            commit: None,
            focus: None,
            style: StyleRefinement::default(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn focus_handle(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }

    pub fn on_change(mut self, callback: impl Fn(&f64, &mut Window, &mut App) + 'static) -> Self {
        self.callback = Some(Rc::new(callback));
        self
    }

    /// Pointer release, or one keyboard/accessibility step, commits the current value.
    pub fn on_commit(mut self, callback: impl Fn(&f64, &mut Window, &mut App) + 'static) -> Self {
        self.commit = Some(Rc::new(callback));
        self
    }
}

impl Styled for Slider {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

fn emit_pointer(
    drag: &Entity<DragState>,
    position: gpui::Point<Pixels>,
    scale: SliderScale,
    callback: &Option<ChangeHandler>,
    window: &mut Window,
    cx: &mut App,
) {
    let value = drag.update(cx, |drag, _| {
        let width = (drag.bounds.size.width - px(16.0)).max(px(1.0));
        let fraction = (position.x - drag.bounds.left() - px(8.0) - drag.grab_offset) / width;
        let value = scale.value_at(fraction);
        if drag.last_emitted == Some(value) {
            return None;
        }
        drag.last_emitted = Some(value);
        Some(value)
    });
    if let Some(value) = value
        && let Some(callback) = callback
    {
        callback(&value, window, cx);
    }
}

fn paint_track(
    bounds: Bounds<Pixels>,
    fraction: f32,
    press: f32,
    colors: &Colors,
    enabled: bool,
    window: &mut Window,
) {
    let width = (bounds.size.width - px(16.0)).max(px(0.0));
    let origin = bounds.origin + point(px(8.0), bounds.size.height / 2.0 - px(2.0));
    let active = if enabled {
        colors.info
    } else {
        colors.disabled
    };
    window.paint_quad(
        fill(Bounds::new(origin, size(width, px(4.0))), colors.border).corner_radii(px(2.0)),
    );
    let x = width * fraction.clamp(0.0, 1.0);
    window.paint_quad(fill(Bounds::new(origin, size(x, px(4.0))), active).corner_radii(px(2.0)));
    let radius = px(7.0 + press);
    window.paint_quad(
        fill(
            Bounds::new(
                origin + point(x - radius, px(2.0) - radius),
                size(radius * 2.0, radius * 2.0),
            ),
            colors.elevated,
        )
        .corner_radii(radius)
        .border_widths(px(1.0))
        .border_color(colors.secondary),
    );
}

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let scale = self.scale;
        let value = self.value;
        let enabled = !self.disabled;
        let drag =
            window.use_keyed_state((self.id.clone(), "drag"), cx, |_, _| DragState::default());
        if !enabled {
            drag.update(cx, |drag, _| drag.active = false);
        }
        let active = drag.read(cx).active;
        let direct = active && drag.read(cx).direct;
        let fraction = control_transition(
            (self.id.clone(), "slider-value"),
            scale.fraction(value),
            direct || !enabled,
            window,
            cx,
        );
        let press = control_transition(
            (self.id.clone(), "slider-press"),
            active,
            !enabled,
            window,
            cx,
        );
        let focus = control_focus(self.id.clone(), self.focus, window, cx);
        let selector = self.id.to_string();
        let mut element = div()
            .id(self.id)
            .key_context("BeckonKitSlider")
            .role(Role::Slider)
            .aria_label(self.label)
            .aria_orientation(gpui::Orientation::Horizontal)
            .aria_numeric_value(value)
            .aria_min_numeric_value(scale.min())
            .aria_max_numeric_value(scale.max())
            .aria_numeric_value_step(scale.step())
            .debug_selector(move || selector)
            .w_full()
            .min_w(px(48.0))
            .h(px(theme.metrics.control_height))
            .rounded(px(theme.metrics.control_radius))
            .border(px(theme.metrics.focus_width))
            .border_color(gpui::transparent_black());
        if enabled {
            let down_drag = drag.clone();
            let down_focus = focus.clone();
            let down_callback = self.callback.clone();
            let key_callback = self.callback.clone();
            let key_commit = self.commit.clone();
            let ring = theme.colors.focus;
            element = element
                .track_focus(&focus)
                .tab_index(0)
                .cursor_pointer()
                .focus_visible(move |style| style.border_color(ring))
                .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                    down_focus.focus(window, cx);
                    down_drag.update(cx, |drag, _| {
                        drag.active = true;
                        let width = (drag.bounds.size.width - px(16.0)).max(px(1.0));
                        let thumb = drag.bounds.left() + px(8.0) + width * drag.presentation;
                        let offset = event.position.x - thumb;
                        drag.direct = offset.abs() <= px(8.0);
                        drag.grab_offset = if drag.direct { offset } else { px(0.0) };
                        drag.origin = event.position;
                        drag.last_emitted = Some(value);
                    });
                    emit_pointer(
                        &down_drag,
                        event.position,
                        scale,
                        &down_callback,
                        window,
                        cx,
                    );
                    window.refresh();
                    cx.stop_propagation();
                })
                .on_key_down(move |event, window, cx| {
                    let modifiers = event.keystroke.modifiers;
                    if modifiers.control || modifiers.platform || modifiers.alt {
                        return;
                    }
                    let steps = if modifiers.shift { 10.0 } else { 1.0 };
                    let next = match event.keystroke.key.as_str() {
                        "left" | "down" => scale.offset(value, -steps),
                        "right" | "up" => scale.offset(value, steps),
                        "pageup" => scale.offset(value, 10.0),
                        "pagedown" => scale.offset(value, -10.0),
                        "home" => scale.min(),
                        "end" => scale.max(),
                        _ => return,
                    };
                    if next != value {
                        if let Some(callback) = &key_callback {
                            callback(&next, window, cx);
                        }
                        if let Some(callback) = &key_commit {
                            callback(&next, window, cx);
                        }
                    }
                    cx.stop_propagation();
                });
            for (action, next) in [
                (Action::Increment, scale.offset(value, 1.0)),
                (Action::Decrement, scale.offset(value, -1.0)),
            ] {
                let callback = self.callback.clone();
                let commit = self.commit.clone();
                element = element.on_a11y_action(action, move |_, window, cx| {
                    if next != value {
                        if let Some(callback) = &callback {
                            callback(&next, window, cx);
                        }
                        if let Some(callback) = &commit {
                            callback(&next, window, cx);
                        }
                    }
                });
            }
            let callback = self.callback.clone();
            let commit = self.commit.clone();
            element = element.on_a11y_action(Action::SetValue, move |data, window, cx| {
                if let Some(ActionData::NumericValue(next)) = data {
                    let next = scale.normalize(*next);
                    if next != value {
                        if let Some(callback) = &callback {
                            callback(&next, window, cx);
                        }
                        if let Some(callback) = &commit {
                            callback(&next, window, cx);
                        }
                    }
                }
            });
        }
        let bounds_drag = drag.clone();
        let callback = self.callback;
        let commit = self.commit;
        let colors = theme.colors.clone();
        element = element.child(
            canvas(
                move |bounds, _, cx| {
                    bounds_drag.update(cx, |drag, _| {
                        drag.bounds = bounds;
                        drag.presentation = fraction.clamp(0.0, 1.0);
                    })
                },
                move |bounds, _, window, _| {
                    paint_track(bounds, fraction, press.0, &colors, enabled, window);
                    if !enabled {
                        return;
                    }
                    let move_drag = drag.clone();
                    let move_callback = callback.clone();
                    window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                        if phase == DispatchPhase::Capture && move_drag.read(cx).active {
                            if event.pressed_button != Some(MouseButton::Left) {
                                move_drag.update(cx, |drag, _| drag.active = false);
                                window.refresh();
                            } else {
                                move_drag.update(cx, |drag, _| {
                                    if (event.position.x - drag.origin.x).abs() > px(2.0) {
                                        drag.direct = true;
                                    }
                                });
                                // A drag tracks the pointer directly, never an eased target.
                                window.refresh();
                                emit_pointer(
                                    &move_drag,
                                    event.position,
                                    scale,
                                    &move_callback,
                                    window,
                                    cx,
                                );
                            }
                        }
                    });
                    window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                        if phase == DispatchPhase::Capture
                            && event.button == MouseButton::Left
                            && drag.read(cx).active
                        {
                            emit_pointer(&drag, event.position, scale, &callback, window, cx);
                            let last = drag.update(cx, |drag, _| {
                                drag.active = false;
                                drag.last_emitted.take()
                            });
                            if let (Some(value), Some(commit)) = (last, &commit) {
                                commit(&value, window, cx);
                            }
                            window.refresh();
                        }
                    });
                },
            )
            .size_full(),
        );
        element.style().refine(&self.style);
        Semantic::new(element).disabled(self.disabled)
    }
}
