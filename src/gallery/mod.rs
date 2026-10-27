//! Component documentation and interactive examples; opt-in gallery feature.

mod canvas;
mod controls;
mod foundations;
mod layout;
mod lists;
mod materials;
mod navigation;
mod scene;
#[cfg(all(test, feature = "test-support"))]
mod tests;
mod window;

use gpui::{AppContext, Context, Entity, SharedString};

use crate::{
    components::{CheckState, TextField, TextFieldEvent},
    theme::{Appearance, Density, Preferences},
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    Controls,
    VirtualList,
    Canvas,
    Materials,
    Window,
    Colors,
    Icons,
}

pub struct Gallery {
    pub(crate) page: Page,
    pub(crate) scroll: gpui::ScrollHandle,
    pub(crate) list_scroll: gpui::UniformListScrollHandle,
    pub(crate) appearance: Appearance,
    pub(crate) density: Density,
    pub(crate) preferences: Preferences,
    pub(crate) input: Entity<TextField>,
    pub(crate) textarea: Entity<TextField>,
    pub(crate) checks: CheckState,
    pub(crate) on: bool,
    pub(crate) mode: SharedString,
    pub(crate) progress: f64,
    pub(crate) clicks: usize,
    pub(crate) last_event: SharedString,
    pub(crate) scene_shift: f64,
    pub(crate) scene_revision: usize,
}

impl Gallery {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextField::new("gallery-input", "命令", cx));
        input.update(cx, |input, cx| input.set_placeholder("搜索命令…", cx));
        cx.subscribe(&input, |gallery, _, event: &TextFieldEvent, cx| {
            gallery.last_event = match event {
                TextFieldEvent::Changed { value, composing } => format!(
                    "{} · {} 个字符",
                    if *composing { "正在组字" } else { "输入" },
                    value.chars().count()
                )
                .into(),
                TextFieldEvent::Submitted(value) => format!("已提交：{value}").into(),
            };
            cx.notify();
        })
        .detach();
        let textarea = cx.new(|cx| {
            let mut input = TextField::new("gallery-textarea", "多行说明", cx);
            input.set_multiline(true, cx);
            input.set_placeholder("输入多行说明…", cx);
            input.set_value(
                "Enter 换行\nCtrl/Cmd+Enter 提交；支持选择、撤销与输入法。",
                cx,
            );
            input
        });
        cx.subscribe(&textarea, |gallery, _, event: &TextFieldEvent, cx| {
            gallery.last_event = match event {
                TextFieldEvent::Changed { value, .. } => {
                    format!("说明 · {} 行", value.lines().count()).into()
                }
                TextFieldEvent::Submitted(value) => format!("已提交说明：{value}").into(),
            };
            cx.notify();
        })
        .detach();
        Self {
            page: Page::default(),
            scroll: gpui::ScrollHandle::new(),
            list_scroll: gpui::UniformListScrollHandle::new(),
            appearance: Appearance::Dark,
            density: Density::default(),
            preferences: Preferences::default(),
            input,
            textarea,
            checks: CheckState::On,
            on: true,
            mode: "list".into(),
            progress: 0.42,
            clicks: 0,
            last_event: "就绪".into(),
            scene_shift: 0.0,
            scene_revision: 0,
        }
    }

    pub fn show_page(&mut self, page: Page, cx: &mut Context<Self>) {
        if self.page != page {
            self.page = page;
            self.scroll
                .set_offset(gpui::point(gpui::px(0.0), gpui::px(0.0)));
            cx.notify();
        }
    }

    pub fn set_appearance(&mut self, appearance: Appearance, cx: &mut Context<Self>) {
        self.appearance = appearance;
        cx.notify();
    }

    pub fn set_density(&mut self, density: Density, cx: &mut Context<Self>) {
        self.density = density;
        cx.notify();
    }

    pub fn set_preferences(&mut self, preferences: Preferences, cx: &mut Context<Self>) {
        self.preferences = preferences;
        cx.notify();
    }

    pub fn scene_revision(&self) -> usize {
        self.scene_revision
    }

    pub fn advance_scene(&mut self, cx: &mut Context<Self>) {
        self.scene_revision = self.scene_revision.wrapping_add(1);
        self.last_event = format!("实时背景 · 第 {} 次更新", self.scene_revision).into();
        cx.notify();
    }

    pub fn set_scene_shift(&mut self, value: f64, cx: &mut Context<Self>) {
        self.scene_shift = if value.is_finite() {
            value.clamp(0.0, 1.0)
        } else {
            0.0
        };
        cx.notify();
    }

    pub fn scene_shift(&self) -> f64 {
        self.scene_shift
    }

    pub fn clicks(&self) -> usize {
        self.clicks
    }
    pub fn toggle_value(&self) -> bool {
        self.on
    }
    pub fn slider_value(&self) -> f64 {
        self.progress
    }
    pub fn text_field(&self) -> &Entity<TextField> {
        &self.input
    }
}
