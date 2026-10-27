use gpui::Refineable;
use gpui::{
    App, ElementId, FocusHandle, IntoElement, ParentElement, RenderOnce, Role, SharedString,
    StyleRefinement, Styled, Window, div, prelude::*, px,
};
use std::rc::Rc;

use crate::{element::Semantic, interaction::control_focus, theme::Theme};

#[derive(Debug, Clone)]
pub struct Segment {
    pub id: SharedString,
    pub label: SharedString,
    pub disabled: bool,
}

impl Segment {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TabVariant {
    #[default]
    Segmented,
    Pills,
    Underline,
}

type ChangeHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// One tab stop, roving arrows; selection stays in the host, not an internal copy.
#[derive(IntoElement)]
pub struct SegmentedControl {
    id: ElementId,
    label: SharedString,
    segments: Vec<Segment>,
    selected: SharedString,
    disabled: bool,
    variant: TabVariant,
    pub(super) tab_semantics: bool,
    focus: Option<FocusHandle>,
    callback: Option<ChangeHandler>,
    style: StyleRefinement,
    item_style: StyleRefinement,
    selected_style: StyleRefinement,
}

impl SegmentedControl {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        segments: impl IntoIterator<Item = Segment>,
        selected: impl Into<SharedString>,
    ) -> Self {
        let segments: Vec<_> = segments.into_iter().collect();
        assert!(
            segments.iter().enumerate().all(|(index, segment)| {
                !segments[..index]
                    .iter()
                    .any(|previous| previous.id == segment.id)
            }),
            "segment identities must be unique"
        );
        Self {
            id: id.into(),
            label: label.into(),
            segments,
            selected: selected.into(),
            disabled: false,
            variant: TabVariant::default(),
            tab_semantics: false,
            focus: None,
            callback: None,
            style: StyleRefinement::default(),
            item_style: StyleRefinement::default(),
            selected_style: StyleRefinement::default(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Refine each item, including padding, height, corner radius and typography.
    pub fn item_style(mut self, style: StyleRefinement) -> Self {
        self.item_style = style;
        self
    }

    /// Refine the selected item after the shared item style; disabled ink remains disabled.
    pub fn selected_style(mut self, style: StyleRefinement) -> Self {
        self.selected_style = style;
        self
    }

    pub fn variant(mut self, variant: TabVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn focus_handle(mut self, focus: &FocusHandle) -> Self {
        self.focus = Some(focus.clone());
        self
    }

    pub fn on_change(
        mut self,
        callback: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.callback = Some(Rc::new(callback));
        self
    }
}

impl Styled for SegmentedControl {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for SegmentedControl {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let enabled: Vec<_> = self
            .segments
            .iter()
            .filter(|segment| !segment.disabled)
            .map(|segment| segment.id.clone())
            .collect();
        let interactive = !self.disabled && !enabled.is_empty();
        let segmented = self.variant == TabVariant::Segmented;
        let underline = self.variant == TabVariant::Underline;
        let transparent = gpui::transparent_black();
        let focus = control_focus(self.id.clone(), self.focus, window, cx);
        let selector = self.id.to_string();
        let mut container = div()
            .id(self.id.clone())
            .key_context("BeckonKitSegmented")
            .role(if self.tab_semantics {
                Role::TabList
            } else {
                Role::RadioGroup
            })
            .aria_label(self.label)
            .debug_selector(move || selector)
            .flex()
            .items_center()
            .gap(px(2.0))
            .p(px(if segmented { 3.0 } else { 0.0 }))
            .rounded(px(if underline {
                0.0
            } else {
                theme.metrics.control_radius + 4.0
            }))
            .bg(if segmented {
                theme.colors.canvas
            } else {
                transparent
            });
        if interactive {
            let focused = focus.clone();
            let selected = self.selected.clone();
            let callback = self.callback.clone();
            let ring = theme.colors.focus;
            container = container
                .track_focus(&focus)
                .tab_index(0)
                .focus_visible(move |style| {
                    style.shadow(vec![
                        gpui::BoxShadow::new(px(0.0), px(0.0), ring).spread_radius(px(2.0)),
                    ])
                })
                .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                    focused.focus(window, cx)
                })
                .on_key_down(move |event, window, cx| {
                    if event.keystroke.modifiers.modified() {
                        return;
                    }
                    let index = enabled.iter().position(|id| id == &selected).unwrap_or(0);
                    let next = match event.keystroke.key.as_str() {
                        "left" | "up" => (index + enabled.len() - 1) % enabled.len(),
                        "right" | "down" => (index + 1) % enabled.len(),
                        "home" => 0,
                        "end" => enabled.len() - 1,
                        _ => return,
                    };
                    if let Some(callback) = &callback {
                        callback(&enabled[next], window, cx);
                    }
                    cx.stop_propagation();
                });
        }
        for segment in self.segments {
            let selected = self.selected == segment.id;
            let disabled = self.disabled || segment.disabled;
            let foreground = if disabled {
                theme.colors.disabled
            } else {
                theme.colors.secondary
            };
            let mut item = div()
                .id((self.id.clone(), segment.id.clone()))
                .role(if self.tab_semantics {
                    Role::Tab
                } else {
                    Role::RadioButton
                })
                .aria_label(segment.label.clone())
                .when(self.tab_semantics, |item| item.aria_selected(selected))
                .when(!self.tab_semantics, |item| {
                    item.aria_toggled(if selected {
                        gpui::Toggled::True
                    } else {
                        gpui::Toggled::False
                    })
                })
                .when(selected, |element| element.aria_active_descendant())
                .debug_selector({
                    let id = format!("{}-{}", self.id, segment.id);
                    move || id
                })
                .flex()
                .items_center()
                .justify_center()
                .flex_1()
                .min_w_0()
                .min_h(px(theme.metrics.control_height))
                .px(px(12.0))
                .border_b(px(if underline { 2.0 } else { 0.0 }))
                .border_color(if underline && selected {
                    theme.colors.info
                } else {
                    transparent
                })
                .rounded(px(if underline {
                    0.0
                } else if segmented {
                    theme.metrics.control_radius + 1.0
                } else {
                    999.0
                }))
                .text_color(if selected && !disabled {
                    theme.colors.info
                } else {
                    foreground
                })
                .font_weight(if selected {
                    theme.typography.control_weight
                } else {
                    gpui::FontWeight::NORMAL
                })
                .when(selected && !underline, |element| {
                    element.bg(theme.colors.selection)
                })
                .child(div().truncate().child(segment.label));
            item.style().refine(&self.item_style);
            if selected {
                item.style().refine(&self.selected_style);
            }
            if disabled {
                item = item.text_color(theme.colors.disabled);
            }
            if !disabled {
                let callback = self.callback.clone();
                let focused = focus.clone();
                let text = theme.colors.text;
                item = item
                    .cursor_pointer()
                    .when(!selected, |item| {
                        item.hover(move |style| style.text_color(text))
                    })
                    .on_click(move |_, window, cx| {
                        focused.focus(window, cx);
                        if !selected && let Some(callback) = &callback {
                            callback(&segment.id, window, cx);
                        }
                        cx.stop_propagation();
                    });
            }
            container = container.child(Semantic::new(item).disabled(disabled));
        }
        container.style().refine(&self.style);
        Semantic::new(container).disabled(self.disabled)
    }
}
