//! Unit tests for the shared [`CurveModel`] editing core — the behaviour BOTH the stroke Shape curve and
//! the selection Convert-to-Curve editor inherit. These pin the ops in one place so the two owners can never
//! drift (Enio's mandate: one system).

use super::*;

/// A dense closed square outline (many points along the edges) — stands in for a raw lasso / traced contour.
fn dense_square(step: f32) -> Vec<[f32; 2]> {
    let mut p = Vec::new();
    let side = 100.0;
    let mut x = 0.0;
    while x < side {
        p.push([x, 0.0]);
        x += step;
    }
    let mut y = 0.0;
    while y < side {
        p.push([side, y]);
        y += step;
    }
    let mut x = side;
    while x > 0.0 {
        p.push([x, side]);
        x -= step;
    }
    let mut y = side;
    while y > 0.0 {
        p.push([0.0, y]);
        y -= step;
    }
    p
}

#[test]
fn from_fit_collapses_a_dense_outline_to_sparse_anchors() {
    // The "muitos pontos" fix: a dense outline fits to a handful of anchors, not the raw contour count.
    let dense = dense_square(2.0);
    assert!(
        dense.len() > 100,
        "the raw outline is dense: {}",
        dense.len()
    );
    let m = CurveModel::from_fit(&dense, &[], true, 4.0, 64).expect("fit");
    assert!(
        m.is_curve(),
        "the fit yields an editable curve (handles present)"
    );
    assert!(
        m.points.len() < 20,
        "fit is sparse: {} anchors from {} points",
        m.points.len(),
        dense.len()
    );
    assert_eq!(m.kinds.len(), m.points.len(), "a kind per anchor");
    assert!(m.closed);
}

#[test]
fn all_five_handle_kinds_apply_and_a_tangent_drag_mirrors() {
    let dense = dense_square(2.0);
    let mut m = CurveModel::from_fit(&dense, &[], true, 4.0, 64).expect("fit");
    m.selected = Some(1);
    for wire in 0..=4u8 {
        assert!(m.set_kind(wire), "kind {wire} applies");
        assert_eq!(m.selected_kind_wire(), Some(wire));
    }
    // Symmetric (wire 4): dragging the OUT handle reflects the IN exactly through the anchor.
    assert!(m.set_kind(4));
    let anchor = m.points[1];
    m.drag(
        CurveGrab::Tangent(1, true),
        [anchor[0] + 10.0, anchor[1] + 5.0],
    );
    let h = m.handles[1];
    assert!(
        (h[0][0] - (anchor[0] - 10.0)).abs() < 1e-3 && (h[0][1] - (anchor[1] - 5.0)).abs() < 1e-3,
        "symmetric IN is the exact reflection of OUT: {h:?}"
    );
}

#[test]
fn insert_then_delete_round_trips_the_anchor_count() {
    let dense = dense_square(2.0);
    let mut m = CurveModel::from_fit(&dense, &[], true, 4.0, 64).expect("fit");
    let n = m.points.len();
    let mid = [50.0, 0.0]; // on the bottom edge
    let idx = m.insert(mid);
    assert_eq!(m.points.len(), n + 1);
    m.selected = Some(idx);
    assert!(m.delete_selected());
    assert_eq!(
        m.points.len(),
        n,
        "insert+delete returns to the original count"
    );
}

#[test]
fn raw_lasso_is_not_a_curve() {
    let m = CurveModel::raw_lasso(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]], true);
    assert!(
        !m.is_curve(),
        "no handles → transform-box mode, not the point editor"
    );
    assert!(m.closed, "a selection lasso is a closed loop");
}
