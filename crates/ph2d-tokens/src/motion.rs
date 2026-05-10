//! Motion tokens. Source: `docs/design/tokens.json` → `motion.*`.
//!
//! Easing as cubic-bezier control points (4 floats); duration in ms
//! as `f32`. Vello/animation playback is the consumer's responsibility
//! — this module just supplies the canonical parameters.

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Easing {
    /// `out` — cubic-bezier(0.2, 0.7, 0.1, 1). Default for element
    /// entry / one-shot transitions.
    Out,
    /// `inout` — cubic-bezier(0.4, 0.0, 0.2, 1). Material standard.
    /// Use for reversible transitions (toggle, open/close).
    InOut,
    /// `spring` — cubic-bezier(0.34, 1.56, 0.64, 1). Slight overshoot;
    /// tactile feedback for press / drop.
    Spring,
}

impl Easing {
    /// 4 control points (x1, y1, x2, y2) of the cubic-bezier curve.
    /// Compatible with CSS `cubic-bezier(...)` and parley/vello custom
    /// easing implementations.
    pub const fn bezier(self) -> [f32; 4] {
        match self {
            Self::Out => [0.2, 0.7, 0.1, 1.0],
            Self::InOut => [0.4, 0.0, 0.2, 1.0],
            Self::Spring => [0.34, 1.56, 0.64, 1.0],
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Duration {
    /// `instant` — 80 ms (state flicker, hover feedback).
    Instant,
    /// `fast` — 150 ms (button press, icon swap).
    Fast,
    /// `default` — 240 ms (panel open/close, page transitions).
    Default,
    /// `slow` — 400 ms (hero animations, onboarding).
    Slow,
}

impl Duration {
    pub const fn ms(self) -> f32 {
        match self {
            Self::Instant => 80.0,
            Self::Fast => 150.0,
            Self::Default => 240.0,
            Self::Slow => 400.0,
        }
    }

    pub const fn secs(self) -> f32 {
        self.ms() / 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_strictly_increasing() {
        assert!(Duration::Instant.ms() < Duration::Fast.ms());
        assert!(Duration::Fast.ms() < Duration::Default.ms());
        assert!(Duration::Default.ms() < Duration::Slow.ms());
    }

    #[test]
    fn easing_bezier_in_unit_range_for_x() {
        // x1 and x2 (indices 0 and 2) MUST be in [0, 1] for a monotonic
        // cubic-bezier in CSS. y can exceed (spring overshoot).
        for ease in [Easing::Out, Easing::InOut, Easing::Spring] {
            let [x1, _y1, x2, _y2] = ease.bezier();
            assert!((0.0..=1.0).contains(&x1), "{ease:?} x1={x1}");
            assert!((0.0..=1.0).contains(&x2), "{ease:?} x2={x2}");
        }
    }

    #[test]
    fn spring_overshoots_y() {
        // Spring must have y1 > 1.0 (overshoots the target then returns).
        let [_, y1, _, _] = Easing::Spring.bezier();
        assert!(y1 > 1.0, "spring y1 = {y1}, expected > 1.0");
    }
}
