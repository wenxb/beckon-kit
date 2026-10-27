//! Window ownership, native-first decoration policy and client-area geometry.
mod close;
mod config;
mod frame;
mod geometry;
mod interaction;
mod overlay;
mod platform;
mod resolve;
mod root;
#[cfg(test)]
mod tests;

pub use close::{CloseDecision, CloseRequest, CloseWindow};
pub use config::{
    ClientFrame, FrameMode, PreparedWindow, TitlebarMode, WindowConfig, WindowConfigError,
    WindowRootConfig,
};
pub use geometry::FrameGeometry;
pub use interaction::DragRegion;
pub use resolve::{FrameOwner, FrameState, WindowFrameStatus};
pub use root::{WindowRoot, WindowRootHandleExt};
