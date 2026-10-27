use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeError;

impl fmt::Display for RangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a slider needs finite min < max and a finite positive step")
    }
}

impl std::error::Error for RangeError {}

/// Finite numeric domain shared by pointer, keyboard and accessibility input.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliderScale {
    min: f64,
    max: f64,
    step: f64,
}

impl SliderScale {
    pub fn new(min: f64, max: f64, step: f64) -> Result<Self, RangeError> {
        if !min.is_finite()
            || !max.is_finite()
            || !(max - min).is_finite()
            || min >= max
            || !step.is_finite()
            || step <= 0.0
            || !((max - min) / step).is_finite()
        {
            return Err(RangeError);
        }
        Ok(Self { min, max, step })
    }

    pub fn min(self) -> f64 {
        self.min
    }

    pub fn max(self) -> f64 {
        self.max
    }

    pub fn step(self) -> f64 {
        self.step
    }

    /// Invalid values resolve to the minimum; both endpoints remain reachable.
    pub fn normalize(self, value: f64) -> f64 {
        if value.is_nan() || value <= self.min {
            return self.min;
        }
        if value >= self.max {
            return self.max;
        }
        let snapped = self.min + ((value - self.min) / self.step).round() * self.step;
        snapped.clamp(self.min, self.max)
    }

    pub fn fraction(self, value: f64) -> f32 {
        ((self.normalize(value) - self.min) / (self.max - self.min)) as f32
    }

    pub fn value_at(self, fraction: f32) -> f64 {
        let fraction = if fraction.is_nan() {
            0.0
        } else {
            fraction.clamp(0.0, 1.0)
        };
        self.normalize(self.min + (self.max - self.min) * f64::from(fraction))
    }

    pub fn offset(self, value: f64, steps: f64) -> f64 {
        let value = self.normalize(value);
        let index = if value == self.max {
            ((self.max - self.min) / self.step).ceil()
        } else {
            ((value - self.min) / self.step).round()
        };
        self.normalize(self.min + (index + steps) * self.step)
    }
}

impl Default for SliderScale {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 1.0,
            step: 0.01,
        }
    }
}
