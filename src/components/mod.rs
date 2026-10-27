//! Controlled recipes for simple controls; entities only for editing state.

mod badge;
mod button;
pub mod canvas;
mod check;
mod icon;
mod markdown;
mod progress;
mod segmented;
mod select;
mod slider;
mod surface;
mod tabs;
mod text_field;
mod virtual_list;

pub use crate::interaction::{RangeError, SliderScale};
pub use badge::{Badge, Status};
pub use button::{Button, ButtonVariant, ControlSize};
pub use canvas::Canvas;
pub use check::{CheckState, Checkbox, Toggle};
pub use icon::{Icon, IconName};
pub use markdown::Markdown;
pub use progress::ProgressBar;
pub use segmented::{Segment, SegmentedControl, TabVariant};
pub use select::{Select, SelectOption};
pub use slider::Slider;
pub use surface::Surface;
pub use tabs::Tabs;
pub use text_field::{TextField, TextFieldEvent, TextFieldVariant};
pub use virtual_list::VirtualList;
