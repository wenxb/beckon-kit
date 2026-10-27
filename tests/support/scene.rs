use gpui::{
    BackdropBlur, Bounds, ContentMask, Corners, Hsla, Quad, ScaledPixels, Scene, point, size,
};

pub fn bounds(x: f32, y: f32, width: f32, height: f32) -> Bounds<ScaledPixels> {
    Bounds::new(
        point(ScaledPixels(x), ScaledPixels(y)),
        size(ScaledPixels(width), ScaledPixels(height)),
    )
}

pub fn quad(bounds: Bounds<ScaledPixels>, color: Hsla) -> Quad {
    Quad {
        bounds,
        content_mask: ContentMask { bounds },
        background: color.into(),
        ..Quad::default()
    }
}

pub fn blur(bounds: Bounds<ScaledPixels>, sigma: f32) -> BackdropBlur {
    BackdropBlur {
        order: 0,
        bounds,
        content_mask: ContentMask { bounds },
        corner_radii: Corners::default(),
        sigma: ScaledPixels(sigma),
        opacity: 1.0,
    }
}

pub fn stripes(offset: u32, color: Hsla) -> Scene {
    let mut scene = Scene::default();
    for x in 0..128 {
        let fill = if (x as u32 + offset).is_multiple_of(2) {
            color
        } else {
            gpui::rgb(0).into()
        };
        scene.insert_primitive(quad(bounds(x as f32, 0.0, 1.0, 96.0), fill));
    }
    scene
}
