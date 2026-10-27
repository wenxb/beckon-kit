mod support;

use beckon_kit::{
    Root,
    components::{TextField, TextFieldEvent},
};
use gpui::{
    Bounds, Context, Entity, EntityInputHandler, IntoElement, Pixels, Render, Window, div,
    prelude::*, px,
};

struct Host {
    input: Entity<TextField>,
    events: Vec<TextFieldEvent>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Root::new("root").child(div().p(px(20.0)).w(px(260.0)).child(self.input.clone()))
    }
}

fn fixture(
    cx: &mut gpui::TestAppContext,
) -> (gpui::AnyWindowHandle, Entity<Host>, Entity<TextField>) {
    let window = support::open(cx, |_, cx| {
        let input = cx.new(|cx| {
            let mut field = TextField::new("textarea", "Notes", cx);
            field.set_multiline(true, cx);
            field
        });
        cx.subscribe(&input, |host: &mut Host, _, event: &TextFieldEvent, _| {
            host.events.push(event.clone())
        })
        .detach();
        Host {
            input,
            events: Vec::new(),
        }
    });
    let host = window.root(cx).unwrap();
    let input = host.read_with(cx, |host, _| host.input.clone());
    support::click(cx, window.into(), "textarea");
    (window.into(), host, input)
}

fn candidate(
    cx: &mut gpui::TestAppContext,
    window: gpui::AnyWindowHandle,
    input: &Entity<TextField>,
    index: usize,
) -> Bounds<Pixels> {
    support::with_window(cx, window, |window, cx| {
        input.update(cx, |field, cx| {
            field
                .bounds_for_range(index..index, Default::default(), window, cx)
                .unwrap()
        })
    })
}

#[gpui::test]
fn multiline_enter_submits_only_with_control_or_command_and_undoes_newlines(
    cx: &mut gpui::TestAppContext,
) {
    let (window, host, input) = fixture(cx);
    cx.simulate_input(window, "first");
    support::press(cx, window, "enter");
    support::with_window(cx, window, |window, cx| {
        input.update(cx, |field, cx| {
            field.replace_text_in_range(None, "文🦀", window, cx);
        })
    });
    support::frame(cx, window);
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "first\n文🦀"
    );
    assert!(host.read_with(cx, |host, _| {
        host.events
            .iter()
            .all(|event| !matches!(event, TextFieldEvent::Submitted(_)))
    }));
    for key in ["ctrl-enter", "cmd-enter"] {
        support::press(cx, window, key);
    }
    assert_eq!(host.read_with(cx, |host, _| host.events.iter().filter(|event| matches!(event, TextFieldEvent::Submitted(value) if &**value == "first\n文🦀")).count()), 2);
    let modifier = if cfg!(target_os = "macos") {
        "cmd"
    } else {
        "ctrl"
    };
    support::press(cx, window, &format!("{modifier}-z"));
    support::press(cx, window, &format!("{modifier}-z"));
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "first"
    );
    support::press(cx, window, &format!("{modifier}-shift-z"));
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "first\n"
    );
}

#[gpui::test]
fn multiline_vertical_selection_keeps_preferred_column_across_short_lines(
    cx: &mut gpui::TestAppContext,
) {
    let (window, _, input) = fixture(cx);
    input.update(cx, |field, cx| field.set_value("abcd\nx\nabcd", cx));
    support::frame(cx, window);
    support::press(cx, window, "up");
    assert_eq!(
        input.read_with(cx, |field, _| field.selected_range_utf16()),
        6..6
    );
    support::press(cx, window, "shift-up");
    assert_eq!(
        input.read_with(cx, |field, _| field.selected_range_utf16()),
        4..6
    );
    support::press(cx, window, "enter");
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "abcd\n\nabcd"
    );
    support::press(cx, window, "end");
    support::press(cx, window, "shift-home");
    assert_eq!(
        input.read_with(cx, |field, _| field.selected_range_utf16()),
        5..5
    );
}

#[gpui::test]
fn multiline_height_tracks_newlines_wrapping_placeholder_and_deletion(
    cx: &mut gpui::TestAppContext,
) {
    let (window, _, input) = fixture(cx);
    let empty = support::bounds(cx, window, "textarea").size.height;
    input.update(cx, |field, cx| field.set_value("a\nb\nc\n", cx));
    support::frame(cx, window);
    let expanded = support::bounds(cx, window, "textarea");
    assert!(expanded.size.height > empty);
    let first = candidate(cx, window, &input, 0);
    let last = candidate(cx, window, &input, 6);
    assert_eq!(last.top() - first.top(), first.size.height * 3.0);
    assert!(last.bottom() <= expanded.bottom());

    support::press(
        cx,
        window,
        if cfg!(target_os = "macos") {
            "cmd-a"
        } else {
            "ctrl-a"
        },
    );
    support::press(cx, window, "backspace");
    assert_eq!(support::bounds(cx, window, "textarea").size.height, empty);
    input.update(cx, |field, cx| {
        field.set_placeholder("first\nsecond\nthird", cx)
    });
    support::frame(cx, window);
    assert!(support::bounds(cx, window, "textarea").size.height > empty);
    assert_eq!(input.read_with(cx, |field, _| field.value().to_owned()), "");

    input.update(cx, |field, cx| {
        field.set_value("abcdefghij ".repeat(8), cx);
        field.set_style(gpui::StyleRefinement::default().w(px(200.0)), cx);
    });
    support::frame(cx, window);
    let wide = support::bounds(cx, window, "textarea").size.height;
    input.update(cx, |field, cx| {
        field.set_style(gpui::StyleRefinement::default().w(px(100.0)), cx);
    });
    support::frame(cx, window);
    assert!(support::bounds(cx, window, "textarea").size.height > wide);
    input.update(cx, |field, cx| {
        field.set_style(gpui::StyleRefinement::default().w(px(200.0)), cx);
    });
    support::frame(cx, window);
    assert_eq!(support::bounds(cx, window, "textarea").size.height, wide);
    input.update(cx, |field, cx| field.set_value("short", cx));
    support::frame(cx, window);
    assert_eq!(support::bounds(cx, window, "textarea").size.height, empty);
    input.update(cx, |field, cx| {
        field.set_variant(beckon_kit::components::TextFieldVariant::Plain, cx);
        field.set_value("one\ntwo\n", cx);
    });
    support::frame(cx, window);
    let plain = support::bounds(cx, window, "textarea");
    let row_height = candidate(cx, window, &input, 0).size.height;
    assert!((plain.size.height - row_height * 3.0).abs() < px(1.0));
}

#[gpui::test]
fn multiline_max_height_caps_growth_and_releases_it_after_deletion(cx: &mut gpui::TestAppContext) {
    let (window, _, input) = fixture(cx);
    let empty = support::bounds(cx, window, "textarea").size.height;
    input.update(cx, |field, cx| {
        field.set_style(gpui::StyleRefinement::default().max_h(px(80.0)), cx);
        field.set_value("line\n".repeat(12), cx);
    });
    support::frame(cx, window);
    let bounds = support::bounds(cx, window, "textarea");
    assert_eq!(bounds.size.height, px(80.0));
    let length = input.read_with(cx, |field, _| field.value().encode_utf16().count());
    let caret = candidate(cx, window, &input, length);
    assert!(caret.top() >= bounds.top() && caret.bottom() <= bounds.bottom());
    input.update(cx, |field, cx| field.set_value("", cx));
    support::frame(cx, window);
    assert_eq!(support::bounds(cx, window, "textarea").size.height, empty);
}

#[gpui::test]
fn multiline_wrapping_hit_testing_candidate_bounds_and_scroll_share_geometry(
    cx: &mut gpui::TestAppContext,
) {
    let (window, _, input) = fixture(cx);
    input.update(cx, |field, cx| {
        field.set_style(gpui::StyleRefinement::default().h(px(96.0)), cx);
    });
    input.update(cx, |field, cx| field.set_value("abcdefghij ".repeat(8), cx));
    support::frame(cx, window);
    // Home/End use visual rows, not the entire logical line.
    support::press(cx, window, "home");
    let row_start = input.read_with(cx, |field, _| field.selected_range_utf16().start);
    assert!(row_start > 0);
    support::press(cx, window, "up");
    let previous = input.read_with(cx, |field, _| field.selected_range_utf16().start);
    assert!(previous < row_start);
    let before = candidate(cx, window, &input, previous);
    let after = candidate(cx, window, &input, row_start);
    assert!(after.top() > before.top());
    let point = before.origin + gpui::point(px(0.2), before.size.height / 2.0);
    support::with_window(cx, window, |window, cx| {
        input.update(cx, |field, cx| {
            assert_eq!(
                field.character_index_for_point(point, window, cx),
                Some(previous)
            );
        })
    });
    support::mouse_down(cx, window, point);
    support::mouse_up(cx, window, point);
    assert_eq!(
        input.read_with(cx, |field, _| field.selected_range_utf16()),
        previous..previous
    );

    input.update(cx, |field, cx| {
        field.set_value("a\nbb\nccc\ndddd\n文🦀\nlast\n", cx)
    });
    support::frame(cx, window);
    let length = input.read_with(cx, |field, _| field.value().encode_utf16().count());
    let caret = candidate(cx, window, &input, length);
    let bounds = support::bounds(cx, window, "textarea");
    assert!(caret.top() >= bounds.top() && caret.bottom() <= bounds.bottom());
    support::press(
        cx,
        window,
        if cfg!(target_os = "macos") {
            "cmd-home"
        } else {
            "ctrl-home"
        },
    );
    let first = candidate(cx, window, &input, 0);
    let second = candidate(cx, window, &input, 2);
    assert_eq!(second.top() - first.top(), first.size.height);
    let second_point = second.origin + gpui::point(px(0.2), second.size.height / 2.0);
    support::mouse_down(cx, window, second_point);
    support::mouse_up(cx, window, second_point);
    assert_eq!(
        input.read_with(cx, |field, _| field.selected_range_utf16()),
        2..2
    );
    // Wheel scrolling must not immediately snap back to the unchanged caret.
    gpui::VisualTestContext::from_window(window, cx).simulate_event(gpui::ScrollWheelEvent {
        position: bounds.center(),
        delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.0), -first.size.height * 2.0)),
        ..Default::default()
    });
    support::frame(cx, window);
    let scrolled = candidate(cx, window, &input, 9);
    assert!(scrolled.top() < first.top() + first.size.height * 3.0);
    assert_eq!(
        input.read_with(cx, |field, _| field.selected_range_utf16()),
        2..2
    );
}

#[gpui::test]
fn multiline_ime_on_later_lines_and_read_only_disabled_contracts(cx: &mut gpui::TestAppContext) {
    let (window, host, input) = fixture(cx);
    input.update(cx, |field, cx| field.set_value("one\n文🦀\n", cx));
    support::frame(cx, window);
    support::with_window(cx, window, |window, cx| {
        input.update(cx, |field, cx| {
            field.replace_and_mark_text_in_range(None, "ni", Some(2..2), window, cx);
            assert_eq!(field.marked_text_range(window, cx), Some(8..10));
        })
    });
    support::frame(cx, window);
    support::press(cx, window, "ctrl-enter");
    assert!(host.read_with(cx, |host, _| {
        host.events
            .iter()
            .all(|event| !matches!(event, TextFieldEvent::Submitted(_)))
    }));
    support::with_window(cx, window, |window, cx| {
        input.update(cx, |field, cx| {
            field.replace_text_in_range(None, "你好", window, cx);
            assert!(!field.is_composing());
        })
    });
    support::frame(cx, window);
    let caret = candidate(cx, window, &input, 10);
    let first = candidate(cx, window, &input, 0);
    assert!(caret.top() > first.top());
    let undo = if cfg!(target_os = "macos") {
        "cmd-z"
    } else {
        "ctrl-z"
    };
    support::press(cx, window, undo);
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "one\n文🦀\n"
    );
    input.update(cx, |field, cx| field.set_read_only(true, cx));
    support::frame(cx, window);
    support::press(cx, window, "enter");
    cx.simulate_input(window, "bad\nvalue");
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "one\n文🦀\n"
    );
    input.update(cx, |field, cx| {
        field.set_read_only(false, cx);
        field.set_disabled(true, cx);
    });
    support::frame(cx, window);
    support::with_window(cx, window, |window, cx| {
        input.update(cx, |field, cx| {
            field.replace_text_in_range(None, "bad\nvalue", window, cx);
            field.replace_and_mark_text_in_range(None, "bad", None, window, cx);
        })
    });
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "one\n文🦀\n"
    );
}
