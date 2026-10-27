mod support;

use beckon_kit::{
    components::Button,
    theme::{Appearance, Theme, ThemeScope},
};
use gpui::{App, Context, IntoElement, Render, RenderOnce, Window, div, prelude::*, px};
use std::{cell::RefCell, rc::Rc};

#[derive(IntoElement)]
struct Probe(Rc<RefCell<Vec<Appearance>>>);

impl RenderOnce for Probe {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        self.0.borrow_mut().push(Theme::current(cx).appearance);
        div().size(px(1.0))
    }
}

struct Scoped {
    observations: Rc<RefCell<Vec<Appearance>>>,
}

impl Render for Scoped {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .child(Probe(self.observations.clone()))
            .child(ThemeScope::new(
                Theme::new(Appearance::Light),
                div()
                    .flex()
                    .child(Probe(self.observations.clone()))
                    .child(ThemeScope::new(
                        Theme::new(Appearance::Dark),
                        Probe(self.observations.clone()),
                    ))
                    .child(Probe(self.observations.clone())),
            ))
            .child(Probe(self.observations.clone()))
    }
}

#[gpui::test]
fn nested_scopes_restore_the_parent_and_never_leak_to_siblings_or_windows(
    cx: &mut gpui::TestAppContext,
) {
    let observations = Rc::new(RefCell::new(Vec::new()));
    let window = support::open(cx, |_, _| Scoped {
        observations: observations.clone(),
    });
    observations.borrow_mut().clear();
    support::frame(cx, window.into());
    assert_eq!(
        *observations.borrow(),
        [
            Appearance::Dark,
            Appearance::Light,
            Appearance::Dark,
            Appearance::Light,
            Appearance::Dark
        ]
    );
    assert_eq!(
        cx.update(|cx| Theme::current(cx).appearance),
        Appearance::Dark
    );
    let second = support::open(cx, |_, _| SingleProbe(observations.clone()));
    observations.borrow_mut().clear();
    support::frame(cx, second.into());
    assert_eq!(*observations.borrow(), [Appearance::Dark]);
}

struct SingleProbe(Rc<RefCell<Vec<Appearance>>>);
impl Render for SingleProbe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Probe(self.0.clone())
    }
}

struct SizedButton {
    theme: Theme,
}
impl Render for SizedButton {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        ThemeScope::new(
            self.theme.clone(),
            div()
                .flex()
                .flex_col()
                .items_start()
                .child(Button::new("sized", "A")),
        )
    }
}

#[gpui::test]
fn typography_scaling_grows_the_control_instead_of_clipping_text(cx: &mut gpui::TestAppContext) {
    let normal = support::open(cx, |_, _| SizedButton {
        theme: Theme::default(),
    });
    let normal_height = support::bounds(cx, normal.into(), "sized").size.height;
    let large = support::open(cx, |_, _| {
        let mut theme = Theme::default();
        theme.typography.body = 24.0;
        SizedButton { theme }
    });
    let large_height = support::bounds(cx, large.into(), "sized").size.height;
    assert!(large_height > normal_height);
}
