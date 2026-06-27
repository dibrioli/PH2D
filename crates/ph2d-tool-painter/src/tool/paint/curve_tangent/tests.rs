//! Unit tests for the Bézier tangent-handle geometry (hit-test, aligned mirror, overlay snapshot).

use super::*;

/// A 3-anchor curve with handles pulled out a clear distance, so the interior anchor's tangents are
/// grabbable and well clear of the `tol` shadow.
fn sample() -> (Vec<[f32; 2]>, Vec<[[f32; 2]; 2]>) {
    let points = vec![[0.0, 0.0], [100.0, 0.0], [200.0, 0.0]];
    let handles = vec![
        [[0.0, 0.0], [40.0, 0.0]],      // anchor 0: in unused, out pulled +x
        [[60.0, -30.0], [140.0, 30.0]], // anchor 1: both pulled out, off-axis
        [[160.0, 0.0], [200.0, 0.0]],   // anchor 2: out unused
    ];
    (points, handles)
}

#[test]
fn tangent_hit_grabs_the_out_handle_near_it() {
    let (p, h) = sample();
    // Near anchor 1's out handle [140,30].
    let hit = tangent_hit(&p, &h, 1, [141.0, 29.0], 6.0);
    assert_eq!(hit, Some((1, true)));
}

#[test]
fn tangent_hit_grabs_the_in_handle_near_it() {
    let (p, h) = sample();
    let hit = tangent_hit(&p, &h, 1, [59.0, -31.0], 6.0);
    assert_eq!(hit, Some((1, false)));
}

#[test]
fn tangent_hit_misses_far_from_any_handle() {
    let (p, h) = sample();
    assert_eq!(tangent_hit(&p, &h, 1, [100.0, 100.0], 6.0), None);
}

#[test]
fn tangent_hit_ignores_the_unused_endpoint_side() {
    let (p, h) = sample();
    // Anchor 0's IN handle is at the anchor (unused) — a point there must not grab a tangent.
    assert_eq!(tangent_hit(&p, &h, 0, [0.0, 0.0], 6.0), None);
    // Anchor 2's OUT handle is unused (last anchor) — sitting on it grabs nothing.
    assert_eq!(tangent_hit(&p, &h, 2, [200.0, 0.0], 6.0), None);
}

#[test]
fn tangent_hit_does_not_steal_a_handle_collapsed_on_the_anchor() {
    let points = vec![[0.0, 0.0], [100.0, 0.0], [200.0, 0.0]];
    // Anchor 1 sharp: both handles on the anchor.
    let handles = vec![
        [[0.0, 0.0], [0.0, 0.0]],
        [[100.0, 0.0], [100.0, 0.0]],
        [[200.0, 0.0], [200.0, 0.0]],
    ];
    assert_eq!(tangent_hit(&points, &handles, 1, [100.0, 0.0], 6.0), None);
}

#[test]
fn mirror_keeps_the_opposite_collinear_and_preserves_its_length() {
    let anchor = [100.0, 0.0];
    // out pulled to [140,30] (just set); in currently [60,-40] → length 50 along (-40,-40)... use a clean one.
    let mut h = [[80.0, 0.0], [140.0, 0.0]]; // in length 20 (-x), out length 40 (+x), already collinear
    // Move out off-axis, then mirror should swing in to the exact opposite direction, length 20 kept.
    h[1] = [100.0 + 30.0, 40.0]; // out = anchor + (30,40), length 50
    mirror_tangent(&mut h, anchor, true, false);
    // in must be anchor - unit(30,40)*20 = anchor - (0.6,0.8)*20 = anchor-(12,16)
    let din = [h[0][0] - anchor[0], h[0][1] - anchor[1]];
    let len = (din[0] * din[0] + din[1] * din[1]).sqrt();
    assert!((len - 20.0).abs() < 1e-3, "preserved length: {len}");
    // Collinear & opposite: in direction == -out direction.
    assert!(
        (din[0] - (-12.0)).abs() < 1e-3 && (din[1] - (-16.0)).abs() < 1e-3,
        "in={din:?}"
    );
}

#[test]
fn mirror_leaves_a_zero_length_opposite_collapsed() {
    let anchor = [50.0, 50.0];
    let mut h = [[50.0, 50.0], [90.0, 50.0]]; // in collapsed on anchor, out pulled
    h[1] = [50.0 + 10.0, 20.0];
    mirror_tangent(&mut h, anchor, true, false);
    assert_eq!(
        h[0], anchor,
        "a sharp side stays sharp (no fabricated tangent)"
    );
}

#[test]
fn symmetric_mirror_reflects_the_opposite_with_equal_length() {
    let anchor = [100.0, 0.0];
    let mut h = [[80.0, 0.0], [140.0, 0.0]]; // in length 20, out length 40 (different)
    h[1] = [100.0 + 30.0, 40.0]; // out pulled to anchor+(30,40), length 50
    mirror_tangent(&mut h, anchor, true, true); // Symmetric: in becomes the exact reflection
    assert_eq!(
        h[0],
        [100.0 - 30.0, -40.0],
        "in is the exact mirror of out (equal length)"
    );
}

#[test]
fn build_tangents_exposes_both_sides_of_an_interior_anchor() {
    let (p, h) = sample();
    let t = build_tangents(&p, &h, 1, None, 6.0).expect("interior anchor has handles");
    assert_eq!(t.anchor, [100.0, 0.0]);
    assert_eq!(t.in_handle, Some([60.0, -30.0]));
    assert_eq!(t.out_handle, Some([140.0, 30.0]));
    assert_eq!(t.grabbed_out, None);
}

#[test]
fn build_tangents_marks_the_grabbed_side() {
    let (p, h) = sample();
    let t = build_tangents(&p, &h, 1, Some((1, true)), 6.0).unwrap();
    assert_eq!(t.grabbed_out, Some(true));
}

#[test]
fn build_tangents_is_none_for_a_sharp_collapsed_anchor() {
    let points = vec![[0.0, 0.0], [100.0, 0.0], [200.0, 0.0]];
    let handles = vec![
        [[0.0, 0.0], [0.0, 0.0]],
        [[100.0, 0.0], [100.0, 0.0]],
        [[200.0, 0.0], [200.0, 0.0]],
    ];
    assert!(build_tangents(&points, &handles, 1, None, 6.0).is_none());
}

#[test]
fn build_tangents_drops_the_unused_endpoint_side() {
    let (p, h) = sample();
    // Anchor 0: in unused → only the out handle is exposed.
    let t = build_tangents(&p, &h, 0, None, 6.0).unwrap();
    assert_eq!(t.in_handle, None);
    assert_eq!(t.out_handle, Some([40.0, 0.0]));
}
