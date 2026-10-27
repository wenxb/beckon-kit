use beckon_kit::{
    WindowConfig,
    window::{FrameMode, TitlebarMode},
};
use gpui::{WindowOptions, px, size};

#[gpui::test]
fn defaults_and_common_customization_prepare(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let prepared = WindowConfig::new("默认窗口").prepare(cx).unwrap();
        assert!(prepared.options.titlebar.unwrap().appears_transparent);
        let prepared = WindowConfig::new("自定义窗口")
            .size(size(px(980.0), px(680.0)))
            .min_size(size(px(420.0), px(380.0)))
            .client_style(|frame| frame.radius(px(16.0)))
            .prepare(cx)
            .unwrap();
        let bounds = prepared.options.window_bounds.unwrap().get_bounds();
        if cfg!(target_os = "linux") {
            assert!(bounds.size.width > px(980.0));
        }
        assert_eq!(prepared.options.app_id, None);
    });
}

#[gpui::test]
fn implicit_initial_size_respects_a_larger_minimum(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let minimum = size(px(1200.0), px(900.0));
        let prepared = WindowConfig::new("大窗口")
            .min_size(minimum)
            .prepare(cx)
            .unwrap();
        let size = prepared.options.window_bounds.unwrap().get_bounds().size;
        assert!(size.width >= minimum.width && size.height >= minimum.height);
    });
}

#[gpui::test]
fn invalid_and_conflicting_options_are_rejected_in_any_order(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        for config in [
            WindowConfig::new("冲突")
                .frame(FrameMode::Native)
                .client_style(|s| s),
            WindowConfig::new("冲突")
                .client_style(|s| s)
                .frame(FrameMode::Native),
            WindowConfig::new("冲突")
                .frame(FrameMode::Client)
                .titlebar(TitlebarMode::System),
            WindowConfig::new("冲突")
                .frame(FrameMode::Native)
                .titlebar(TitlebarMode::None),
            WindowConfig::new("无效").size(size(px(f32::NAN), px(100.0))),
            WindowConfig::new("无效").min_size(size(px(-1.0), px(100.0))),
            WindowConfig::new("无效").client_style(|s| s.radius(px(-1.0))),
            WindowConfig::new("无效").client_style(|s| {
                s.shadows(vec![gpui::BoxShadow::new(
                    px(0.0),
                    px(1000.0),
                    gpui::black(),
                )])
            }),
            WindowConfig::new("冲突").options(WindowOptions {
                window_min_size: Some(size(px(1.0), px(1.0))),
                ..Default::default()
            }),
            WindowConfig::new("冲突")
                .size(size(px(100.0), px(100.0)))
                .options(WindowOptions {
                    window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds::new(
                        gpui::point(px(0.0), px(0.0)),
                        size(px(200.0), px(200.0)),
                    ))),
                    ..Default::default()
                }),
        ] {
            assert!(config.prepare(cx).is_err());
        }
    });
}
