//! Tokens and subtree-scoped themes, independent of business preferences.

mod motion;
mod scope;
mod tokens;

pub(crate) use motion::control_transition;
pub use scope::ThemeScope;
pub use tokens::{Appearance, Colors, Density, FrostedMaterial, Metrics, Preferences, Typography};

use gpui::{App, Global, TextStyleRefinement, px, relative};
use std::{cell::RefCell, rc::Rc, sync::Arc};

#[derive(Debug, Clone)]
pub struct Theme {
    pub appearance: Appearance,
    pub density: Density,
    pub colors: Colors,
    pub frosted: FrostedMaterial,
    pub metrics: Metrics,
    pub typography: Typography,
    pub preferences: Preferences,
}

impl Theme {
    pub fn new(appearance: Appearance) -> Self {
        Self {
            appearance,
            density: Density::default(),
            colors: Colors::for_appearance(appearance),
            frosted: FrostedMaterial::for_appearance(appearance),
            metrics: Metrics::for_density(Density::default()),
            typography: Typography::default(),
            preferences: Preferences::default(),
        }
    }

    pub fn density(mut self, density: Density) -> Self {
        self.density = density;
        self.metrics = Metrics::for_density(density);
        self
    }

    pub fn preferences(mut self, preferences: Preferences) -> Self {
        self.preferences = preferences;
        self
    }

    pub(crate) fn text_style(&self) -> TextStyleRefinement {
        TextStyleRefinement {
            font_family: Some(self.typography.family.clone()),
            font_size: Some(px(self.typography.body).into()),
            line_height: Some(relative(self.typography.line_height)),
            color: Some(self.colors.text),
            ..Default::default()
        }
    }

    /// Snapshot the environment while drawing, never inside event callbacks.
    pub fn current(cx: &App) -> Arc<Self> {
        cx.try_global::<Environment>()
            .map(|environment| {
                environment
                    .scopes
                    .borrow()
                    .last()
                    .unwrap_or(&environment.default)
                    .clone()
            })
            .unwrap_or_else(|| Arc::new(Self::default()))
    }

    /// App-wide theme for windows without an explicit override; ignores subtree scopes.
    pub fn global(cx: &App) -> Arc<Self> {
        cx.try_global::<Environment>()
            .map(|environment| environment.default.clone())
            .unwrap_or_else(|| Arc::new(Self::default()))
    }

    /// Update all following windows and controls. Hosts own system appearance policy.
    pub fn install(self, cx: &mut App) {
        ensure_environment(cx);
        cx.global_mut::<Environment>().default = Arc::new(self);
        cx.refresh_windows();
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new(Appearance::default())
    }
}

struct Environment {
    default: Arc<Theme>,
    scopes: Rc<RefCell<Vec<Arc<Theme>>>>,
}

impl Global for Environment {}

pub(crate) fn ensure_environment(cx: &mut App) {
    if !cx.has_global::<Environment>() {
        cx.set_global(Environment {
            default: Arc::new(Theme::default()),
            scopes: Rc::default(),
        });
    }
}

fn with_theme<R>(theme: Arc<Theme>, cx: &mut App, f: impl FnOnce(&mut App) -> R) -> R {
    ensure_environment(cx);
    struct Restore(Rc<RefCell<Vec<Arc<Theme>>>>);
    impl Drop for Restore {
        fn drop(&mut self) {
            self.0.borrow_mut().pop();
        }
    }
    let scopes = cx.global::<Environment>().scopes.clone();
    scopes.borrow_mut().push(theme);
    let _restore = Restore(scopes);
    f(cx)
}
