use gpui::Refineable;
use gpui::{
    AnyElement, App, ClickEvent, Context, ElementId, FocusHandle, IntoElement, ParentElement,
    Render, RenderOnce, Role, SharedString, StyleRefinement, Styled, Window, div, prelude::*, px,
};
use std::{rc::Rc, sync::Arc};

use crate::{
    element::Semantic,
    interaction::{ClickHandler, Pressable},
    theme::Theme,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    #[default]
    Secondary,
    /// Transparent fill with a hairline edge.
    Outline,
    /// Info-tinted fill for low-emphasis actions.
    Soft,
    Quiet,
    /// Text-only action with no hover surface.
    Link,
    Danger,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ControlSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl ControlSize {
    pub(crate) fn height(self, theme: &Theme) -> f32 {
        let adjustment = match self {
            Self::Small => -6.0,
            Self::Medium => 0.0,
            Self::Large => 8.0,
        };
        (theme.metrics.control_height + adjustment).max(theme.typography.body * 1.5 + 8.0)
    }
}

/// A controlled action. Press feedback and cancellation use GPUI's input lifecycle.
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    variant: ButtonVariant,
    size: ControlSize,
    disabled: bool,
    busy: bool,
    leading: Option<AnyElement>,
    content: Option<AnyElement>,
    icon_only: bool,
    tooltip: bool,
    frosted: bool,
    toggled: Option<bool>,
    window_control: Option<gpui::WindowControlArea>,
    focus: Option<FocusHandle>,
    callback: Option<ClickHandler>,
    style: StyleRefinement,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: ButtonVariant::default(),
            size: ControlSize::default(),
            disabled: false,
            busy: false,
            leading: None,
            content: None,
            icon_only: false,
            tooltip: true,
            frosted: false,
            toggled: None,
            window_control: None,
            focus: None,
            callback: None,
            style: StyleRefinement::default(),
        }
    }

    /// A square icon-only action; the required label names it for assistive technology and hover.
    pub fn icon(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        icon: impl IntoElement,
    ) -> Self {
        Self {
            leading: Some(icon.into_any_element()),
            icon_only: true,
            ..Self::new(id, label)
        }
    }

    /// Disable the hover hint without removing the accessible label.
    pub fn tooltip(mut self, enabled: bool) -> Self {
        self.tooltip = enabled;
        self
    }

    /// Enable the shared theme's frosted material, with an opaque accessibility fallback.
    pub fn frosted(mut self) -> Self {
        self.frosted = true;
        self
    }

    /// Controlled toggle-button state; the host still owns the value and appearance.
    pub fn toggled(mut self, toggled: bool) -> Self {
        self.toggled = Some(toggled);
        self
    }

    pub(crate) fn window_control_area(mut self, area: gpui::WindowControlArea) -> Self {
        self.window_control = Some(area);
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    pub fn leading(mut self, leading: impl IntoElement) -> Self {
        self.leading = Some(leading.into_any_element());
        self
    }

    /// Replace the visible label while retaining its accessible name and button behavior.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    pub fn focus_handle(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }

    pub fn on_click(
        mut self,
        callback: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callback = Some(Rc::new(callback));
        self
    }
}

impl Styled for Button {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let colors = &theme.colors;
        let enabled = !self.disabled && !self.busy;
        let transparent = gpui::transparent_black();
        let (mut bg, mut foreground, hover, pressed, mut border) = match self.variant {
            ButtonVariant::Primary => (
                colors.primary,
                colors.on_primary,
                colors.primary_hover,
                colors.primary_pressed,
                transparent,
            ),
            ButtonVariant::Secondary => (
                colors.elevated,
                colors.text,
                colors.hover,
                colors.pressed,
                transparent,
            ),
            ButtonVariant::Outline => (
                transparent,
                colors.text,
                colors.hover,
                colors.pressed,
                colors.border,
            ),
            ButtonVariant::Soft => (
                colors.selection,
                colors.info,
                colors.info.opacity(0.13),
                colors.info.opacity(0.14),
                transparent,
            ),
            ButtonVariant::Quiet => (
                gpui::transparent_black(),
                colors.text,
                colors.hover,
                colors.pressed,
                gpui::transparent_black(),
            ),
            ButtonVariant::Link => (
                transparent,
                colors.info,
                transparent,
                transparent,
                transparent,
            ),
            ButtonVariant::Danger => (
                colors.danger,
                colors.on_danger,
                colors.danger.opacity(0.90),
                colors.danger.opacity(0.80),
                transparent,
            ),
        };
        if self.frosted {
            bg = transparent;
            foreground = match self.variant {
                ButtonVariant::Danger => colors.danger,
                ButtonVariant::Soft | ButtonVariant::Link => colors.info,
                _ => colors.text,
            };
        }
        if !enabled {
            bg = bg.opacity(0.45);
            foreground = colors.disabled;
            border = border.opacity(0.5);
        } else if theme.preferences.increased_contrast
            && !matches!(self.variant, ButtonVariant::Quiet | ButtonVariant::Link)
        {
            border = colors.secondary;
        }
        let group: SharedString = format!("{}-material", self.id).into();
        let (element, keyboard_pressed) = Pressable {
            id: self.id,
            label: self.label.clone(),
            role: Role::Button,
            enabled,
            focus: self.focus,
            callback: self.callback,
        }
        .render(window, cx);
        let height = self.size.height(&theme);
        let mut element = element
            .when(enabled, |element| {
                element.when_some(self.window_control, |element, area| {
                    element.window_control_area(area)
                })
            })
            .when_some(self.toggled, |element, toggled| {
                element.aria_toggled(if toggled {
                    gpui::Toggled::True
                } else {
                    gpui::Toggled::False
                })
            })
            .flex()
            .items_center()
            .justify_center()
            .gap(px(7.0))
            .min_w(px(if self.icon_only { height } else { 48.0 }))
            .min_h(px(height))
            .px(px(if self.icon_only {
                0.0
            } else {
                theme.metrics.control_padding
            }))
            .py(px(if self.icon_only { 0.0 } else { 4.0 }))
            .border(px(1.0))
            .border_color(border)
            .rounded(px(theme.metrics.control_radius + 2.0))
            .bg(bg)
            .text_color(foreground)
            .font_weight(theme.typography.control_weight)
            .when(self.icon_only, |element| {
                element.size(px(height)).flex_shrink_0()
            });
        if enabled {
            let info_hover = colors.info_hover;
            let info_pressed = colors.info_pressed;
            let link = self.variant == ButtonVariant::Link;
            let hover = if self.frosted { transparent } else { hover };
            let pressed = if self.frosted { transparent } else { pressed };
            element = element
                .hover(move |style| {
                    let style = style.bg(hover);
                    if link {
                        style.text_color(info_hover).underline()
                    } else {
                        style
                    }
                })
                .active(move |style| {
                    let style = style.bg(pressed);
                    if link {
                        style.text_color(info_pressed)
                    } else {
                        style
                    }
                })
                .when(keyboard_pressed, |element| {
                    element
                        .bg(pressed)
                        .when(link, |element| element.text_color(info_pressed))
                });
        }
        element.style().refine(&self.style);
        if self.frosted {
            let mut style = gpui::Style::default();
            style.refine(element.style());
            let mut background = super::surface::material_background(
                crate::material::Material::Frosted,
                None,
                &style,
                style.box_shadow.clone(),
                &theme,
                window,
            );
            background.plan.border = transparent;
            // Paint the material once; never cover the sampled scene with a Div fill.
            element.style().background = None;
            element.style().box_shadow = None;
            let hover_tint = colors.text.alpha(0.08);
            let pressed_tint = colors.text.alpha(0.14);
            let edge = style.border_widths.to_pixels(window.rem_size());
            let overlay = div()
                .id("material-feedback")
                .absolute()
                .top(-edge.top)
                .right(-edge.right)
                .bottom(-edge.bottom)
                .left(-edge.left)
                .when(enabled, |overlay| {
                    overlay
                        .group_hover(group.clone(), move |style| style.bg(hover_tint))
                        .group_active(group.clone(), move |style| style.bg(pressed_tint))
                        .when(keyboard_pressed, |overlay| overlay.bg(pressed_tint))
                });
            let mut overlay = overlay;
            overlay.style().corner_radii = gpui::CornersRefinement {
                top_left: Some(style.corner_radii.top_left),
                top_right: Some(style.corner_radii.top_right),
                bottom_right: Some(style.corner_radii.bottom_right),
                bottom_left: Some(style.corner_radii.bottom_left),
            };
            element = element
                .group(group)
                .child(background)
                .child(super::surface::Layered::new(overlay));
        }
        if self.busy {
            element = element.child(super::surface::Layered::new(
                div().w(px(14.0)).text_center().child("…"),
            ));
        } else if let Some(leading) = self.leading {
            element = element.child(super::surface::Layered::new(leading));
        }
        if self.icon_only && self.tooltip {
            let label = self.label;
            element = element.tooltip(move |_, cx| {
                cx.new(|_| IconHint {
                    label: label.clone(),
                    theme: theme.clone(),
                })
                .into()
            });
        } else if let Some(content) = self.content {
            element = element.child(super::surface::Layered::new(content));
        } else if !self.icon_only {
            element = element.child(super::surface::Layered::new(
                div().min_w_0().truncate().child(self.label),
            ));
        }
        Semantic::new(element)
            .disabled(self.disabled)
            .busy(self.busy)
    }
}

struct IconHint {
    label: SharedString,
    theme: Arc<Theme>,
}

impl Render for IconHint {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("button-tooltip")
            .debug_selector(|| "button-tooltip".into())
            .role(Role::Tooltip)
            .aria_label(self.label.clone())
            .px(px(8.0))
            .py(px(5.0))
            .rounded(px(6.0))
            .border(px(1.0))
            .border_color(self.theme.colors.border)
            .bg(self.theme.colors.elevated)
            .text_color(self.theme.colors.text)
            .font_family(self.theme.typography.family.clone())
            .text_size(px(self.theme.typography.body))
            .child(self.label.clone())
    }
}
