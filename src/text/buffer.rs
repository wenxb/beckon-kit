use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

use super::offset::{byte_floor, from_utf16, single_line, to_utf16};

const HISTORY_LIMIT: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Snapshot {
    text: String,
    anchor: usize,
    head: usize,
}

/// UTF-8 storage, grapheme navigation, UTF-16 protocol, bounded edit history.
#[derive(Debug, Default)]
pub(crate) struct TextBuffer {
    text: String,
    anchor: usize,
    head: usize,
    marked: Option<Range<usize>>,
    composition_start: Option<Snapshot>,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    multiline: bool,
}

impl TextBuffer {
    pub(crate) fn multiline(&self) -> bool {
        self.multiline
    }

    pub(crate) fn set_multiline(&mut self, multiline: bool) {
        if self.multiline == multiline {
            return;
        }
        self.unmark();
        self.multiline = multiline;
        if !multiline {
            let text = single_line(&self.text);
            self.set_text(&text);
        }
    }

    fn normalize(&self, text: &str) -> String {
        if !self.multiline {
            return single_line(text);
        }
        let mut result = String::with_capacity(text.len());
        let mut chars = text.chars().peekable();
        while let Some(character) = chars.next() {
            result.push(match character {
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    '\n'
                }
                '\n' => '\n',
                character if character.is_control() => ' ',
                character => character,
            });
        }
        result
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }
    pub(crate) fn head(&self) -> usize {
        self.head
    }
    pub(crate) fn anchor(&self) -> usize {
        self.anchor
    }
    pub(crate) fn marked(&self) -> Option<Range<usize>> {
        self.marked.clone()
    }
    pub(crate) fn composing(&self) -> bool {
        self.composition_start.is_some()
    }

    pub(crate) fn selection(&self) -> Range<usize> {
        self.anchor.min(self.head)..self.anchor.max(self.head)
    }

    pub(crate) fn selection_utf16(&self) -> Range<usize> {
        to_utf16(&self.text, self.selection())
    }

    pub(crate) fn marked_utf16(&self) -> Option<Range<usize>> {
        self.marked.clone().map(|range| to_utf16(&self.text, range))
    }

    pub(crate) fn text_for_utf16(&self, range: Range<usize>) -> (String, Range<usize>) {
        let bytes = from_utf16(&self.text, range);
        (self.text[bytes.clone()].into(), to_utf16(&self.text, bytes))
    }

    pub(crate) fn bytes_for_utf16(&self, range: Range<usize>) -> Range<usize> {
        from_utf16(&self.text, range)
    }

    pub(crate) fn set_text(&mut self, text: &str) -> bool {
        let text = self.normalize(text);
        let changed = self.text != text;
        self.text = text;
        self.anchor = self.text.len();
        self.head = self.anchor;
        self.marked = None;
        self.composition_start = None;
        self.undo.clear();
        self.redo.clear();
        changed
    }

    pub(crate) fn select(&mut self, anchor: usize, head: usize) {
        self.anchor = byte_floor(&self.text, anchor);
        self.head = byte_floor(&self.text, head);
    }

    pub(crate) fn select_utf16(&mut self, range: Range<usize>) {
        let range = from_utf16(&self.text, range);
        self.select(range.start, range.end);
    }

    pub(crate) fn select_all(&mut self) {
        self.select(0, self.text.len());
    }

    pub(crate) fn snap_grapheme(&self, index: usize) -> usize {
        self.text
            .grapheme_indices(true)
            .map(|(start, _)| start)
            .chain([self.text.len()])
            .take_while(|boundary| *boundary <= index)
            .last()
            .unwrap_or(0)
    }

    pub(crate) fn move_to(&mut self, position: usize, extend: bool) {
        let position = byte_floor(&self.text, position);
        if !extend {
            self.anchor = position;
        }
        self.head = position;
    }

    pub(crate) fn move_horizontal(&mut self, forward: bool, extend: bool, word: bool) {
        let selection = self.selection();
        let position = if !extend && !selection.is_empty() {
            if forward {
                selection.end
            } else {
                selection.start
            }
        } else if word {
            if forward {
                self.next_word()
            } else {
                self.previous_word()
            }
        } else if forward {
            self.next_grapheme()
        } else {
            self.previous_grapheme()
        };
        self.move_to(position, extend);
    }

    fn previous_grapheme(&self) -> usize {
        self.text
            .grapheme_indices(true)
            .map(|(start, _)| start)
            .take_while(|start| *start < self.head)
            .last()
            .unwrap_or(0)
    }

    fn next_grapheme(&self) -> usize {
        self.text
            .grapheme_indices(true)
            .map(|(start, _)| start)
            .find(|start| *start > self.head)
            .unwrap_or(self.text.len())
    }

    fn previous_word(&self) -> usize {
        self.text
            .unicode_word_indices()
            .map(|(start, _)| start)
            .take_while(|start| *start < self.head)
            .last()
            .unwrap_or(0)
    }

    fn next_word(&self) -> usize {
        self.text
            .unicode_word_indices()
            .map(|(start, word)| start + word.len())
            .find(|end| *end > self.head)
            .unwrap_or(self.text.len())
    }

    pub(crate) fn select_word(&mut self, index: usize) {
        if let Some((start, word)) = self
            .text
            .unicode_word_indices()
            .find(|(start, word)| *start <= index && index < start + word.len())
        {
            self.select(start, start + word.len());
        } else {
            let start = self.snap_grapheme(index);
            let end = self.text[start..]
                .graphemes(true)
                .next()
                .map_or(start, |grapheme| start + grapheme.len());
            self.select(start, end);
        }
    }

    pub(crate) fn replace(&mut self, range_utf16: Option<Range<usize>>, text: &str) -> bool {
        let range = self.replacement_range(range_utf16);
        let text = self.normalize(text);
        let before = self
            .composition_start
            .take()
            .unwrap_or_else(|| self.snapshot());
        let changed = self.text[range.clone()] != text;
        self.text.replace_range(range.clone(), &text);
        self.anchor = range.start + text.len();
        self.head = self.anchor;
        self.marked = None;
        if before.text != self.text {
            self.record(before);
        }
        changed
    }

    pub(crate) fn compose(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        selection_utf16: Option<Range<usize>>,
    ) {
        if self.composition_start.is_none() {
            self.composition_start = Some(self.snapshot());
        }
        let range = self.replacement_range(range_utf16);
        let text = self.normalize(text);
        self.text.replace_range(range.clone(), &text);
        let marked = range.start..range.start + text.len();
        let selection = selection_utf16
            .map(|selection| from_utf16(&text, selection))
            .unwrap_or(text.len()..text.len());
        self.anchor = marked.start + selection.start;
        self.head = marked.start + selection.end;
        self.marked = Some(marked);
    }

    pub(crate) fn unmark(&mut self) {
        self.marked = None;
        if let Some(before) = self.composition_start.take()
            && before.text != self.text
        {
            self.record(before);
        }
    }

    fn replacement_range(&self, range_utf16: Option<Range<usize>>) -> Range<usize> {
        range_utf16
            .map(|range| from_utf16(&self.text, range))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selection())
    }

    pub(crate) fn delete(&mut self, forward: bool, word: bool) -> bool {
        if self.selection().is_empty() {
            self.move_horizontal(forward, true, word);
        }
        self.replace(None, "")
    }

    pub(crate) fn undo(&mut self) -> bool {
        self.unmark();
        let Some(snapshot) = self.undo.pop() else {
            return false;
        };
        self.redo.push(self.snapshot());
        self.restore(snapshot);
        true
    }

    pub(crate) fn redo(&mut self) -> bool {
        let Some(snapshot) = self.redo.pop() else {
            return false;
        };
        self.undo.push(self.snapshot());
        self.restore(snapshot);
        true
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            text: self.text.clone(),
            anchor: self.anchor,
            head: self.head,
        }
    }

    fn restore(&mut self, snapshot: Snapshot) {
        self.text = snapshot.text;
        self.anchor = snapshot.anchor;
        self.head = snapshot.head;
        self.marked = None;
        self.composition_start = None;
    }

    fn record(&mut self, snapshot: Snapshot) {
        if self.undo.len() == HISTORY_LIMIT {
            self.undo.remove(0);
        }
        self.undo.push(snapshot);
        self.redo.clear();
    }
}

#[cfg(test)]
mod tests;
