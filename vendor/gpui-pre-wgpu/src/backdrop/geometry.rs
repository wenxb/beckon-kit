use bytemuck::{Pod, Zeroable};
use gpui::{BackdropBlur, Bounds, DevicePixels, ScaledPixels, Size, point, size};

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub(super) struct Params {
    pub viewport: [f32; 2],
    pub origin: [f32; 2],
    pub extent: [f32; 2],
    pub samples: [f32; 2],
    pub bounds: [f32; 4],
    pub clip: [f32; 4],
    pub corners: [f32; 4],
    pub effect: [f32; 4],
}

pub(super) struct Geometry {
    pub params: Params,
    pub origin: [u32; 2],
    pub extent: [u32; 2],
    pub samples: [u32; 2],
    pub scissor: [u32; 4],
}

impl Geometry {
    pub fn new(blur: &BackdropBlur, viewport: Size<DevicePixels>) -> Option<Self> {
        let sigma = blur.sigma.0;
        let bounds = blur.bounds;
        let values = [
            sigma,
            blur.opacity,
            bounds.origin.x.0,
            bounds.origin.y.0,
            bounds.size.width.0,
            bounds.size.height.0,
        ];
        if values.iter().any(|value| !value.is_finite()) || sigma <= 0.0 || blur.opacity <= 0.0 {
            return None;
        }
        let viewport_bounds = Bounds::new(
            point(ScaledPixels(0.0), ScaledPixels(0.0)),
            size(
                ScaledPixels(viewport.width.0 as f32),
                ScaledPixels(viewport.height.0 as f32),
            ),
        );
        let visible = bounds
            .intersect(&blur.content_mask.bounds)
            .intersect(&viewport_bounds);
        if visible.is_empty() {
            return None;
        }
        let halo = (3.0 * sigma).ceil() + 1.0;
        let left = (visible.left().0 - halo).floor().max(0.0) as u32;
        let top = (visible.top().0 - halo).floor().max(0.0) as u32;
        let right = (visible.right().0 + halo)
            .ceil()
            .min(viewport.width.0 as f32) as u32;
        let bottom = (visible.bottom().0 + halo)
            .ceil()
            .min(viewport.height.0 as f32) as u32;
        let extent = [right - left, bottom - top];
        // Keep the separable Gaussian bounded while prefiltering high-frequency content.
        let step = (sigma / 6.0).ceil().max(1.0);
        let samples = extent.map(|length| ((length as f32 / step).ceil() as u32).max(1));
        let x = visible.left().0.floor().max(0.0) as u32;
        let y = visible.top().0.floor().max(0.0) as u32;
        let scissor = [
            x,
            y,
            visible.right().0.ceil() as u32 - x,
            visible.bottom().0.ceil() as u32 - y,
        ];
        let corners = blur.corner_radii.clamp_radii_for_quad_size(bounds.size);
        Some(Self {
            params: Params {
                viewport: [viewport.width.0 as f32, viewport.height.0 as f32],
                origin: [left as f32, top as f32],
                extent: extent.map(|length| length as f32),
                samples: samples.map(|length| length as f32),
                bounds: [
                    bounds.left().0,
                    bounds.top().0,
                    bounds.size.width.0,
                    bounds.size.height.0,
                ],
                clip: [
                    visible.left().0,
                    visible.top().0,
                    visible.right().0,
                    visible.bottom().0,
                ],
                corners: [
                    corners.top_left.0,
                    corners.top_right.0,
                    corners.bottom_right.0,
                    corners.bottom_left.0,
                ],
                effect: [sigma, blur.opacity.clamp(0.0, 1.0), 0.0, 0.0],
            },
            origin: [left, top],
            extent,
            samples,
            scissor,
        })
    }
}
