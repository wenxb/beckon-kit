use std::{fmt, sync::Arc};

use gpui::{
    App, Bounds, BoxShadow, Corners, Entity, Hsla, Pixels, Render, SharedString, Size,
    TitlebarOptions, Window, WindowBackgroundAppearance, WindowBounds, WindowDecorations,
    WindowHandle, WindowOptions, point, px, size,
};

use super::{WindowRoot, close::CloseHandler, geometry::normal_inset, platform::Platform};
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FrameMode {
    #[default]
    Adaptive,
    Native,
    Client,
    /// Content owns the whole native surface; no kit background, frame or titlebar.
    None,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TitlebarMode {
    System,
    #[default]
    Integrated,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientFrame {
    pub corner_radii: Corners<Pixels>,
    pub border_width: Pixels,
    pub border_color: Hsla,
    pub shadows: Vec<BoxShadow>,
    overrides: FrameOverrides,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct FrameOverrides {
    corners: bool,
    border: bool,
    shadows: bool,
}

impl ClientFrame {
    pub fn radius(mut self, radius: Pixels) -> Self {
        self.corner_radii = Corners::all(radius);
        self.overrides.corners = true;
        self
    }
    pub fn corners(mut self, radii: Corners<Pixels>) -> Self {
        self.corner_radii = radii;
        self.overrides.corners = true;
        self
    }
    pub fn border(mut self, width: Pixels, color: Hsla) -> Self {
        self.border_width = width;
        self.border_color = color;
        self.overrides.border = true;
        self
    }
    pub fn shadows(mut self, shadows: Vec<BoxShadow>) -> Self {
        self.shadows = shadows;
        self.overrides.shadows = true;
        self
    }
    pub(crate) fn themed(theme: &Theme) -> Self {
        Self {
            corner_radii: Corners::all(px(theme.metrics.surface_radius)),
            border_width: px(1.0),
            border_color: theme.colors.window_border,
            shadows: vec![BoxShadow {
                color: theme.colors.shadow,
                offset: point(px(0.0), px(4.0)),
                blur_radius: px(8.0),
                spread_radius: px(0.0),
                inset: false,
            }],
            overrides: FrameOverrides::default(),
        }
    }
    /// Keep explicit customizations while updating values inherited from the old theme.
    pub(crate) fn retheme(&mut self, before: &Theme, after: &Theme) {
        let old = Self::themed(before);
        let new = Self::themed(after);
        self.overrides.corners |= self.corner_radii != old.corner_radii;
        self.overrides.border |= self.border_color != old.border_color;
        self.overrides.shadows |= self.shadows != old.shadows;
        if !self.overrides.corners {
            self.corner_radii = new.corner_radii;
        }
        if !self.overrides.border {
            self.border_color = new.border_color;
        }
        if !self.overrides.shadows {
            self.shadows = new.shadows;
        }
    }
    pub(crate) fn validate(&self) -> Result<(), WindowConfigError> {
        for (name, value) in [
            ("top_left", self.corner_radii.top_left),
            ("top_right", self.corner_radii.top_right),
            ("bottom_left", self.corner_radii.bottom_left),
            ("bottom_right", self.corner_radii.bottom_right),
            ("border_width", self.border_width),
        ] {
            if !f32::from(value).is_finite() || value < px(0.0) {
                return Err(WindowConfigError::new(
                    name,
                    format!("请求 {value:?}，必须是有限非负像素"),
                ));
            }
        }
        let valid_color = |c: Hsla| {
            [c.h, c.s, c.l, c.a]
                .iter()
                .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
        };
        if !valid_color(self.border_color) {
            return Err(WindowConfigError::new(
                "border_color",
                "HSLA 必须在 0..=1 内",
            ));
        }
        if self.shadows.len() > 8 {
            return Err(WindowConfigError::new("shadows", "最多 8 层投影"));
        }
        for shadow in &self.shadows {
            if !valid_color(shadow.color)
                || [
                    shadow.offset.x,
                    shadow.offset.y,
                    shadow.blur_radius,
                    shadow.spread_radius,
                ]
                .iter()
                .any(|p| !f32::from(*p).is_finite())
                || shadow.blur_radius < px(0.0)
                || shadow.inset
            {
                return Err(WindowConfigError::new(
                    "shadows",
                    "颜色、偏移及半径必须有效，blur_radius 不得为负；外框不接受 inset 投影",
                ));
            }
        }
        if normal_inset(self, true) > px(256.0) {
            return Err(WindowConfigError::new(
                "shadows",
                "投影外扩超过 256px；请减小 blur / spread / offset",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowConfigError {
    pub property: &'static str,
    pub reason: String,
}
impl WindowConfigError {
    pub(crate) fn new(property: &'static str, reason: impl Into<String>) -> Self {
        Self {
            property,
            reason: reason.into(),
        }
    }
}
impl fmt::Display for WindowConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "窗口配置 {}：{}", self.property, self.reason)
    }
}
impl std::error::Error for WindowConfigError {}

/// Structural options are consumed at creation; only title, theme and client style are mutable.
pub struct WindowConfig {
    title: SharedString,
    frame: FrameMode,
    titlebar: TitlebarMode,
    client: Option<ClientFrame>,
    theme: Option<Arc<Theme>>,
    size: Option<Size<Pixels>>,
    min_size: Option<Size<Pixels>>,
    options: WindowOptions,
    close: Option<CloseHandler>,
}

pub struct PreparedWindow {
    pub options: WindowOptions,
    pub root: WindowRootConfig,
}

pub struct WindowRootConfig {
    pub(crate) title: SharedString,
    pub(crate) frame: FrameMode,
    pub(crate) titlebar: TitlebarMode,
    pub(crate) client: ClientFrame,
    pub(crate) theme: Option<Arc<Theme>>,
    pub(crate) min_size: Option<Size<Pixels>>,
    pub(crate) initial_inset: Option<Pixels>,
    pub(crate) resizable: bool,
    pub(crate) minimizable: bool,
    pub(crate) movable: bool,
    pub(crate) close: Option<CloseHandler>,
}

impl WindowConfig {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            frame: FrameMode::Adaptive,
            titlebar: TitlebarMode::Integrated,
            client: None,
            theme: None,
            size: None,
            min_size: None,
            options: WindowOptions::default(),
            close: None,
        }
    }
    pub fn size(mut self, value: Size<Pixels>) -> Self {
        self.size = Some(value);
        self
    }
    pub fn min_size(mut self, value: Size<Pixels>) -> Self {
        self.min_size = Some(value);
        self
    }
    pub fn frame(mut self, mode: FrameMode) -> Self {
        self.frame = mode;
        self
    }
    pub fn titlebar(mut self, mode: TitlebarMode) -> Self {
        self.titlebar = mode;
        self
    }
    /// Override the app theme for this window only; omit to follow `Theme::install`.
    pub fn theme(mut self, theme: impl Into<Arc<Theme>>) -> Self {
        let theme = theme.into();
        if let Some(client) = &mut self.client {
            client.retheme(self.theme.as_deref().unwrap_or(&Theme::default()), &theme);
        }
        self.theme = Some(theme);
        self
    }
    pub fn client_style(mut self, update: impl FnOnce(ClientFrame) -> ClientFrame) -> Self {
        let frame = self.client.take().unwrap_or_else(|| {
            ClientFrame::themed(self.theme.as_deref().unwrap_or(&Theme::default()))
        });
        self.client = Some(update(frame));
        self
    }
    /// GPUI escape hatch. Managed fields are rejected at prepare, not silently overwritten.
    pub fn options(mut self, options: WindowOptions) -> Self {
        self.options = options;
        self
    }
    pub fn on_close(
        mut self,
        handler: impl Fn(super::CloseRequest, &mut Window, &mut App) -> super::CloseDecision + 'static,
    ) -> Self {
        self.close = Some(std::rc::Rc::new(handler));
        self
    }

    pub fn prepare(self, cx: &App) -> Result<PreparedWindow, WindowConfigError> {
        let platform = Platform::current();
        if self.frame == FrameMode::None
            && (self.titlebar != TitlebarMode::None || self.client.is_some())
        {
            return Err(WindowConfigError::new(
                "frame=None",
                "无外框窗口必须使用 titlebar=None，且不能指定客户端外框样式",
            ));
        }
        if self.frame == FrameMode::Native && self.client.is_some() {
            return Err(WindowConfigError::new(
                "frame=Native + client_style",
                "原生外框不接收客户端样式；使用 Adaptive 或 Client",
            ));
        }
        if self.frame == FrameMode::Client && self.titlebar == TitlebarMode::System {
            return Err(WindowConfigError::new(
                "frame=Client + titlebar=System",
                "请选择 Integrated 或 None",
            ));
        }
        if self.frame == FrameMode::Native && self.titlebar == TitlebarMode::None {
            return Err(WindowConfigError::new(
                "frame=Native + titlebar=None",
                "系统标题区不能独立隐藏全部按钮；请选择 Integrated 或 Linux Client",
            ));
        }
        if platform != Platform::Linux && self.frame == FrameMode::Client {
            return Err(WindowConfigError::new(
                "frame=Client",
                "此平台尚无完整 GPU 客户端外框；请使用 Adaptive / Native",
            ));
        }
        if platform == Platform::Other && self.titlebar == TitlebarMode::None {
            return Err(WindowConfigError::new(
                "titlebar=None",
                "此平台尚无无标题区窗口支持",
            ));
        }
        let mut options = self.options;
        let titlebar_conflict = options.titlebar.as_ref().is_none_or(|t| {
            t.title.is_some() || t.appears_transparent || t.traffic_light_position.is_some()
        });
        if titlebar_conflict
            || options.window_decorations.is_some()
            || options.window_min_size.is_some()
            || options.app_owns_titlebar_drag
            || options.window_background != WindowBackgroundAppearance::Opaque
        {
            return Err(WindowConfigError::new(
                "options",
                "titlebar / decorations / min_size / background / app_owns_titlebar_drag 由外壳管理；使用对应 builder",
            ));
        }
        if self.size.is_some() && options.window_bounds.is_some() {
            return Err(WindowConfigError::new(
                "size + options.window_bounds",
                "客户区尺寸与原生 bounds 不能混用；仅选择一种",
            ));
        }
        let theme = self.theme.clone().unwrap_or_else(|| Theme::global(cx));
        let client = match self.client {
            Some(mut client) => {
                if self.theme.is_none() {
                    client.retheme(&Theme::default(), &theme);
                }
                client
            }
            None => ClientFrame::themed(&theme),
        };
        client.validate()?;
        for (name, value) in [("size", self.size), ("min_size", self.min_size)] {
            if let Some(value) = value {
                validate_size(name, value)?;
            }
        }
        if let (Some(s), Some(m)) = (self.size, self.min_size)
            && (s.width < m.width || s.height < m.height)
        {
            return Err(WindowConfigError::new("size", "不得小于 min_size"));
        }
        let expect_client = platform == Platform::Linux
            && !matches!(self.frame, FrameMode::Native | FrameMode::None)
            && self.titlebar != TitlebarMode::System;
        let inset = if expect_client {
            normal_inset(&client, options.is_resizable)
        } else {
            px(0.0)
        };
        let initial_inset = options.window_bounds.is_none().then_some(inset);
        if options.window_bounds.is_none() {
            let visible = self.size.unwrap_or_else(|| {
                let minimum = self.min_size.unwrap_or_default();
                size(px(980.0).max(minimum.width), px(680.0).max(minimum.height))
            });
            options.window_bounds = Some(WindowBounds::Windowed(Bounds::centered(
                options.display_id,
                visible + size(inset * 2.0, inset * 2.0),
                cx,
            )));
        }
        options.window_min_size = self.min_size.map(|s| {
            if platform == Platform::Linux {
                s + size(inset * 2.0, inset * 2.0)
            } else {
                s
            }
        });
        options.titlebar = Some(TitlebarOptions {
            title: Some(self.title.clone()),
            appears_transparent: self.titlebar != TitlebarMode::System,
            ..Default::default()
        });
        options.app_owns_titlebar_drag =
            platform == Platform::Mac && self.titlebar != TitlebarMode::System;
        if self.titlebar == TitlebarMode::None {
            options.window_background = WindowBackgroundAppearance::Transparent;
        }
        if platform == Platform::Linux {
            options.window_background = WindowBackgroundAppearance::Transparent;
            options.window_decorations = Some(if expect_client || self.frame == FrameMode::None {
                WindowDecorations::Client
            } else {
                WindowDecorations::Server
            });
        }
        let root = WindowRootConfig {
            title: self.title,
            frame: self.frame,
            titlebar: self.titlebar,
            client,
            theme: self.theme,
            min_size: self.min_size,
            initial_inset,
            resizable: options.is_resizable,
            minimizable: options.is_minimizable,
            movable: options.is_movable,
            close: self.close,
        };
        Ok(PreparedWindow { options, root })
    }

    pub fn open<V: Render + 'static>(
        self,
        cx: &mut App,
        content: impl FnOnce(&mut Window, &mut App) -> Entity<V>,
    ) -> gpui::Result<WindowHandle<WindowRoot<V>>> {
        crate::init(cx);
        let PreparedWindow { options, root } = self.prepare(cx)?;
        cx.try_open_window(options, move |window, cx| {
            // Reserve decoration ownership before constructing arbitrary application content.
            WindowRoot::<V>::mount(root, window, cx, content)
        })
    }
}

fn validate_size(property: &'static str, value: Size<Pixels>) -> Result<(), WindowConfigError> {
    if [value.width, value.height]
        .iter()
        .any(|p| !f32::from(*p).is_finite() || *p <= px(0.0) || *p > px(16384.0))
    {
        return Err(WindowConfigError::new(
            property,
            "宽高必须在 (0, 16384] 逻辑像素内",
        ));
    }
    Ok(())
}
