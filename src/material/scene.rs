use gpui::{App, Bounds, Corners, Pixels, Window};

use super::{Backdrop, BackdropKind};

/// Real GPU scene sampling; the host does not provide a background image.
#[derive(Debug, Clone, Copy)]
pub struct SceneBackdrop {
    sigma: Pixels,
}

impl SceneBackdrop {
    /// Logical-pixel Gaussian sigma. Invalid values are rejected by the renderer
    /// and produce the surface's opaque fallback, never fake translucency.
    pub fn new(sigma: Pixels) -> Self {
        Self { sigma }
    }

    pub fn sigma(&self) -> Pixels {
        self.sigma
    }
}

impl Backdrop for SceneBackdrop {
    fn kind(&self) -> BackdropKind {
        BackdropKind::Scene
    }

    fn paint(
        &self,
        bounds: Bounds<Pixels>,
        corners: Corners<Pixels>,
        window: &mut Window,
        _: &mut App,
    ) -> gpui::Result<bool> {
        Ok(window.paint_backdrop_blur(bounds, corners, self.sigma))
    }
}
