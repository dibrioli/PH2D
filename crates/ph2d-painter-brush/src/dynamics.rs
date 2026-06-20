//! Pressure dynamics — how pen pressure drives the dab size and coverage.
//!
//! Behavioural reference (clean-room, no code copied): Blender maps pen pressure onto brush
//! parameters via the per-channel pressure toggles shown in the brush popover (the pencil icons
//! next to Radius/Strength). Each enabled channel scales linearly from a per-channel minimum at
//! pressure 0 up to the full value at pressure 1. A mouse (no pressure) reports `1.0`, so dynamics
//! are a no-op there.

/// Which brush channels respond to pen pressure, and how far they fall at zero pressure.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dynamics {
    /// Radius follows pressure.
    pub size_pressure: bool,
    /// Coverage (per-dab opacity) follows pressure.
    pub strength_pressure: bool,
    /// Radius multiplier at pressure 0 when [`Self::size_pressure`], `0..1`.
    pub size_min: f32,
    /// Coverage multiplier at pressure 0 when [`Self::strength_pressure`], `0..1`.
    pub strength_min: f32,
}

impl Default for Dynamics {
    /// Size follows pressure (the iconic pen behaviour), coverage does not. `size_min = 0.1` keeps
    /// a faint mark at the lightest touch instead of vanishing.
    fn default() -> Self {
        Self {
            size_pressure: true,
            strength_pressure: false,
            size_min: 0.1,
            strength_min: 0.0,
        }
    }
}

#[inline]
fn ramp(enabled: bool, min: f32, pressure: f32) -> f32 {
    if enabled {
        let p = pressure.clamp(0.0, 1.0);
        let min = min.clamp(0.0, 1.0);
        min + (1.0 - min) * p
    } else {
        1.0
    }
}

impl Dynamics {
    /// Radius multiplier for a given pressure in `[0, 1]`.
    #[must_use]
    pub fn radius_scale(&self, pressure: f32) -> f32 {
        ramp(self.size_pressure, self.size_min, pressure)
    }

    /// Coverage multiplier for a given pressure in `[0, 1]`.
    #[must_use]
    pub fn coverage_scale(&self, pressure: f32) -> f32 {
        ramp(self.strength_pressure, self.strength_min, pressure)
    }
}

#[cfg(test)]
mod tests {
    use super::Dynamics;

    #[test]
    fn mouse_pressure_is_noop() {
        let d = Dynamics { size_pressure: true, strength_pressure: true, ..Default::default() };
        assert!((d.radius_scale(1.0) - 1.0).abs() < 1e-6);
        assert!((d.coverage_scale(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn disabled_channel_is_full() {
        let d = Dynamics {
            size_pressure: false,
            strength_pressure: false,
            size_min: 0.1,
            strength_min: 0.1,
        };
        assert!((d.radius_scale(0.0) - 1.0).abs() < 1e-6);
        assert!((d.coverage_scale(0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn size_min_floor_at_zero_pressure() {
        let d = Dynamics { size_pressure: true, size_min: 0.25, ..Default::default() };
        assert!((d.radius_scale(0.0) - 0.25).abs() < 1e-6);
        assert!((d.radius_scale(0.5) - 0.625).abs() < 1e-6); // 0.25 + 0.75*0.5
    }
}
