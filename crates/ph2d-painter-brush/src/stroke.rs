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
use crate::spec::BrushSpec;
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
    /// The point *before* `last_pos` — the trailing neighbour the Catmull-Rom smoother needs for the
    /// centred tangent at the start of each segment. `None` until the second move after
    /// [`Stroke::begin`] (the first segment has no trailing neighbour, so it starts straight).
    prev_prev: Option<[f32; 2]>,
    /// The stabilizer's lazy-mouse filtered position (lags the cursor by the stabilizer intensity).
    /// The path is built from this, not the raw cursor — that is what regularises a shaky hand.
    stab_pos: [f32; 2],
    /// The most recent *raw* (un-stabilized) sample, so [`Stroke::finish`] can catch the lagged
    /// stroke up to the real release point on pointer-up.
    last_raw_pos: [f32; 2],
    last_raw_pressure: f32,
}

/// Smallest lazy-mouse blend factor, reached at stabilizer `1.0` (heaviest smoothing / most lag).
/// At stabilizer `0.0` the factor is `1.0` (the filtered point IS the cursor — no stabilization).
const STABILIZER_MIN_BLEND: f32 = 0.08;

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
            prev_prev: None,
            stab_pos: [0.0, 0.0],
            last_raw_pos: [0.0, 0.0],
            last_raw_pressure: 1.0,
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
        self.prev_prev = None;
        self.stab_pos = p.pos;
        self.last_raw_pos = p.pos;
        self.last_raw_pressure = p.pressure;
        self.sampler.reset(p);
        self.started = true;
        if self.spec.stroke_method.emits_on_begin() {
            let pr = self.method_pressure(p.pressure);
            let dab = self.dab_at(p.pos, pr, self.method_overlap());
            out.push(dab);
            self.tot_samples = self.tot_samples.wrapping_add(1);
        }
    }

    /// Extend the stroke to the raw sample `raw`: average it into the input-sample window, run it
    /// through the stabilizer, then emit dabs per the stroke method. No-op until [`Stroke::begin`].
    pub fn extend(&mut self, raw: StrokePoint, out: &mut Vec<Dab>) {
        out.clear();
        if !self.started {
            return;
        }
        let avg = self.sampler.push_average(raw);
        self.last_raw_pos = avg.pos;
        self.last_raw_pressure = avg.pressure;
        let target = self.stabilize(avg);
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

    /// Close the stroke on pointer-up. With the stabilizer on, the painted path lags the cursor, so
    /// flush the lag: walk the spline from the last filtered point up to the true release point so
    /// the stroke ends exactly where the pen lifted (Space only — the per-event methods stamp at the
    /// cursor each event and have nothing pending). Then clear the smoother state.
    pub fn finish(&mut self, out: &mut Vec<Dab>) {
        out.clear();
        if self.started && self.spec.stroke_method == StrokeMethod::Space {
            self.walk_smoothed(
                StrokePoint {
                    pos: self.last_raw_pos,
                    pressure: self.last_raw_pressure,
                },
                out,
            );
        }
        self.prev_prev = None;
    }

    /// Lazy-mouse stabilizer: blend the running filtered position [`Self::stab_pos`] toward the
    /// incoming sample by `1 − intensity` (clamped to a floor), so a higher intensity lags more and
    /// filters out hand tremor. At intensity `0` the filtered point is the sample itself (raw,
    /// real-time). Position only — pressure passes through.
    fn stabilize(&mut self, sample: StrokePoint) -> StrokePoint {
        let s = self.spec.stabilizer.clamp(0.0, 1.0);
        if s <= f32::EPSILON {
            self.stab_pos = sample.pos;
            return sample;
        }
        let blend = 1.0 - s * (1.0 - STABILIZER_MIN_BLEND);
        self.stab_pos = [
            self.stab_pos[0] + (sample.pos[0] - self.stab_pos[0]) * blend,
            self.stab_pos[1] + (sample.pos[1] - self.stab_pos[1]) * blend,
        ];
        StrokePoint {
            pos: self.stab_pos,
            pressure: sample.pressure,
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

    /// Freehand smoother for the `Space` method — a **Catmull-Rom spline** through the input points.
    /// Each `extend` paints the segment from the last point `a` to the new point `p`, so the stroke
    /// follows the cursor in real time (no held-back tail), and the spline interpolates *through*
    /// every sample with a smooth, continuous tangent, so sparse / coalesced input reads as a clean
    /// curve instead of the connected straight facets the old scheme produced.
    ///
    /// The segment `a → p` is a cubic Hermite with the Catmull-Rom tangents: at `a` the centripetal
    /// tangent `(p − prev_prev)/2` (its two neighbours — smooth join with the previous segment), and
    /// at `p` the causal chord `p − a` (there is no next sample yet). The first segment after
    /// [`Stroke::begin`] has no `prev_prev`, so its start tangent is the chord (straight). The cubic
    /// is flattened into short chords fed through [`Stroke::walk_space`] (which owns spacing / dash /
    /// jitter / attenuation and chains via `last_pos`). Collinear input keeps the tangents on the
    /// line, so straight strokes stay straight.
    fn walk_smoothed(&mut self, p: StrokePoint, out: &mut Vec<Dab>) {
        let a = self.last_pos;
        let a_pr = self.last_pressure;
        let b = p.pos;
        let seg = dist(a, b);
        if seg <= f32::EPSILON {
            self.last_pressure = p.pressure;
            return;
        }
        // Catmull-Rom tangents (at `a`: centred on its neighbours `prev_prev` and `b`; the first
        // segment uses the chord. at `b`: the causal chord), scaled by the stabilizer intensity `w`:
        // `w = 0` ⇒ zero tangents ⇒ the Hermite is the straight chord `a→b` (raw, faceted path);
        // `w → 1` ⇒ full curvature between samples. So the one knob ramps from raw to smooth.
        let w = self.spec.stabilizer.clamp(0.0, 1.0);
        let m_a = match self.prev_prev {
            Some(pp) => [(b[0] - pp[0]) * 0.5 * w, (b[1] - pp[1]) * 0.5 * w],
            None => [(b[0] - a[0]) * w, (b[1] - a[1]) * w],
        };
        let m_b = [(b[0] - a[0]) * w, (b[1] - a[1]) * w];
        // Flatten the Hermite `a → b` into short chords (denser than the dab spacing so the curve
        // never facets), each chained through `walk_space`.
        let n = ((seg / 3.0).ceil() as usize).clamp(1, 96);
        for i in 1..=n {
            let t = i as f32 / n as f32;
            self.walk_space(
                StrokePoint {
                    pos: hermite(a, m_a, b, m_b, t),
                    pressure: lerp(a_pr, p.pressure, t),
                },
                out,
            );
        }
        // This segment's start point becomes the next segment's `prev_prev` neighbour.
        self.prev_prev = Some(a);
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

/// Cubic Hermite at `t∈[0,1]`: endpoints `p0`,`p1` with tangents `m0`,`m1`. With Catmull-Rom
/// tangents this evaluates the spline segment between two input points.
#[inline]
fn hermite(p0: [f32; 2], m0: [f32; 2], p1: [f32; 2], m1: [f32; 2], t: f32) -> [f32; 2] {
    let t2 = t * t;
    let t3 = t2 * t;
    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;
    [
        h00 * p0[0] + h10 * m0[0] + h01 * p1[0] + h11 * m1[0],
        h00 * p0[1] + h10 * m0[1] + h01 * p1[1] + h11 * m1[1],
    ]
}

#[cfg(test)]
mod tests;
