use super::*;

#[test]
fn demo_has_fill_and_stroke_paths() {
    let scene = VecScene::demo();
    assert_eq!(scene.paths().len(), 2);
    assert!(scene.paths()[0].fill.is_some() && scene.paths()[0].closed);
    assert!(scene.paths()[1].stroke.is_some() && !scene.paths()[1].closed);
}

#[test]
fn push_path_assigns_monotonic_ids() {
    let mut scene = VecScene::new();
    let a = scene.push_path(VecPath {
        id: 999,
        verts: vec![VecVertex::corner([0.0, 0.0])],
        closed: false,
        fill: None,
        stroke: None,
    });
    let b = scene.push_path(VecPath {
        id: 999,
        verts: vec![VecVertex::corner([1.0, 1.0])],
        closed: false,
        fill: None,
        stroke: None,
    });
    assert_eq!((a, b), (0, 1));
    assert_eq!(scene.paths()[0].id, 0);
}

fn path_at(p: [f64; 2]) -> VecPath {
    VecPath {
        id: 0,
        verts: vec![VecVertex::corner(p)],
        closed: false,
        fill: None,
        stroke: None,
    }
}

#[test]
fn reorder_path_moves_within_the_stack() {
    use ZOrder::*;
    let mut scene = VecScene::new();
    let a = scene.push_path(path_at([0.0, 0.0]));
    let b = scene.push_path(path_at([1.0, 0.0]));
    let c = scene.push_path(path_at([2.0, 0.0]));
    let order = |s: &VecScene| s.paths().iter().map(|p| p.id).collect::<Vec<_>>();
    assert_eq!(order(&scene), vec![a, b, c]);

    assert!(scene.reorder_path(a, Raise));
    assert_eq!(order(&scene), vec![b, a, c]);
    assert!(scene.reorder_path(a, Lower));
    assert_eq!(order(&scene), vec![a, b, c]);
    assert!(scene.reorder_path(a, ToFront));
    assert_eq!(order(&scene), vec![b, c, a]);
    assert!(scene.reorder_path(a, ToBack));
    assert_eq!(order(&scene), vec![a, b, c]);

    // Edge no-ops: already at the extreme, or unknown id.
    assert!(!scene.reorder_path(a, ToBack));
    assert!(!scene.reorder_path(a, Lower));
    assert!(!scene.reorder_path(c, ToFront));
    assert!(!scene.reorder_path(c, Raise));
    assert!(!scene.reorder_path(999, Raise));
    assert_eq!(order(&scene), vec![a, b, c]);
}

#[test]
fn duplicate_path_offsets_every_point_and_stacks_on_top() {
    let mut scene = VecScene::new();
    let src = scene.push_path(rectangle([0.0, 0.0], [10.0, 4.0]));
    let dup = scene.duplicate_path(src, 5.0, 7.0).unwrap();
    assert_ne!(src, dup, "clone gets a fresh id");
    assert_eq!(scene.paths().len(), 2);
    assert_eq!(
        scene.paths().last().unwrap().id,
        dup,
        "duplicate stacks on top"
    );
    let orig = scene.paths().iter().find(|p| p.id == src).unwrap().clone();
    let copy = scene.paths().iter().find(|p| p.id == dup).unwrap();
    for (o, n) in orig.verts.iter().zip(&copy.verts) {
        assert_eq!(n.anchor, [o.anchor[0] + 5.0, o.anchor[1] + 7.0]);
        assert_eq!(n.in_handle, [o.in_handle[0] + 5.0, o.in_handle[1] + 7.0]);
        assert_eq!(n.out_handle, [o.out_handle[0] + 5.0, o.out_handle[1] + 7.0]);
    }
    assert!(scene.duplicate_path(999, 1.0, 1.0).is_none());
}

#[test]
fn demo_grid_count() {
    assert_eq!(VecScene::demo_grid(50).paths().len(), 50);
    assert!(VecScene::demo_grid(0).is_empty());
}

#[test]
fn postcard_roundtrip_is_identity() {
    let scene = VecScene::demo();
    let bytes = scene.to_bytes().unwrap();
    let back = VecScene::from_bytes(&bytes).unwrap();
    assert_eq!(scene, back);
}

#[test]
fn from_bytes_rejects_garbage() {
    assert!(VecScene::from_bytes(&[0xFF, 0xFF, 0xFF]).is_err());
}

fn anchor_bbox(p: &VecPath) -> ([f64; 2], [f64; 2]) {
    let mut mn = [f64::MAX; 2];
    let mut mx = [f64::MIN; 2];
    for v in &p.verts {
        mn[0] = mn[0].min(v.anchor[0]);
        mn[1] = mn[1].min(v.anchor[1]);
        mx[0] = mx[0].max(v.anchor[0]);
        mx[1] = mx[1].max(v.anchor[1]);
    }
    (mn, mx)
}

#[test]
fn rectangle_is_closed_four_corners_spanning_the_bbox() {
    // Corners passed in arbitrary order → normalized bbox.
    let r = rectangle([3.0, 5.0], [-1.0, -2.0]);
    assert!(r.closed && r.fill.is_none() && r.stroke.is_none());
    assert_eq!(r.verts.len(), 4);
    assert!(r.verts.iter().all(|v| v.kind == VertexKind::Corner));
    let (mn, mx) = anchor_bbox(&r);
    assert_eq!((mn, mx), ([-1.0, -2.0], [3.0, 5.0]));
}

#[test]
fn ellipse_matches_blob_when_radii_equal() {
    // `blob` now delegates to `ellipse`; the demo circle must be byte-identical
    // (guards the postcard/demo determinism after the refactor).
    let mut e = ellipse([0.0, 0.0], 1.2, 1.2);
    e.fill = Some(Rgba8::new(90, 150, 230, 255));
    assert_eq!(e, blob([0.0, 0.0], 1.2, Rgba8::new(90, 150, 230, 255)));
    assert!(e.verts.iter().all(|v| v.kind == VertexKind::Smooth));
}

#[test]
fn ellipse_anchors_touch_the_bbox_extents() {
    let e = ellipse([2.0, 3.0], 4.0, 1.0);
    let (mn, mx) = anchor_bbox(&e);
    assert_eq!((mn, mx), ([-2.0, 2.0], [6.0, 4.0]));
}

#[test]
fn regular_polygon_has_sides_corner_verts_and_clamps() {
    let p = regular_polygon([0.0, 0.0], 2.0, 2.0, 5);
    assert!(p.closed);
    assert_eq!(p.verts.len(), 5);
    assert!(p.verts.iter().all(|v| v.kind == VertexKind::Corner));
    // Clamp: sides < 3 → 3.
    assert_eq!(regular_polygon([0.0, 0.0], 1.0, 1.0, 0).verts.len(), 3);
    assert_eq!(
        regular_polygon([0.0, 0.0], 1.0, 1.0, MAX_POLYGON_SIDES + 99)
            .verts
            .len(),
        MAX_POLYGON_SIDES as usize
    );
}

#[test]
fn regular_polygon_first_vertex_is_at_top() {
    // Angle 0 = top (−Y): first anchor sits at (cx, cy − ry).
    let p = regular_polygon([1.0, 1.0], 3.0, 2.0, 6);
    let a = p.verts[0].anchor;
    assert!((a[0] - 1.0).abs() < 1e-9, "x centered");
    assert!((a[1] - (1.0 - 2.0)).abs() < 1e-9, "y at top of bbox");
}

/// A closed triangle of straight corners (degenerate handles).
fn corner_triangle() -> VecPath {
    VecPath {
        id: 0,
        verts: vec![
            VecVertex::corner([0.0, 0.0]),
            VecVertex::corner([4.0, 0.0]),
            VecVertex::corner([2.0, 3.0]),
        ],
        closed: true,
        fill: None,
        stroke: None,
    }
}

fn handles_rel(v: &VecVertex) -> ([f64; 2], [f64; 2]) {
    (
        [v.in_handle[0] - v.anchor[0], v.in_handle[1] - v.anchor[1]],
        [v.out_handle[0] - v.anchor[0], v.out_handle[1] - v.anchor[1]],
    )
}

fn cross(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn norm(v: [f64; 2]) -> f64 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}

#[test]
fn retype_corner_to_smooth_auto_synthesizes_colinear_handles_from_neighbors() {
    let mut p = corner_triangle();
    // Vertex 1 (bbox apex on the base) had degenerate handles → synthesized.
    assert!(retype_vertex(&mut p, 1, VertexKind::Smooth));
    let v = p.verts[1];
    assert_eq!(v.kind, VertexKind::Smooth);
    let (in_rel, out_rel) = handles_rel(&v);
    assert!(norm(in_rel) > 1e-6 && norm(out_rel) > 1e-6, "handles grew");
    // Colinear + opposite (tangent continuous).
    assert!(cross(in_rel, out_rel).abs() < 1e-9, "colinear");
    assert!(dot(in_rel, out_rel) < 0.0, "opposite sides of the anchor");
}

#[test]
fn retype_to_symmetric_equalizes_handle_lengths() {
    let mut p = VecPath {
        id: 0,
        verts: vec![
            VecVertex::corner([0.0, 0.0]),
            // Asymmetric colinear-ish handles on the middle vertex.
            VecVertex::smooth([4.0, 0.0], [3.0, 0.0], [5.0, 0.5]),
            VecVertex::corner([8.0, 0.0]),
        ],
        closed: false,
        fill: None,
        stroke: None,
    };
    assert!(retype_vertex(&mut p, 1, VertexKind::Symmetric));
    let v = p.verts[1];
    let (in_rel, out_rel) = handles_rel(&v);
    assert!((norm(in_rel) - norm(out_rel)).abs() < 1e-9, "equal length");
    assert!(cross(in_rel, out_rel).abs() < 1e-9, "colinear");
    assert!(dot(in_rel, out_rel) < 0.0, "opposite");
}

#[test]
fn retype_to_corner_keeps_handle_positions_as_a_cusp() {
    let mut p = corner_triangle();
    let _ = retype_vertex(&mut p, 1, VertexKind::Symmetric); // grow handles
    let grown = p.verts[1];
    assert!(retype_vertex(&mut p, 1, VertexKind::Corner));
    let cusp = p.verts[1];
    assert_eq!(cusp.kind, VertexKind::Corner);
    // Handles unchanged — Corner just releases the colinear constraint.
    assert_eq!(cusp.in_handle, grown.in_handle);
    assert_eq!(cusp.out_handle, grown.out_handle);
}

#[test]
fn retype_is_noop_when_kind_and_geometry_already_match() {
    let mut p = corner_triangle();
    // Already Corner with degenerate handles → Corner is a true no-op.
    assert!(!retype_vertex(&mut p, 1, VertexKind::Corner));
    // Out-of-bounds index.
    assert!(!retype_vertex(&mut p, 99, VertexKind::Smooth));
}

/// A curved open segment for split tests.
fn curved_segment() -> VecPath {
    VecPath {
        id: 0,
        verts: vec![
            VecVertex::smooth([0.0, 0.0], [0.0, 0.0], [1.0, 2.0]),
            VecVertex::smooth([3.0, 0.0], [2.0, 2.0], [3.0, 0.0]),
        ],
        closed: false,
        fill: None,
        stroke: None,
    }
}

#[test]
fn split_segment_inserts_a_smooth_vertex_on_the_curve() {
    let mut p = curved_segment();
    let (p0, p1, p2, p3) = ([0.0, 0.0], [1.0, 2.0], [2.0, 2.0], [3.0, 0.0]);
    let ni = split_segment(&mut p, 0, 0.4).unwrap();
    assert_eq!(ni, 1);
    assert_eq!(p.verts.len(), 3);
    assert_eq!(p.verts[1].kind, VertexKind::Smooth);
    // The new anchor lies exactly on the original curve at t = 0.4.
    let want = cubic_at(p0, p1, p2, p3, 0.4);
    assert!((p.verts[1].anchor[0] - want[0]).abs() < 1e-9);
    assert!((p.verts[1].anchor[1] - want[1]).abs() < 1e-9);
}

#[test]
fn split_preserves_the_shape_exactly() {
    let mut p = curved_segment();
    let (p0, p1, p2, p3) = ([0.0, 0.0], [1.0, 2.0], [2.0, 2.0], [3.0, 0.0]);
    split_segment(&mut p, 0, 0.4).unwrap();
    // Left sub-cubic spans original t∈[0,0.4]; its local u maps to orig 0.4·u.
    let (la0, la1) = (p.verts[0].anchor, p.verts[0].out_handle);
    let (la2, la3) = (p.verts[1].in_handle, p.verts[1].anchor);
    for &u in &[0.0_f64, 0.5, 1.0] {
        let got = cubic_at(la0, la1, la2, la3, u);
        let want = cubic_at(p0, p1, p2, p3, 0.4 * u);
        assert!((got[0] - want[0]).abs() < 1e-9 && (got[1] - want[1]).abs() < 1e-9);
    }
    // Right sub-cubic spans original t∈[0.4,1].
    let (ra0, ra1) = (p.verts[1].anchor, p.verts[1].out_handle);
    let (ra2, ra3) = (p.verts[2].in_handle, p.verts[2].anchor);
    for &u in &[0.0_f64, 0.5, 1.0] {
        let got = cubic_at(ra0, ra1, ra2, ra3, u);
        let want = cubic_at(p0, p1, p2, p3, 0.4 + 0.6 * u);
        assert!((got[0] - want[0]).abs() < 1e-9 && (got[1] - want[1]).abs() < 1e-9);
    }
}

#[test]
fn split_handles_the_closing_segment_of_a_closed_path() {
    let mut p = corner_triangle(); // 3 verts, closed → 3 segments (incl. closing)
    // Closing segment is index 2 (v2 → v0).
    let ni = split_segment(&mut p, 2, 0.5).unwrap();
    assert_eq!(ni, 3); // appended at the end
    assert_eq!(p.verts.len(), 4);
    // Out of range segment → None.
    assert!(split_segment(&mut p, 99, 0.5).is_none());
}

#[test]
fn star_has_2n_alternating_corner_verts() {
    let s = star([0.0, 0.0], 4.0, 4.0, 5, 0.5);
    assert!(s.closed);
    assert_eq!(s.verts.len(), 10); // 2 · 5
    assert!(s.verts.iter().all(|v| v.kind == VertexKind::Corner));
    // Even indices are outer (radius 4), odd are inner (radius 2).
    let rad = |v: &VecVertex| (v.anchor[0].powi(2) + v.anchor[1].powi(2)).sqrt();
    assert!((rad(&s.verts[0]) - 4.0).abs() < 1e-9, "outer");
    assert!((rad(&s.verts[1]) - 2.0).abs() < 1e-9, "inner = 4·0.5");
    // First point at the top (−Y).
    assert!(s.verts[0].anchor[0].abs() < 1e-9 && (s.verts[0].anchor[1] + 4.0).abs() < 1e-9);
    // Clamps.
    assert_eq!(star([0.0, 0.0], 1.0, 1.0, 2, 0.5).verts.len(), 6); // points → 3
}

#[test]
fn rounded_rect_is_eight_corners_within_the_bbox() {
    let rr = rounded_rect([0.0, 0.0], [10.0, 6.0], 2.0);
    assert!(rr.closed);
    assert_eq!(rr.verts.len(), 8);
    assert!(rr.verts.iter().all(|v| v.kind == VertexKind::Corner));
    // Every anchor sits inside the bbox.
    assert!(
        rr.verts
            .iter()
            .all(|v| (0.0..=10.0).contains(&v.anchor[0]) && (0.0..=6.0).contains(&v.anchor[1]))
    );
    // At least one handle is offset (the arcs) — the shape is rounded.
    assert!(
        rr.verts
            .iter()
            .any(|v| v.out_handle != v.anchor || v.in_handle != v.anchor)
    );
}

#[test]
fn rounded_rect_degenerates_to_rectangle_at_zero_radius() {
    let rr = rounded_rect([0.0, 0.0], [4.0, 3.0], 0.0);
    assert_eq!(rr.verts.len(), 4);
    assert_eq!(rr, rectangle([0.0, 0.0], [4.0, 3.0]));
}

#[test]
fn rounded_rect_clamps_radius_to_half_the_smaller_side() {
    // 4×10 rect, huge radius → clamps to 2 (half of 4). Anchors still valid.
    let rr = rounded_rect([0.0, 0.0], [4.0, 10.0], 999.0);
    assert_eq!(rr.verts.len(), 8);
    assert!(
        rr.verts
            .iter()
            .all(|v| (0.0..=4.0).contains(&v.anchor[0]) && (0.0..=10.0).contains(&v.anchor[1]))
    );
}

#[test]
fn nearest_point_on_path_finds_the_click_segment_and_t() {
    let p = curved_segment();
    // A point near the midpoint of the (only) segment.
    let mid = cubic_at([0.0, 0.0], [1.0, 2.0], [2.0, 2.0], [3.0, 0.0], 0.5);
    let probe = [mid[0], mid[1] + 0.05];
    let (seg, t, d2) = nearest_point_on_path(&p, probe, 64).unwrap();
    assert_eq!(seg, 0);
    assert!((t - 0.5).abs() < 0.1, "t near the middle");
    assert!(d2.sqrt() < 0.1, "close to the curve");
    // A single-vertex path has no segments.
    let dot = VecPath {
        id: 0,
        verts: vec![VecVertex::corner([0.0, 0.0])],
        closed: false,
        fill: None,
        stroke: None,
    };
    assert!(nearest_point_on_path(&dot, [0.0, 0.0], 8).is_none());
}
