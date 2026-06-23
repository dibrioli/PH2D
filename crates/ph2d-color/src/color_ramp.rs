//! **Color Ramp** — a reusable gradient that maps a scalar factor `t ∈ [0, 1]` to an RGBA color by
//! interpolating between positioned color *stops*. A clean-room model of Blender's *Color Ramp*
//! (`ColorBand`): the same stop model, the RGB / HSV / HSL interpolation spaces, and the Linear /
//! Ease / Cardinal / B-Spline / Constant interpolation modes, plus the HSV/HSL hue paths.
//!
//! This lives in `ph2d-color` (not in any one tool) so the **same** ramp object drives brush
//! textures, gradient fills, node color maps, adjustment LUTs, and design tooling. Colors are stored
//! and interpolated as **linear** RGBA `[f32; 4]` (the crate's math space); a consumer that needs
//! sRGB converts at the boundary. The hot path bakes a 256-entry LUT once ([`ColorRamp::bake_into`])
//! and indexes it per sample.

mod convert;
use convert::{
    bspline_weights, catmull_rom_weights, cub, hsl_to_rgba, hsv_to_rgba, lerp, lerp_hue, lerp4,
    rgb_to_hsl, rgb_to_hsv, smoothstep, unwrap_hues,
};

/// Largest number of stops a ramp can hold (matches Blender's `MAXCOLORBAND`).
pub const MAX_RAMP_STOPS: usize = 32;

/// The space in which adjacent stop colors are interpolated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum RampColorMode {
    /// Interpolate the linear R, G, B channels directly.
    #[default]
    Rgb,
    /// Interpolate in Hue / Saturation / Value (hue takes the [`RampHue`] path).
    Hsv,
    /// Interpolate in Hue / Saturation / Lightness (hue takes the [`RampHue`] path).
    Hsl,
}

/// How the factor between two bracketing stops is shaped (and, for the cubic modes, how four
/// surrounding stops combine). Mirrors Blender's `ipotype`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum RampInterp {
    /// Smoothstep ease-in/out between the two stops.
    Ease,
    /// Catmull–Rom cubic through the stops (overshoots slightly; smooth tangents).
    Cardinal,
    /// Straight linear blend between the two stops.
    #[default]
    Linear,
    /// Uniform cubic B-spline (very smooth; does not pass through the stops).
    BSpline,
    /// No blend — hold the left stop's color until the next (a hard step).
    Constant,
}

/// Which way around the hue circle [`RampColorMode::Hsv`] / [`RampColorMode::Hsl`] interpolates.
/// Mirrors Blender's `ipotype_hue`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum RampHue {
    /// Shortest arc between the two hues.
    #[default]
    Near,
    /// Longest arc between the two hues.
    Far,
    /// Force the clockwise (decreasing-hue) arc.
    Cw,
    /// Force the counter-clockwise (increasing-hue) arc.
    Ccw,
}

impl RampColorMode {
    /// Number of modes (drives a dropdown's decode range).
    pub const COUNT: u8 = 3;
    /// Stable wire discriminant.
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Rgb => 0,
            Self::Hsv => 1,
            Self::Hsl => 2,
        }
    }
    /// Inverse of [`Self::to_u8`]; unknown → [`Self::Rgb`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Hsv,
            2 => Self::Hsl,
            _ => Self::Rgb,
        }
    }
    /// English label for a picker.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Rgb => "RGB",
            Self::Hsv => "HSV",
            Self::Hsl => "HSL",
        }
    }
}

impl RampInterp {
    /// Number of interpolation modes (drives a dropdown's decode range).
    pub const COUNT: u8 = 5;
    /// Stable wire discriminant (ordered as Blender's menu: Ease/Cardinal/Linear/B-Spline/Constant).
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Ease => 0,
            Self::Cardinal => 1,
            Self::Linear => 2,
            Self::BSpline => 3,
            Self::Constant => 4,
        }
    }
    /// Inverse of [`Self::to_u8`]; unknown → [`Self::Linear`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Ease,
            1 => Self::Cardinal,
            3 => Self::BSpline,
            4 => Self::Constant,
            _ => Self::Linear,
        }
    }
    /// English label for a picker.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Ease => "Ease",
            Self::Cardinal => "Cardinal",
            Self::Linear => "Linear",
            Self::BSpline => "B-Spline",
            Self::Constant => "Constant",
        }
    }
}

impl RampHue {
    /// Number of hue paths (drives a dropdown's decode range).
    pub const COUNT: u8 = 4;
    /// Stable wire discriminant.
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Near => 0,
            Self::Far => 1,
            Self::Cw => 2,
            Self::Ccw => 3,
        }
    }
    /// Inverse of [`Self::to_u8`]; unknown → [`Self::Near`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Far,
            2 => Self::Cw,
            3 => Self::Ccw,
            _ => Self::Near,
        }
    }
    /// English label for a picker.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Near => "Near",
            Self::Far => "Far",
            Self::Cw => "CW",
            Self::Ccw => "CCW",
        }
    }
}

/// One color stop: a position on the ramp and its linear RGBA color.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RampStop {
    /// Position on the ramp in `[0, 1]`.
    pub pos: f32,
    /// Linear RGBA color, each channel `[0, 1]`.
    pub color: [f32; 4],
    /// **Stable identity**, preserved across re-sorts — so an editor can drag a stop *past* its
    /// neighbours and keep tracking the same stop. Assigned by the owning [`ColorRamp`];
    /// [`RampStop::new`] leaves it `0` until the ramp adopts the stop.
    pub id: u8,
}

impl RampStop {
    /// A stop at `pos` with linear color `color` (id `0` until a [`ColorRamp`] adopts it).
    #[must_use]
    pub fn new(pos: f32, color: [f32; 4]) -> Self {
        Self {
            pos: pos.clamp(0.0, 1.0),
            color,
            id: 0,
        }
    }
}

/// A gradient of color stops with a chosen interpolation space + mode. Stops are kept **sorted by
/// position**; there is always at least one. Evaluate with [`Self::eval`] or bake a LUT with
/// [`Self::bake_into`].
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ColorRamp {
    /// Sorted by `pos` (ascending); invariant: non-empty, len ≤ [`MAX_RAMP_STOPS`].
    stops: Vec<RampStop>,
    /// Interpolation space (RGB / HSV / HSL).
    pub color_mode: RampColorMode,
    /// Interpolation mode (Linear / Ease / Cardinal / B-Spline / Constant).
    pub interp: RampInterp,
    /// Hue direction for HSV / HSL modes.
    pub hue: RampHue,
}

impl Default for ColorRamp {
    /// Blender's default: black at `0.0` → white at `1.0`, RGB / Linear.
    fn default() -> Self {
        Self::new(
            vec![
                RampStop::new(0.0, [0.0, 0.0, 0.0, 1.0]),
                RampStop::new(1.0, [1.0, 1.0, 1.0, 1.0]),
            ],
            RampColorMode::Rgb,
            RampInterp::Linear,
        )
    }
}

impl ColorRamp {
    /// A ramp from `stops` (cloned, clamped, sorted). Falls back to a single black stop if empty.
    #[must_use]
    pub fn new(mut stops: Vec<RampStop>, color_mode: RampColorMode, interp: RampInterp) -> Self {
        if stops.is_empty() {
            stops.push(RampStop::new(0.0, [0.0, 0.0, 0.0, 1.0]));
        }
        stops.truncate(MAX_RAMP_STOPS);
        let mut ramp = Self {
            stops,
            color_mode,
            interp,
            hue: RampHue::Near,
        };
        ramp.sort();
        // Assign fresh stable ids to the initial stops (callers' `RampStop`s come in with id 0).
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
    pub fn stops(&self) -> &[RampStop] {
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
    pub fn add_stop(&mut self, stop: RampStop) -> usize {
        if self.stops.len() >= MAX_RAMP_STOPS {
            return self.nearest(stop.pos);
        }
        let mut stop = RampStop::new(stop.pos, stop.color);
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

    /// Mutate stop `index`'s color; the position is unchanged (so the order is preserved).
    pub fn set_color(&mut self, index: usize, color: [f32; 4]) {
        if let Some(s) = self.stops.get_mut(index) {
            s.color = color;
        }
    }

    /// Move stop `index` to `pos` (clamped `[0,1]`), re-sorting; returns the stop's new index. The
    /// stop's color + **stable id are preserved**, so it may be dragged *past* a neighbour and stay
    /// the same stop. Find a stop by id with [`Self::index_of_id`].
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

    /// Evaluate the ramp at `t`, returning a linear RGBA color. `t` is clamped to `[0, 1]`; outside
    /// the first/last stop the end color is held (no extrapolation).
    #[must_use]
    pub fn eval(&self, t: f32) -> [f32; 4] {
        let n = self.stops.len();
        let first = self.stops[0];
        if n == 1 || t <= first.pos {
            return first.color;
        }
        let last = self.stops[n - 1];
        if t >= last.pos {
            return last.color;
        }
        // Bracket: stops[i].pos <= t < stops[i+1].pos.
        let i = self.stops.partition_point(|s| s.pos <= t) - 1;
        let (a, b) = (self.stops[i], self.stops[i + 1]);
        let span = (b.pos - a.pos).max(1e-8);
        let fac = ((t - a.pos) / span).clamp(0.0, 1.0);
        match self.interp {
            RampInterp::Constant => a.color,
            RampInterp::Linear => self.mix2(a.color, b.color, fac),
            RampInterp::Ease => self.mix2(a.color, b.color, smoothstep(fac)),
            RampInterp::Cardinal | RampInterp::BSpline => {
                // Four control stops (clamp at the ends), combined by the cubic basis weights.
                let c0 = self.stops[i.saturating_sub(1)].color;
                let c1 = a.color;
                let c2 = b.color;
                let c3 = self.stops[(i + 2).min(n - 1)].color;
                let w = if self.interp == RampInterp::Cardinal {
                    catmull_rom_weights(fac)
                } else {
                    bspline_weights(fac)
                };
                self.cubic(c0, c1, c2, c3, w)
            }
        }
    }

    /// Fill `lut` by sampling the ramp uniformly across `[0, 1]` (`lut[i] = eval(i / (len-1))`). The
    /// hot-path form: bake once, then index `lut[(t * (len-1)) as usize]`.
    pub fn bake_into(&self, lut: &mut [[f32; 4]]) {
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

    /// Two-stop interpolation in the ramp's color mode.
    fn mix2(&self, a: [f32; 4], b: [f32; 4], fac: f32) -> [f32; 4] {
        match self.color_mode {
            RampColorMode::Rgb => lerp4(a, b, fac),
            RampColorMode::Hsv => {
                let (h1, s1, v1) = rgb_to_hsv(a);
                let (h2, s2, v2) = rgb_to_hsv(b);
                let h = lerp_hue(h1, h2, fac, self.hue);
                hsv_to_rgba(
                    h,
                    lerp(s1, s2, fac),
                    lerp(v1, v2, fac),
                    lerp(a[3], b[3], fac),
                )
            }
            RampColorMode::Hsl => {
                let (h1, s1, l1) = rgb_to_hsl(a);
                let (h2, s2, l2) = rgb_to_hsl(b);
                let h = lerp_hue(h1, h2, fac, self.hue);
                hsl_to_rgba(
                    h,
                    lerp(s1, s2, fac),
                    lerp(l1, l2, fac),
                    lerp(a[3], b[3], fac),
                )
            }
        }
    }

    /// Four-stop cubic combine in the ramp's color mode (hues are unwrapped to stay continuous).
    fn cubic(
        &self,
        c0: [f32; 4],
        c1: [f32; 4],
        c2: [f32; 4],
        c3: [f32; 4],
        w: [f32; 4],
    ) -> [f32; 4] {
        match self.color_mode {
            RampColorMode::Rgb => [
                cub(w, c0[0], c1[0], c2[0], c3[0]),
                cub(w, c0[1], c1[1], c2[1], c3[1]),
                cub(w, c0[2], c1[2], c2[2], c3[2]),
                cub(w, c0[3], c1[3], c2[3], c3[3]),
            ],
            RampColorMode::Hsv => {
                let p = [
                    rgb_to_hsv(c0),
                    rgb_to_hsv(c1),
                    rgb_to_hsv(c2),
                    rgb_to_hsv(c3),
                ];
                let hs = unwrap_hues([p[0].0, p[1].0, p[2].0, p[3].0]);
                let h = w[0] * hs[0] + w[1] * hs[1] + w[2] * hs[2] + w[3] * hs[3];
                let s = cub(w, p[0].1, p[1].1, p[2].1, p[3].1);
                let v = cub(w, p[0].2, p[1].2, p[2].2, p[3].2);
                hsv_to_rgba(h.rem_euclid(1.0), s, v, cub(w, c0[3], c1[3], c2[3], c3[3]))
            }
            RampColorMode::Hsl => {
                let p = [
                    rgb_to_hsl(c0),
                    rgb_to_hsl(c1),
                    rgb_to_hsl(c2),
                    rgb_to_hsl(c3),
                ];
                let hs = unwrap_hues([p[0].0, p[1].0, p[2].0, p[3].0]);
                let h = w[0] * hs[0] + w[1] * hs[1] + w[2] * hs[2] + w[3] * hs[3];
                let s = cub(w, p[0].1, p[1].1, p[2].1, p[3].1);
                let l = cub(w, p[0].2, p[1].2, p[2].2, p[3].2);
                hsl_to_rgba(h.rem_euclid(1.0), s, l, cub(w, c0[3], c1[3], c2[3], c3[3]))
            }
        }
    }
}

#[cfg(test)]
mod tests;
