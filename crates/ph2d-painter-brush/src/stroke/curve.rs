//! The **Curve** stroke method's geometry: the shared Catmull-Rom flattener and the spaced-dab fill
//! along it. A child module of [`super`] (`stroke`) so it keeps private access to `Stroke`'s walk
//! internals (`walk_space`/`dab_at`/`method_*`) and the `dist`/`hermite` free fns; split out to keep
//! `stroke.rs` under the workspace LOC cap. See `stroke_method::StrokeMethod::Curve` for the model:
//! the artist authors control points and the engine auto-smooths between them.

use super::*;

impl Stroke {
    /// Fill an authored Curve through the control points `pts` with spaced dabs for the live preview:
    /// auto-smooth the polygon with a Catmull-Rom spline ([`flatten_catmull_rom`]) and lay dabs at the
    /// brush spacing along it (constant full pressure — the curve is a geometric path, no per-point
    /// pen pressure), gated by dash, with jitter. Spacing carries continuously across the whole spine
    /// (not reset per segment), so the dab cadence reads as one curve. Self-contained: it resets the
    /// walk state from `pts[0]` and does NOT restore it, so the tool builds a FRESH `Stroke` per
    /// re-fill (each move) — identical control points ⇒ identical dabs (no shimmer), and the live
    /// brush spec (size / spacing / dash) is always reflected. The tool routes the result through the
    /// restore+re-stamp preview, so the growing/reshaping curve leaves no trail; commit keeps the last
    /// fill. `< 2` points fills nothing.
    pub fn fill_curve_preview(&mut self, pts: &[[f32; 2]], out: &mut Vec<Dab>) {
        out.clear();
        if pts.len() < 2 {
            return;
        }
        let mut spine = Vec::new();
        flatten_catmull_rom(pts, &mut spine);
        self.last_pos = spine[0];
        self.last_pressure = 1.0;
        self.accum = 0.0;
        self.tot_samples = 0;
        // First dab at the curve start (dash-gated), like `fill_segment`.
        if self.spec.dash_on(self.tot_samples) {
            let pr = self.method_pressure(1.0);
            let d = self.dab_at(spine[0], pr, self.method_overlap());
            out.push(d);
        }
        self.tot_samples = self.tot_samples.wrapping_add(1);
        // Walk the flattened spine with continuous spacing (`walk_space` carries `accum`/dash/jitter
        // and chains via `last_pos`), so the whole curve dabs as one path.
        for &p in &spine[1..] {
            self.walk_space(
                StrokePoint {
                    pos: p,
                    pressure: 1.0,
                },
                out,
            );
        }
    }
}

/// Flatten a **Catmull-Rom spline** through `pts` (the authored Curve control points, clamped
/// endpoints) into a dense polyline, written to `out` (cleared first; starts at `pts[0]`). Each
/// segment `pts[i] → pts[i+1]` is the Hermite with uniform Catmull-Rom tangents `(next − prev)/2`
/// (the trailing/leading neighbour clamped to the endpoint), subdivided finer than the dab spacing
/// so the curve never facets. Collinear control points keep the tangents on the line, so a straight
/// run stays straight (the initial 3-point line reads as a line until a point is dragged off it).
///
/// Shared by [`Stroke::fill_curve_preview`] (the painted dabs) and the tool's editor overlay (the
/// drawn spine), so the on-screen guide matches the dabs exactly.
pub fn flatten_catmull_rom(pts: &[[f32; 2]], out: &mut Vec<[f32; 2]>) {
    out.clear();
    if pts.is_empty() {
        return;
    }
    out.push(pts[0]);
    let last = pts.len() - 1;
    for i in 0..last {
        let p0 = pts[i];
        let p1 = pts[i + 1];
        let prev = if i == 0 { p0 } else { pts[i - 1] };
        let next = if i + 2 <= last { pts[i + 2] } else { p1 };
        let m0 = [(p1[0] - prev[0]) * 0.5, (p1[1] - prev[1]) * 0.5];
        let m1 = [(next[0] - p0[0]) * 0.5, (next[1] - p0[1]) * 0.5];
        let seg = dist(p0, p1);
        let n = ((seg / 3.0).ceil() as usize).clamp(1, 96);
        for k in 1..=n {
            let t = k as f32 / n as f32;
            out.push(hermite(p0, m0, p1, m1, t));
        }
    }
}
