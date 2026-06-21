//! The "Space" stroke engine — turns a pointer path into evenly-spaced dabs.
//!
//! Behavioural reference (clean-room, no code copied): Blender
//! `editors/sculpt_paint/paint_stroke.cc::paint_space_stroke` — dabs are emitted at fixed
//! arc-length intervals (`spacing × diameter`) along the interpolated path, with pen pressure
//! interpolated between input samples. Only the default "Space" method is implemented here;
//! Smooth Stroke (stabilizer) and Airbrush (time-based emission) are deferred (see
//! `docs/Painter/02_plano_de_implementacao.md` T2.3/T2.4).

use crate::dynamics::Dynamics;
use crate::spec::BrushSpec;

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
    /// Per-dab opacity in `[0, 1]` (brush strength × pressure coverage-scale).
    pub coverage: f32,
}

/// Incremental stroke state. Feed it pointer samples; it emits dabs at the brush spacing.
///
/// Usage: [`Stroke::begin`] on pointer-down, [`Stroke::extend`] on every move. Both fill a
/// caller-provided `Vec<Dab>` (cleared first) so a hot pointer loop allocates nothing per call.
#[derive(Clone, Debug)]
pub struct Stroke {
    spec: BrushSpec,
    dynamics: Dynamics,
    last_pos: [f32; 2],
    last_pressure: f32,
    /// Distance travelled since the last emitted dab.
    accum: f32,
    /// State for the dep-free jitter RNG (splitmix64).
    rng: u64,
    started: bool,
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
        }
    }

    /// Update the live brush parameters mid-stroke (e.g. the artist drags the size slider).
    pub fn set_spec(&mut self, spec: BrushSpec) {
        self.spec = spec;
    }

    /// Begin the stroke at `p`, emitting the first dab at the down point.
    pub fn begin(&mut self, p: StrokePoint, out: &mut Vec<Dab>) {
        out.clear();
        self.last_pos = p.pos;
        self.last_pressure = p.pressure;
        self.accum = 0.0;
        self.started = true;
        out.push(self.dab_at(p.pos, p.pressure));
    }

    /// Extend the stroke to `p`, emitting a dab every `spacing × diameter` of arc length since the
    /// last dab. Pressure is interpolated along the segment. No-op until [`Stroke::begin`].
    pub fn extend(&mut self, p: StrokePoint, out: &mut Vec<Dab>) {
        out.clear();
        if !self.started {
            return;
        }
        let from = self.last_pos;
        let to = p.pos;
        let seg = dist(from, to);
        if seg <= f32::EPSILON {
            self.last_pressure = p.pressure;
            return;
        }
        let step = self.spec.dab_spacing_px();
        let dir = [(to[0] - from[0]) / seg, (to[1] - from[1]) / seg];
        let mut traveled = 0.0;
        loop {
            let to_next = step - self.accum;
            if traveled + to_next > seg {
                break;
            }
            traveled += to_next;
            let f = traveled / seg;
            let pos = [from[0] + dir[0] * traveled, from[1] + dir[1] * traveled];
            let pressure = lerp(self.last_pressure, p.pressure, f);
            out.push(self.dab_at(pos, pressure));
            self.accum = 0.0;
        }
        self.accum += seg - traveled;
        self.last_pos = to;
        self.last_pressure = p.pressure;
    }

    /// Build a dab at `pos`/`pressure`, applying pressure dynamics and jitter.
    fn dab_at(&mut self, pos: [f32; 2], pressure: f32) -> Dab {
        let radius = self.spec.clamped_radius() * self.dynamics.radius_scale(pressure);
        let coverage =
            (self.spec.strength * self.dynamics.coverage_scale(pressure)).clamp(0.0, 1.0);
        let center = self.apply_jitter(pos, radius);
        Dab {
            center,
            radius_px: radius,
            coverage,
        }
    }

    /// Offset the dab centre by up to `jitter × radius` in a random direction.
    fn apply_jitter(&mut self, pos: [f32; 2], radius: f32) -> [f32; 2] {
        let j = self.spec.jitter.clamp(0.0, 1.0);
        if j <= 0.0 {
            return pos;
        }
        let angle = self.next_f32() * std::f32::consts::TAU;
        let mag = self.next_f32() * j * radius;
        [pos[0] + angle.cos() * mag, pos[1] + angle.sin() * mag]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::falloff::Falloff;

    fn straight_spec(radius: f32, spacing: f32) -> BrushSpec {
        BrushSpec {
            radius_px: radius,
            spacing,
            falloff: Falloff::Constant,
            ..Default::default()
        }
    }

    fn no_dynamics() -> Dynamics {
        Dynamics {
            size_pressure: false,
            strength_pressure: false,
            ..Default::default()
        }
    }

    #[test]
    fn begin_emits_one_dab_at_down() {
        let mut s = Stroke::new(straight_spec(10.0, 0.5), no_dynamics(), 1);
        let mut out = Vec::new();
        s.begin(
            StrokePoint {
                pos: [5.0, 5.0],
                pressure: 1.0,
            },
            &mut out,
        );
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].center, [5.0, 5.0]);
    }

    #[test]
    fn space_method_emits_at_arc_length_intervals() {
        // radius 10 → diameter 20; spacing 0.5 → step 10 px.
        let mut s = Stroke::new(straight_spec(10.0, 0.5), no_dynamics(), 1);
        let mut out = Vec::new();
        s.begin(
            StrokePoint {
                pos: [0.0, 0.0],
                pressure: 1.0,
            },
            &mut out,
        );
        // Move 100 px along +x: expect dabs at 10,20,...,100 ⟹ 10 dabs.
        s.extend(
            StrokePoint {
                pos: [100.0, 0.0],
                pressure: 1.0,
            },
            &mut out,
        );
        assert_eq!(
            out.len(),
            10,
            "got {:?}",
            out.iter().map(|d| d.center[0]).collect::<Vec<_>>()
        );
        assert!((out[0].center[0] - 10.0).abs() < 1e-3);
        assert!((out[9].center[0] - 100.0).abs() < 1e-3);
        for d in &out {
            assert!((d.center[1]).abs() < 1e-4, "stayed on the x axis");
        }
    }

    #[test]
    fn accumulates_across_short_segments() {
        // step = 10. Two 6-px moves (total 12) ⟹ exactly one dab (at arc-length 10).
        let mut s = Stroke::new(straight_spec(10.0, 0.5), no_dynamics(), 1);
        let mut out = Vec::new();
        s.begin(
            StrokePoint {
                pos: [0.0, 0.0],
                pressure: 1.0,
            },
            &mut out,
        );
        s.extend(
            StrokePoint {
                pos: [6.0, 0.0],
                pressure: 1.0,
            },
            &mut out,
        );
        assert_eq!(out.len(), 0, "6 < 10, no dab yet");
        s.extend(
            StrokePoint {
                pos: [12.0, 0.0],
                pressure: 1.0,
            },
            &mut out,
        );
        assert_eq!(out.len(), 1, "crossed 10 between 6 and 12");
        assert!((out[0].center[0] - 10.0).abs() < 1e-3);
    }

    #[test]
    fn zero_length_move_emits_nothing() {
        let mut s = Stroke::new(straight_spec(10.0, 0.5), no_dynamics(), 1);
        let mut out = Vec::new();
        s.begin(
            StrokePoint {
                pos: [3.0, 3.0],
                pressure: 1.0,
            },
            &mut out,
        );
        s.extend(
            StrokePoint {
                pos: [3.0, 3.0],
                pressure: 1.0,
            },
            &mut out,
        );
        assert_eq!(out.len(), 0);
    }

    #[test]
    fn pressure_interpolates_along_segment() {
        // size follows pressure; check the dab radius grows along the segment.
        let dyn_ = Dynamics {
            size_pressure: true,
            size_min: 0.0,
            ..Default::default()
        };
        let mut s = Stroke::new(straight_spec(10.0, 0.5), dyn_, 1);
        let mut out = Vec::new();
        s.begin(
            StrokePoint {
                pos: [0.0, 0.0],
                pressure: 0.0,
            },
            &mut out,
        );
        s.extend(
            StrokePoint {
                pos: [100.0, 0.0],
                pressure: 1.0,
            },
            &mut out,
        );
        assert!(out.len() >= 2);
        assert!(
            out[0].radius_px < out[out.len() - 1].radius_px,
            "radius rises with pressure"
        );
    }

    #[test]
    fn jitter_is_deterministic_for_a_seed() {
        let spec = BrushSpec {
            jitter: 0.5,
            ..straight_spec(10.0, 0.5)
        };
        let run = || {
            let mut s = Stroke::new(spec, no_dynamics(), 42);
            let mut out = Vec::new();
            s.begin(
                StrokePoint {
                    pos: [50.0, 50.0],
                    pressure: 1.0,
                },
                &mut out,
            );
            s.extend(
                StrokePoint {
                    pos: [150.0, 50.0],
                    pressure: 1.0,
                },
                &mut out,
            );
            out
        };
        assert_eq!(run(), run(), "same seed ⟹ identical jittered dabs");
    }
}
