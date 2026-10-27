use gpui::{App, ElementId, SpringTarget, Window};
use std::time::Duration;

const TRANSITION: Duration = Duration::from_millis(320);

struct Transition {
    from: f32,
    value: f32,
    target: f32,
    elapsed: Duration,
}

impl Transition {
    fn sample(&mut self, elapsed: Duration) {
        self.elapsed = self.elapsed.saturating_add(elapsed).min(TRANSITION);
        let t = self.elapsed.as_secs_f32() / TRANSITION.as_secs_f32();
        let eased = 1.0 - (1.0 - t).powi(3);
        self.value = if self.elapsed == TRANSITION {
            self.target
        } else {
            self.from + (self.target - self.from) * eased
        };
    }
}

/// Finite ease-out: respond immediately, retarget from presentation, never coast past a reversal.
/// The keyed state is presentation only; hosts still own the requested value.
pub(crate) fn control_transition<T: SpringTarget>(
    id: impl Into<ElementId>,
    target: T,
    immediate: bool,
    window: &mut Window,
    cx: &mut App,
) -> T::Output {
    let now = cx.background_executor().now();
    let immediate = immediate || cx.reduce_motion();
    let next = target.target();
    let state = window.use_keyed_state(id, cx, |_, _| {
        (
            Transition {
                from: next,
                value: next,
                target: next,
                elapsed: TRANSITION,
            },
            now,
        )
    });
    let (value, running) = state.update(cx, |(transition, updated_at), _| {
        transition.sample(now.duration_since(*updated_at));
        *updated_at = now;
        if transition.target != next {
            transition.from = transition.value;
            transition.target = next;
            transition.elapsed = Duration::ZERO;
        }
        if immediate {
            transition.value = next;
            transition.from = next;
            transition.elapsed = TRANSITION;
        }
        (
            transition.value,
            transition.value != next && transition.elapsed < TRANSITION,
        )
    });
    if running {
        window.request_animation_frame();
    }
    target.resolve(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ease_out_is_front_loaded_bounded_and_finishes_without_a_tail() {
        let mut transition = Transition {
            from: 0.0,
            value: 0.0,
            target: 1.0,
            elapsed: Duration::ZERO,
        };
        transition.sample(TRANSITION / 4);
        assert_eq!(transition.value, 0.578125);
        transition.sample(TRANSITION / 4);
        assert_eq!(transition.value, 0.875);
        transition.sample(TRANSITION / 2);
        assert_eq!(transition.value, 1.0);
        transition.sample(TRANSITION);
        assert_eq!(transition.value, 1.0);
    }
}
