use super::{FrameGeometry, FrameMode, TitlebarMode, WindowRootConfig, platform::Platform};
use gpui::{Decorations, Pixels, Size, Tiling, Window, WindowControls};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameOwner {
    System,
    Client,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameState {
    Requested,
    Negotiating,
    ClientApplied,
    SystemManaged,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowFrameStatus {
    pub requested: FrameMode,
    pub owner: FrameOwner,
    pub state: FrameState,
    pub geometry: FrameGeometry,
    pub controls: WindowControls,
    pub active: bool,
    pub visible: bool,
    pub maximized: bool,
    pub fullscreen: bool,
    pub scale_factor: f32,
    pub reason: Option<String>,
}

pub(crate) struct WindowSnapshot {
    pub platform: Platform,
    pub decorations: Decorations,
    pub negotiated: bool,
    pub surface: Size<Pixels>,
    pub controls: WindowControls,
    pub active: bool,
    pub visible: bool,
    pub maximized: bool,
    pub fullscreen: bool,
    pub scale_factor: f32,
    pub rounded_clip: bool,
}
impl WindowSnapshot {
    pub fn read(window: &Window) -> Self {
        Self {
            platform: Platform::current(),
            decorations: window.window_decorations(),
            negotiated: window.window_decorations_negotiated(),
            surface: window.viewport_size(),
            controls: window.window_controls(),
            active: window.is_window_active(),
            visible: window.is_visible(),
            maximized: window.is_maximized(),
            fullscreen: window.is_fullscreen(),
            scale_factor: window.scale_factor(),
            rounded_clip: window.is_rounded_clip_supported(),
        }
    }
}

pub(crate) fn resolve(config: &WindowRootConfig, snapshot: &WindowSnapshot) -> WindowFrameStatus {
    let linux = snapshot.platform == Platform::Linux;
    let frameless = config.frame == FrameMode::None;
    let client = !frameless && linux && matches!(snapshot.decorations, Decorations::Client { .. });
    let wants_client = !frameless
        && linux
        && config.frame != FrameMode::Native
        && config.titlebar != TitlebarMode::System;
    let unsupported = (client && !snapshot.rounded_clip)
        || (config.frame == FrameMode::Client && !client)
        || (config.frame == FrameMode::Native && client)
        || (config.titlebar == TitlebarMode::System && client);
    let reason = if frameless {
        None
    } else if !snapshot.negotiated {
        Some("等待合成器确认装饰模式".into())
    } else if client && !snapshot.rounded_clip {
        Some("GPU 后端不支持完整圆角裁剪；请求未满足".into())
    } else if client != wants_client && linux {
        Some(format!(
            "平台采用了{}装饰{}",
            if client { "客户端" } else { "系统" },
            if unsupported {
                "；显式请求未满足"
            } else {
                "；Adaptive 已适应"
            }
        ))
    } else {
        None
    };
    let owner = if client {
        FrameOwner::Client
    } else {
        FrameOwner::System
    };
    let tiling = match snapshot.decorations {
        Decorations::Client { tiling } => tiling,
        _ => Tiling::default(),
    };
    let tiling = if linux && snapshot.maximized {
        Tiling::tiled()
    } else {
        tiling
    };
    let integrated = config.titlebar == TitlebarMode::Integrated && (!linux || client);
    WindowFrameStatus {
        requested: config.frame,
        owner,
        state: if !frameless && !snapshot.negotiated {
            FrameState::Negotiating
        } else if unsupported {
            FrameState::Unsupported
        } else if client {
            FrameState::ClientApplied
        } else {
            FrameState::SystemManaged
        },
        geometry: FrameGeometry::new(
            snapshot.surface,
            client,
            &config.client,
            config.resizable,
            tiling,
            snapshot.fullscreen,
            integrated,
        ),
        controls: snapshot.controls,
        active: snapshot.active,
        visible: snapshot.visible,
        maximized: snapshot.maximized,
        fullscreen: snapshot.fullscreen,
        scale_factor: snapshot.scale_factor,
        reason,
    }
}
