use super::TestAppContext;
use crate::{AnyWindowHandle, Decorations, WindowControls, WindowFrameRequest};

impl TestAppContext {
    /// Native action requests, without synthesizing a compositor reply.
    pub fn window_frame_requests(&self, window: AnyWindowHandle) -> Vec<WindowFrameRequest> {
        self.test_window(window).frame_requests()
    }
    /// Deliver decoration/state negotiation through the real bounds-observer path.
    pub fn simulate_window_frame_state(
        &mut self,
        window: AnyWindowHandle,
        decorations: Decorations,
        maximized: bool,
        fullscreen: bool,
        controls: WindowControls,
    ) {
        self.test_window(window)
            .simulate_frame_state(decorations, maximized, fullscreen, controls);
        self.background_executor.run_until_parked();
    }
}
