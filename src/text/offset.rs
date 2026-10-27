use std::ops::Range;

pub(crate) fn byte_floor(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

pub(crate) fn utf16_to_byte(text: &str, offset: usize, round_up: bool) -> usize {
    let mut units = 0;
    for (index, character) in text.char_indices() {
        if units >= offset {
            return index;
        }
        let next = units + character.len_utf16();
        if offset < next {
            return if round_up {
                index + character.len_utf8()
            } else {
                index
            };
        }
        units = next;
    }
    text.len()
}

pub(crate) fn byte_to_utf16(text: &str, offset: usize) -> usize {
    text[..byte_floor(text, offset)].encode_utf16().count()
}

pub(crate) fn from_utf16(text: &str, range: Range<usize>) -> Range<usize> {
    let start = range.start.min(range.end);
    let end = range.start.max(range.end);
    let start = utf16_to_byte(text, start, false);
    // A collapsed selection inside a surrogate pair stays collapsed.
    let end = if range.start == range.end {
        start
    } else {
        utf16_to_byte(text, end, true)
    };
    start..end
}

pub(crate) fn to_utf16(text: &str, range: Range<usize>) -> Range<usize> {
    byte_to_utf16(text, range.start)..byte_to_utf16(text, range.end)
}

pub(crate) fn single_line(text: &str) -> String {
    text.chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect()
}
