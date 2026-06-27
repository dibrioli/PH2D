//! Unit tests for the per-anchor handle-kind model + derived geometry.

use super::*;

#[test]
fn wire_round_trips_every_kind() {
    for k in [
        HandleKind::Free,
        HandleKind::Aligned,
        HandleKind::Symmetric,
        HandleKind::Vector,
        HandleKind::Auto,
    ] {
        assert_eq!(HandleKind::from_wire(k.wire()), Some(k));
    }
    assert_eq!(HandleKind::from_wire(5), None);
}

#[test]
fn only_free_aligned_and_symmetric_are_manual() {
    assert!(HandleKind::Free.is_manual());
    assert!(HandleKind::Symmetric.is_manual());
    assert!(HandleKind::Aligned.is_manual());
    assert!(!HandleKind::Vector.is_manual());
    assert!(!HandleKind::Auto.is_manual());
}

#[test]
fn vector_handles_point_one_third_toward_the_neighbours() {
    let pts = [[0.0, 0.0], [30.0, 0.0], [60.0, 0.0]];
    let h = vector_handle(&pts, 1, false);
    // in toward pts[0] (−x), out toward pts[2] (+x), each 1/3 of the 30px gap = 10px.
    assert_eq!(h[0], [20.0, 0.0]);
    assert_eq!(h[1], [40.0, 0.0]);
}

#[test]
fn vector_endpoint_sides_collapse_onto_the_anchor() {
    let pts = [[0.0, 0.0], [30.0, 0.0], [60.0, 0.0]];
    let first = vector_handle(&pts, 0, false);
    assert_eq!(
        first[0],
        [0.0, 0.0],
        "first anchor has no incoming neighbour"
    );
    assert_eq!(first[1], [10.0, 0.0]);
    let last = vector_handle(&pts, 2, false);
    assert_eq!(
        last[1],
        [60.0, 0.0],
        "last anchor has no outgoing neighbour"
    );
}

#[test]
fn closed_loop_seam_anchors_keep_both_arms() {
    // Regression (Enio 2026-06-28): a converted Polygon / Circle is a CLOSED loop, so the seam (first / last)
    // anchor must get BOTH tangent arms — the open-curve build collapses one side → a one-armed handle that
    // can't be made symmetric. Vector + Auto both wrap their neighbours when `closed`.
    let pts = [[0.0, 0.0], [30.0, 0.0], [60.0, 0.0]];
    let first = vector_handle(&pts, 0, true);
    assert_ne!(
        first[0],
        [0.0, 0.0],
        "closed: first anchor's in-arm wraps to the last point"
    );
    assert_ne!(
        first[1],
        [0.0, 0.0],
        "closed: first anchor's out-arm is present"
    );
    let last = vector_handle(&pts, 2, true);
    assert_ne!(
        last[1],
        [60.0, 0.0],
        "closed: last anchor's out-arm wraps to the first point"
    );
    // Auto (cardinal-spline) likewise gives every anchor two arms off the wrapped neighbours.
    let auto = auto_handles(&pts, true);
    for (i, h) in auto.iter().enumerate() {
        assert_ne!(h[0], pts[i], "closed Auto in-arm present at {i}");
        assert_ne!(h[1], pts[i], "closed Auto out-arm present at {i}");
    }
}

#[test]
fn rebuild_recomputes_auto_and_vector_but_preserves_manual() {
    let pts = vec![[0.0, 0.0], [30.0, 10.0], [60.0, 0.0]];
    let kinds = vec![HandleKind::Auto, HandleKind::Free, HandleKind::Vector];
    // Seed a clearly-bogus Free handle that rebuild must NOT touch, and bogus Auto/Vector it MUST replace.
    let mut handles = vec![
        [[99.0, 99.0], [99.0, 99.0]], // Auto — should be overwritten by chordal
        [[7.0, 7.0], [8.0, 8.0]],     // Free — must be preserved verbatim
        [[99.0, 99.0], [99.0, 99.0]], // Vector — should become neighbour-pointing
    ];
    rebuild(&pts, &kinds, &mut handles, false);
    assert_eq!(handles[1], [[7.0, 7.0], [8.0, 8.0]], "Free preserved");
    assert_ne!(handles[0], [[99.0, 99.0], [99.0, 99.0]], "Auto recomputed");
    // Vector at idx 2: in toward pts[1] (1/3 of (-30,10)) = (-10, 3.333) added to (60,0).
    let v = handles[2][0];
    assert!(
        (v[0] - 50.0).abs() < 1e-3 && (v[1] - 10.0 / 3.0).abs() < 1e-3,
        "Vector in = {v:?}"
    );
}

#[test]
fn rebuild_sizes_an_empty_handle_buffer() {
    let pts = vec![[0.0, 0.0], [10.0, 0.0]];
    let kinds = vec![HandleKind::Auto, HandleKind::Auto];
    let mut handles = Vec::new();
    rebuild(&pts, &kinds, &mut handles, false);
    assert_eq!(handles.len(), 2, "empty buffer is sized to the point count");
}

#[test]
fn rebuild_is_a_noop_on_a_length_mismatch() {
    let pts = vec![[0.0, 0.0], [10.0, 0.0]];
    let kinds = vec![HandleKind::Auto]; // mismatched
    let mut handles = vec![[[1.0, 1.0], [2.0, 2.0]]];
    rebuild(&pts, &kinds, &mut handles, false);
    assert_eq!(
        handles,
        vec![[[1.0, 1.0], [2.0, 2.0]]],
        "untouched on mismatch"
    );
}

#[test]
fn is_collapsed_detects_a_sharp_point() {
    let anchor = [5.0, 5.0];
    assert!(is_collapsed(Some(&[[5.0, 5.0], [5.0, 5.0]]), anchor));
    assert!(is_collapsed(None, anchor));
    assert!(!is_collapsed(Some(&[[5.0, 5.0], [9.0, 5.0]]), anchor));
}
