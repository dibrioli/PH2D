use super::*;

/// Sample a half-circle arc (radius 100) as a dense, slightly noisy capture.
fn arc_capture() -> Vec<P> {
    // Transcendental-free sampling: walk a unit half-circle via a fine polyline of precomputed points
    // is overkill; instead trace a parametric arc using the chord-stepped midpoint method. For the test
    // a coarse set of true-arc points (computed offline) is enough — 13 points across a quarter turn.
    let raw: &[P] = &[
        [0.0, 0.0],
        [10.0, 4.0],
        [20.0, 9.0],
        [30.0, 16.0],
        [40.0, 24.0],
        [50.0, 34.0],
        [60.0, 46.0],
        [70.0, 60.0],
        [78.0, 74.0],
        [85.0, 90.0],
        [90.0, 107.0],
        [94.0, 125.0],
        [96.0, 143.0],
    ];
    raw.to_vec()
}

#[test]
fn fit_is_far_fewer_points_than_the_input() {
    let pts = arc_capture();
    let f = fit_curve(&pts, 4.0);
    assert!(
        f.anchors.len() < pts.len(),
        "fit must reduce the point count: {} -> {}",
        pts.len(),
        f.anchors.len()
    );
    assert!(f.anchors.len() >= 2, "always at least the two endpoints");
    assert_eq!(
        f.anchors.len(),
        f.handles.len(),
        "one [in,out] handle pair per anchor"
    );
}

#[test]
fn fit_preserves_the_endpoints() {
    let pts = arc_capture();
    let f = fit_curve(&pts, 6.0);
    assert_eq!(f.anchors[0], pts[0], "first anchor is the start");
    assert_eq!(
        *f.anchors.last().unwrap(),
        *pts.last().unwrap(),
        "last anchor is the end"
    );
}

#[test]
fn larger_tolerance_yields_fewer_or_equal_points() {
    let pts = arc_capture();
    let tight = fit_curve(&pts, 2.0).anchors.len();
    let loose = fit_curve(&pts, 20.0).anchors.len();
    assert!(
        loose <= tight,
        "a looser tolerance must not add points: tight={tight} loose={loose}"
    );
}

#[test]
fn a_straight_line_collapses_to_two_points() {
    let pts: Vec<P> = (0..=20).map(|i| [i as f32 * 5.0, i as f32 * 5.0]).collect();
    assert_eq!(
        fit_curve(&pts, 1.0).anchors.len(),
        2,
        "a clean straight line needs one segment"
    );
}

#[test]
fn degenerate_inputs_pass_through() {
    assert_eq!(fit_curve(&[], 4.0).anchors.len(), 0);
    assert_eq!(fit_curve(&[[1.0, 2.0]], 4.0).anchors.len(), 1);
    assert_eq!(fit_curve(&[[1.0, 2.0], [3.0, 4.0]], 4.0).anchors.len(), 2);
    // Consecutive exact duplicates collapse (degenerate tangents) before fitting.
    let dup = vec![[0.0, 0.0], [0.0, 0.0], [10.0, 0.0], [10.0, 0.0]];
    assert_eq!(fit_curve(&dup, 4.0).anchors, vec![[0.0, 0.0], [10.0, 0.0]]);
}

#[test]
fn fit_is_deterministic() {
    let pts = arc_capture();
    let (a, b) = (fit_curve(&pts, 4.0), fit_curve(&pts, 4.0));
    assert_eq!(a.anchors, b.anchors, "bit-identical anchors (HR-5)");
    assert_eq!(a.handles, b.handles, "bit-identical handles (HR-5)");
}

#[test]
fn flattened_fit_follows_the_captured_points() {
    // The point of the Bézier handles: flattening the fit stays CLOSE to the captured stroke (no
    // Catmull-Rom deformation). Every capture point is within ~tolerance of the flattened spine.
    let pts = arc_capture();
    let f = fit_curve(&pts, 4.0);
    let mut spine = Vec::new();
    flatten_bezier(&f.anchors, &f.handles, &mut spine);
    for &p in &pts {
        let best = spine
            .iter()
            .map(|&s| {
                let d = [s[0] - p[0], s[1] - p[1]];
                d[0] * d[0] + d[1] * d[1]
            })
            .fold(f32::INFINITY, f32::min);
        assert!(
            best.sqrt() <= 6.0,
            "capture point {p:?} is {:.2}px off the spine",
            best.sqrt()
        );
    }
}

#[test]
fn auto_handles_keep_a_straight_run_straight() {
    // Evenly-spaced collinear points → the chordal auto-handles lie ON the line (handle = chord/6), so a
    // straight authored run stays straight (no bulge). Reduces to the uniform Catmull-Rom here.
    let pts: Vec<P> = (0..=4).map(|i| [i as f32 * 10.0, 0.0]).collect();
    let h = auto_handles(&pts);
    assert_eq!(h.len(), pts.len(), "one [in,out] pair per point");
    for (i, pair) in h.iter().enumerate() {
        for hp in pair {
            assert!(
                (hp[1]).abs() < 1e-3,
                "handle {i} left the line: y={}",
                hp[1]
            );
        }
    }
}

#[test]
fn auto_handles_do_not_overshoot_an_uneven_corner() {
    // A sharp corner with a long then a short segment: uniform Catmull-Rom overshoots past the corner;
    // the chordal handles stay within the bounding box of the control points (no loop/overshoot).
    let pts: Vec<P> = vec![[0.0, 0.0], [100.0, 0.0], [105.0, 5.0], [110.0, 100.0]];
    let h = auto_handles(&pts);
    let mut spine = Vec::new();
    flatten_bezier(&pts, &h, &mut spine);
    for &s in &spine {
        assert!(
            (-2.0..=112.0).contains(&s[0]) && (-2.0..=102.0).contains(&s[1]),
            "spine point {s:?} overshot the control polygon's bounds"
        );
    }
}
