//! The stroke engine — turns a pointer path into dabs per the Blender "Stroke" panel.
//!
//! Behavioural reference (clean-room, no code copied): Blender
//! `editors/sculpt_paint/paint_stroke.cc`. The per-raw-sample pipeline is:
//! **input-samples box-average** ([`crate::sampler`]) → **stabilize spring** (smooth-stroke,
//! dead-zone + lerp) → per-[`StrokeMethod`] emission → per-dab **dash gate** + **jitter** +
//! pressure **dynamics** + **space-attenuation**. `Space` resamples the path at `spacing ×
//! diameter`; the per-event methods (`Dots`/`DragDot`/`Airbrush`) emit one dab per processed
//! sample. The *interactive* methods (`Anchored`/`Line`/`Curve` — live preview + finalise) are
//! owned by the tool/shell; the engine exposes [`Stroke::fill_segment`] (line/curve fill) and
//! [`Stroke::tick`] (airbrush timer) as the primitives those drive. See
//! `docs/Painter/03_algoritmos_referencia_blender.md` §3.

use crate::dynamics::Dynamics;
use crate::sampler::InputSampler;
use crate::spec::{
    BrushSpec, SMOOTH_FACTOR_MAX, SMOOTH_FACTOR_MIN, SMOOTH_RADIUS_MAX_PX, SMOOTH_RADIUS_MIN_PX,
};
use crate::stroke_method::{JitterUnit, StrokeMethod};

/// One input sample from the pointer device, in image-space pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrokePoint {
    /// Position in image pixels.
    pub pos: [f32; 2],
    /// Pen pressure in `[0, 1]`; `1.0` for devices without pressure (mouse).
    pub pressure: f32,
}

/// One dab to stamp: where, how big, and how opaque. The falloff profile / blend / colour come
/// from the [`BrushSpec`]; only the pressure-varying parts live here.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dab {
    /// Centre in image pixels.
    pub center: [f32; 2],
    /// Radius in pixels (brush radius × pressure size-scale).
    pub radius_px: f32,
    /// Per-dab opacity in `[0, 1]` (brush strength × pressure coverage-scale × space-attenuation).
    pub coverage: f32,
}

/// Incremental stroke state. Feed it pointer samples; it emits dabs per the brush's stroke method.
///
/// Usage: [`Stroke::begin`] on pointer-down, [`Stroke::extend`] on every move. Both fill a
/// caller-provided `Vec<Dab>` (cleared first) so a hot pointer loop allocates nothing per call.
#[derive(Clone, Debug)]
pub struct Stroke {
    spec: BrushSpec,
    dynamics: Dynamics,
    /// Last point the path was walked to — the spacing segment start AND the stabilize spring
    /// anchor (Blender's `last_mouse_position`).
    last_pos: [f32; 2],
    last_pressure: f32,
    /// Distance travelled since the last emitted dab (carried across segments).
    accum: f32,
    /// State for the dep-free jitter RNG (splitmix64).
    rng: u64,
    started: bool,
    /// Input-samples box-average ring (front of the pipeline).
    sampler: InputSampler,
    /// Monotonic emitted-dab-slot counter that drives the dash pattern.
    tot_samples: u32,
    /// Cached "Adjust Strength for Spacing" multiplier (recomputed on every spec change).
    overlap: f32,
    /// Airbrush time accumulator in seconds, consumed by [`Stroke::tick`].
    airbrush_accum_s: f32,
    /// Previous input point for the freehand path smoother (quadratic-midpoint). `None` until the
    /// first move after [`Stroke::begin`]. The smoother turns the sparse, possibly-faceted input
    /// polyline into a curve so the stroke reads smooth regardless of input sample density.
    sp_prev: Option<StrokePoint>,
}

impl Stroke {
    /// Create a stroke for `spec`/`dynamics`. `seed` seeds the jitter RNG (use a per-stroke
    /// counter; never a global/thread RNG — keeps replays reproducible, HR-5).
    #[must_use]
    pub fn new(spec: BrushSpec, dynamics: Dynamics, seed: u64) -> Self {
        Self {
            spec,
            dynamics,
            last_pos: [0.0, 0.0],
            last_pressure: 1.0,
            accum: 0.0,
            rng: seed ^ 0x9E37_79B9_7F4A_7C15,
            started: false,
            sampler: InputSampler::new(spec.input_samples),
            tot_samples: 0,
            overlap: spec.space_overlap_factor(),
            airbrush_accum_s: 0.0,
            sp_prev: None,
        }
    }

    /// Update the live brush parameters mid-stroke (e.g. the artist drags a slider). Recomputes the
    /// cached space-attenuation factor and re-clamps the input-samples window.
    pub fn set_spec(&mut self, spec: BrushSpec) {
        self.spec = spec;
        self.overlap = spec.space_overlap_factor();
        self.sampler.set_window(spec.input_samples);
    }

    /// Begin the stroke at `p`. For the continuous methods this emits the first dab at the down
    /// point; the interactive methods (Anchored/Line/Curve) only record the anchor (they paint on
    /// finalise via [`Stroke::fill_segment`]).
    pub fn begin(&mut self, p: StrokePoint, out: &mut Vec<Dab>) {
        out.clear();
        self.last_pos = p.pos;
        self.last_pressure = p.pressure;
        self.accum = 0.0;
        self.tot_samples = 0;
        self.airbrush_accum_s = 0.0;
        self.sp_prev = None;
        self.sampler.reset(p);
        self.started = true;
        if self.spec.stroke_method.emits_on_begin() {
            let pr = self.method_pressure(p.pressure);
            let dab = self.dab_at(p.pos, pr, self.method_overlap());
            out.push(dab);
            self.tot_samples = self.tot_samples.wrapping_add(1);
        }
    }

    /// Extend the stroke to the raw sample `raw`: average it into the input-sample window, apply
    /// the stabilize spring, then emit dabs per the stroke method. No-op until [`Stroke::begin`].
    pub fn extend(&mut self, raw: StrokePoint, out: &mut Vec<Dab>) {
        out.clear();
        if !self.started {
            return;
        }
        let avg = self.sampler.push_average(raw);
        let target = match self.stabilized(avg) {
            Some(t) => t,
            None => return, // inside the stabilize dead-zone — no dab this event
        };
        match self.spec.stroke_method {
            StrokeMethod::Space => self.walk_smoothed(target, out),
            StrokeMethod::Dots | StrokeMethod::Airbrush => {
                self.emit_single(target.pos, self.method_pressure(target.pressure), out);
                self.advance_anchor(target);
            }
            StrokeMethod::DragDot => {
                self.emit_single(target.pos, 1.0, out);
                self.advance_anchor(target);
            }
            // Interactive: the engine doesn't paint per-move. The tool drives `fill_segment` /
            // a resized stamp on finalise; we still track the anchor so a preview can read it.
            StrokeMethod::Anchored | StrokeMethod::Line | StrokeMethod::Curve => {
                self.advance_anchor(target);
            }
        }
    }

    /// Fill the straight segment `a → b` with spaced dabs (Blender LINE / CURVE-segment finalise).
    /// Pressure is constant (Blender fills lines/curves at pressure `1.0`). Appends to `out` (the
    /// caller clears) and continues the dash counter so a multi-segment curve dashes continuously.
    pub fn fill_segment(&mut self, a: [f32; 2], b: [f32; 2], pressure: f32, out: &mut Vec<Dab>) {
        self.last_pos = a;
        self.last_pressure = pressure;
        self.accum = 0.0;
        if self.spec.dash_on(self.tot_samples) {
            let pr = self.method_pressure(pressure);
            let d = self.dab_at(a, pr, self.method_overlap());
            out.push(d);
        }
        self.tot_samples = self.tot_samples.wrapping_add(1);
        self.walk_space(StrokePoint { pos: b, pressure }, out);
    }

    /// Flush the freehand smoother's tail — the final straight bit from the last emitted midpoint
    /// to the final input point. Call once on pointer-up (after the last [`Stroke::extend`]) so the
    /// stroke reaches the cursor's release position. No-op for non-`Space` methods.
    pub fn finish(&mut self, out: &mut Vec<Dab>) {
        out.clear();
        if !self.started {
            return;
        }
        if let Some(prev) = self.sp_prev.take() {
            self.walk_space(prev, out);
        }
    }

    /// Advance the airbrush timer by `dt` seconds, emitting a dab at the current cursor every
    /// `rate` seconds (Blender airbrush TIMER). No-op unless the method is [`StrokeMethod::Airbrush`].
    /// The tool drives this from its tick while the button is held and the cursor is parked.
    pub fn tick(&mut self, dt: f32, out: &mut Vec<Dab>) {
        out.clear();
        if !self.started || self.spec.stroke_method != StrokeMethod::Airbrush {
            return;
        }
        let rate = self.spec.airbrush_rate_s.max(1e-3);
        self.airbrush_accum_s += dt.max(0.0);
        while self.airbrush_accum_s >= rate {
            self.airbrush_accum_s -= rate;
            let pr = self.method_pressure(self.last_pressure);
            let d = self.dab_at(self.last_pos, pr, 1.0); // per-event: no spacing attenuation
            out.push(d);
            self.tot_samples = self.tot_samples.wrapping_add(1);
        }
    }

    // ── internals ───────────────────────────────────────────────────────────────────

    /// Apply the stabilize spring to the averaged sample. Returns `None` when the cursor is still
    /// inside the dead-zone (skip this event entirely — what lets sharp turns settle).
    fn stabilized(&mut self, avg: StrokePoint) -> Option<StrokePoint> {
        if !self.spec.smooth_stroke || !self.spec.stroke_method.supports_smooth() {
            return Some(avg);
        }
        // Clamp to Blender's hard RNA range so a save/LLM-authored value can't drop the stabilizer
        // into the sub-floor regime that tracks the raw cursor (the "not fluid at low values" bug).
        let r = self
            .spec
            .smooth_radius_px
            .clamp(SMOOTH_RADIUS_MIN_PX, SMOOTH_RADIUS_MAX_PX);
        if dist(self.last_pos, avg.pos) < r {
            return None;
        }
        let u = self
            .spec
            .smooth_factor
            .clamp(SMOOTH_FACTOR_MIN, SMOOTH_FACTOR_MAX);
        Some(StrokePoint {
            pos: lerp2(avg.pos, self.last_pos, u),
            pressure: lerp(avg.pressure, self.last_pressure, u),
        })
    }

    /// Space spacing walk from `last_pos` → `target`, emitting a dab every `spacing × diameter` of
    /// arc length and carrying the residual distance across calls (`accum`).
    fn walk_space(&mut self, target: StrokePoint, out: &mut Vec<Dab>) {
        let from = self.last_pos;
        let to = target.pos;
        let seg = dist(from, to);
        if seg <= f32::EPSILON {
            self.last_pressure = target.pressure;
            return;
        }
        let step = self.spec.dab_spacing_px();
        let dir = [(to[0] - from[0]) / seg, (to[1] - from[1]) / seg];
        let overlap = self.method_overlap();
        let mut traveled = 0.0;
        loop {
            let to_next = step - self.accum;
            if traveled + to_next > seg {
                break;
            }
            traveled += to_next;
            let f = traveled / seg;
            let pos = [from[0] + dir[0] * traveled, from[1] + dir[1] * traveled];
            let pressure = lerp(self.last_pressure, target.pressure, f);
            if self.spec.dash_on(self.tot_samples) {
                let d = self.dab_at(pos, pressure, overlap);
                out.push(d);
            }
            self.tot_samples = self.tot_samples.wrapping_add(1);
            self.accum = 0.0;
        }
        self.accum += seg - traveled;
        self.last_pos = to;
        self.last_pressure = target.pressure;
    }

    /// Freehand path smoother for the `Space` method: turns the input polyline into a quadratic
    /// curve so it never reads as straight chords, regardless of how sparse / faceted the input
    /// samples are (low frame-rate, the stabilize dead-zone, coalesced device events).
    ///
    /// Quadratic-midpoint scheme (the standard smooth-freehand technique): each input point is the
    /// Bézier *control*; consecutive segment midpoints are the curve endpoints. So a new point `p`
    /// paints the quadratic from the previous midpoint through the previous point to
    /// `mid(prev, p)`, with one input-point of lag (flushed by [`Stroke::finish`] on pointer-up).
    /// The curve is flattened into short chords and fed through [`Stroke::walk_space`], which keeps
    /// the spacing / dash / jitter / attenuation behaviour and chains via `last_pos`. Collinear
    /// input collapses to a straight line (no change for straight strokes).
    fn walk_smoothed(&mut self, p: StrokePoint, out: &mut Vec<Dab>) {
        match self.sp_prev {
            None => {
                // Warm-up: straight from the down point to the first segment's midpoint.
                let mid = StrokePoint {
                    pos: midpoint(self.last_pos, p.pos),
                    pressure: 0.5 * (self.last_pressure + p.pressure),
                };
                self.walk_space(mid, out);
                self.sp_prev = Some(p);
            }
            Some(prev) => {
                let pen = self.last_pos;
                let pen_pr = self.last_pressure;
                let end = StrokePoint {
                    pos: midpoint(prev.pos, p.pos),
                    pressure: 0.5 * (prev.pressure + p.pressure),
                };
                // Flatten the quadratic pen → (control = prev) → end into ~4 px chords, feeding
                // each through `walk_space` (which chains off `last_pos`).
                let approx_len = dist(pen, prev.pos) + dist(prev.pos, end.pos);
                let n = ((approx_len / 4.0).ceil() as usize).clamp(1, 64);
                for i in 1..=n {
                    let t = i as f32 / n as f32;
                    self.walk_space(
                        StrokePoint {
                            pos: quad_bezier(pen, prev.pos, end.pos, t),
                            pressure: lerp(pen_pr, end.pressure, t),
                        },
                        out,
                    );
                }
                self.sp_prev = Some(p);
            }
        }
    }

    /// Emit one dab at `pos`/`pressure` (the per-event methods). Per-event dabs carry no
    /// space-attenuation (that normalises *dense spacing*, which these don't have).
    fn emit_single(&mut self, pos: [f32; 2], pressure: f32, out: &mut Vec<Dab>) {
        let d = self.dab_at(pos, pressure, 1.0);
        out.push(d);
        self.tot_samples = self.tot_samples.wrapping_add(1);
    }

    /// Move the spacing/spring anchor to `target` without resetting the spacing residual (used by
    /// the per-event + interactive methods, which don't accumulate distance).
    fn advance_anchor(&mut self, target: StrokePoint) {
        self.last_pos = target.pos;
        self.last_pressure = target.pressure;
    }

    /// Pressure to use given the method (DragDot/Anchored/Line force full pressure).
    fn method_pressure(&self, pressure: f32) -> f32 {
        if self.spec.stroke_method.forces_full_pressure() {
            1.0
        } else {
            pressure
        }
    }

    /// Space-attenuation multiplier for the current method — applied to the spaced fills
    /// (Space/Line/Curve), `1.0` for the per-event methods.
    fn method_overlap(&self) -> f32 {
        match self.spec.stroke_method {
            StrokeMethod::Space | StrokeMethod::Line | StrokeMethod::Curve => self.overlap,
            _ => 1.0,
        }
    }

    /// Build a dab at `pos`/`pressure`, applying pressure dynamics, the space-attenuation
    /// `overlap` multiplier, and jitter.
    fn dab_at(&mut self, pos: [f32; 2], pressure: f32, overlap: f32) -> Dab {
        let radius = self.spec.clamped_radius() * self.dynamics.radius_scale(pressure);
        let coverage =
            (self.spec.strength * self.dynamics.coverage_scale(pressure) * overlap).clamp(0.0, 1.0);
        let center = self.apply_jitter(pos, radius);
        Dab {
            center,
            radius_px: radius,
            coverage,
        }
    }

    /// Offset the dab centre by a random vector sampled uniformly inside a disc, sized by the
    /// jitter unit: `Brush` → radius `jitter × diameter`; `View` → radius `2 × jitter_absolute_px`
    /// (Blender `BKE_brush_jitter_pos`). No-op for methods that disable jitter (DragDot/Anchored).
    fn apply_jitter(&mut self, pos: [f32; 2], radius: f32) -> [f32; 2] {
        if !self.spec.stroke_method.allows_jitter() {
            return pos;
        }
        let max_offset = match self.spec.jitter_unit {
            JitterUnit::Brush => self.spec.jitter.clamp(0.0, 1.0) * (2.0 * radius),
            JitterUnit::View => 2.0 * self.spec.jitter_absolute_px.max(0.0),
        };
        if max_offset <= 0.0 {
            return pos;
        }
        let (dx, dy) = self.disc_sample();
        [pos[0] + dx * max_offset, pos[1] + dy * max_offset]
    }

    /// Uniform sample inside the unit disc by rejection (matches Blender's jitter sampling).
    fn disc_sample(&mut self) -> (f32, f32) {
        loop {
            let x = self.next_f32() * 2.0 - 1.0;
            let y = self.next_f32() * 2.0 - 1.0;
            if x * x + y * y <= 1.0 {
                return (x, y);
            }
        }
    }

    /// Dep-free deterministic `[0,1)` RNG (splitmix64 → top 24 bits).
    fn next_f32(&mut self) -> f32 {
        self.rng = self.rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        // Top 24 bits → [0,1).
        ((z >> 40) as f32) / ((1u32 << 24) as f32)
    }
}

#[inline]
fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    (dx * dx + dy * dy).sqrt()
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline]
fn lerp2(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    [lerp(a[0], b[0], t), lerp(a[1], b[1], t)]
}

#[inline]
fn midpoint(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]
}

/// Quadratic Bézier `B(t) = (1−t)² p0 + 2(1−t)t c + t² p1`.
#[inline]
fn quad_bezier(p0: [f32; 2], c: [f32; 2], p1: [f32; 2], t: f32) -> [f32; 2] {
    let mt = 1.0 - t;
    let a = mt * mt;
    let b = 2.0 * mt * t;
    let d = t * t;
    [
        a * p0[0] + b * c[0] + d * p1[0],
        a * p0[1] + b * c[1] + d * p1[1],
    ]
}

#[cfg(test)]
mod tests;
