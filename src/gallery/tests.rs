use gpui::{
    AnyWindowHandle, AppContext, Entity, EntityInputHandler, InputEvent, KeyDownEvent, KeyUpEvent,
    Keystroke, TestAppContext, px, size,
};

use super::{Gallery, Page};
use crate::test_support;

fn fixture(cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<Gallery>) {
    cx.update(crate::init);
    let handle = cx.open_window(size(px(980.0), px(820.0)), |_, cx| Gallery::new(cx));
    let gallery = handle.root(cx).unwrap();
    let window = handle.into();
    test_support::render(cx, window);
    (window, gallery)
}

fn assert_feedback(gallery: &Entity<Gallery>, cx: &TestAppContext, expected: &str) {
    gallery.read_with(cx, |gallery, _| {
        assert_eq!(gallery.last_event.as_ref(), expected);
    });
}

fn press_key(cx: &mut TestAppContext, window: AnyWindowHandle, key: &str) {
    let keystroke = Keystroke::parse(key).unwrap();
    for event in [
        KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        }
        .to_platform_input(),
        KeyUpEvent { keystroke }.to_platform_input(),
    ] {
        cx.update_window(window, |_, window, cx| {
            window.dispatch_event(event, cx);
        })
        .unwrap();
        cx.run_until_parked();
        test_support::render(cx, window);
    }
}

#[gpui::test]
fn action_feedback_is_chinese_after_real_clicks(cx: &mut TestAppContext) {
    let (window, gallery) = fixture(cx);
    assert_feedback(&gallery, cx, "就绪");
    for (selector, expected) in [
        ("gallery-primary", "已执行 1 次"),
        ("gallery-secondary", "已请求复制"),
        ("gallery-quiet", "已请求更多操作"),
        ("gallery-soft", "已点击柔和"),
        ("gallery-outline", "已点击描边"),
        ("gallery-link", "已点击了解详情"),
        ("gallery-danger", "已请求移除"),
        ("gallery-toggle", "最近命令：已隐藏"),
        ("gallery-toggle", "最近命令：已显示"),
    ] {
        test_support::click(cx, window, selector);
        assert_feedback(&gallery, cx, expected);
    }
}

#[gpui::test]
fn select_specimen_shares_controlled_layout_and_markdown_has_bounds(cx: &mut TestAppContext) {
    let (window, gallery) = fixture(cx);
    test_support::bounds(cx, window, "gallery-markdown");
    test_support::click(cx, window, "gallery-select");
    press_key(cx, window, "down");
    press_key(cx, window, "enter");
    assert_eq!(gallery.read_with(cx, |g, _| g.mode.clone()), "grid");
}

#[gpui::test]
fn tabs_variants_share_selection_and_sidebar_keeps_keyboard_focus(cx: &mut TestAppContext) {
    let (window, gallery) = fixture(cx);
    test_support::click(cx, window, "gallery-pills-grid");
    assert_eq!(gallery.read_with(cx, |g, _| g.mode.clone()), "grid");
    test_support::click(cx, window, "gallery-underline-detail");
    assert_eq!(gallery.read_with(cx, |g, _| g.mode.clone()), "detail");
    test_support::click(cx, window, "page-materials");
    assert_eq!(gallery.read_with(cx, |g, _| g.page), Page::Materials);
    gpui::VisualTestContext::from_window(window, cx).simulate_keystrokes("left");
    cx.run_until_parked();
    test_support::render(cx, window);
    assert_eq!(gallery.read_with(cx, |g, _| g.page), Page::Canvas);
    test_support::bounds(cx, window, "gallery-canvas");
    for (key, expected) in [
        ("home", Page::Colors),
        ("down", Page::Icons),
        ("end", Page::Window),
    ] {
        gpui::VisualTestContext::from_window(window, cx).simulate_keystrokes(key);
        cx.run_until_parked();
        test_support::render(cx, window);
        assert_eq!(gallery.read_with(cx, |g, _| g.page), expected);
    }
}

#[gpui::test]
fn document_scroll_leaves_navigation_fixed_and_resets_on_page_changes(cx: &mut TestAppContext) {
    let (window, gallery) = fixture(cx);
    test_support::click(cx, window, "page-icons");
    let sidebar = test_support::bounds(cx, window, "gallery-sidebar");
    let header = test_support::bounds(cx, window, "gallery-header");
    let content = test_support::bounds(cx, window, "gallery-scroll");
    gpui::VisualTestContext::from_window(window, cx).simulate_event(gpui::ScrollWheelEvent {
        position: content.center(),
        delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.0), px(-500.0))),
        ..Default::default()
    });
    cx.run_until_parked();
    test_support::render(cx, window);
    assert!(gallery.read_with(cx, |g, _| g.scroll.offset().y) < px(0.0));
    assert_eq!(test_support::bounds(cx, window, "gallery-sidebar"), sidebar);
    assert_eq!(test_support::bounds(cx, window, "gallery-header"), header);
    test_support::click(cx, window, "page-colors");
    assert_eq!(gallery.read_with(cx, |g, _| g.scroll.offset().y), px(0.0));
    test_support::click(cx, window, "gallery-theme");
    assert_eq!(
        gallery.read_with(cx, |g, _| g.appearance),
        crate::theme::Appearance::Light
    );
    test_support::click(cx, window, "gallery-density");
    assert_eq!(
        gallery.read_with(cx, |g, _| g.density),
        crate::theme::Density::Compact
    );
}

#[gpui::test]
fn header_icon_actions_keep_their_keyboard_paths_and_fit_narrow_windows(cx: &mut TestAppContext) {
    use crate::theme::{Appearance, Density};

    let (window, gallery) = fixture(cx);
    for width in [980.0, 420.0] {
        cx.simulate_window_resize(window, size(px(width), px(820.0)));
        test_support::render(cx, window);
        test_support::click(cx, window, "gallery-density");
        assert_eq!(gallery.read_with(cx, |g, _| g.density), Density::Compact);
        press_key(cx, window, "space");
        assert_eq!(
            gallery.read_with(cx, |g, _| g.density),
            Density::Comfortable
        );
        press_key(cx, window, "tab");
        press_key(cx, window, "enter");
        assert_eq!(
            gallery.read_with(cx, |g, _| g.appearance),
            Appearance::Light
        );
        for selector in ["gallery-density", "gallery-theme"] {
            let bounds = test_support::bounds(cx, window, selector);
            let header = test_support::bounds(cx, window, "gallery-header");
            assert_eq!(bounds.size.width, bounds.size.height);
            assert!(bounds.left() >= header.left() && bounds.right() <= header.right());
            assert!(bounds.top() >= header.top() && bounds.bottom() <= header.bottom());
        }
        test_support::click(cx, window, "gallery-theme");
        assert_eq!(gallery.read_with(cx, |g, _| g.appearance), Appearance::Dark);
    }
}

#[gpui::test]
fn chinese_input_and_composition_report_localized_feedback(cx: &mut TestAppContext) {
    let (window, gallery) = fixture(cx);
    let input = gallery.read_with(cx, |gallery, _| gallery.input.clone());
    test_support::click(cx, window, "gallery-input");
    cx.update_window(window, |_, window, cx| {
        input.update(cx, |field, cx| {
            field.replace_and_mark_text_in_range(None, "ni", Some(2..2), window, cx);
        });
    })
    .unwrap();
    cx.run_until_parked();
    test_support::render(cx, window);
    assert_feedback(&gallery, cx, "正在组字 · 2 个字符");

    cx.simulate_input(window, "你好");
    cx.run_until_parked();
    test_support::render(cx, window);
    assert_feedback(&gallery, cx, "输入 · 2 个字符");
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "你好"
    );
    gpui::VisualTestContext::from_window(window, cx).simulate_keystrokes("enter");
    cx.run_until_parked();
    test_support::render(cx, window);
    assert_feedback(&gallery, cx, "已提交：你好");
}

#[gpui::test]
fn translated_materials_keep_stable_identities_and_chinese_feedback(cx: &mut TestAppContext) {
    let (window, gallery) = fixture(cx);
    test_support::click(cx, window, "page-materials");
    test_support::click(cx, window, "gallery-scene-change");
    assert_feedback(&gallery, cx, "实时背景 · 第 1 次更新");
    for selector in ["material-card", "material-details", "material-actions"] {
        test_support::bounds(cx, window, selector);
    }
    test_support::click(cx, window, "material-action-details");
    assert_feedback(&gallery, cx, "已查看材质");
    cx.simulate_backdrop_blur_support(window, true);
    test_support::render(cx, window);
    let bounds = test_support::bounds(cx, window, "gallery-glass-close");
    cx.update_window(window, |_, window, _| {
        assert!(window.painted_backdrop_blurs().iter().any(|blur| {
            blur.bounds == bounds.scale(window.scale_factor())
                && blur.corner_radii.top_left.0 == blur.bounds.size.width.0 / 2.0
        }));
    })
    .unwrap();
    test_support::click(cx, window, "gallery-glass-close");
    assert_feedback(&gallery, cx, "已点击磨砂按钮");
}

#[gpui::test]
fn virtual_list_specimen_jumps_and_reports_the_clicked_row(cx: &mut TestAppContext) {
    let (window, gallery) = fixture(cx);
    test_support::click(cx, window, "page-virtual-list");
    test_support::click(cx, window, "gallery-list-jump");
    test_support::click(cx, window, "gallery-list-row-5000");
    assert_feedback(&gallery, cx, "已选择第 5001 行");
}

#[gpui::test]
fn textarea_specimen_inserts_newlines_and_submits_localized_feedback(cx: &mut TestAppContext) {
    let (window, gallery) = fixture(cx);
    let input = gallery.read_with(cx, |gallery, _| gallery.textarea.clone());
    input.update(cx, |field, cx| field.set_value("说明", cx));
    test_support::render(cx, window);
    cx.update_window(window, |_, window, cx| {
        input.update(cx, |field, cx| field.focus(window, cx))
    })
    .unwrap();
    press_key(cx, window, "enter");
    assert_eq!(
        input.read_with(cx, |field, _| field.value().to_owned()),
        "说明\n"
    );
    press_key(cx, window, "ctrl-enter");
    assert_feedback(&gallery, cx, "已提交说明：说明\n");
}
