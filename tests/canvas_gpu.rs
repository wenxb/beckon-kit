#![cfg(all(target_os = "linux", feature = "test-support"))]

use beckon_kit::{
    components::{
        Canvas, Surface,
        canvas::{Node, Point, Rect, Scene, Stroke, Transform},
    },
    test_support,
};
use gpui::{
    AppContext, Context, HeadlessAppContext, IntoElement, Render, Window, div, prelude::*, px, size,
};
use std::sync::Arc;

struct Fixture(Arc<Scene>);

impl Render for Fixture {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Surface::new("canvas-surface")
            .size_full()
            .p(px(0.0))
            .shadow(Vec::new())
            .child(
                div()
                    .size_full()
                    .bg(gpui::black())
                    .p(px(20.0))
                    .child(Canvas::new("canvas-gpu", self.0.clone())),
            )
    }
}

#[test]
fn painter_order_crosses_native_primitive_types_and_clips_transformed_groups() {
    let mut cx = HeadlessAppContext::with_platform(
        gpui_platform::current_platform(true).text_system(),
        Arc::new(()),
        gpui_platform::current_headless_renderer,
    );
    cx.update(beckon_kit::init);
    let scene = Arc::new(Scene {
        width: 200.0,
        height: 120.0,
        transform: Transform::default(),
        children: vec![
            Node::Arrow {
                id: "arrow".into(),
                from: Point { x: 10.0, y: 20.0 },
                to: Point { x: 150.0, y: 20.0 },
                stroke: Stroke {
                    color: gpui::rgb(0xff0000).into(),
                    width: 6.0,
                },
            },
            Node::Text {
                id: "text".into(),
                origin: Point { x: 10.0, y: 40.0 },
                text: "MMMMMMMMMM".into(),
                size: 20.0,
                color: gpui::white(),
            },
            Node::Rect {
                id: "cover".into(),
                rect: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 90.0,
                },
                fill: Some(gpui::rgb(0x0000ff).into()),
                stroke: None,
            },
            Node::Group {
                transform: Transform {
                    x: 120.0,
                    y: 60.0,
                    scale: 2.0,
                },
                clip: Some(Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 20.0,
                    height: 20.0,
                }),
                children: vec![Node::Rect {
                    id: "clipped".into(),
                    rect: Rect {
                        x: -10.0,
                        y: -10.0,
                        width: 40.0,
                        height: 40.0,
                    },
                    fill: Some(gpui::rgb(0x00ff00).into()),
                    stroke: None,
                }],
            },
        ],
    });
    let window = cx
        .open_window(size(px(240.0), px(160.0)), |_, cx| {
            cx.new(|_| Fixture(scene))
        })
        .unwrap();
    cx.run_until_parked();
    test_support::render(&mut cx, window.into());
    let pixels = test_support::render_pixels(&mut cx, window.into());
    let scale = cx
        .update_window(window.into(), |_, window, _| window.scale_factor())
        .unwrap();
    let pixel = |x: f32, y: f32| {
        pixels
            .get_pixel(((20.0 + x) * scale) as u32, ((20.0 + y) * scale) as u32)
            .0
    };
    assert_eq!(
        pixel(50.0, 20.0),
        [0, 0, 255, 255],
        "later rectangle must cover earlier arrow"
    );
    for y in 45..65 {
        for x in 15..95 {
            assert_eq!(
                pixel(x as f32, y as f32),
                [0, 0, 255, 255],
                "later rectangle must cover earlier glyphs"
            );
        }
    }
    assert_eq!(
        pixel(130.0, 20.0),
        [255, 0, 0, 255],
        "uncovered arrow remains visible"
    );
    assert_eq!(
        pixel(125.0, 65.0),
        [0, 255, 0, 255],
        "group scale/translation maps its clip"
    );
    for (x, y) in [(115.0, 65.0), (165.0, 65.0), (125.0, 55.0), (125.0, 105.0)] {
        assert_eq!(
            pixel(x, y),
            [0, 0, 0, 255],
            "group content must not escape its clip"
        );
    }
}
