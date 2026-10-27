use beckon_kit::{
    WindowConfig,
    window::{FrameMode, TitlebarMode},
};
use gpui::{WindowBackgroundAppearance, px, size};

#[gpui::test]
fn frameless_content_keeps_requested_extent_and_rejects_chrome(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let config = WindowConfig::new("Plugin")
            .frame(FrameMode::None)
            .titlebar(TitlebarMode::None)
            .size(size(px(760.0), px(560.0)))
            .prepare(cx)
            .unwrap();
        assert_eq!(
            config.options.window_bounds.unwrap().get_bounds().size,
            size(px(760.0), px(560.0))
        );
        assert_eq!(
            config.options.window_background,
            WindowBackgroundAppearance::Transparent
        );
        assert!(
            WindowConfig::new("Conflict")
                .frame(FrameMode::None)
                .prepare(cx)
                .is_err()
        );
        assert!(
            WindowConfig::new("Conflict")
                .frame(FrameMode::None)
                .titlebar(TitlebarMode::None)
                .client_style(|style| style)
                .prepare(cx)
                .is_err()
        );
    });
}
#[gpui::test]
fn unsupported_overlay_is_not_a_fullscreen_fallback(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let error = WindowConfig::new("Overlay")
            .overlay(cx)
            .err()
            .expect("test backend cannot enforce overlay");
        assert!(error.to_string().contains("不会退回普通全屏"));
    });
}
