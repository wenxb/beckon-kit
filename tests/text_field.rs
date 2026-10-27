mod support;

use beckon_kit::{
    Root,
    components::{TextField, TextFieldEvent, TextFieldVariant},
};
use gpui::{Context, Entity, EntityInputHandler, IntoElement, Render, Window, div, prelude::*, px};

struct FieldHost {
    input: Entity<TextField>,
    events: Vec<TextFieldEvent>,
}

impl Render for FieldHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Root::new("root").child(div().p(px(20.0)).w(px(260.0)).child(self.input.clone()))
    }
}

fn fixture(
    cx: &mut gpui::TestAppContext,
) -> (
    gpui::AnyWindowHandle,
    gpui::Entity<FieldHost>,
    Entity<TextField>,
) {
    let window = support::open(cx, |_, cx| {
        let input = cx.new(|cx| TextField::new("input", "Search", cx));
        cx.subscribe(
            &input,
            |host: &mut FieldHost, _, event: &TextFieldEvent, _| host.events.push(event.clone()),
        )
        .detach();
        FieldHost {
            input,
            events: Vec::new(),
        }
    });
    let host = window.root(cx).unwrap();
    let input = host.read_with(cx, |host, _| host.input.clone());
    support::click(cx, window.into(), "input");
    (window.into(), host, input)
}

#[gpui::test]
fn masking_hides_accessible_value_and_blocks_clipboard_export(cx: &mut gpui::TestAppContext) {
    let (window, _, input) = fixture(cx);
    input.update(cx, |field, cx| {
        field.set_masked(true, cx);
        field.set_value("文🦀e\u{301}", cx);
    });
    support::frame(cx, window);
    support::with_window(cx, window, |window, _| {
        let field = gpui::test::observation::find(window, &[], &"input".into()).unwrap();
        assert_eq!(field.role(), gpui::Role::PasswordInput);
        assert_eq!(field.value(), Some("•••"));
    });
    let modifier = if cfg!(target_os = "macos") {
        "cmd"
    } else {
        "ctrl"
    };
    support::press(cx, window, &format!("{modifier}-a"));
    support::press(cx, window, &format!("{modifier}-c"));
    assert!(cx.read_from_clipboard().is_none());
    support::press(cx, window, &format!("{modifier}-x"));
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "文🦀e\u{301}"
    );
    cx.simulate_input(window, "new");
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "new"
    );
}

#[gpui::test]
fn plain_and_style_overrides_change_layout_without_changing_editing(cx: &mut gpui::TestAppContext) {
    let (window, _, input) = fixture(cx);
    let original = support::bounds(cx, window, "input");
    input.update(cx, |field, cx| {
        field.set_variant(TextFieldVariant::Plain, cx);
        field.set_style(
            gpui::StyleRefinement::default()
                .text_size(px(24.0))
                .line_height(gpui::relative(1.25)),
            cx,
        );
    });
    support::frame(cx, window);
    let plain = support::bounds(cx, window, "input");
    assert_eq!(plain.size.height, px(30.0));
    cx.simulate_input(window, "hello");
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "hello"
    );
    input.update(cx, |field, cx| {
        field.set_variant(TextFieldVariant::Outline, cx);
        field.set_style(gpui::StyleRefinement::default(), cx);
    });
    support::frame(cx, window);
    assert_eq!(
        support::bounds(cx, window, "input").size.height,
        original.size.height
    );
}

#[gpui::test]
fn native_input_unicode_deletion_clipboard_and_undo_work_through_the_window(
    cx: &mut gpui::TestAppContext,
) {
    let (window, _, input) = fixture(cx);
    cx.simulate_input(window, "你好🦀e\u{301}");
    support::frame(cx, window);
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "你好🦀e\u{301}"
    );
    support::press(cx, window, "backspace");
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "你好🦀"
    );
    let modifier = if cfg!(target_os = "macos") {
        "cmd"
    } else {
        "ctrl"
    };
    support::press(cx, window, &format!("{modifier}-a"));
    support::press(cx, window, &format!("{modifier}-c"));
    assert_eq!(cx.read_from_clipboard().unwrap().text().unwrap(), "你好🦀");
    support::press(cx, window, &format!("{modifier}-x"));
    assert_eq!(input.read_with(cx, |field, _| field.value().to_owned()), "");
    support::press(cx, window, &format!("{modifier}-v"));
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "你好🦀"
    );
    support::press(cx, window, &format!("{modifier}-z"));
    assert_eq!(input.read_with(cx, |field, _| field.value().to_owned()), "");
    support::press(cx, window, &format!("{modifier}-shift-z"));
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "你好🦀"
    );
}

#[gpui::test]
fn ime_preedit_commits_one_edit_and_enter_does_not_submit_during_composition(
    cx: &mut gpui::TestAppContext,
) {
    let (window, host, input) = fixture(cx);
    support::with_window(cx, window, |window, cx| {
        input.update(cx, |field, cx| {
            field.replace_and_mark_text_in_range(None, "ni", Some(2..2), window, cx);
            assert_eq!(field.marked_text_range(window, cx), Some(0..2));
        })
    });
    support::frame(cx, window);
    support::press(cx, window, "enter");
    assert!(
        host.read_with(cx, |host, _| host.events.clone())
            .iter()
            .all(|event| !matches!(event, TextFieldEvent::Submitted(_)))
    );
    support::with_window(cx, window, |window, cx| {
        input.update(cx, |field, cx| {
            field.replace_and_mark_text_in_range(None, "你好", Some(2..2), window, cx);
            field.replace_text_in_range(None, "你好", window, cx);
        })
    });
    support::frame(cx, window);
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "你好"
    );
    assert!(!input.read_with(cx, |field, _| field.is_composing()));
    assert!(
        host.read_with(cx, |host, _| host.events.clone())
            .iter()
            .any(|event| matches!(
                event,
                TextFieldEvent::Changed {
                    composing: true,
                    ..
                }
            ))
    );
    support::press(cx, window, "enter");
    assert!(
        host.read_with(cx, |host, _| host.events.clone())
            .contains(&TextFieldEvent::Submitted("你好".into()))
    );
    let undo = if cfg!(target_os = "macos") {
        "cmd-z"
    } else {
        "ctrl-z"
    };
    support::press(cx, window, undo);
    assert_eq!(input.read_with(cx, |field, _| field.value().to_owned()), "");
}

#[gpui::test]
fn utf16_selection_and_candidate_bounds_are_valid_after_horizontal_scroll(
    cx: &mut gpui::TestAppContext,
) {
    let (window, _, input) = fixture(cx);
    input.update(cx, |field, cx| {
        field.set_value(
            "文🦀abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz",
            cx,
        )
    });
    support::frame(cx, window);
    support::with_window(cx, window, |window, cx| {
        input.update(cx, |field, cx| {
            let length = field.value().encode_utf16().count();
            let bounds = field
                .bounds_for_range(length..length, Default::default(), window, cx)
                .unwrap();
            assert!(bounds.size.width >= px(1.0));
            assert!(bounds.left() >= px(20.0) && bounds.right() <= px(260.0));
            let mut adjusted = None;
            assert_eq!(
                field
                    .text_for_range(2..3, &mut adjusted, window, cx)
                    .unwrap(),
                "🦀"
            );
            assert_eq!(adjusted, Some(1..3));
            field.set_selected_text_range(1..3, window, cx);
            assert_eq!(field.selected_range_utf16(), 1..3);
        })
    });
    support::frame(cx, window);
    support::press(cx, window, "backspace");
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "文abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz"
    );
}

#[gpui::test]
fn ime_anchor_stays_at_viewport_top_when_a_line_does_not_fit(cx: &mut gpui::TestAppContext) {
    let (window, _, input) = fixture(cx);
    for multiline in [false, true] {
        input.update(cx, |field, cx| {
            field.set_multiline(multiline, cx);
            field.set_value("文🦀", cx);
            field.set_style(gpui::StyleRefinement::default().flex_col().h(px(12.0)), cx);
        });
        support::frame(cx, window);
        let viewport = support::bounds(cx, window, "input");
        support::with_window(cx, window, |window, cx| {
            input.update(cx, |field, cx| {
                let length = field.value().encode_utf16().count();
                let caret = field
                    .bounds_for_range(length..length, Default::default(), window, cx)
                    .unwrap();
                assert!(caret.size.height > viewport.size.height);
                assert!(caret.top() >= viewport.top() && caret.top() < viewport.bottom());
                assert_eq!(
                    field.character_index_for_point(caret.origin, window, cx),
                    Some(length)
                );
            });
        });
    }
}

#[gpui::test]
fn read_only_and_disabled_fields_reject_editing_but_read_only_allows_copy(
    cx: &mut gpui::TestAppContext,
) {
    let (window, host, input) = fixture(cx);
    input.update(cx, |field, cx| {
        field.set_value("protected", cx);
        field.set_read_only(true, cx);
    });
    support::frame(cx, window);
    cx.simulate_input(window, "x");
    support::press(cx, window, "backspace");
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "protected"
    );
    let modifier = if cfg!(target_os = "macos") {
        "cmd"
    } else {
        "ctrl"
    };
    support::press(cx, window, &format!("{modifier}-a"));
    support::press(cx, window, &format!("{modifier}-c"));
    assert_eq!(
        cx.read_from_clipboard().unwrap().text().unwrap(),
        "protected"
    );
    input.update(cx, |field, cx| {
        field.set_read_only(false, cx);
        field.set_disabled(true, cx);
    });
    support::frame(cx, window);
    support::with_window(cx, window, |window, cx| {
        input.update(cx, |field, cx| {
            field.replace_text_in_range(None, "bad", window, cx);
            field.replace_and_mark_text_in_range(None, "bad", None, window, cx);
        })
    });
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "protected"
    );
    assert!(
        host.read_with(cx, |host, _| host.events.clone()).is_empty(),
        "programmatic values do not emit user changes"
    );
}

#[gpui::test]
fn user_selection_and_placeholder_do_not_replace_the_value(cx: &mut gpui::TestAppContext) {
    let (window, _, input) = fixture(cx);
    input.update(cx, |field, cx| {
        field.set_value("abc", cx);
        field.set_placeholder("Hint", cx);
    });
    support::frame(cx, window);
    support::press(cx, window, "shift-left");
    assert_eq!(
        input.read_with(cx, |field, _| field.selected_range_utf16()),
        2..3
    );
    cx.simulate_input(window, "文");
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "ab文"
    );
}

#[gpui::test]
fn word_deletion_and_multiline_placeholders_do_not_break_the_editor(cx: &mut gpui::TestAppContext) {
    let (window, _, input) = fixture(cx);
    input.update(cx, |field, cx| {
        field.set_value("one two", cx);
        field.set_placeholder("Line\nNext", cx);
    });
    support::frame(cx, window);
    let word_delete = if cfg!(target_os = "macos") {
        "alt-backspace"
    } else {
        "ctrl-backspace"
    };
    support::press(cx, window, word_delete);
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "one "
    );
    input.update(cx, |field, cx| field.set_value("", cx));
    support::frame(cx, window);
    assert_eq!(input.read_with(cx, |field, _| field.value().to_owned()), "");
}
