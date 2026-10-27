use super::ClientFrame;
use gpui::{Bounds, Corners, Edges, Pixels, Size, Tiling, point, px, size};

/// All coordinates are logical pixels in the native content surface.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameGeometry {
    pub surface: Bounds<Pixels>,
    pub visible: Bounds<Pixels>,
    pub content: Bounds<Pixels>,
    pub insets: Edges<Pixels>,
    pub corner_radii: Corners<Pixels>,
    pub border_widths: Edges<Pixels>,
    /// Normal-state scalar inset stays stable while maximized or tiled.
    pub normal_inset: Pixels,
    pub resize_width: Pixels,
}

pub(crate) fn normal_inset(style: &ClientFrame, resizable: bool) -> Pixels {
    style
        .shadows
        .iter()
        .fold(if resizable { px(6.0) } else { px(0.0) }, |inset, s| {
            inset
                .max(s.blur_radius * 3.0 + s.spread_radius + s.offset.x.abs().max(s.offset.y.abs()))
        })
        .max(px(0.0))
}

impl FrameGeometry {
    pub(crate) fn new(
        surface: Size<Pixels>,
        client: bool,
        style: &ClientFrame,
        resizable: bool,
        tiling: Tiling,
        fullscreen: bool,
        titlebar: bool,
    ) -> Self {
        let normal = if client {
            normal_inset(style, resizable)
        } else {
            px(0.0)
        };
        let tiled = if fullscreen { Tiling::tiled() } else { tiling };
        let edge = |blocked| if blocked { px(0.0) } else { normal };
        let insets = Edges {
            top: edge(tiled.top),
            right: edge(tiled.right),
            bottom: edge(tiled.bottom),
            left: edge(tiled.left),
        };
        let visible = Bounds::new(
            point(insets.left, insets.top),
            size(
                (surface.width - insets.left - insets.right).max(px(0.0)),
                (surface.height - insets.top - insets.bottom).max(px(0.0)),
            ),
        );
        let mut radii = if client {
            style.corner_radii
        } else {
            Corners::all(px(0.0))
        };
        if tiled.top || tiled.left {
            radii.top_left = px(0.0);
        }
        if tiled.top || tiled.right {
            radii.top_right = px(0.0);
        }
        if tiled.bottom || tiled.left {
            radii.bottom_left = px(0.0);
        }
        if tiled.bottom || tiled.right {
            radii.bottom_right = px(0.0);
        }
        radii = radii.clamp_radii_for_quad_size(visible.size);
        let title_height = if titlebar && !fullscreen {
            px(38.0).min(visible.size.height)
        } else {
            px(0.0)
        };
        let content = Bounds::new(
            visible.origin + point(px(0.0), title_height),
            size(visible.size.width, visible.size.height - title_height),
        );
        Self {
            surface: Bounds::new(point(px(0.0), px(0.0)), surface),
            visible,
            content,
            insets,
            corner_radii: radii,
            border_widths: Edges {
                top: if client && !tiled.top {
                    style.border_width
                } else {
                    px(0.0)
                },
                right: if client && !tiled.right {
                    style.border_width
                } else {
                    px(0.0)
                },
                bottom: if client && !tiled.bottom {
                    style.border_width
                } else {
                    px(0.0)
                },
                left: if client && !tiled.left {
                    style.border_width
                } else {
                    px(0.0)
                },
            },
            normal_inset: normal,
            resize_width: if client && resizable && !fullscreen {
                px(6.0)
            } else {
                px(0.0)
            },
        }
    }
}
