use crate::{Decorations, Pixels, Point, ResizeEdge, Size, TestWindow, WindowControls};

/// Requests are recorded without pretending a native compositor replied.
#[derive(Debug, Clone, PartialEq)]
pub enum WindowFrameRequest {
    /// Begin a compositor-controlled window move.
    Move,
    /// Begin a compositor-controlled resize at the specified edge.
    Resize(ResizeEdge),
    /// Request minimization without changing simulated state.
    Minimize,
    /// Request maximize/restore without changing simulated state.
    Zoom,
    /// Open the native window menu at a surface-local position.
    Menu(Point<Pixels>),
    /// Update the minimum native content-surface size.
    Minimum(Size<Pixels>),
    /// Update the normal-state client decoration inset.
    Inset(Pixels),
}
impl TestWindow {
    pub fn frame_requests(&self) -> Vec<WindowFrameRequest> {
        self.0.lock().frame_requests.clone()
    }
    pub fn simulate_frame_state(
        &mut self,
        decorations: Decorations,
        maximized: bool,
        fullscreen: bool,
        controls: WindowControls,
    ) {
        let size = {
            let mut state = self.0.lock();
            state.decorations = decorations;
            state.maximized = maximized;
            state.is_fullscreen = fullscreen;
            state.controls = controls;
            state.bounds.size
        };
        self.simulate_resize(size);
    }
}
