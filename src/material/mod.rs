//! Frosted materials, not refractive or liquid glass.
//!
//! A backend must supply actual blurred pixels. Unsupported and inaccessible
//! surfaces are opaque; reducing alpha alone is never treated as blur support.

mod image;
mod scene;

pub use image::{BackdropImage, ImagePlane, PreparationError, PreparedImage};
pub use scene::SceneBackdrop;

use gpui::{App, Bounds, Corners, Hsla, Pixels, Window};

use crate::theme::Theme;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Material {
    #[default]
    Solid,
    Frosted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackdropKind {
    Unavailable,
    PreparedImage,
    /// Only backends that really sample the rendered scene may return this.
    Scene,
}

/// Renderer boundary. No preparation, convolution or readback belongs in paint.
pub trait Backdrop: 'static {
    fn kind(&self) -> BackdropKind;

    /// Return false if this region is not available; the surface will be opaque.
    fn paint(
        &self,
        bounds: Bounds<Pixels>,
        corners: Corners<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::Result<bool>;
}

#[derive(Debug, Clone, Copy)]
pub struct MaterialPlan {
    pub fill: Hsla,
    pub border: Hsla,
    pub highlight: Hsla,
    pub uses_backdrop: bool,
}

impl Material {
    pub fn resolve(self, theme: &Theme, backdrop: BackdropKind) -> MaterialPlan {
        let uses_backdrop = self == Self::Frosted
            && backdrop != BackdropKind::Unavailable
            && theme.frosted.is_valid()
            && !theme.preferences.reduced_transparency
            && !theme.preferences.increased_contrast;
        let alpha = if uses_backdrop {
            theme.frosted.tint_opacity
        } else {
            1.0
        };
        MaterialPlan {
            fill: theme.colors.surface.alpha(alpha),
            border: if theme.preferences.increased_contrast {
                theme.colors.secondary
            } else {
                theme.colors.border
            },
            highlight: theme.colors.highlight,
            uses_backdrop,
        }
    }
}
