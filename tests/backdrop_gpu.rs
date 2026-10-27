#![cfg(target_os = "linux")]

mod support;

use gpui::{Corners, DevicePixels, PlatformHeadlessRenderer, ScaledPixels, Scene, size};
use support::scene::{blur, bounds, quad, stripes};

fn renderer() -> Box<dyn PlatformHeadlessRenderer> {
    let renderer = gpui_platform::current_headless_renderer()
        .expect("initialize GPU")
        .expect("real renderer required");
    assert!(
        renderer.is_backdrop_blur_supported(),
        "WGPU must implement real backdrop blur"
    );
    renderer
}

fn render(
    renderer: &mut dyn PlatformHeadlessRenderer,
    scene: &mut Scene,
    width: i32,
    height: i32,
) -> image::RgbaImage {
    scene.finish();
    renderer
        .render_scene_to_image(scene, size(DevicePixels(width), DevicePixels(height)))
        .expect("real GPU rendering and validation")
}

#[test]
fn gpu_blurs_live_quads_without_an_image_and_keeps_foreground_sharp() {
    let mut renderer = renderer();
    let mut scene = stripes(0, gpui::rgb(0xffffff).into());
    scene.insert_primitive(blur(bounds(24.0, 16.0, 80.0, 64.0), 6.0));
    scene.insert_primitive(quad(
        bounds(52.0, 38.0, 12.0, 12.0),
        gpui::rgb(0xff0011).into(),
    ));
    let pixels = render(renderer.as_mut(), &mut scene, 128, 96);
    let values: Vec<_> = (36..90).map(|x| pixels.get_pixel(x, 28)[0]).collect();
    assert!(
        values.iter().max().unwrap() - values.iter().min().unwrap() < 5,
        "high-frequency detail must be filtered"
    );
    assert!(
        (170..205).contains(&values[0]),
        "black/white mix must be filtered in linear space: {}",
        values[0]
    );
    assert_eq!(pixels.get_pixel(10, 28).0, [255, 255, 255, 255]);
    assert_eq!(pixels.get_pixel(11, 28).0, [0, 0, 0, 255]);
    assert_eq!(pixels.get_pixel(52, 40).0, [255, 0, 17, 255]);
    assert_ne!(
        pixels.get_pixel(51, 40).0,
        [255, 0, 17, 255],
        "no foreground bloom or refraction"
    );
}

#[test]
fn changed_scene_updates_in_the_same_frame_and_multiple_regions_have_distinct_uniforms() {
    let mut renderer = renderer();
    for color in [0xff0000, 0x00ff00, 0x0000ff] {
        let mut scene = stripes(0, gpui::rgb(color).into());
        scene.insert_primitive(blur(bounds(8.0, 8.0, 40.0, 72.0), 4.0));
        scene.insert_primitive(blur(bounds(72.0, 8.0, 40.0, 72.0), 12.0));
        let pixels = render(renderer.as_mut(), &mut scene, 128, 96);
        let channel = match color {
            0xff0000 => 0,
            0x00ff00 => 1,
            _ => 2,
        };
        for x in [28, 92] {
            let pixel = pixels.get_pixel(x, 44);
            assert!(
                pixel[channel] > 170 && pixel[channel] < 205,
                "region at {x} must reflect the current scene: {pixel:?}"
            );
            assert_eq!(pixel[(channel + 1) % 3], 0);
            assert_eq!(pixel[(channel + 2) % 3], 0);
        }
    }
}

#[test]
fn rounded_output_and_content_mask_do_not_clip_the_sampling_halo() {
    let mut renderer = renderer();
    let mut scene = Scene::default();
    scene.insert_primitive(quad(bounds(0.0, 0.0, 128.0, 96.0), gpui::rgb(0).into()));
    scene.insert_primitive(quad(
        bounds(0.0, 0.0, 24.0, 96.0),
        gpui::rgb(0xffffff).into(),
    ));
    let mut region = blur(bounds(24.0, 16.0, 80.0, 64.0), 8.0);
    region.corner_radii = Corners::all(ScaledPixels(16.0));
    region.content_mask.bounds = bounds(24.0, 0.0, 40.0, 96.0);
    scene.insert_primitive(region);
    let pixels = render(renderer.as_mut(), &mut scene, 128, 96);
    assert_eq!(
        pixels.get_pixel(24, 16).0,
        [0, 0, 0, 255],
        "rounded corner must not receive blur"
    );
    assert!(
        pixels.get_pixel(25, 48)[0] > 150,
        "white outside the output bounds must contribute to blur"
    );
    assert_eq!(
        pixels.get_pixel(65, 48).0,
        [0, 0, 0, 255],
        "content mask must clip output"
    );
}

#[test]
fn gpu_handles_window_edges_large_blur_and_repeated_resizes() {
    let mut renderer = renderer();
    for (width, height, sigma) in [
        (128, 96, 1.0),
        (71, 53, 64.0),
        (208, 111, 24.0),
        (128, 96, 6.0),
    ] {
        let mut scene = Scene::default();
        scene.insert_primitive(quad(
            bounds(0.0, 0.0, width as f32, height as f32),
            gpui::rgb(0x336699).into(),
        ));
        scene.insert_primitive(blur(
            bounds(-20.0, -10.0, width as f32 + 40.0, height as f32 + 20.0),
            sigma,
        ));
        let pixels = render(renderer.as_mut(), &mut scene, width, height);
        assert_eq!(pixels.dimensions(), (width as u32, height as u32));
        for (x, y) in [
            (0, 0),
            (width as u32 / 2, height as u32 / 2),
            (width as u32 - 1, height as u32 - 1),
        ] {
            let pixel = pixels.get_pixel(x, y);
            for (actual, expected) in pixel.0.into_iter().zip([51u8, 102, 153, 255]) {
                assert!(
                    actual.abs_diff(expected) <= 1,
                    "constant edges must not darken: {pixel:?}"
                );
            }
        }
    }
}

#[test]
fn large_and_fractional_sigma_prefilter_every_source_pixel_without_aliasing() {
    let mut renderer = renderer();
    for sigma in [0.5, 10.5, 16.0, 30.5, 64.0, 80.0] {
        for offset in [0, 1] {
            let mut scene = stripes(offset, gpui::rgb(0xffffff).into());
            scene.insert_primitive(blur(bounds(8.25, 8.5, 111.5, 79.25), sigma));
            let pixels = render(renderer.as_mut(), &mut scene, 128, 96);
            if sigma >= 10.0 {
                let pixel = pixels.get_pixel(64, 48);
                assert!(
                    pixel[0].abs_diff(188) <= 3,
                    "sigma={sigma}, phase={offset}: {pixel:?}"
                );
            }
            assert_eq!(
                pixels.get_pixel(4, 48)[0],
                if offset == 0 { 255 } else { 0 }
            );
        }
    }
}

#[test]
fn a_later_neighbor_cannot_leak_into_an_earlier_blur_sampling_halo() {
    let mut renderer = renderer();
    let mut scene = Scene::default();
    scene.insert_primitive(quad(bounds(0.0, 0.0, 128.0, 96.0), gpui::rgb(0).into()));
    scene.push_layer(bounds(0.0, 0.0, 128.0, 96.0));
    scene.insert_primitive(blur(bounds(40.0, 16.0, 48.0, 64.0), 8.0));
    scene.insert_primitive(quad(
        bounds(24.0, 16.0, 16.0, 64.0),
        gpui::rgb(0xffffff).into(),
    ));
    scene.pop_layer();
    let pixels = render(renderer.as_mut(), &mut scene, 128, 96);
    assert_eq!(pixels.get_pixel(39, 48).0, [255, 255, 255, 255]);
    assert_eq!(
        pixels.get_pixel(40, 48).0,
        [0, 0, 0, 255],
        "future white geometry must not be sampled"
    );
}

#[test]
fn replayed_blur_uses_current_background_and_nested_blur_sees_prior_foreground() {
    let mut renderer = renderer();
    let mut cached = stripes(0, gpui::rgb(0xff0000).into());
    let start = cached.len();
    cached.push_layer(bounds(16.0, 16.0, 96.0, 64.0));
    cached.insert_primitive(blur(bounds(16.0, 16.0, 96.0, 64.0), 4.0));
    cached.insert_primitive(quad(
        bounds(60.0, 16.0, 10.0, 64.0),
        gpui::rgb(0x0000ff).into(),
    ));
    cached.pop_layer();
    let _ = render(renderer.as_mut(), &mut cached, 128, 96);

    let mut scene = stripes(0, gpui::rgb(0x00ff00).into());
    scene.replay(start..cached.len(), &cached);
    scene.insert_primitive(blur(bounds(48.0, 30.0, 52.0, 36.0), 6.0));
    scene.insert_primitive(quad(
        bounds(78.0, 42.0, 4.0, 10.0),
        gpui::rgb(0xffff00).into(),
    ));
    let pixels = render(renderer.as_mut(), &mut scene, 128, 96);
    let first = pixels.get_pixel(32, 44);
    assert!(
        first[1].abs_diff(188) <= 3 && first[0] == 0 && first[2] == 0,
        "replay must sample current green: {first:?}"
    );
    let nested = pixels.get_pixel(58, 44);
    assert!(
        nested[2] > 70 && nested[0] == 0,
        "nested glass must sample the earlier blue foreground: {nested:?}"
    );
    assert_eq!(
        pixels.get_pixel(62, 20).0,
        [0, 0, 255, 255],
        "outside the nested region remains sharp"
    );
    assert_eq!(
        pixels.get_pixel(78, 44).0,
        [255, 255, 0, 255],
        "new foreground remains sharp"
    );
}

#[test]
fn transparent_backdrops_filter_premultiplied_alpha_and_mix_instead_of_blending_twice() {
    let mut renderer = gpui_wgpu::WgpuHeadlessRenderer::new_with_transparency(true)
        .expect("transparent GPU renderer");
    let mut full_alpha = None;
    for opacity in [1.0, 0.5] {
        let mut scene = Scene::default();
        scene.insert_primitive(quad(
            bounds(0.0, 0.0, 64.0, 96.0),
            gpui::rgb(0xff0000).opacity(0.5).into(),
        ));
        let mut region = blur(bounds(16.0, 16.0, 96.0, 64.0), 8.0);
        region.opacity = opacity;
        scene.insert_primitive(region);
        let pixels = render(&mut renderer, &mut scene, 128, 96);
        assert_eq!(pixels.get_pixel(10, 48).0, [128, 0, 0, 128]);
        for x in 50..90 {
            let pixel = pixels.get_pixel(x, 48);
            assert!(
                pixel[0].abs_diff(pixel[3]) <= 1,
                "transparent red must not acquire black fringes: {pixel:?}"
            );
            assert_eq!((pixel[1], pixel[2]), (0, 0));
        }
        let edge = pixels.get_pixel(70, 48)[3];
        assert!(edge > 0 && edge < 128);
        if let Some(full) = full_alpha {
            assert!(edge.abs_diff(full / 2) <= 1);
        } else {
            full_alpha = Some(edge);
        }
    }

    let mut scene = Scene::default();
    scene.insert_primitive(quad(
        bounds(0.0, 0.0, 128.0, 96.0),
        gpui::rgb(0xff0000).opacity(0.5).into(),
    ));
    for sigma in [4.0, 16.0] {
        let mut region = blur(bounds(16.0, 16.0, 96.0, 64.0), sigma);
        region.opacity = 0.5;
        scene.insert_primitive(region);
    }
    scene.insert_primitive(quad(
        bounds(50.0, 40.0, 10.0, 10.0),
        gpui::rgb(0x0000ff).opacity(0.25).into(),
    ));
    let pixels = render(&mut renderer, &mut scene, 128, 96);
    assert_eq!(
        pixels.get_pixel(30, 48).0,
        [128, 0, 0, 128],
        "repeated blur must not increase alpha"
    );
    for (actual, expected) in pixels
        .get_pixel(54, 44)
        .0
        .into_iter()
        .zip([96u8, 0, 64, 160])
    {
        assert!(
            actual.abs_diff(expected) <= 1,
            "foreground composites once over the filtered scene"
        );
    }
}
