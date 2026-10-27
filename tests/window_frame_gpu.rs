#![cfg(all(target_os = "linux", feature = "test-support"))]
mod support;

use gpui::{
    Bounds, ContentMask, Corners, DevicePixels, PlatformHeadlessRenderer, Quad,
    RoundedClipBoundary, ScaledPixels, Scene, point, size,
};

fn bounds(x: f32, y: f32, w: f32, h: f32) -> Bounds<ScaledPixels> {
    Bounds::new(
        point(ScaledPixels(x), ScaledPixels(y)),
        size(ScaledPixels(w), ScaledPixels(h)),
    )
}
fn quad(b: Bounds<ScaledPixels>, color: u32) -> Quad {
    Quad {
        bounds: b,
        content_mask: ContentMask { bounds: b },
        background: gpui::rgb(color).into(),
        ..Default::default()
    }
}
fn boundary(order: u32, begin: bool, b: Bounds<ScaledPixels>, radius: f32) -> RoundedClipBoundary {
    RoundedClipBoundary {
        order,
        begin,
        bounds: b,
        radii: Corners::all(ScaledPixels(radius)),
    }
}

#[test]
fn group_coverage_applies_once_to_overlapping_layers_and_reuses_after_resize() {
    let mut renderer = gpui_wgpu::WgpuHeadlessRenderer::new_with_transparency(true).unwrap();
    assert!(renderer.is_rounded_clip_supported());
    for width in [64, 96, 64] {
        let b = bounds(4.0, 4.0, 48.0, 48.0);
        let mut one = Scene::default();
        one.insert_primitive(quad(b, 0xff0000));
        one.finish();
        one.quads[0].order = 2;
        one.rounded_clips = vec![boundary(1, true, b, 16.0), boundary(3, false, b, 16.0)];
        let a = renderer
            .render_scene_to_image(&one, size(DevicePixels(width), DevicePixels(64)))
            .unwrap();
        let mut layered = Scene::default();
        layered.insert_primitive(quad(b, 0x0000ff));
        layered.insert_primitive(quad(b, 0xff0000));
        layered.finish();
        for (index, q) in layered.quads.iter_mut().enumerate() {
            q.order = index as u32 + 2;
        }
        layered.rounded_clips = vec![boundary(1, true, b, 16.0), boundary(4, false, b, 16.0)];
        let b = renderer
            .render_scene_to_image(&layered, size(DevicePixels(width), DevicePixels(64)))
            .unwrap();
        assert_eq!(
            a.as_raw(),
            b.as_raw(),
            "overlapping backgrounds must not compound corner coverage"
        );
        assert_eq!(a.get_pixel(4, 4)[3], 0);
        assert_eq!(a.get_pixel(28, 28).0, [255, 0, 0, 255]);
        assert!(
            a.pixels().any(|p| p[3] > 0 && p[3] < 255),
            "anti-aliased transparent edge"
        );
    }
}

#[test]
fn rounded_group_clips_only_blur_output_not_its_sampling_halo() {
    let mut renderer = gpui_wgpu::WgpuHeadlessRenderer::new_with_transparency(true).unwrap();
    let all = bounds(0.0, 0.0, 128.0, 96.0);
    let output = bounds(24.0, 16.0, 80.0, 64.0);
    let mut scene = Scene::default();
    scene.insert_primitive(quad(all, 0));
    scene.insert_primitive(quad(bounds(0.0, 0.0, 24.0, 96.0), 0xffffff));
    scene.insert_primitive(support::scene::blur(output, 8.0));
    scene.finish();
    scene.quads[0].order = 0;
    scene.quads[1].order = 1;
    scene.backdrop_blurs[0].order = 3;
    scene.rounded_clips = vec![
        boundary(2, true, output, 16.0),
        boundary(4, false, output, 16.0),
    ];
    let pixels = renderer
        .render_scene_to_image(&scene, size(DevicePixels(128), DevicePixels(96)))
        .unwrap();
    assert_eq!(pixels.get_pixel(24, 16).0, [0, 0, 0, 255]);
    assert!(
        pixels.get_pixel(25, 48)[0] > 150,
        "white outside group contributes to blur halo"
    );
    assert_eq!(pixels.get_pixel(10, 48).0, [255, 255, 255, 255]);
}

#[test]
fn nested_scopes_keep_background_and_outer_shadow_unclipped() {
    let mut renderer = gpui_wgpu::WgpuHeadlessRenderer::new_with_transparency(true).unwrap();
    let all = bounds(0.0, 0.0, 64.0, 64.0);
    let outer = bounds(4.0, 4.0, 56.0, 56.0);
    let inner = bounds(16.0, 16.0, 32.0, 32.0);
    let mut scene = Scene::default();
    scene.insert_primitive(quad(all, 0x0000ff));
    scene.insert_primitive(quad(all, 0xff0000));
    scene.insert_primitive(quad(all, 0x00ff00));
    scene.finish();
    for (q, order) in scene.quads.iter_mut().zip([0, 2, 4]) {
        q.order = order;
    }
    scene.rounded_clips = vec![
        boundary(1, true, outer, 16.0),
        boundary(3, true, inner, 8.0),
        boundary(5, false, inner, 8.0),
        boundary(6, false, outer, 16.0),
    ];
    let image = renderer
        .render_scene_to_image(&scene, size(DevicePixels(64), DevicePixels(64)))
        .unwrap();
    assert_eq!(image.get_pixel(0, 0).0, [0, 0, 255, 255]);
    assert_eq!(image.get_pixel(32, 8).0, [255, 0, 0, 255]);
    assert_eq!(image.get_pixel(32, 32).0, [0, 255, 0, 255]);
    assert_eq!(image.get_pixel(16, 16).0, [255, 0, 0, 255]);
}
