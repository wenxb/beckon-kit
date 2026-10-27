use gpui::{Bounds, Context, EntityInputHandler, Pixels, Point, UTF16Selection, Window};
use std::ops::Range;

use super::TextField;

impl EntityInputHandler for TextField {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        adjusted: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let (text, range) = self.buffer.text_for_utf16(range);
        *adjusted = Some(range);
        Some(text)
    }

    fn selected_text_range(
        &mut self,
        ignore_disabled: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        if self.disabled && !ignore_disabled {
            return None;
        }
        Some(UTF16Selection {
            range: self.buffer.selection_utf16(),
            reversed: self.buffer.head() < self.buffer.anchor(),
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.buffer.marked_utf16()
    }

    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        if self.buffer.composing() {
            self.buffer.unmark();
            self.changed(cx);
        }
    }

    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled || self.read_only {
            return;
        }
        let was_composing = self.buffer.composing();
        if self.buffer.replace(range, text) || was_composing {
            self.changed(cx);
        } else {
            cx.notify();
        }
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selection: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled || self.read_only {
            return;
        }
        self.buffer.compose(range, text, selection);
        self.changed(cx);
    }

    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        _: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let geometry = self.geometry.as_ref()?;
        let range = self.buffer.bytes_for_utf16(range);
        let start = geometry.position(range.start);
        let end = geometry.position(range.end);
        let left = start
            .x
            .clamp(geometry.bounds.left(), geometry.bounds.right());
        let right = if start.y == end.y {
            end.x
        } else {
            geometry.bounds.right()
        }
        .clamp(left, geometry.bounds.right());
        Some(Bounds::new(
            gpui::point(
                left,
                start.y.clamp(
                    geometry.bounds.top(),
                    (geometry.bounds.bottom() - geometry.line_height).max(geometry.bounds.top()),
                ),
            ),
            gpui::size((right - left).max(gpui::px(1.0)), geometry.line_height),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let geometry = self.geometry.as_ref()?;
        let index = self.buffer.snap_grapheme(geometry.index_at(point));
        Some(self.buffer.text()[..index].encode_utf16().count())
    }

    fn set_selected_text_range(
        &mut self,
        range: Range<usize>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        self.buffer.select_utf16(range);
        self.preferred_x = None;
        self.reveal_caret = true;
        cx.notify();
    }

    fn text_length_utf16(&mut self, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {
        Some(self.buffer.text().encode_utf16().count())
    }

    fn accepts_text_input(&self, _: &mut Window, _: &mut Context<Self>) -> bool {
        !self.disabled && !self.read_only
    }
}
