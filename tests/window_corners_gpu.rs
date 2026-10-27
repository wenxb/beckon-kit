#![cfg(all(target_os = "linux", feature = "test-support"))]
use beckon_kit::{
    WindowConfig, WindowRootHandleExt, test_support,
    theme::{Appearance, Theme},
    window::TitlebarMode,
};
use gpui::{
    AppContext, Context, HeadlessAppContext, IntoElement, Render, Window, div, prelude::*, px, size,
};
use std::sync::Arc;

struct Fill(u32);
impl Render for Fill {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().bg(gpui::rgb(self.0))
    }
}

#[test]
fn opaque_content_reaches_every_inner_corner_without_a_dark_seam() {
    let mut cx = HeadlessAppContext::with_platform(
        gpui_platform::current_platform(true).text_system(),
        Arc::new(()),
        || {
            Ok(Some(Box::new(
                gpui_wgpu::WgpuHeadlessRenderer::new_with_transparency(true)?,
            )))
        },
    );
    let handle = cx.update(|cx| {
        WindowConfig::new("Corners")
            .size(size(px(180.0), px(140.0)))
            .titlebar(TitlebarMode::None)
            .theme(Theme::new(Appearance::Light))
            .client_style(|frame| {
                frame
                    .radius(px(16.0))
                    .shadows(Vec::new())
                    .border(px(1.0), gpui::rgb(0xd0d0d0).into())
            })
            .open(cx, |_, cx| cx.new(|_| Fill(0xffffff)))
            .unwrap()
    });
    for color in [0xffffff, 0x262626] {
        handle
            .update_content(&mut cx, |fill, _, cx| {
                fill.0 = color;
                cx.notify();
            })
            .unwrap();
        let image = test_support::render_pixels(&mut cx, handle.into());
        let (bounds, scale) = handle
            .read_with(&cx, |root, _| {
                (
                    root.frame_status().geometry.visible,
                    root.frame_status().scale_factor,
                )
            })
            .unwrap();
        let x = (f32::from(bounds.left()) * scale).round() as u32;
        let y = (f32::from(bounds.top()) * scale).round() as u32;
        let width = (f32::from(bounds.size.width) * scale).round() as u32;
        let height = (f32::from(bounds.size.height) * scale).round() as u32;
        let radius = 16.0 * scale;
        let mut checked = 0;
        for dy in 0..radius as u32 {
            for dx in 0..radius as u32 {
                let distance = ((dx as f32 + 0.5 - radius).powi(2)
                    + (dy as f32 + 0.5 - radius).powi(2))
                .sqrt()
                    - radius;
                for (right, bottom) in [(false, false), (true, false), (false, true), (true, true)]
                {
                    let p = image
                        .get_pixel(
                            x + if right { width - 1 - dx } else { dx },
                            y + if bottom { height - 1 - dy } else { dy },
                        )
                        .0;
                    if distance >= 0.5 {
                        assert_eq!(
                            p[3], 0,
                            "corner exterior must stay transparent: {dx},{dy} {p:?}"
                        );
                    }
                    if color == 0xffffff && (-scale - 0.5..=-0.5).contains(&distance) {
                        assert_eq!(p[3], 255, "border inner antialias stays opaque");
                        assert!(
                            p[..3].iter().all(|channel| *channel >= 207),
                            "border must blend between gray and white, never darken to black: {dx},{dy} d={distance} {p:?}"
                        );
                    }
                    if (-scale - 2.0..=-scale - 0.5).contains(&distance) {
                        let expected = (color & 0xff) as u8;
                        assert_eq!(p[3], 255, "inner corner must be opaque: {dx},{dy} {p:?}");
                        assert!(
                            p[..3].iter().all(|channel| channel.abs_diff(expected) <= 1),
                            "inner corner must match content, not a black seam: {dx},{dy} {p:?}"
                        );
                        checked += 1;
                    }
                }
            }
        }
        assert!(checked > 24);
    }
}
