#![cfg(all(target_os = "linux", feature = "test-support"))]

use beckon_kit::{
    components::{Icon, IconName},
    test_support,
};
use gpui::{
    AppContext, AssetSource, Context, HeadlessAppContext, IntoElement, Render, SharedString,
    Window, div, prelude::*, px, size,
};
use std::{borrow::Cow, sync::Arc};

struct Assets;
impl AssetSource for Assets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        Ok((path == "custom/check.svg").then(|| Cow::Borrowed(IconName::Check.svg())))
    }
    fn list(&self, _: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(vec!["custom/check.svg".into()])
    }
}

struct Sources;
impl Render for Sources {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let custom = Icon::data(IconName::Check.svg());
        div()
            .size_full()
            .flex()
            .bg(gpui::black())
            .text_color(gpui::rgb(0x00ff00))
            .child(Icon::new(IconName::Check).size(px(24.0)))
            .child(custom.clone().size(px(24.0)))
            .child(Icon::path("custom/check.svg").size(px(24.0)))
            .child(custom.size(px(24.0)).text_color(gpui::rgb(0xff0000)))
    }
}

#[test]
fn named_bytes_and_asset_icons_share_geometry_and_inherit_or_override_color() {
    let mut cx = HeadlessAppContext::with_platform(
        gpui_platform::current_platform(true).text_system(),
        Arc::new(Assets),
        gpui_platform::current_headless_renderer,
    );
    let window = cx
        .open_window(size(px(96.0), px(24.0)), |_, cx| cx.new(|_| Sources))
        .unwrap();
    cx.run_until_parked();
    let pixels = test_support::render_pixels(&mut cx, window.into());
    let width = pixels.width() / 4;
    let mut green_strokes = 0;
    let mut red_strokes = 0;
    for y in 0..pixels.height() {
        for x in 0..width {
            let named = pixels.get_pixel(x, y);
            assert_eq!(named, pixels.get_pixel(x + width, y));
            assert_eq!(named, pixels.get_pixel(x + width * 2, y));
            green_strokes += usize::from(named[1] > 50 && named[0] == 0 && named[2] == 0);
            let explicit = pixels.get_pixel(x + width * 3, y);
            red_strokes += usize::from(explicit[0] > 50 && explicit[1] == 0 && explicit[2] == 0);
        }
    }
    assert!(
        green_strokes > 10,
        "named, byte and asset icons must inherit green foreground"
    );
    assert!(
        red_strokes > 10,
        "explicit foreground must override inheritance without changing the source clone"
    );
}
