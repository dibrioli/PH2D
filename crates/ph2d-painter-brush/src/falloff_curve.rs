//! Editable custom falloff curve — the [`Falloff::Custom`](crate::Falloff::Custom) profile.
//!
//! Behavioural reference (clean-room, no code copied): Blender's brush falloff
//! "Custom" `CurveMapping` (`blenkernel/intern/colortools.cc`
//! `BKE_curvemapping_evaluate`, sampled into the per-texel dab mask by
//! `editors/sculpt_paint/mesh/paint_image_2d_curve_mask.cc`). Blender's
//! `CurveMapping` is a general spline; this port uses a MONOTONE cubic-Hermite
//! (Fritsch–Carlson) spline — the same well-behaved evaluation the Painter's
//! adjustment Curves editor uses — so a falloff never wiggles out of its control
//! points' range. `x` is the normalized distance from the dab centre (`0` =
//! centre, `1` = rim); `y` is the strength.

/// Maximum control points in a custom falloff curve (matches the Painter's
/// adjustment-curve cap; keeps [`FalloffCurve`] `Copy` and alloc-free).
pub const MAX_FALLOFF_POINTS: usize = 8;

/// An editable custom falloff profile: 2..=[`MAX_FALLOFF_POINTS`] control points
/// `[distance, strength]`, ascending by distance, evaluated as a monotone
/// cubic-Hermite spline. Cheap to copy — the stroke engine reads it per dab.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FalloffCurve {
    points: [[f32; 2]; MAX_FALLOFF_POINTS],
    len: u8,
}

impl Default for FalloffCurve {
    /// A linear ramp from full strength at the centre to zero at the rim
    /// (`(0,1) → (1,0)`) — the clearest editable starting shape (matches
    /// [`Falloff::Linear`](crate::Falloff::Linear)).
    fn default() -> Self {
        let mut points = [[0.0; 2]; MAX_FALLOFF_POINTS];
        points[0] = [0.0, 1.0];
        points[1] = [1.0, 0.0];
        Self { points, len: 2 }
    }
}

impl FalloffCurve {
    /// The active control points `[distance, strength]`, ascending by distance.
    #[must_use]
    pub fn points(&self) -> &[[f32; 2]] {
        &self.points[..self.len as usize]
    }

    /// Number of active control points (`2..=MAX_FALLOFF_POINTS`).
    #[must_use]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// A falloff curve always keeps ≥2 points, so this is never `true`; provided
    /// only to satisfy `clippy::len_without_is_empty`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Strength at normalized distance `t` (`0` = centre, `1` = rim). Forced to
    /// `0` at/after the rim (`t >= 1`) so a dab never paints beyond its radius,
    /// matching the preset [`Falloff::weight`](crate::Falloff::weight).
    #[must_use]
    pub fn weight(&self, t: f32) -> f32 {
        if t >= 1.0 {
            return 0.0;
        }
        eval_falloff_curve(self.points(), t.max(0.0))
    }

    /// Move control point `index` to `[x, y]` (clamped to `[0,1]²` and between
    /// its neighbours so the points stay ordered without re-sorting — keeping the
    /// dragged handle's index stable across the gesture, mirror of the Painter's
    /// `set_curve_point`). No-op out of range.
    pub fn set_point(&mut self, index: usize, x: f32, y: f32) {
        let n = self.len as usize;
        if index >= n {
            return;
        }
        let left = if index == 0 {
            0.0
        } else {
            self.points[index - 1][0]
        };
        let right = if index + 1 == n {
            1.0
        } else {
            self.points[index + 1][0]
        };
        self.points[index][0] = x.clamp(0.0, 1.0).clamp(left, right);
        self.points[index][1] = y.clamp(0.0, 1.0);
    }

    /// Insert a control point at the midpoint of the widest distance-gap, its
    /// strength sampled ON the current curve (so the profile is unchanged until
    /// the new point is dragged). Returns the inserted index, or `None` at the
    /// [`MAX_FALLOFF_POINTS`] cap.
    pub fn add_point(&mut self) -> Option<usize> {
        let n = self.len as usize;
        if !(2..MAX_FALLOFF_POINTS).contains(&n) {
            return None;
        }
        let mut best_gap = -1.0_f32;
        let mut new_x = 0.5_f32;
        let mut insert_at = n;
        for i in 0..n - 1 {
            let gap = self.points[i + 1][0] - self.points[i][0];
            if gap > best_gap {
                best_gap = gap;
                new_x = (self.points[i][0] + self.points[i + 1][0]) * 0.5;
                insert_at = i + 1;
            }
        }
        let new_y = eval_falloff_curve(self.points(), new_x);
        for j in (insert_at..n).rev() {
            self.points[j + 1] = self.points[j];
        }
        self.points[insert_at] = [new_x, new_y];
        self.len += 1;
        Some(insert_at)
    }

    /// Remove control point `index`. No-op when only the two endpoints remain (a
    /// curve needs ≥2 points) or out of range.
    pub fn remove_point(&mut self, index: usize) {
        let n = self.len as usize;
        if n <= 2 || index >= n {
            return;
        }
        for j in index..n - 1 {
            self.points[j] = self.points[j + 1];
        }
        self.points[n - 1] = [0.0, 0.0];
        self.len -= 1;
    }
}

/// Fritsch–Carlson monotone tangent at control point `i` of `points` — the
/// Hermite slope used by the segments on either side, clamped against the
/// adjacent secants so the spline stays monotone (a falloff must not wiggle out
/// of its control points' range). Standard published construction.
fn monotone_tangent(points: &[[f32; 2]], i: usize) -> f32 {
    let n = points.len();
    let secant = |a: usize, b: usize| {
        let dx = points[b][0] - points[a][0];
        if dx.abs() <= 1e-9 {
            0.0
        } else {
            (points[b][1] - points[a][1]) / dx
        }
    };
    let mut m = if i == 0 {
        secant(0, 1)
    } else if i == n - 1 {
        secant(n - 2, n - 1)
    } else {
        0.5 * (secant(i - 1, i) + secant(i, i + 1))
    };
    let neighbors = [
        (i > 0).then(|| secant(i - 1, i)),
        (i + 1 < n).then(|| secant(i, i + 1)),
    ];
    for d in neighbors.into_iter().flatten() {
        if d == 0.0 {
            m = 0.0;
        } else {
            let r = m / d;
            if r < 0.0 {
                m = 0.0;
            } else if r > 3.0 {
                m = 3.0 * d;
            }
        }
    }
    m
}

/// Evaluate a falloff curve (`[distance, strength]` points, ascending distance)
/// at `t`. Empty → `1` (full); one point → its strength; ≥2 → a monotone
/// cubic-Hermite spline ([`monotone_tangent`]), endpoints extended flat. The
/// panel plots the live preview with this, so the graph matches the dab.
#[must_use]
pub fn eval_falloff_curve(points: &[[f32; 2]], t: f32) -> f32 {
    match points.len() {
        0 => return 1.0,
        1 => return points[0][1].clamp(0.0, 1.0),
        _ => {}
    }
    let t = t.clamp(0.0, 1.0);
    let n = points.len();
    if t <= points[0][0] {
        return points[0][1].clamp(0.0, 1.0);
    }
    if t >= points[n - 1][0] {
        return points[n - 1][1].clamp(0.0, 1.0);
    }
    let mut i = 0;
    while i + 1 < n && points[i + 1][0] < t {
        i += 1;
    }
    let (x0, y0) = (points[i][0], points[i][1]);
    let (x1, y1) = (points[i + 1][0], points[i + 1][1]);
    let h = x1 - x0;
    if h <= 1e-9 {
        return y1.clamp(0.0, 1.0);
    }
    let m0 = monotone_tangent(points, i);
    let m1 = monotone_tangent(points, i + 1);
    let s = (t - x0) / h;
    let (s2, s3) = (s * s, s * s * s);
    let h00 = 2.0 * s3 - 3.0 * s2 + 1.0;
    let h10 = s3 - 2.0 * s2 + s;
    let h01 = -2.0 * s3 + 3.0 * s2;
    let h11 = s3 - s2;
    (h00 * y0 + h10 * h * m0 + h01 * y1 + h11 * h * m1).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_linear_ramp() {
        let c = FalloffCurve::default();
        assert_eq!(c.len(), 2);
        assert!((c.weight(0.0) - 1.0).abs() < 1e-6, "centre full");
        assert_eq!(c.weight(1.0), 0.0, "rim zero (forced)");
        // Linear midpoint.
        assert!((c.weight(0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn weight_forces_zero_at_and_past_rim() {
        let mut c = FalloffCurve::default();
        // Even a curve whose last point is high gets cut at the rim.
        c.set_point(1, 1.0, 1.0);
        assert_eq!(c.weight(1.0), 0.0);
        assert_eq!(c.weight(1.5), 0.0);
    }

    #[test]
    fn add_then_remove_round_trips_len() {
        let mut c = FalloffCurve::default();
        let before = c.weight(0.5);
        let idx = c.add_point().expect("inserted");
        assert_eq!(c.len(), 3);
        // The new point sits ON the curve → profile unchanged until dragged.
        assert!((c.weight(0.5) - before).abs() < 1e-3);
        c.remove_point(idx);
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn add_point_caps_at_max() {
        let mut c = FalloffCurve::default();
        while c.add_point().is_some() {}
        assert_eq!(c.len(), MAX_FALLOFF_POINTS);
        assert!(c.add_point().is_none(), "no insert past the cap");
    }

    #[test]
    fn remove_keeps_two_endpoints() {
        let mut c = FalloffCurve::default();
        c.remove_point(0);
        assert_eq!(c.len(), 2, "a curve needs ≥2 points");
    }

    #[test]
    fn set_point_clamps_between_neighbours() {
        let mut c = FalloffCurve::default();
        c.add_point(); // now 3 points; middle at x≈0.5
        // Drag the middle point far right past the rim point — it clamps to the
        // right neighbour's x (1.0), staying ordered.
        c.set_point(1, 9.0, 0.5);
        let pts = c.points();
        assert!(pts[1][0] <= pts[2][0], "stays ordered after a wild drag");
        assert!((0.0..=1.0).contains(&pts[1][1]));
    }

    #[test]
    fn shape_is_monotone_non_increasing_for_a_falling_curve() {
        let c = FalloffCurve::default();
        let mut prev = c.weight(0.0);
        let mut t = 0.0;
        while t <= 0.999 {
            let w = c.weight(t);
            assert!(w <= prev + 1e-6, "monotone at t={t}: {w} > {prev}");
            prev = w;
            t += 0.01;
        }
    }
}
