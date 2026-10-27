use beckon_kit::{
    components::SliderScale,
    material::{BackdropKind, Material, PreparationError, PreparedImage},
    theme::{Appearance, Density, Preferences, Theme},
};
use image::{Rgba, RgbaImage};

#[test]
fn slider_domain_rejects_invalid_values_and_handles_endpoint_not_divisible_by_step() {
    for (min, max, step) in [
        (0.0, 0.0, 1.0),
        (2.0, 1.0, 1.0),
        (0.0, f64::INFINITY, 1.0),
        (0.0, 1.0, 0.0),
        (0.0, 1.0, f64::NAN),
        (-f64::MAX, f64::MAX, 1.0),
    ] {
        assert!(SliderScale::new(min, max, step).is_err());
    }
    let scale = SliderScale::new(1.0, 2.0, 0.3).unwrap();
    assert_eq!(scale.value_at(1.0), 2.0);
    assert!((scale.offset(2.0, -1.0) - 1.9).abs() < 1e-10);
    assert_eq!(scale.normalize(f64::NAN), 1.0);
    assert_eq!(scale.normalize(f64::INFINITY), 2.0);
    assert_eq!(scale.value_at(f32::NEG_INFINITY), 1.0);
}

#[test]
fn frost_requires_an_actual_backdrop_and_respects_accessibility_preferences() {
    let material = Material::Frosted;
    for appearance in [Appearance::Light, Appearance::Dark] {
        let theme = Theme::new(appearance);
        let unavailable = material.resolve(&theme, BackdropKind::Unavailable);
        assert!(!unavailable.uses_backdrop);
        assert_eq!(unavailable.fill.a, 1.0);
        for backend in [BackdropKind::PreparedImage, BackdropKind::Scene] {
            assert!(material.resolve(&theme, backend).uses_backdrop);
            for preferences in [
                Preferences {
                    reduced_transparency: true,
                    increased_contrast: false,
                },
                Preferences {
                    reduced_transparency: false,
                    increased_contrast: true,
                },
            ] {
                let plan = material.resolve(&theme.clone().preferences(preferences), backend);
                assert!(!plan.uses_backdrop);
                assert_eq!(plan.fill.a, 1.0);
            }
        }
    }
}

#[test]
fn frosted_material_uses_theme_tokens_and_compact_density_is_explicit() {
    let mut theme = Theme::default();
    theme.frosted.tint_opacity = 0.8;
    assert_eq!(
        Material::Frosted
            .resolve(&theme, BackdropKind::Scene)
            .fill
            .a,
        0.8
    );
    assert!(
        theme
            .clone()
            .density(Density::Compact)
            .metrics
            .control_height
            < theme.metrics.control_height
    );
}

#[test]
fn invalid_frosted_theme_tokens_fall_back_to_opaque_material() {
    for (sigma, opacity) in [
        (0.0, 0.9),
        (-1.0, 0.9),
        (f32::NAN, 0.9),
        (f32::INFINITY, 0.9),
        (65.0, 0.9),
        (16.0, -0.1),
        (16.0, 1.1),
        (16.0, f32::NAN),
        (16.0, f32::INFINITY),
    ] {
        let mut theme = Theme::default();
        theme.frosted.blur_sigma = gpui::px(sigma);
        theme.frosted.tint_opacity = opacity;
        for backend in [BackdropKind::Scene, BackdropKind::PreparedImage] {
            let plan = Material::Frosted.resolve(&theme, backend);
            assert!(!plan.uses_backdrop);
            assert_eq!(plan.fill.a, 1.0);
        }
    }
    for opacity in [0.0, 1.0] {
        let mut theme = Theme::default();
        theme.frosted.blur_sigma = gpui::px(64.0);
        theme.frosted.tint_opacity = opacity;
        let plan = Material::Frosted.resolve(&theme, BackdropKind::Scene);
        assert!(plan.uses_backdrop);
        assert_eq!(plan.fill.a, opacity);
    }
}

#[test]
fn preparation_reduces_high_frequency_detail_and_preserves_image_identity() {
    let source = RgbaImage::from_fn(32, 32, |x, y| {
        if (x + y) % 2 == 0 {
            Rgba([255, 255, 255, 255])
        } else {
            Rgba([0, 0, 0, 255])
        }
    });
    let prepared = PreparedImage::new(source, 3.0).unwrap();
    let original = prepared.original();
    let blurred = prepared.blurred();
    assert!(std::sync::Arc::ptr_eq(&blurred, &prepared.blurred()));
    assert_ne!(original.id, blurred.id);
    let contrast = |bytes: &[u8]| {
        let pixels = bytes.as_chunks::<4>().0;
        pixels.iter().map(|pixel| pixel[0]).max().unwrap()
            - pixels.iter().map(|pixel| pixel[0]).min().unwrap()
    };
    assert!(contrast(blurred.as_bytes(0).unwrap()) < contrast(original.as_bytes(0).unwrap()) / 2);
}

#[test]
fn blur_uses_premultiplied_color_and_rgba_is_converted_to_gpui_bgra() {
    let source = RgbaImage::from_fn(12, 12, |x, _| {
        if x < 6 {
            Rgba([255, 0, 0, 255])
        } else {
            Rgba([0, 0, 0, 0])
        }
    });
    let prepared = PreparedImage::new(source, 2.0).unwrap();
    assert_eq!(
        &prepared.original().as_bytes(0).unwrap()[..4],
        &[0, 0, 255, 255]
    );
    let blurred = prepared.blurred();
    for pixel in blurred
        .as_bytes(0)
        .unwrap()
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|pixel| pixel[3] > 0)
    {
        assert_eq!(pixel[0], 0);
        assert_eq!(pixel[1], 0);
        assert_eq!(
            pixel[2], 255,
            "transparent edge should remain red, not darken"
        );
    }
}

#[test]
fn backdrop_preparation_rejects_empty_images_and_unbounded_sigma() {
    assert!(matches!(
        PreparedImage::new(RgbaImage::new(0, 1), 1.0),
        Err(PreparationError::EmptyImage)
    ));
    for sigma in [0.0, -1.0, f32::NAN, f32::INFINITY, 65.0] {
        assert!(matches!(
            PreparedImage::new(RgbaImage::new(1, 1), sigma),
            Err(PreparationError::InvalidSigma)
        ));
    }
}

fn contrast(foreground: gpui::Hsla, background: gpui::Hsla) -> f32 {
    fn luminance(color: gpui::Hsla) -> f32 {
        let color = color.to_rgb();
        let linear = |x: f32| {
            if x <= 0.04045 {
                x / 12.92
            } else {
                ((x + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
    }
    let a = luminance(foreground);
    let b = luminance(background);
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[test]
fn neutral_and_semantic_themes_meet_normal_text_contrast() {
    for appearance in [Appearance::Light, Appearance::Dark] {
        let colors = Theme::new(appearance).colors;
        for background in [colors.canvas, colors.surface, colors.elevated] {
            assert!(contrast(colors.text, background) >= 4.5);
            assert!(contrast(colors.secondary, background) >= 4.5);
        }
        for (foreground, background) in [
            (colors.on_primary, colors.primary),
            (colors.on_info, colors.info),
            (colors.on_success, colors.success),
        ] {
            assert!(contrast(foreground, background) >= 4.5);
        }
        for neutral in [
            colors.canvas,
            colors.surface,
            colors.elevated,
            colors.primary,
        ] {
            assert!(
                neutral.s < 0.15,
                "neutral structure must not carry a brand hue"
            );
        }
        for background in [colors.canvas, colors.surface, colors.elevated] {
            let base = background.to_rgb();
            for tint in [
                colors.selection,
                colors.info.opacity(0.13),
                colors.info.opacity(0.14),
            ] {
                let tint = tint.to_rgb();
                let selected = gpui::Hsla::from(gpui::Rgba {
                    r: tint.r * tint.a + base.r * (1.0 - tint.a),
                    g: tint.g * tint.a + base.g * (1.0 - tint.a),
                    b: tint.b * tint.a + base.b * (1.0 - tint.a),
                    a: 1.0,
                });
                assert!(
                    contrast(colors.info, selected) >= 4.5,
                    "{appearance:?}: selected/soft text must remain legible on {background:?}"
                );
            }
        }
        assert_eq!(colors.focus, colors.info);
        assert_ne!(colors.info, colors.success);
        assert!(contrast(colors.on_danger, colors.danger) >= 4.5);
    }
}

#[test]
fn frosted_text_keeps_contrast_over_worst_case_black_or_white_backdrops() {
    for appearance in [Appearance::Light, Appearance::Dark] {
        let theme = Theme::new(appearance);
        let fill = Material::Frosted.resolve(&theme, BackdropKind::Scene).fill;
        let foreground = fill.to_rgb();
        for background in [0.0, 1.0] {
            let composite = gpui::Hsla::from(gpui::Rgba {
                r: foreground.r * foreground.a + background * (1.0 - foreground.a),
                g: foreground.g * foreground.a + background * (1.0 - foreground.a),
                b: foreground.b * foreground.a + background * (1.0 - foreground.a),
                a: 1.0,
            });
            assert!(contrast(theme.colors.text, composite) >= 4.5);
            assert!(contrast(theme.colors.secondary, composite) >= 4.5);
        }
    }
}
