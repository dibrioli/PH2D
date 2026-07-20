//! Live **dimensions** for the active Line segment — the CAD-style dx/dy distance annotation between the
//! point being placed and the previous one, plus the raw geometry for the corner angle at the previous
//! vertex. A sibling of [`super::line`] (LOC-cap split). Pure data: this module does NO trigonometry (HR-5
//! — the tool crate stays transcendental-free); it passes the three corner points and the shell overlay
//! computes the angle in degrees for the label (presentation only, off the deterministic path).

/// The active Line segment's live dimensions, image-space. Built by [`compute`] while drawing (never in
/// the editing phase); the shell maps these to screen and draws thin translucent guides in the line
/// colour with px / degree labels, then drops them when the line is finished.
pub struct LineDimensions {
    /// The segment start (also the corner vertex for the angle) — image px.
    pub prev: [f32; 2],
    /// The current / moving point — image px.
    pub cur: [f32; 2],
    /// Horizontal distance `|cur.x − prev.x|` in image px — the dx label.
    pub dx: f32,
    /// Vertical distance `|cur.y − prev.y|` in image px — the dy label.
    pub dy: f32,
    /// The point BEFORE `prev` (the incoming edge's far end) when a prior segment exists, so the shell can
    /// draw the corner angle at `prev` between `angle_from → prev` and `prev → cur`. `None` on the first
    /// segment (no corner yet).
    pub angle_from: Option<[f32; 2]>,
}

/// Live dimensions for the LAST segment of `points` (the active one being drawn): `points[n-2] → [n-1]`.
/// `None` with fewer than two points. No trig — just the distances + the neighbour point for the angle.
pub(super) fn compute(points: &[[f32; 2]]) -> Option<LineDimensions> {
    let n = points.len();
    if n < 2 {
        return None;
    }
    let cur = points[n - 1];
    let prev = points[n - 2];
    Some(LineDimensions {
        prev,
        cur,
        dx: (cur[0] - prev[0]).abs(),
        dy: (cur[1] - prev[1]).abs(),
        angle_from: (n >= 3).then(|| points[n - 3]),
    })
}

#[cfg(test)]
mod tests {
    use super::compute;

    #[test]
    fn dimensions_measure_the_last_segment_and_expose_the_corner() {
        // Fewer than two points → nothing.
        assert!(compute(&[[0.0, 0.0]]).is_none());
        // A single segment: dx/dy of the last segment, no corner.
        let d = compute(&[[10.0, 20.0], [40.0, 60.0]]).expect("two points");
        assert_eq!(d.prev, [10.0, 20.0]);
        assert_eq!(d.cur, [40.0, 60.0]);
        assert_eq!(d.dx, 30.0);
        assert_eq!(d.dy, 40.0);
        assert!(d.angle_from.is_none(), "no corner on the first segment");
        // Three points → the corner's far end is the point before `prev`.
        let d = compute(&[[0.0, 0.0], [10.0, 0.0], [10.0, 25.0]]).expect("three points");
        assert_eq!(d.prev, [10.0, 0.0]);
        assert_eq!(d.cur, [10.0, 25.0]);
        assert_eq!(d.dx, 0.0);
        assert_eq!(d.dy, 25.0);
        assert_eq!(d.angle_from, Some([0.0, 0.0]));
    }
}
