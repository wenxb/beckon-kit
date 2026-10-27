use super::Window;
use crate::{BackdropBlur, Bounds, Corners, Pixels};

impl Window {
    /// Whether this window has GPU support for real, in-scene backdrop blur.
    pub fn is_backdrop_blur_supported(&self) -> bool {
        self.platform_window.is_backdrop_blur_supported()
    }

    /// Filter previously painted scene content, keeping subsequently painted
    /// foreground sharp. `sigma` is a logical-pixel Gaussian standard deviation.
    /// Unsupported, invalid or clipped-out regions return false without painting.
    /// This must be called during paint, before the surface's tint and foreground.
    pub fn paint_backdrop_blur(
        &mut self,
        bounds: Bounds<Pixels>,
        corner_radii: Corners<Pixels>,
        sigma: Pixels,
    ) -> bool {
        self.invalidator.debug_assert_paint();
        let sigma_value = f32::from(sigma);
        if !self.is_backdrop_blur_supported()
            || !sigma_value.is_finite()
            || !(0.0..=64.0).contains(&sigma_value)
            || sigma_value == 0.0
        {
            return false;
        }
        let content_mask = self.snapped_content_mask();
        let bounds = self.cover_bounds(bounds);
        if bounds.intersect(&content_mask.bounds).is_empty() || self.element_opacity() <= 0.0 {
            return false;
        }
        self.next_frame.scene.insert_primitive(BackdropBlur {
            order: 0,
            bounds,
            content_mask,
            corner_radii: corner_radii
                .scale(self.scale_factor())
                .clamp_radii_for_quad_size(bounds.size),
            sigma: sigma.scale(self.scale_factor()),
            opacity: self.element_opacity(),
        });
        true
    }

    /// Blur primitives from the completed frame, without rendering or reading images.
    #[cfg(any(test, feature = "test-support"))]
    pub fn painted_backdrop_blurs(&self) -> Vec<BackdropBlur> {
        self.rendered_frame.scene.backdrop_blurs.clone()
    }
}
