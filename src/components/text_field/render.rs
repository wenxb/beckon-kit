use gpui::{
    Bounds, ContentMask, Context, DispatchPhase, ElementInputHandler, IntoElement, MouseButton,
    MouseMoveEvent, MouseUpEvent, Render, Role, TextAlign, TextRun, UnderlineStyle, Window, div,
    fill, point, prelude::*, px, size,
};

use super::{
    TextField, TextFieldVariant,
    display::DisplayText,
    layout::{FieldCanvas, FieldLayout},
};
use crate::{element::Semantic, theme::Theme};
use gpui::Refineable;

impl Render for TextField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::current(cx);
        let enabled = !self.disabled;
        let mut text_style = window.text_style().clone();
        let multiline = self.buffer.multiline();
        text_style.refine(&theme.text_style());
        text_style.refine(&self.style.text);
        let ink = if enabled {
            text_style.color
        } else {
            theme.colors.disabled
        };
        let focus = self.focus.clone();
        let entity = cx.entity();
        let paint_entity = entity.clone();
        let label = self.label.clone();
        let display = DisplayText::new(self.buffer.text(), self.masked);
        let text = display.text.clone();
        let empty = text.is_empty();
        let placeholder = self.placeholder.clone();
        let marked = self.buffer.marked().map(|range| display.range(range));
        let selection = self.buffer.selection();
        let head = self.buffer.head();
        let font_size = text_style.font_size.to_pixels(window.rem_size());
        let line_height = text_style
            .line_height
            .to_pixels(font_size.into(), window.rem_size());
        let font = text_style.font();
        let mut runs = Vec::new();
        if let Some(marked) = marked {
            for (range, underline) in [
                (0..marked.start, false),
                (marked.clone(), true),
                (marked.end..text.len(), false),
            ] {
                if range.is_empty() {
                    continue;
                }
                runs.push(TextRun {
                    len: range.len(),
                    font: font.clone(),
                    color: ink,
                    underline: underline.then_some(UnderlineStyle {
                        thickness: px(1.0),
                        color: Some(theme.colors.focus),
                        wavy: false,
                    }),
                    ..Default::default()
                });
            }
        } else {
            runs.push(TextRun {
                len: text.len(),
                font: font.clone(),
                color: ink,
                ..Default::default()
            });
        }
        let paint_focus = focus.clone();
        let selection_color = theme.colors.selection;
        let placeholder_color = theme.colors.secondary;
        let selector = self.id.to_string();
        let mut element = div()
            .id(self.id.clone())
            .role(if self.masked {
                Role::PasswordInput
            } else {
                Role::TextInput
            })
            .aria_label(label)
            .aria_value(text.clone())
            .aria_placeholder(placeholder.clone())
            .debug_selector(move || selector)
            .key_context(if multiline {
                "BeckonKitTextArea"
            } else {
                "BeckonKitTextField"
            })
            .flex()
            .items_center()
            .w_full()
            .min_w(px(64.0))
            .h(px(theme.metrics.control_height).max(line_height + px(10.0)))
            .px(px(10.0))
            .rounded(px(theme.metrics.control_radius))
            .bg(theme.colors.elevated)
            .border(px(1.0))
            .border_color(if self.invalid.is_some() {
                theme.colors.danger
            } else {
                theme.colors.border
            });
        if self.variant == TextFieldVariant::Plain {
            element = element
                .h(line_height)
                .px(px(0.0))
                .border(px(0.0))
                .bg(gpui::transparent_black());
        }
        if multiline {
            element = element.h_auto().flex_col().justify_center();
            if self.variant != TextFieldVariant::Plain {
                element = element.py(px(5.0));
            }
        }
        element.style().refine(&self.style);
        if let Some(message) = &self.invalid {
            element = element.aria_description(message.clone());
        }
        if enabled {
            let down_focus = focus.clone();
            let ring = theme.colors.focus;
            element = element
                .track_focus(&focus)
                .tab_index(0)
                .cursor_text()
                .focus_visible(move |style| style.border_color(ring))
                .on_key_down(cx.listener(Self::key_down))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |field, event: &gpui::MouseDownEvent, window, cx| {
                        down_focus.focus(window, cx);
                        field.dragging = true;
                        field.select_at(
                            event.position,
                            event.modifiers.shift,
                            event.click_count,
                            cx,
                        );
                        cx.stop_propagation();
                    }),
                );
            if multiline {
                element = element.on_scroll_wheel(cx.listener(
                    |field, event: &gpui::ScrollWheelEvent, _, cx| {
                        if let Some(geometry) = &field.geometry {
                            let delta = event.delta.pixel_delta(geometry.line_height);
                            let maximum =
                                (geometry.height() - geometry.bounds.size.height).max(px(0.0));
                            let next = (field.scroll_y - delta.y).clamp(px(0.0), maximum);
                            if next != field.scroll_y {
                                field.scroll_y = next;
                                field.reveal_caret = false;
                                cx.notify();
                                cx.stop_propagation();
                            }
                        }
                    },
                ));
            }
        }
        element = element.child(
            FieldCanvas::new(
                FieldLayout::new(
                    display,
                    placeholder.clone(),
                    runs,
                    TextRun {
                        len: placeholder.len(),
                        font,
                        color: placeholder_color,
                        ..Default::default()
                    },
                    font_size,
                    line_height,
                    multiline,
                ),
                move |bounds, mut geometry, placeholder, _, cx| {
                    let width = (bounds.size.width - px(2.0)).max(px(1.0));
                    let geometry = entity.update(cx, |field, _| {
                        let cursor = geometry.position(head) - bounds.origin;
                        if field.reveal_caret {
                            if cursor.x < field.scroll_x {
                                field.scroll_x = cursor.x;
                            }
                            if cursor.x > field.scroll_x + width {
                                field.scroll_x = cursor.x - width;
                            }
                            if cursor.y < field.scroll_y {
                                field.scroll_y = cursor.y;
                            }
                            if cursor.y + line_height > field.scroll_y + bounds.size.height {
                                field.scroll_y = cursor.y + line_height - bounds.size.height;
                            }
                        }
                        field.scroll_x = field
                            .scroll_x
                            .clamp(px(0.0), (geometry.width() - width).max(px(0.0)));
                        field.scroll_y = field.scroll_y.clamp(
                            px(0.0),
                            (geometry.height() - bounds.size.height).max(px(0.0)),
                        );
                        geometry.origin -= point(field.scroll_x, field.scroll_y);
                        field.reveal_caret = false;
                        field.geometry = Some(geometry.clone());
                        geometry
                    });
                    (geometry, placeholder)
                },
                move |bounds, (geometry, placeholder), window, cx| {
                    if enabled {
                        window.handle_input(
                            &paint_focus,
                            ElementInputHandler::new(bounds, paint_entity.clone()),
                            cx,
                        );
                    }
                    window.with_content_mask(Some(ContentMask { bounds }), |window| {
                        let focused = enabled && paint_focus.is_focused(window);
                        if focused && !selection.is_empty() {
                            geometry.selection_bounds(selection.clone(), |bounds| {
                                window.paint_quad(fill(bounds, selection_color));
                            });
                        }
                        if empty {
                            if let Some(placeholder) = placeholder {
                                for line in placeholder.lines.iter() {
                                    line.line
                                        .paint(
                                            bounds.origin + point(px(0.0), line.y),
                                            line_height,
                                            TextAlign::Left,
                                            None,
                                            window,
                                            cx,
                                        )
                                        .expect("paint placeholder");
                                }
                            }
                        } else {
                            for line in geometry.lines.iter() {
                                line.line
                                    .paint(
                                        geometry.origin + point(px(0.0), line.y),
                                        line_height,
                                        TextAlign::Left,
                                        None,
                                        window,
                                        cx,
                                    )
                                    .expect("paint field text");
                            }
                        }
                        if focused && selection.is_empty() {
                            let cursor = geometry.position(head) + point(px(0.0), px(2.0));
                            window.paint_quad(fill(
                                Bounds::new(
                                    cursor,
                                    size(px(1.0), (line_height - px(4.0)).max(px(1.0))),
                                ),
                                ink,
                            ));
                        }
                    });
                    if !enabled {
                        return;
                    }
                    let move_entity = paint_entity.clone();
                    window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
                        if phase != DispatchPhase::Capture {
                            return;
                        }
                        move_entity.update(cx, |field, cx| {
                            if field.dragging {
                                if event.pressed_button == Some(MouseButton::Left) {
                                    field.select_at(event.position, true, 1, cx);
                                } else {
                                    field.dragging = false;
                                }
                            }
                        });
                    });
                    window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
                        if phase == DispatchPhase::Capture && event.button == MouseButton::Left {
                            paint_entity.update(cx, |field, _| field.dragging = false);
                        }
                    });
                },
            )
            .w_full()
            .min_h_0(),
        );
        Semantic::new(element)
            .disabled(self.disabled)
            .read_only(self.read_only)
            .invalid(self.invalid.is_some())
            .multiline(multiline)
    }
}
