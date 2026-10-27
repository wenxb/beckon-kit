use gpui::SharedString;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

/// Byte boundaries map shaped password bullets back to the original document.
#[derive(Clone)]
pub(super) struct DisplayText {
    pub text: SharedString,
    boundaries: Vec<(usize, usize)>,
}

impl DisplayText {
    pub fn new(source: &str, masked: bool) -> Self {
        if !masked {
            return Self {
                text: source.to_owned().into(),
                boundaries: Vec::new(),
            };
        }
        let mut text = String::new();
        let mut boundaries = Vec::new();
        for (index, grapheme) in source.grapheme_indices(true) {
            boundaries.push((index, text.len()));
            text.push(if grapheme == "\n" { '\n' } else { '•' });
        }
        boundaries.push((source.len(), text.len()));
        Self {
            text: text.into(),
            boundaries,
        }
    }

    pub fn display_index(&self, source: usize) -> usize {
        self.boundaries
            .iter()
            .rev()
            .find(|(index, _)| *index <= source)
            .map_or(source, |(_, index)| *index)
    }

    pub fn source_index(&self, display: usize) -> usize {
        self.boundaries
            .iter()
            .rev()
            .find(|(_, index)| *index <= display)
            .map_or(display, |(index, _)| *index)
    }

    pub fn range(&self, range: Range<usize>) -> Range<usize> {
        self.display_index(range.start)..self.display_index(range.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_geometry_maps_graphemes_not_utf8_bytes() {
        let display = DisplayText::new("文🦀e\u{301}", true);
        assert_eq!(&*display.text, "•••");
        assert_eq!(display.range(3..7), 3..6);
        assert_eq!(display.source_index(6), 7);
        assert_eq!(display.display_index(10), 9);
    }

    #[test]
    fn masked_multiline_geometry_retains_line_breaks() {
        let display = DisplayText::new("文\n🦀", true);
        assert_eq!(&*display.text, "•\n•");
        assert_eq!(display.display_index(4), 4);
        assert_eq!(display.source_index(7), 8);
    }
}
