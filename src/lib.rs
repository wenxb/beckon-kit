//! Original, application-independent components built directly on GPUI.
//!
//! Components own their visuals and semantics. GPUI owns layout, input dispatch,
//! text shaping and platform integration; no other component library is linked.

#![doc = include_str!("../README.md")]

pub mod components;
pub mod material;
pub mod theme;
pub mod window;
pub use window::{WindowConfig, WindowRoot, WindowRootHandleExt};

mod element;
mod interaction;
mod root;
mod text;

#[cfg(feature = "gallery")]
pub mod gallery;
#[cfg(feature = "test-support")]
pub mod test_support;

/// GPUI types, traits and macros share the component library's engine identity.
pub use gpui;
/// Opt-in native application bootstrap and headless platform support.
#[cfg(feature = "platform")]
pub use gpui_platform as platform;
pub use root::{Root, init};

pub mod prelude {
    pub use crate::components::*;
    pub use crate::material::{BackdropImage, ImagePlane, Material, PreparedImage, SceneBackdrop};
    pub use crate::theme::{Appearance, Density, FrostedMaterial, Preferences, Theme, ThemeScope};
    pub use crate::window::*;
    pub use crate::{Root, init};
    pub use gpui::prelude::*;
}
