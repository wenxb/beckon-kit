use crate::{Bounds, ContentMask, Corners, DrawOrder, Primitive, ScaledPixels};

/// A rounded region that filters the scene already drawn behind it.
/// No refraction, distortion, image asset or desktop capture is involved.
#[derive(Debug, Clone, Copy)]
pub struct BackdropBlur {
    /// Assigned by the scene as an ordering barrier.
    pub order: DrawOrder,
    /// Output region in physical pixels.
    pub bounds: Bounds<ScaledPixels>,
    /// Output clipping; the sampling halo may extend beyond this mask.
    pub content_mask: ContentMask<ScaledPixels>,
    /// Output corner radii in physical pixels.
    pub corner_radii: Corners<ScaledPixels>,
    /// Gaussian standard deviation in physical pixels.
    pub sigma: ScaledPixels,
    /// Mix between the original and filtered backdrop, including ancestor opacity.
    pub opacity: f32,
}

impl From<BackdropBlur> for Primitive {
    fn from(blur: BackdropBlur) -> Self {
        Self::BackdropBlur(blur)
    }
}
