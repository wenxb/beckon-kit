use gpui::{
    AnyElement, App, Div, Global, IntoElement, KeyBinding, ParentElement, RenderOnce, SharedString,
    Styled, Window, actions, div, prelude::*, px, relative,
};

use crate::theme::{Theme, ensure_environment};

actions!(beckon_kit, [FocusNext, FocusPrevious]);

struct Installed;
impl Global for Installed {}

/// Idempotent bootstrap. It does not create windows or choose system appearance.
pub fn init(cx: &mut App) {
    ensure_environment(cx);
    if !cx.has_global::<Installed>() {
        cx.bind_keys([
            KeyBinding::new("tab", FocusNext, Some("BeckonKit")),
            KeyBinding::new("shift-tab", FocusPrevious, Some("BeckonKit")),
        ]);
        cx.bind_keys([KeyBinding::new(
            if cfg!(target_os = "macos") {
                "cmd-w"
            } else {
                "ctrl-w"
            },
            crate::window::CloseWindow,
            Some("BeckonWindow"),
        )]);
        // Component keys must reach their local listeners before ancestor shortcuts.
        for (context, keys) in [
            ("BeckonKitPressable", &["enter", "space"][..]),
            (
                "BeckonKitTextField",
                &["enter", "left", "right", "home", "end"][..],
            ),
            (
                "BeckonKitTextArea",
                &[
                    "enter",
                    "left",
                    "right",
                    "home",
                    "end",
                    "up",
                    "down",
                    "pageup",
                    "pagedown",
                    "shift-left",
                    "shift-right",
                    "shift-home",
                    "shift-end",
                    "shift-up",
                    "shift-down",
                    "shift-pageup",
                    "shift-pagedown",
                ][..],
            ),
            (
                "BeckonKitSlider",
                &[
                    "left", "right", "up", "down", "home", "end", "pageup", "pagedown",
                ][..],
            ),
            (
                "BeckonKitSegmented",
                &["left", "right", "up", "down", "home", "end"][..],
            ),
            (
                "BeckonKitSelect",
                &["enter", "space", "up", "down", "home", "end"][..],
            ),
            (
                "BeckonKitSelectOpen",
                &["enter", "space", "up", "down", "home", "end", "escape"][..],
            ),
        ] {
            cx.bind_keys(
                keys.iter()
                    .map(|key| KeyBinding::new(key, gpui::NoAction, Some(context))),
            );
        }
        cx.set_global(Installed);
    }
}

#[derive(IntoElement)]
pub struct Root {
    id: SharedString,
    label: SharedString,
    children: Vec<AnyElement>,
}

impl Root {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: "Content".into(),
            children: Vec::new(),
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }
}

impl ParentElement for Root {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Root {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let focus = window
            .use_keyed_state(
                (gpui::ElementId::from(self.id.clone()), "root-focus"),
                cx,
                |window, cx| {
                    let focus = cx.focus_handle().tab_stop(false);
                    // Bootstrap keyboard navigation once; repaint must respect explicit blur.
                    if window.focused(cx).is_none() {
                        focus.focus(window, cx);
                    }
                    focus
                },
            )
            .read(cx)
            .clone();
        let root: gpui::Stateful<Div> = div()
            .id(self.id)
            .role(gpui::Role::Pane)
            .aria_label(self.label)
            .track_focus(&focus)
            .key_context("BeckonKit")
            .tab_group()
            .size_full()
            .font_family(theme.typography.family.clone())
            .text_size(px(theme.typography.body))
            .line_height(relative(theme.typography.line_height))
            .text_color(theme.colors.text)
            .on_action(|_: &FocusNext, window, cx| window.focus_next(cx))
            .on_action(|_: &FocusPrevious, window, cx| window.focus_prev(cx))
            .children(self.children);
        root
    }
}
