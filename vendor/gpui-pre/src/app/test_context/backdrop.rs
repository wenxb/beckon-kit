use super::TestAppContext;
use crate::{AnyWindowHandle, AppContext};

impl TestAppContext {
    /// Override only the test window's advertised capability for scene assertions.
    /// This does not install a renderer; GPU validation needs a headless renderer.
    pub fn simulate_backdrop_blur_support(&mut self, window: AnyWindowHandle, supported: bool) {
        self.test_window(window)
            .0
            .lock()
            .backdrop_blur_supported_override = Some(supported);
        self.update_window(window, |_, window, _| window.refresh())
            .expect("refresh test blur capability");
    }
}
