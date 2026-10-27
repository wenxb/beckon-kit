#![cfg(all(target_os = "linux", feature = "gallery", feature = "test-support"))]

use beckon_kit::{WindowConfig, gallery::Gallery, test_support};
use gpui::{AppContext, TestAppContext, px, size};

#[gpui::test]
fn window_settings_update_the_shell_through_real_input_without_reentering_it(
    cx: &mut TestAppContext,
) {
    let handle = cx.update(|cx| {
        WindowConfig::new("窗口设置")
            .size(size(px(980.0), px(820.0)))
            .open(cx, |_, cx| cx.new(Gallery::new))
            .unwrap()
    });
    let window = handle.into();
    test_support::click(cx, window, "page-window");
    test_support::click(cx, window, "window-radius");
    cx.simulate_keystrokes(window, "end");
    cx.run_until_parked();
    handle
        .read_with(cx, |root, _| {
            assert_eq!(root.client_style().corner_radii.top_left, px(32.0));
        })
        .unwrap();
    let visible = handle
        .read_with(cx, |root, _| root.frame_status().geometry.visible.size)
        .unwrap();
    test_support::click(cx, window, "window-shadow");
    handle
        .read_with(cx, |root, _| {
            assert!(root.client_style().shadows.is_empty());
            assert_eq!(root.frame_status().geometry.visible.size, visible);
            assert_eq!(root.frame_status().geometry.normal_inset, px(6.0));
        })
        .unwrap();
    test_support::click(cx, window, "window-shadow");
    handle
        .read_with(cx, |root, _| {
            assert!(!root.client_style().shadows.is_empty());
            assert_eq!(root.frame_status().geometry.visible.size, visible);
        })
        .unwrap();
}
