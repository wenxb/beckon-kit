use gpui::App;
#[cfg(all(target_os = "linux", feature = "platform"))]
use gpui::{WindowBounds, WindowKind, WindowOptions};

#[cfg(all(target_os = "linux", feature = "platform"))]
use super::{FrameMode, TitlebarMode};
use super::{WindowConfig, WindowConfigError};

impl WindowConfig {
    /// Full-output overlay with compositor-enforced top layer and exclusive keyboard focus.
    /// Unsupported backends fail rather than opening an ordinary fullscreen window.
    pub fn overlay(self, cx: &App) -> Result<Self, WindowConfigError> {
        #[cfg(all(target_os = "linux", feature = "platform"))]
        if cx.compositor_name() == "Wayland" {
            use gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
            let display = cx
                .primary_display()
                .or_else(|| cx.displays().into_iter().next())
                .ok_or_else(|| WindowConfigError::new("overlay", "没有可用显示器"))?;
            return Ok(self
                .frame(FrameMode::None)
                .titlebar(TitlebarMode::None)
                .options(WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(display.bounds())),
                    display_id: Some(display.id()),
                    kind: WindowKind::LayerShell(LayerShellOptions {
                        namespace: "beckon-plugin-overlay".into(),
                        layer: Layer::Overlay,
                        anchor: Anchor::all(),
                        exclusive_zone: Some(gpui::px(-1.0)),
                        keyboard_interactivity: KeyboardInteractivity::Exclusive,
                        ..Default::default()
                    }),
                    is_resizable: false,
                    is_minimizable: false,
                    is_movable: false,
                    ..Default::default()
                }));
        }
        let _ = (self, cx);
        Err(WindowConfigError::new(
            "overlay",
            "此窗口后端未提供置顶且独占输入的覆盖层；不会退回普通全屏",
        ))
    }
}
