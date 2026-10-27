use gpui::Refineable;
use gpui::{
    App, ElementId, FocusHandle, IntoElement, ParentElement, PathBuilder, RenderOnce, Role,
    SharedString, StyleRefinement, Styled, Toggled, Window, canvas, div, point, prelude::*, px,
};
use std::rc::Rc;

use crate::{
    element::Semantic,
    interaction::Pressable,
    theme::{Theme, control_transition},
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CheckState {
    #[default]
    Off,
    On,
    Mixed,
}

impl CheckState {
    pub fn next(self) -> Self {
        match self {
            Self::Off | Self::Mixed => Self::On,
            Self::On => Self::Off,
        }
    }

    fn toggled(self) -> Toggled {
        match self {
            Self::Off => Toggled::False,
            Self::On => Toggled::True,
            Self::Mixed => Toggled::Mixed,
        }
    }
}

type ChangeHandler = Rc<dyn Fn(&CheckState, &mut Window, &mut App)>;

struct CheckRecipe {
    id: ElementId,
    label: SharedString,
    label_visible: bool,
    state: CheckState,
    disabled: bool,
    focus: Option<FocusHandle>,
    callback: Option<ChangeHandler>,
    style: StyleRefinement,
    track_size: gpui::Size<gpui::Pixels>,
    track_colors: Option<(gpui::Hsla, gpui::Hsla, gpui::Hsla)>,
}

impl CheckRecipe {
    fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, state: CheckState) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            label_visible: true,
            state,
            disabled: false,
            focus: None,
            callback: None,
            style: StyleRefinement::default(),
            track_size: gpui::size(px(34.0), px(20.0)),
            track_colors: None,
        }
    }

    fn render(self, switch: bool, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let state = self.state;
        let selected = state != CheckState::Off;
        let enabled = !self.disabled;
        let phase = control_transition(
            (self.id.clone(), "check-selection"),
            selected,
            !enabled,
            window,
            cx,
        );
        let mixed = control_transition(
            (self.id.clone(), "check-mark"),
            state == CheckState::Mixed,
            !enabled,
            window,
            cx,
        );
        let callback = self.callback.map(|callback| {
            Rc::new(
                move |_: &gpui::ClickEvent, window: &mut Window, cx: &mut App| {
                    callback(&state.next(), window, cx);
                },
            ) as crate::interaction::ClickHandler
        });
        let foreground = if enabled {
            theme.colors.text
        } else {
            theme.colors.disabled
        };
        let (element, _) = Pressable {
            id: self.id,
            label: self.label.clone(),
            role: if switch { Role::Switch } else { Role::CheckBox },
            enabled,
            focus: self.focus,
            callback,
        }
        .render(window, cx);
        let mut element = element
            .aria_toggled(state.toggled())
            .flex()
            .items_center()
            .gap(px(10.0))
            .min_h(px(theme.metrics.control_height))
            .border(px(theme.metrics.focus_width))
            .border_color(gpui::transparent_black())
            .rounded(px(theme.metrics.control_radius))
            .px(px(5.0))
            .text_color(foreground);
        let colors = &theme.colors;
        let (off_color, on_color, thumb_color) =
            self.track_colors
                .unwrap_or((colors.elevated, colors.success, colors.on_success));
        let selected_color = if !enabled {
            colors.disabled
        } else if switch {
            on_color
        } else {
            colors.info
        };
        let selected_ink = if switch {
            colors.on_success
        } else {
            colors.on_info
        };
        let indicator = div().flex_shrink_0();
        let track = self.track_size;
        let thumb = (track.height - px(6.0)).max(px(0.0));
        let travel = (track.width - thumb - px(6.0)).max(px(0.0));
        // Blend channels, not hue: neutral-to-state transitions must not sweep other colors.
        let fill = phase.interpolate_clamped(
            if switch { off_color } else { colors.elevated }.to_rgb(),
            selected_color.to_rgb(),
        );
        element = element.child(if switch {
            indicator
                .relative()
                .w(track.width)
                .h(track.height)
                .rounded(track.height / 2.0)
                .bg(fill)
                .child(
                    div()
                        .absolute()
                        .top(px(3.0))
                        .left(px(3.0) + phase.interpolate(px(0.0), travel))
                        .size(thumb)
                        .rounded_full()
                        .bg(if self.track_colors.is_some() {
                            thumb_color.to_rgb()
                        } else {
                            phase.interpolate_clamped(
                                colors.secondary.to_rgb(),
                                selected_ink.to_rgb(),
                            )
                        }),
                )
        } else {
            let ink = selected_ink.opacity(phase.0.clamp(0.0, 1.0));
            indicator
                .size(px(18.0))
                .rounded(px(4.0))
                .bg(fill)
                .border(px(1.0))
                .border_color(
                    phase.interpolate_clamped(colors.border.to_rgb(), selected_color.to_rgb()),
                )
                .child(
                    div().size_full().child(
                        canvas(
                            |_, _, _| (),
                            move |bounds, _, window, _| {
                                if ink.a == 0.0 {
                                    return;
                                }
                                let mut path = PathBuilder::stroke(px(1.6));
                                let x = |a, b| mixed.interpolate(px(a), px(b));
                                path.move_to(bounds.origin + point(x(3.0, 4.0), px(8.0)));
                                path.line_to(bounds.origin + point(x(6.5, 8.0), x(11.0, 8.0)));
                                path.line_to(bounds.origin + point(px(12.0), x(4.0, 8.0)));
                                window.paint_path(path.build().expect("valid check mark"), ink);
                            },
                        )
                        .size_full(),
                    ),
                )
        });
        if self.label_visible {
            element = element.child(div().min_w_0().child(self.label));
        }
        element.style().refine(&self.style);
        Semantic::new(element).disabled(self.disabled)
    }
}

#[derive(IntoElement)]
pub struct Checkbox(CheckRecipe);

impl Checkbox {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: CheckState,
    ) -> Self {
        Self(CheckRecipe::new(id, label, state))
    }

    /// Hide duplicate text beside an externally labelled field; keep the accessible name.
    pub fn label_visible(mut self, visible: bool) -> Self {
        self.0.label_visible = visible;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.0.disabled = disabled;
        self
    }

    pub fn focus_handle(mut self, focus: &FocusHandle) -> Self {
        self.0.focus = Some(focus.clone());
        self
    }

    pub fn on_change(
        mut self,
        callback: impl Fn(&CheckState, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.0.callback = Some(Rc::new(callback));
        self
    }
}

impl Styled for Checkbox {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.0.style
    }
}

impl RenderOnce for Checkbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.0.render(false, window, cx)
    }
}

#[derive(IntoElement)]
pub struct Toggle(CheckRecipe);

impl Toggle {
    /// Override the track's off/on fills and thumb; disabled state still suppresses the on fill.
    pub fn track_colors(mut self, off: gpui::Hsla, on: gpui::Hsla, thumb: gpui::Hsla) -> Self {
        self.0.track_colors = Some((off, on, thumb));
        self
    }

    /// Resize the indicator without changing the row's label, focus or input behavior.
    pub fn track_size(mut self, size: gpui::Size<gpui::Pixels>) -> Self {
        assert!(
            f32::from(size.width).is_finite()
                && f32::from(size.height).is_finite()
                && size.height > px(6.0)
                && size.width >= size.height
        );
        self.0.track_size = size;
        self
    }

    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, on: bool) -> Self {
        Self(CheckRecipe::new(
            id,
            label,
            if on { CheckState::On } else { CheckState::Off },
        ))
    }

    /// Hide duplicate text beside an externally labelled field; keep the accessible name.
    pub fn label_visible(mut self, visible: bool) -> Self {
        self.0.label_visible = visible;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.0.disabled = disabled;
        self
    }

    pub fn focus_handle(mut self, focus: &FocusHandle) -> Self {
        self.0.focus = Some(focus.clone());
        self
    }

    pub fn on_change(mut self, callback: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.0.callback = Some(Rc::new(move |state, window, cx| {
            callback(&(*state == CheckState::On), window, cx)
        }));
        self
    }
}

impl Styled for Toggle {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.0.style
    }
}

impl RenderOnce for Toggle {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.0.render(true, window, cx)
    }
}
