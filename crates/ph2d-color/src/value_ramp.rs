//! **Value Ramp** — a reusable grayscale gradient that maps a scalar factor `t ∈ [0, 1]` to a single
//! scalar value `[0, 1]` by interpolating between positioned *value stops*. The B&W twin of
//! [`crate::ColorRamp`]: the same stop model + stable ids + Linear / Ease / Cardinal / B-Spline /
//! Constant interpolation ([`RampInterp`]), but one channel instead of RGBA.
//!
//! Lives in `ph2d-color` (not in any one tool) so the **same** object drives a brush Shape's tonal
//! remap, an opacity / mask curve, a height/displacement ramp, a node value map, and design tooling.
//! The hot path bakes a LUT once ([`ValueRamp::bake_into`]) and indexes it per sample.

use crate::color_ramp::RampInterp;
use crate::color_ramp::convert::{bspline_weights, catmull_rom_weights, cub, lerp, smoothstep};

/// Largest number of stops a value ramp can hold (matches [`crate::MAX_RAMP_STOPS`]).
pub const MAX_VALUE_RAMP_STOPS: usize = crate::MAX_RAMP_STOPS;

/// One value stop: a position on the ramp and its scalar value.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ValueStop {
    /// Position on the ramp in `[0, 1]`.
    pub pos: f32,
    /// Scalar value, `[0, 1]`.
    pub value: f32,
    /// **Stable identity**, preserved across re-sorts — so an editor can drag a stop *past* its
    /// neighbours and keep tracking the same stop. Assigned by the owning [`ValueRamp`];
    /// [`ValueStop::new`] leaves it `0` until the ramp adopts the stop.
    pub id: u8,
}

impl ValueStop {
    /// A stop at `pos` with scalar `value` (id `0` until a [`ValueRamp`] adopts it).
    #[must_use]
    pub fn new(pos: f32, value: f32) -> Self {
        Self {
            pos: pos.clamp(0.0, 1.0),
            value: value.clamp(0.0, 1.0),
            id: 0,
        }
    }
}

/// A grayscale gradient of value stops with a chosen interpolation mode. Stops are kept **sorted by
/// position**; there is always at least one. Evaluate with [`Self::eval`] or bake a LUT with
/// [`Self::bake_into`].
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ValueRamp {
    /// Sorted by `pos` (ascending); invariant: non-empty, len ≤ [`MAX_VALUE_RAMP_STOPS`].
    stops: Vec<ValueStop>,
    /// Interpolation mode (Linear / Ease / Cardinal / B-Spline / Constant).
    pub interp: RampInterp,
}

impl Default for ValueRamp {
    /// Identity remap: black at `0.0` → white at `1.0`, Linear (so `eval(t) == t`).
    fn default() -> Self {
        Self::new(
            vec![ValueStop::new(0.0, 0.0), ValueStop::new(1.0, 1.0)],
            RampInterp::Linear,
        )
    }
}

impl ValueRamp {
    /// A ramp from `stops` (cloned, clamped, sorted). Falls back to a single black stop if empty.
    #[must_use]
    pub fn new(mut stops: Vec<ValueStop>, interp: RampInterp) -> Self {
        if stops.is_empty() {
            stops.push(ValueStop::new(0.0, 0.0));
        }
        stops.truncate(MAX_VALUE_RAMP_STOPS);
        let mut ramp = Self { stops, interp };
        ramp.sort();
        for (i, s) in ramp.stops.iter_mut().enumerate() {
            s.id = i as u8;
        }
        ramp
    }

    /// Smallest stable id not currently in use (`0..=255`) — assigned to a freshly added stop.
    fn alloc_id(&self) -> u8 {
        (0u8..=u8::MAX)
            .find(|id| !self.stops.iter().any(|s| s.id == *id))
            .unwrap_or(0)
    }

    /// The stops, sorted by position (read-only).
    #[must_use]
    pub fn stops(&self) -> &[ValueStop] {
        &self.stops
    }

    /// Number of stops (≥ 1).
    #[must_use]
    pub fn len(&self) -> usize {
        self.stops.len()
    }

    /// Always false — a ramp keeps at least one stop (here for clippy / API symmetry).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Insert a stop, keeping the sorted invariant; returns its index. No-op at the cap (returns the
    /// nearest existing stop's index).
    pub fn add_stop(&mut self, stop: ValueStop) -> usize {
        if self.stops.len() >= MAX_VALUE_RAMP_STOPS {
            return self.nearest(stop.pos);
        }
        let mut stop = ValueStop::new(stop.pos, stop.value);
        stop.id = self.alloc_id();
        let idx = self.stops.partition_point(|s| s.pos < stop.pos);
        self.stops.insert(idx, stop);
        idx
    }

    /// Remove the stop at `index` (ignored if it would empty the ramp or is out of range).
    pub fn remove_stop(&mut self, index: usize) {
        if self.stops.len() > 1 && index < self.stops.len() {
            self.stops.remove(index);
        }
    }

    /// Mutate stop `index`'s value; the position is unchanged (so the order is preserved).
    pub fn set_value(&mut self, index: usize, value: f32) {
        if let Some(s) = self.stops.get_mut(index) {
            s.value = value.clamp(0.0, 1.0);
        }
    }

    /// Move stop `index` to `pos` (clamped `[0,1]`), re-sorting; returns the stop's new index. The
    /// stop's value + **stable id are preserved**, so it may be dragged *past* a neighbour and stay the
    /// same stop. Find a stop by id with [`Self::index_of_id`].
    pub fn set_position(&mut self, index: usize, pos: f32) -> usize {
        if index >= self.stops.len() {
            return index;
        }
        let mut stop = self.stops.remove(index);
        stop.pos = pos.clamp(0.0, 1.0);
        let idx = self.stops.partition_point(|s| s.pos < stop.pos);
        self.stops.insert(idx, stop);
        idx
    }

    /// **Invert** the ramp left↔right: mirror every stop's position about the centre (`pos → 1 − pos`),
    /// then re-sort. Values + stable ids are preserved, so an editor's selection survives.
    pub fn invert(&mut self) {
        for s in &mut self.stops {
            s.pos = 1.0 - s.pos;
        }
        self.sort();
    }

    /// The current index of the stop with stable `id`, if present.
    #[must_use]
    pub fn index_of_id(&self, id: u8) -> Option<usize> {
        self.stops.iter().position(|s| s.id == id)
    }

    /// Index of the stop nearest `pos` (used when adding at the cap).
    #[must_use]
    fn nearest(&self, pos: f32) -> usize {
        let mut best = 0;
        let mut best_d = f32::INFINITY;
        for (i, s) in self.stops.iter().enumerate() {
            let d = (s.pos - pos).abs();
            if d < best_d {
                best_d = d;
                best = i;
            }
        }
        best
    }

    fn sort(&mut self) {
        self.stops.sort_by(|a, b| a.pos.total_cmp(&b.pos));
    }

    /// Evaluate the ramp at `t`, returning a scalar value `[0, 1]`. `t` is clamped to `[0, 1]`; outside
    /// the first/last stop the end value is held (no extrapolation). Cubic modes are clamped (Cardinal
    /// can overshoot).
    #[must_use]
    pub fn eval(&self, t: f32) -> f32 {
        let n = self.stops.len();
        let first = self.stops[0];
        if n == 1 || t <= first.pos {
            return first.value;
        }
        let last = self.stops[n - 1];
        if t >= last.pos {
            return last.value;
        }
        let i = self.stops.partition_point(|s| s.pos <= t) - 1;
        let (a, b) = (self.stops[i], self.stops[i + 1]);
        let span = (b.pos - a.pos).max(1e-8);
        let fac = ((t - a.pos) / span).clamp(0.0, 1.0);
        match self.interp {
            RampInterp::Constant => a.value,
            RampInterp::Linear => lerp(a.value, b.value, fac),
            RampInterp::Ease => lerp(a.value, b.value, smoothstep(fac)),
            RampInterp::Cardinal | RampInterp::BSpline => {
                let v0 = self.stops[i.saturating_sub(1)].value;
                let v3 = self.stops[(i + 2).min(n - 1)].value;
                let w = if self.interp == RampInterp::Cardinal {
                    catmull_rom_weights(fac)
                } else {
                    bspline_weights(fac)
                };
                cub(w, v0, a.value, b.value, v3).clamp(0.0, 1.0)
            }
        }
    }

    /// Fill `lut` by sampling the ramp uniformly across `[0, 1]` (`lut[i] = eval(i / (len-1))`). The
    /// hot-path form: bake once, then index `lut[(t * (len-1)) as usize]`.
    pub fn bake_into(&self, lut: &mut [f32]) {
        let n = lut.len();
        if n == 0 {
            return;
        }
        if n == 1 {
            lut[0] = self.eval(0.5);
            return;
        }
        let inv = 1.0 / (n as f32 - 1.0);
        for (i, slot) in lut.iter_mut().enumerate() {
            *slot = self.eval(i as f32 * inv);
        }
    }
}

#[cfg(test)]
mod tests;
