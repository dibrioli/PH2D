//! The **Ellipse** stroke method's geometry: a transcendental-free ellipse perimeter generator and
//! the spaced-dab fill along it. A child module of [`super`] (`stroke`) so it keeps private access
//! to `Stroke`'s walk internals; split out for the workspace LOC cap. See
//! `stroke_method::StrokeMethod::Ellipse` — a PH2D shape extension (editable ellipse: 4 axis handles
//! + rotation).
//!
//! ## Why no `sin`/`cos`
//!
//! `sin`/`cos`/`atan2` are libm-dependent and NOT bit-reproducible across platforms (HR-5), so the
//! perimeter is built WITHOUT them: start from the four EXACT cardinal unit vectors and recursively
//! insert arc midpoints via `normalize(a + b)` (which lands on the unit circle, using only `sqrt` —
//! IEEE-correctly-rounded, deterministic). The ellipse orientation is carried as a unit VECTOR `u`
//! (from the rotation handle / `normalize`), never an angle, so rotation is transcendental-free too.

use super::*;

impl Stroke {
    /// Fill the perimeter of the ellipse `(center, u, rx, ry)` with spaced dabs for the live preview:
    /// `u` is the local x-axis unit vector (orientation), `rx`/`ry` the half-axes. Lays dabs at the
    /// brush spacing continuously around the closed loop (gated by dash, with jitter), at constant
    /// full pressure (a geometric path). Self-contained + fresh-per-fill like
    /// [`Stroke::fill_curve_preview`]: identical params ⇒ identical dabs (no shimmer). The tool routes
    /// the result through the restore + re-stamp preview, so the reshaping ellipse leaves no trail;
    /// commit keeps the last fill. A degenerate (`< 0.5 px` either axis) ellipse fills nothing. (The Offset
    /// slider grows/shrinks `rx`/`ry` in the tool, so the fill stays a plain perimeter walk.)
    pub fn fill_ellipse_preview(
        &mut self,
        center: [f32; 2],
        u: [f32; 2],
        rx: f32,
        ry: f32,
        out: &mut Vec<Dab>,
    ) {
        out.clear();
        if rx < 0.5 || ry < 0.5 {
            return;
        }
        let mut perim = Vec::new();
        ellipse_perimeter(center, u, rx, ry, &mut perim);
        if perim.len() < 2 {
            return;
        }
        self.last_pos = perim[0];
        self.last_pressure = 1.0;
        self.accum = 0.0;
        self.tot_samples = 0;
        self.heading = [0.0, 0.0]; // fresh fill → the Rake heading re-aims along the perimeter from the start
        self.arc_len = 0.0; // fresh fill → the Flow along-coordinate starts at the perimeter's origin
        // First dab at the perimeter start (dash-gated), like `fill_segment`.
        if self.spec.dash_on(self.tot_samples) {
            let pr = self.method_pressure(1.0);
            let d = self.dab_at(perim[0], pr, self.method_overlap(), self.arc_len);
            out.push(d);
        }
        self.tot_samples = self.tot_samples.wrapping_add(1);
        // Walk the perimeter with continuous spacing, then close the loop back to the start.
        for &p in &perim[1..] {
            self.walk_space(
                StrokePoint {
                    pos: p,
                    pressure: 1.0,
                },
                out,
            );
        }
        self.walk_space(
            StrokePoint {
                pos: perim[0],
                pressure: 1.0,
            },
            out,
        );
    }
}

/// Generate the closed perimeter of the ellipse `(center, u, rx, ry)` as a dense polyline written to
/// `out` (cleared first; the start point is NOT duplicated at the end — the caller closes the loop).
/// `u` is the local x-axis unit vector; the y-axis is its left perpendicular. Transcendental-free
/// (module header): unit-circle points are the four cardinals refined by `normalize`-midpoint
/// subdivision, the depth scaling with the radius so big circles stay smooth without faceting.
///
/// Shared by [`Stroke::fill_ellipse_preview`] (the painted dabs) and the tool's editor overlay (the
/// drawn outline), so the on-screen guide matches the dabs exactly.
pub fn ellipse_perimeter(center: [f32; 2], u: [f32; 2], rx: f32, ry: f32, out: &mut Vec<[f32; 2]>) {
    out.clear();
    let un = unit_or(u, [1.0, 0.0]); // local x-axis (orientation), defended against a zero vector
    let v = [-un[1], un[0]]; // local y-axis (left perpendicular)
    // Subdivision depth from the larger radius — more segments for bigger circles so the chord
    // deviation stays sub-pixel. A pure integer ladder (no `log`), so it's transcendental-free.
    let max_r = rx.max(ry);
    let depth: usize = if max_r > 512.0 {
        8
    } else if max_r > 256.0 {
        7
    } else if max_r > 128.0 {
        6
    } else if max_r > 48.0 {
        5
    } else if max_r > 16.0 {
        4
    } else {
        3
    };
    // Unit circle: the four exact cardinals, then `depth` rounds of arc-midpoint insertion. After
    // `depth` rounds there are `4 * 2^depth` points evenly around the circle.
    let mut unit: Vec<[f32; 2]> = vec![[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]];
    for _ in 0..depth {
        let mut next = Vec::with_capacity(unit.len() * 2);
        for i in 0..unit.len() {
            let a = unit[i];
            let b = unit[(i + 1) % unit.len()];
            next.push(a);
            next.push(unit_or([a[0] + b[0], a[1] + b[1]], a)); // arc midpoint on the unit circle
        }
        unit = next;
    }
    // Map each unit point onto the ellipse: center + rx·cx·u + ry·cy·v.
    out.reserve(unit.len());
    for c in unit {
        out.push([
            center[0] + rx * c[0] * un[0] + ry * c[1] * v[0],
            center[1] + rx * c[0] * un[1] + ry * c[1] * v[1],
        ]);
    }
}

/// Normalize `p` to a unit vector, falling back to `fallback` when `p` is (near) zero-length. Uses
/// only `sqrt` (deterministic), never `atan2`/trig.
fn unit_or(p: [f32; 2], fallback: [f32; 2]) -> [f32; 2] {
    let len = (p[0] * p[0] + p[1] * p[1]).sqrt();
    if len > 1e-6 {
        [p[0] / len, p[1] / len]
    } else {
        fallback
    }
}
