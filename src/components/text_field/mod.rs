use gpui::{
    App, Context, EventEmitter, FocusHandle, Focusable, Pixels, SharedString, StyleRefinement,
    Window, px,
};

use crate::text::TextBuffer;

mod display;
mod geometry;
mod input;
mod keyboard;
mod layout;
mod render;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextFieldEvent {
    Changed {
        value: SharedString,
        composing: bool,
    },
    Submitted(SharedString),
}

/// Outline is the default control; Plain keeps editing without a visual container.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TextFieldVariant {
    #[default]
    Outline,
    Plain,
}

/// An editor with native IME, grapheme editing, wrapping and bounded undo.
pub struct TextField {
    id: SharedString,
    label: SharedString,
    placeholder: SharedString,
    focus: FocusHandle,
    buffer: TextBuffer,
    disabled: bool,
    read_only: bool,
    invalid: Option<SharedString>,
    geometry: Option<geometry::FieldGeometry>,
    scroll_x: Pixels,
    scroll_y: Pixels,
    preferred_x: Option<Pixels>,
    reveal_caret: bool,
    dragging: bool,
    masked: bool,
    variant: TextFieldVariant,
    style: StyleRefinement,
}

impl EventEmitter<TextFieldEvent> for TextField {}

impl Focusable for TextField {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl TextField {
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            placeholder: SharedString::default(),
            focus: cx.focus_handle().tab_stop(true),
            buffer: TextBuffer::default(),
            disabled: false,
            read_only: false,
            invalid: None,
            geometry: None,
            scroll_x: px(0.0),
            scroll_y: px(0.0),
            preferred_x: None,
            reveal_caret: true,
            dragging: false,
            masked: false,
            variant: TextFieldVariant::default(),
            style: StyleRefinement::default(),
        }
    }

    /// Visual overrides use the same GPUI style vocabulary as stateless controls.
    pub fn set_style(&mut self, style: StyleRefinement, cx: &mut Context<Self>) {
        use gpui::Refineable as _;
        if !self.style.is_superset_of(&style) || !style.is_superset_of(&self.style) {
            self.style = style;
            cx.notify();
        }
    }

    pub fn set_variant(&mut self, variant: TextFieldVariant, cx: &mut Context<Self>) {
        if self.variant != variant {
            self.variant = variant;
            cx.notify();
        }
    }

    /// Multiline wraps and sizes to content; explicit height/max-height bounds its viewport.
    /// Enter inserts a newline; Ctrl/Cmd+Enter submits.
    /// Switching back to single-line normalizes the value and clears edit history.
    pub fn set_multiline(&mut self, multiline: bool, cx: &mut Context<Self>) {
        if self.buffer.multiline() != multiline {
            self.buffer.set_multiline(multiline);
            if !multiline {
                self.placeholder = crate::text::single_line(&self.placeholder).into();
            }
            self.geometry = None;
            self.scroll_x = px(0.0);
            self.scroll_y = px(0.0);
            self.preferred_x = None;
            self.reveal_caret = true;
            cx.notify();
        }
    }

    pub fn is_multiline(&self) -> bool {
        self.buffer.multiline()
    }

    /// Password mode masks painting and accessibility, and disables clipboard export.
    pub fn set_masked(&mut self, masked: bool, cx: &mut Context<Self>) {
        if self.masked != masked {
            self.masked = masked;
            cx.notify();
        }
    }

    pub fn set_label(&mut self, label: impl Into<SharedString>, cx: &mut Context<Self>) {
        let label = label.into();
        if self.label != label {
            self.label = label;
            cx.notify();
        }
    }

    pub fn value(&self) -> &str {
        self.buffer.text()
    }
    pub fn is_composing(&self) -> bool {
        self.buffer.composing()
    }
    pub fn selected_range_utf16(&self) -> std::ops::Range<usize> {
        self.buffer.selection_utf16()
    }

    /// Programmatic replacement clears history and does not emit an input event.
    pub fn set_value(&mut self, value: impl AsRef<str>, cx: &mut Context<Self>) {
        self.buffer.set_text(value.as_ref());
        self.scroll_x = px(0.0);
        self.scroll_y = px(0.0);
        self.preferred_x = None;
        self.reveal_caret = true;
        cx.notify();
    }

    pub fn set_placeholder(
        &mut self,
        placeholder: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) {
        let placeholder = placeholder.into();
        let placeholder = if self.buffer.multiline() {
            placeholder
        } else {
            crate::text::single_line(&placeholder).into()
        };
        if self.placeholder != placeholder {
            self.placeholder = placeholder;
            cx.notify();
        }
    }

    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        self.disabled = disabled;
        if disabled {
            self.dragging = false;
            self.buffer.unmark();
        }
        cx.notify();
    }

    pub fn set_read_only(&mut self, read_only: bool, cx: &mut Context<Self>) {
        self.read_only = read_only;
        if read_only {
            self.buffer.unmark();
        }
        cx.notify();
    }

    pub fn set_invalid(&mut self, message: Option<SharedString>, cx: &mut Context<Self>) {
        self.invalid = message;
        cx.notify();
    }

    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        if !self.disabled {
            self.focus.focus(window, cx);
        }
    }

    fn changed(&mut self, cx: &mut Context<Self>) {
        self.preferred_x = None;
        self.reveal_caret = true;
        cx.emit(TextFieldEvent::Changed {
            value: self.buffer.text().to_owned().into(),
            composing: self.buffer.composing(),
        });
        cx.notify();
    }

    fn select_at(
        &mut self,
        position: gpui::Point<Pixels>,
        extend: bool,
        count: usize,
        cx: &mut Context<Self>,
    ) {
        let Some(geometry) = &self.geometry else {
            return;
        };
        let index = self.buffer.snap_grapheme(geometry.index_at(position));
        if count >= 3 {
            self.buffer.select_all();
        } else if count == 2 {
            self.buffer.select_word(index);
        } else {
            self.buffer.move_to(index, extend);
        }
        self.preferred_x = None;
        self.reveal_caret = true;
        cx.notify();
    }
}
