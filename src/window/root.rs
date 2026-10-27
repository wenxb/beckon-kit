use super::{
    ClientFrame, CloseRequest, FrameOwner, TitlebarMode, WindowConfigError, WindowFrameStatus,
    WindowRootConfig,
    close::CloseGate,
    frame::Frame,
    interaction::titlebar,
    platform::Platform,
    resolve::{WindowSnapshot, resolve},
};
use crate::{
    Root,
    theme::{Theme, ThemeScope},
};
use gpui::{
    App, AppContext, Context, Entity, EventEmitter, IntoElement, ParentElement, Render,
    Subscription, Window, WindowHandle, prelude::*, px, size,
};
use std::{cell::RefCell, rc::Rc, sync::Arc};

/// One native window, one shell owner. Use `mount` only inside `App::open_window`.
pub struct WindowRoot<V: Render + 'static> {
    content: Entity<V>,
    overlay: Option<gpui::AnyView>,
    titlebar_content: Option<gpui::AnyView>,
    config: WindowRootConfig,
    theme: Arc<Theme>,
    status: WindowFrameStatus,
    close: Rc<RefCell<CloseGate>>,
    clip: gpui::RoundedClip,
    pending_resize: Option<(gpui::Size<gpui::Pixels>, gpui::Size<gpui::Pixels>)>,
    _subscriptions: Vec<Subscription>,
}
impl<V: Render + 'static> EventEmitter<WindowFrameStatus> for WindowRoot<V> {}

impl<V: Render + 'static> WindowRoot<V> {
    pub fn mount(
        config: WindowRootConfig,
        window: &mut Window,
        cx: &mut App,
        build: impl FnOnce(&mut Window, &mut App) -> Entity<V>,
    ) -> gpui::Result<Entity<Self>> {
        if !window.claim_decoration_owner() {
            return Err(WindowConfigError::new(
                "WindowRoot",
                "一个原生窗口只能挂载一个装饰所有者；请在开窗构造闭包内挂载",
            )
            .into());
        }
        if config.frame != super::FrameMode::None
            && matches!(
                window.window_decorations(),
                gpui::Decorations::Client { .. }
            )
            && !window.is_rounded_clip_supported()
        {
            return Err(WindowConfigError::new(
                "frame=Client",
                "GPU 后端缺少完整圆角裁剪；无法装配客户端外框",
            )
            .into());
        }
        if config.titlebar == TitlebarMode::None
            && !window.set_native_caption_buttons_visible(false)
        {
            return Err(WindowConfigError::new("titlebar=None", "隐藏平台原生窗控失败").into());
        }
        crate::init(cx);
        let content = build(window, cx);
        Ok(cx.new(|cx| Self::new(config, content, window, cx)))
    }
    fn new(
        mut config: WindowRootConfig,
        content: Entity<V>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let close = Rc::new(RefCell::new(CloseGate::new(config.close.take())));
        let native_gate = close.clone();
        window.on_window_should_close(cx, move |window, cx| {
            native_gate.borrow_mut().request(window, cx)
        });
        let status = resolve(&config, &WindowSnapshot::read(window));
        let subscriptions = vec![
            cx.observe_window_bounds(window, Self::refresh),
            cx.observe_window_activation(window, Self::refresh),
            cx.observe_window_visibility(window, |root, _, window, cx| root.refresh(window, cx)),
            cx.observe_window_appearance(window, Self::refresh),
            cx.observe_window_display(window, Self::refresh),
        ];
        let clip = gpui::RoundedClip::new(status.geometry.visible, status.geometry.corner_radii);
        let theme = config.theme.clone().unwrap_or_else(|| Theme::global(cx));
        let mut root = Self {
            content,
            overlay: None,
            titlebar_content: None,
            config,
            theme,
            status,
            close,
            clip,
            pending_resize: None,
            _subscriptions: subscriptions,
        };
        root.apply_geometry(None, window, cx);
        root
    }
    pub fn content(&self) -> &Entity<V> {
        &self.content
    }
    /// Window-local overlay content; the caller owns focus and dismissal behavior.
    pub fn set_overlay(&mut self, overlay: Option<gpui::AnyView>, cx: &mut Context<Self>) {
        self.overlay = overlay;
        cx.notify();
    }
    /// Optional integrated-titlebar content, beside the drag area and platform controls.
    pub fn set_titlebar_content(&mut self, content: Option<gpui::AnyView>, cx: &mut Context<Self>) {
        self.titlebar_content = content;
        cx.notify();
    }
    pub fn frame_status(&self) -> &WindowFrameStatus {
        &self.status
    }
    pub fn client_style(&self) -> &ClientFrame {
        &self.config.client
    }
    pub fn title(&self) -> &gpui::SharedString {
        &self.config.title
    }
    pub fn titlebar_mode(&self) -> TitlebarMode {
        self.config.titlebar
    }
    pub fn set_title(
        &mut self,
        title: impl Into<gpui::SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let title = title.into();
        if self.config.title != title {
            window.set_window_title(&title);
            self.config.title = title;
            cx.notify();
        }
    }
    /// Pin this window to a local theme; other windows keep following the app theme.
    pub fn set_theme(&mut self, theme: impl Into<Arc<Theme>>, cx: &mut Context<Self>) {
        self.config.theme = Some(theme.into());
        cx.refresh_windows();
        cx.notify();
    }
    pub fn set_client_style(
        &mut self,
        update: impl FnOnce(ClientFrame) -> ClientFrame,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), WindowConfigError> {
        if matches!(
            self.config.frame,
            super::FrameMode::Native | super::FrameMode::None
        ) {
            return Err(WindowConfigError::new(
                "client_style + Native/None",
                "请选择 Adaptive / Client",
            ));
        }
        let next = update(self.config.client.clone());
        next.validate()?;
        self.config.client = next;
        self.refresh(window, cx);
        cx.notify();
        Ok(())
    }
    pub fn request_close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.close.borrow_mut().request(window, cx) {
            window.remove_window();
        }
    }
    pub fn resume_close(
        &mut self,
        request: CloseRequest,
        allow: bool,
        window: &mut Window,
        _: &mut Context<Self>,
    ) {
        if self.close.borrow_mut().resume(request, allow) {
            window.remove_window();
        }
    }
    fn refresh(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let status = resolve(&self.config, &WindowSnapshot::read(window));
        if status == self.status {
            return;
        }
        let previous = std::mem::replace(&mut self.status, status);
        self.apply_geometry(Some(&previous), window, cx);
        if self.status.geometry != previous.geometry {
            window.refresh();
        }
        cx.emit(self.status.clone());
        cx.notify();
    }
    fn apply_geometry(
        &mut self,
        previous: Option<&WindowFrameStatus>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let inset = self.status.geometry.normal_inset;
        let old = window.client_inset().unwrap_or(px(0.0));
        if window.supports_input_region()
            && previous
                .is_none_or(|p| p.geometry != self.status.geometry || p.owner != self.status.owner)
        {
            let g = &self.status.geometry;
            if self.status.owner == FrameOwner::Client {
                window.set_input_region(Some(&[g
                    .visible
                    .dilate(g.resize_width)
                    .intersect(&g.surface)]));
            } else {
                window.set_input_region(None);
            }
        }
        if inset != old
            || previous.is_none_or(|p| {
                p.geometry.insets != self.status.geometry.insets
                    || p.scale_factor != self.status.scale_factor
            })
        {
            window.set_client_inset(inset);
        }
        let initial = previous.is_none();
        let previous_inset = if initial {
            self.config.initial_inset.take().unwrap_or(inset)
        } else {
            old
        };
        if inset != previous_inset && !self.status.fullscreen && !self.status.maximized {
            // Keep the first visible size across coalesced style/negotiation changes.
            if self.pending_resize.is_none() {
                let surface = window.viewport_size();
                let before = previous.map_or_else(
                    || surface - size(previous_inset * 2.0, previous_inset * 2.0),
                    |status| status.geometry.visible.size,
                );
                self.pending_resize = Some((surface, before));
                let root = cx.weak_entity();
                window.defer(cx, move |window, cx| {
                    let _ = root.update(cx, |root, _| {
                        let Some((surface, before)) = root.pending_resize.take() else {
                            return;
                        };
                        if window.viewport_size() == surface
                            && !root.status.fullscreen
                            && !root.status.maximized
                        {
                            let g = &root.status.geometry;
                            window.resize(
                                before
                                    + size(
                                        g.insets.left + g.insets.right,
                                        g.insets.top + g.insets.bottom,
                                    ),
                            );
                        }
                    });
                });
            }
        }
        if (inset != old || previous.is_none_or(|p| p.scale_factor != self.status.scale_factor))
            && let Some(minimum) = self.config.min_size
        {
            window.set_window_min_size(minimum + size(inset * 2.0, inset * 2.0));
        }
    }
}

impl<V: Render + 'static> Render for WindowRoot<V> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self
            .config
            .theme
            .clone()
            .unwrap_or_else(|| Theme::global(cx));
        if !Arc::ptr_eq(&theme, &self.theme) {
            self.config.client.retheme(&self.theme, &theme);
            self.theme = theme;
            self.refresh(window, cx);
        }
        let client = self.status.owner == FrameOwner::Client;
        let linux = Platform::current() == Platform::Linux;
        let integrated = self.config.titlebar == TitlebarMode::Integrated
            && (!linux || client)
            && !self.status.fullscreen;
        let mut body = gpui::div().relative().size_full().flex().flex_col();
        if self.config.frame != super::FrameMode::None {
            body = body.bg(self.theme.colors.canvas);
        }
        if integrated {
            body = body.child(titlebar(
                &self.config,
                &self.status,
                self.close.clone(),
                self.titlebar_content.clone(),
            ));
        }
        body = body.child(gpui::div().flex_1().min_h_0().child(self.content.clone()));
        if let Some(overlay) = &self.overlay {
            body = body.child(gpui::div().absolute().inset_0().child(overlay.clone()));
        }
        let body = Root::new("beckon-window-root")
            .label(self.config.title.clone())
            .child(body);
        let body = gpui::div()
            .size_full()
            .key_context("BeckonWindow")
            .on_action(cx.listener(|root, _: &super::CloseWindow, window, cx| {
                root.request_close(window, cx)
            }))
            .child(body);
        ThemeScope::new(
            self.theme.clone(),
            Frame::new(
                self.status.geometry.clone(),
                self.config.client.clone(),
                client,
                {
                    let mut clip = self.clip.clone();
                    clip.bounds = self.status.geometry.visible;
                    clip.radii = self.status.geometry.corner_radii;
                    clip
                },
                body,
            ),
        )
    }
}

pub trait WindowRootHandleExt<V: Render + 'static> {
    fn content<C: AppContext>(&self, cx: &C) -> gpui::Result<Entity<V>>;
    fn update_content<C: AppContext, R>(
        &self,
        cx: &mut C,
        update: impl FnOnce(&mut V, &mut Window, &mut Context<V>) -> R,
    ) -> gpui::Result<R>;
}
impl<V: Render + 'static> WindowRootHandleExt<V> for WindowHandle<WindowRoot<V>> {
    fn content<C: AppContext>(&self, cx: &C) -> gpui::Result<Entity<V>> {
        self.read_with(cx, |root, _| root.content.clone())
    }
    fn update_content<C: AppContext, R>(
        &self,
        cx: &mut C,
        update: impl FnOnce(&mut V, &mut Window, &mut Context<V>) -> R,
    ) -> gpui::Result<R> {
        self.update(cx, |root, window, cx| {
            root.content
                .update(cx, |content, cx| update(content, window, cx))
        })
    }
}
