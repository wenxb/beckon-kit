//! Retained native paths and shaped text; text editing belongs to an external `TextField`.

mod compiled;
mod hit;
mod input;
mod model;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod geometry_tests;

pub use model::{Node, Phase, Point, PointerEvent, Rect, Scene, Stroke, Transform};

use std::{rc::Rc, sync::Arc};

use gpui::{
    App, DispatchPhase, ElementId, FocusHandle, IntoElement, MouseButton, MouseMoveEvent,
    MouseUpEvent, RenderOnce, Role, SharedString, StyleRefinement, Styled, Window, canvas, div,
    prelude::*, px,
};

use self::{compiled::Compiled, input::Input};
use crate::{interaction::control_focus, theme::Theme};

type PointerHandler = Rc<dyn Fn(&PointerEvent, &mut Window, &mut App)>;

/// A clipped logical-pixel viewport over an immutable scene, with primary-pointer capture.
#[derive(IntoElement)]
pub struct Canvas {
    id: ElementId,
    scene: Arc<Scene>,
    callback: Option<PointerHandler>,
    focus: Option<FocusHandle>,
    label: SharedString,
    style: StyleRefinement,
}

impl Canvas {
    pub fn new(id: impl Into<ElementId>, scene: Arc<Scene>) -> Self {
        Self {
            id: id.into(),
            scene,
            callback: None,
            focus: None,
            label: "Canvas".into(),
            style: StyleRefinement::default(),
        }
    }

    pub fn on_pointer(
        mut self,
        callback: impl Fn(&PointerEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callback = Some(Rc::new(callback));
        self
    }

    pub fn focus_handle(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }
}

impl Styled for Canvas {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Canvas {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let mut text_style = window.text_style();
        text_style.refine(&theme.text_style());
        text_style.refine(&self.style.text);
        let font = text_style.font();
        let compiled =
            window.use_keyed_state((self.id.clone(), "canvas-scene"), cx, |window, _| {
                Compiled::new(self.scene.clone(), font.clone(), window)
            });
        if !Arc::ptr_eq(&compiled.read(cx).scene, &self.scene) || compiled.read(cx).font != font {
            compiled.update(cx, |compiled, _| {
                *compiled = Compiled::new(self.scene.clone(), font, window);
            });
        }
        let input = window.use_keyed_state((self.id.clone(), "canvas-input"), cx, |_, _| {
            Input::default()
        });
        input.update(cx, |input, _| input.transform = self.scene.transform);
        let focus = control_focus(self.id.clone(), self.focus, window, cx);
        let down_input = input.clone();
        let down_compiled = compiled.clone();
        let down_callback = self.callback.clone();
        let selector = self.id.to_string();
        let ring = theme.colors.focus;
        let width = px(self.scene.width);
        let height = px(self.scene.height);
        let mut element = div()
            .id(self.id)
            .role(Role::Image)
            .aria_label(self.label)
            .aria_description(compiled.read(cx).alternative.clone())
            .debug_selector(move || selector)
            .key_context("BeckonKitCanvas")
            .track_focus(&focus)
            .tab_index(0)
            .w(width)
            .h(height)
            .flex_shrink_0()
            .focus_visible(move |style| {
                style.shadow(vec![
                    gpui::BoxShadow::new(px(0.0), px(0.0), ring).spread_radius(px(2.0)),
                ])
            })
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                let state = down_input.read(cx);
                let local = state.local(event.position);
                if !state.bounds.contains(&event.position)
                    || !down_compiled.read(cx).scene.viewport().contains(local)
                {
                    return;
                }
                let target = down_compiled.read(cx).hit_test(local).map(str::to_owned);
                focus.focus(window, cx);
                let event = down_input.update(cx, |input, _| input.down(event.position, target));
                if let (Some(event), Some(callback)) = (event, &down_callback) {
                    callback(&event, window, cx);
                }
                cx.stop_propagation();
            });
        let bounds_input = input.clone();
        let bounds_compiled = compiled.clone();
        let callback = self.callback;
        element = element.child(
            canvas(
                move |bounds, _, cx| {
                    bounds_input.update(cx, |input, _| input.bounds = bounds);
                    bounds_compiled.update(cx, |compiled, _| compiled.place(bounds.origin));
                },
                move |bounds, _, window, cx| {
                    compiled.update(cx, |compiled, cx| compiled.paint(bounds, window, cx));
                    let move_input = input.clone();
                    let move_callback = callback.clone();
                    window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                        if phase != DispatchPhase::Capture || !move_input.read(cx).active() {
                            return;
                        }
                        let phase = if event.pressed_button == Some(MouseButton::Left) {
                            Phase::Move
                        } else {
                            Phase::Cancel
                        };
                        let event =
                            move_input.update(cx, |input, _| input.next(phase, event.position));
                        if let (Some(event), Some(callback)) = (event, &move_callback) {
                            callback(&event, window, cx);
                            cx.stop_propagation();
                        }
                    });
                    window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                        if phase != DispatchPhase::Capture
                            || event.button != MouseButton::Left
                            || !input.read(cx).active()
                        {
                            return;
                        }
                        let event =
                            input.update(cx, |input, _| input.next(Phase::Up, event.position));
                        if let (Some(event), Some(callback)) = (event, &callback) {
                            callback(&event, window, cx);
                            cx.stop_propagation();
                        }
                    });
                },
            )
            .size_full(),
        );
        element.style().refine(&self.style);
        element.max_w(width).max_h(height).overflow_hidden()
    }
}
