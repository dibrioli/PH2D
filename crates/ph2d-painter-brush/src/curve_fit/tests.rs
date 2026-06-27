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
    let anchors = fit_curve(&pts, 4.0);
    assert!(
        anchors.len() < pts.len(),
        "fit must reduce the point count: {} -> {}",
        pts.len(),
        anchors.len()
    );
    assert!(anchors.len() >= 2, "always at least the two endpoints");
}

#[test]
fn fit_preserves_the_endpoints() {
    let pts = arc_capture();
    let anchors = fit_curve(&pts, 6.0);
    assert_eq!(anchors[0], pts[0], "first anchor is the start");
    assert_eq!(
        *anchors.last().unwrap(),
        *pts.last().unwrap(),
        "last anchor is the end"
    );
}

#[test]
fn larger_tolerance_yields_fewer_or_equal_points() {
    let pts = arc_capture();
    let tight = fit_curve(&pts, 2.0).len();
    let loose = fit_curve(&pts, 20.0).len();
    assert!(
        loose <= tight,
        "a looser tolerance must not add points: tight={tight} loose={loose}"
    );
}

#[test]
fn a_straight_line_collapses_to_two_points() {
    let pts: Vec<P> = (0..=20).map(|i| [i as f32 * 5.0, i as f32 * 5.0]).collect();
    let anchors = fit_curve(&pts, 1.0);
    assert_eq!(anchors.len(), 2, "a clean straight line needs one segment");
}

#[test]
fn degenerate_inputs_pass_through() {
    assert_eq!(fit_curve(&[], 4.0).len(), 0);
    assert_eq!(fit_curve(&[[1.0, 2.0]], 4.0).len(), 1);
    assert_eq!(fit_curve(&[[1.0, 2.0], [3.0, 4.0]], 4.0).len(), 2);
    // Consecutive exact duplicates collapse (degenerate tangents) before fitting.
    let dup = vec![[0.0, 0.0], [0.0, 0.0], [10.0, 0.0], [10.0, 0.0]];
    assert_eq!(fit_curve(&dup, 4.0), vec![[0.0, 0.0], [10.0, 0.0]]);
}

#[test]
fn fit_is_deterministic() {
    let pts = arc_capture();
    assert_eq!(
        fit_curve(&pts, 4.0),
        fit_curve(&pts, 4.0),
        "bit-identical re-run (HR-5)"
    );
}
