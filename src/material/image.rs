use gpui::{App, Bounds, Corners, Pixels, RenderImage, Window, point, px, size};
use image::{Frame, Rgba, Rgba32FImage, RgbaImage};
use std::{fmt, sync::Arc};

use super::{Backdrop, BackdropKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparationError {
    EmptyImage,
    InvalidSigma,
}

impl fmt::Display for PreparationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EmptyImage => "a backdrop image must not be empty",
            Self::InvalidSigma => "blur sigma must be finite and in (0, 64] image pixels",
        })
    }
}
impl std::error::Error for PreparationError {}

/// Immutable, transferable preparation. Run this on a worker before rendering.
/// Every surface shares these GPU image identities; paint only crops and samples.
pub struct PreparedImage {
    original: Arc<RenderImage>,
    blurred: Arc<RenderImage>,
    dimensions: (u32, u32),
    sigma: f32,
}

impl PreparedImage {
    pub fn new(source: RgbaImage, sigma: f32) -> Result<Self, PreparationError> {
        if source.width() == 0 || source.height() == 0 {
            return Err(PreparationError::EmptyImage);
        }
        if !sigma.is_finite() || sigma <= 0.0 || sigma > 64.0 {
            return Err(PreparationError::InvalidSigma);
        }
        // Blur linear, premultiplied pixels so transparent edges cannot bleed black.
        let linear = Rgba32FImage::from_fn(source.width(), source.height(), |x, y| {
            let pixel = source.get_pixel(x, y);
            let alpha = f32::from(pixel[3]) / 255.0;
            Rgba([
                to_linear(f32::from(pixel[0]) / 255.0) * alpha,
                to_linear(f32::from(pixel[1]) / 255.0) * alpha,
                to_linear(f32::from(pixel[2]) / 255.0) * alpha,
                alpha,
            ])
        });
        let blurred = image::imageops::blur(&linear, sigma);
        let blurred = RgbaImage::from_fn(source.width(), source.height(), |x, y| {
            let pixel = blurred.get_pixel(x, y);
            let alpha = pixel[3].clamp(0.0, 1.0);
            let channel = |value: f32| {
                if alpha <= f32::EPSILON {
                    0
                } else {
                    quantize(to_srgb((value / alpha).clamp(0.0, 1.0)))
                }
            };
            Rgba([
                channel(pixel[0]),
                channel(pixel[1]),
                channel(pixel[2]),
                quantize(alpha),
            ])
        });
        Ok(Self {
            dimensions: source.dimensions(),
            sigma,
            original: render_image(source),
            blurred: render_image(blurred),
        })
    }

    pub fn original(&self) -> Arc<RenderImage> {
        self.original.clone()
    }
    pub fn blurred(&self) -> Arc<RenderImage> {
        self.blurred.clone()
    }
    pub fn sigma(&self) -> f32 {
        self.sigma
    }
    pub fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }
}

fn render_image(mut pixels: RgbaImage) -> Arc<RenderImage> {
    // GPUI RenderImage expects BGRA, while image operations use RGBA.
    for pixel in pixels.pixels_mut() {
        pixel.0.swap(0, 2);
    }
    Arc::new(RenderImage::new([Frame::new(pixels)]))
}

fn to_linear(value: f32) -> f32 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn to_srgb(value: f32) -> f32 {
    if value <= 0.0031308 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}

fn quantize(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ImagePlane {
    /// Fit the image to this surface. Useful for an isolated material specimen.
    #[default]
    Local,
    /// Match an image drawn with object-fit: cover across the whole window.
    Viewport,
    /// Match a captured or owned image occupying these window coordinates.
    Bounds(Bounds<Pixels>),
}

#[derive(Clone)]
pub struct BackdropImage {
    image: Arc<PreparedImage>,
    plane: ImagePlane,
}

impl BackdropImage {
    pub fn new(image: Arc<PreparedImage>, plane: ImagePlane) -> Self {
        Self { image, plane }
    }

    /// Draw the sharp source using the exact same coordinate mapping as the frost.
    pub fn paint_original(&self, bounds: Bounds<Pixels>, window: &mut Window) -> gpui::Result<()> {
        let image_bounds = self.image_bounds(bounds, window);
        window.paint_image(
            bounds,
            image_bounds,
            Corners::default(),
            self.image.original(),
            0,
            false,
        )
    }

    fn image_bounds(&self, bounds: Bounds<Pixels>, window: &Window) -> Bounds<Pixels> {
        let plane = match self.plane {
            ImagePlane::Local => bounds,
            ImagePlane::Viewport => Bounds::new(point(px(0.0), px(0.0)), window.viewport_size()),
            ImagePlane::Bounds(bounds) => bounds,
        };
        let (width, height) = self.image.dimensions();
        let scale =
            (plane.size.width / px(width as f32)).max(plane.size.height / px(height as f32));
        let image_size = size(px(width as f32) * scale, px(height as f32) * scale);
        Bounds::new(
            plane.center() - point(image_size.width / 2.0, image_size.height / 2.0),
            image_size,
        )
    }
}

impl Backdrop for BackdropImage {
    fn kind(&self) -> BackdropKind {
        BackdropKind::PreparedImage
    }

    fn paint(
        &self,
        bounds: Bounds<Pixels>,
        corners: Corners<Pixels>,
        window: &mut Window,
        _: &mut App,
    ) -> gpui::Result<bool> {
        let image_bounds = self.image_bounds(bounds, window);
        if image_bounds.left() > bounds.left()
            || image_bounds.top() > bounds.top()
            || image_bounds.right() < bounds.right()
            || image_bounds.bottom() < bounds.bottom()
            || image_bounds.is_empty()
        {
            return Ok(false);
        }
        window.paint_image(
            bounds,
            image_bounds,
            corners,
            self.image.blurred(),
            0,
            false,
        )?;
        Ok(true)
    }
}
