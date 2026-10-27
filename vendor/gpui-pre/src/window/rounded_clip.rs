use super::Window;
use crate::RoundedClip;

impl Window {
    /// Whether the decoration mode reflects an acknowledged platform decision.
    pub fn window_decorations_negotiated(&self) -> bool {
        self.platform_window.window_decorations_negotiated()
    }

    /// Whether `set_input_region` restricts native input on this platform.
    pub fn supports_input_region(&self) -> bool {
        self.platform_window.supports_input_region()
    }

    /// True only for renderers with group coverage (not per-primitive rounded alpha).
    pub fn is_rounded_clip_supported(&self) -> bool {
        self.platform_window.is_rounded_clip_supported()
    }

    /// Clip all descendant drawing and application hitboxes without changing layout.
    /// Pass the same identity in prepaint and paint; deferred draws inherit the scope.
    pub fn with_rounded_clip<R>(
        &mut self,
        clip: &RoundedClip,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        assert!(
            self.rounded_clip_stack.len() < 8,
            "maximum rounded clip depth is eight"
        );
        self.invalidator.debug_assert_paint_or_prepaint();
        let painting = self.invalidator.inner.borrow().draw_phase == super::DrawPhase::Paint;
        self.rounded_clip_stack.push(clip.clone());
        if painting {
            self.next_frame
                .scene
                .push_rounded_clip(clip, self.scale_factor());
        }
        let result = f(self);
        if painting {
            self.next_frame.scene.pop_rounded_clip();
        }
        self.rounded_clip_stack.pop();
        result
    }

    pub(crate) fn with_rounded_clip_stack<R>(
        &mut self,
        clips: &[RoundedClip],
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        if let Some((clip, remaining)) = clips.split_first() {
            self.with_rounded_clip(clip, |window| window.with_rounded_clip_stack(remaining, f))
        } else {
            f(self)
        }
    }

    /// Reserve the unique native decoration owner before the first frame.
    /// Returns false if another owner or an existing native root is already present.
    pub fn claim_decoration_owner(&mut self) -> bool {
        if self.decoration_owner || self.root.is_some() {
            return false;
        }
        self.decoration_owner = true;
        true
    }
}
