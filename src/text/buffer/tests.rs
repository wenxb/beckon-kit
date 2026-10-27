use super::TextBuffer;

#[test]
fn backspace_deletes_a_whole_grapheme() {
    let mut buffer = TextBuffer::default();
    buffer.set_text("中👨‍👩‍👧‍👦e\u{301}");
    assert!(buffer.delete(false, false));
    assert_eq!(buffer.text(), "中👨‍👩‍👧‍👦");
    assert!(buffer.delete(false, false));
    assert_eq!(buffer.text(), "中");
    assert!(buffer.undo());
    assert_eq!(buffer.text(), "中👨‍👩‍👧‍👦");
    assert!(buffer.redo());
    assert_eq!(buffer.text(), "中");
}

#[test]
fn utf16_ranges_never_split_a_surrogate_pair() {
    let mut buffer = TextBuffer::default();
    buffer.set_text("a🦀文");
    assert_eq!(buffer.text_for_utf16(2..3), ("🦀".into(), 1..3));
    buffer.select_utf16(2..2);
    assert_eq!(buffer.selection_utf16(), 1..1);
    buffer.replace(Some(1..3), "好");
    assert_eq!(buffer.text(), "a好文");
}

#[test]
fn composition_replaces_preedit_and_undoes_as_one_transaction() {
    let mut buffer = TextBuffer::default();
    buffer.set_text("hello ");
    buffer.compose(None, "ni", Some(2..2));
    assert_eq!(buffer.marked_utf16(), Some(6..8));
    buffer.compose(None, "你好", Some(2..2));
    assert_eq!(buffer.text(), "hello 你好");
    buffer.replace(None, "你好");
    assert!(!buffer.composing());
    assert!(buffer.undo());
    assert_eq!(buffer.text(), "hello ");
    assert!(buffer.redo());
    assert_eq!(buffer.text(), "hello 你好");
}

#[test]
fn selection_direction_and_word_navigation_are_preserved() {
    let mut buffer = TextBuffer::default();
    buffer.set_text("one two");
    buffer.move_horizontal(false, true, true);
    assert_eq!(buffer.selection(), 4..7);
    assert!(buffer.head() < buffer.anchor());
    buffer.move_horizontal(false, false, false);
    assert_eq!(buffer.selection(), 4..4);
    buffer.select_word(1);
    assert_eq!(buffer.selection(), 0..3);
}

#[test]
fn external_values_and_paste_remain_single_line() {
    let mut buffer = TextBuffer::default();
    buffer.set_text("line\nnext\tend");
    assert_eq!(buffer.text(), "line next end");
    buffer.select_all();
    buffer.replace(None, "new\r\nvalue");
    assert_eq!(buffer.text(), "new  value");
    buffer.set_text("reset");
    assert!(!buffer.undo());
}

#[test]
fn multiline_normalizes_newlines_and_preserves_history_and_ime() {
    let mut buffer = TextBuffer::default();
    buffer.set_multiline(true);
    buffer.set_text("one\r\ntwo\rthree\n");
    assert_eq!(buffer.text(), "one\ntwo\nthree\n");
    buffer.replace(None, "文\n🦀");
    assert_eq!(buffer.selection_utf16(), 18..18);
    assert!(buffer.undo());
    assert_eq!(buffer.text(), "one\ntwo\nthree\n");
    assert!(buffer.redo());
    buffer.compose(None, "ni\nhao", Some(2..2));
    assert!(buffer.composing());
    buffer.replace(None, "你\n好");
    assert!(buffer.text().ends_with("你\n好"));
    assert!(buffer.undo());
    assert_eq!(buffer.text(), "one\ntwo\nthree\n文\n🦀");
    buffer.set_multiline(false);
    assert_eq!(buffer.text(), "one two three 文 🦀");
    assert!(!buffer.undo());
    buffer.replace(None, "\nend");
    assert!(buffer.text().ends_with(" end"));
}
