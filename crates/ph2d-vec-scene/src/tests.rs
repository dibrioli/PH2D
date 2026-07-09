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
fn flip_path_mirrors_around_the_bbox_center() {
    let mut scene = VecScene::new();
    // Rectangle spanning x∈[0,10], y∈[0,4] → centers (5, 2).
    let id = scene.push_path(rectangle([0.0, 0.0], [10.0, 4.0]));
    let before: Vec<_> = scene.paths()[0].verts.iter().map(|v| v.anchor).collect();

    assert!(scene.flip_path(id, FlipAxis::Horizontal));
    for (b, v) in before.iter().zip(&scene.paths()[0].verts) {
        assert!(
            (v.anchor[0] - (10.0 - b[0])).abs() < 1e-9,
            "X mirrored about 5"
        );
        assert!((v.anchor[1] - b[1]).abs() < 1e-9, "Y unchanged");
    }
    // Flipping the same axis twice is the identity.
    assert!(scene.flip_path(id, FlipAxis::Horizontal));
    for (b, v) in before.iter().zip(&scene.paths()[0].verts) {
        assert!((v.anchor[0] - b[0]).abs() < 1e-9);
    }

    assert!(scene.flip_path(id, FlipAxis::Vertical));
    for (b, v) in before.iter().zip(&scene.paths()[0].verts) {
        assert!(
            (v.anchor[1] - (4.0 - b[1])).abs() < 1e-9,
            "Y mirrored about 2"
        );
        assert!((v.anchor[0] - b[0]).abs() < 1e-9, "X unchanged");
    }
    assert!(!scene.flip_path(999, FlipAxis::Horizontal));
}

#[test]
fn rotate_path_quarter_turn_is_cyclic_and_exact() {
    let mut scene = VecScene::new();
    // Rectangle x∈[0,10], y∈[0,4] → bbox center (5, 2), invariant under rotation.
    let id = scene.push_path(rectangle([0.0, 0.0], [10.0, 4.0]));
    let before: Vec<_> = scene.paths()[0].verts.iter().map(|v| v.anchor).collect();
    let same = |scene: &VecScene, want: &[[f64; 2]]| {
        want.iter()
            .zip(&scene.paths()[0].verts)
            .all(|(b, v)| (v.anchor[0] - b[0]).abs() < 1e-9 && (v.anchor[1] - b[1]).abs() < 1e-9)
    };

    // 4× CW = full turn = identity.
    for _ in 0..4 {
        assert!(scene.rotate_path(id, Rotate90::Cw));
    }
    assert!(same(&scene, &before), "4× CW returns to the original");
    // CW then CCW = identity.
    assert!(scene.rotate_path(id, Rotate90::Cw));
    assert!(scene.rotate_path(id, Rotate90::Ccw));
    assert!(same(&scene, &before), "CW·CCW cancels");

    // One CW quarter-turn about (5,2): the (0,0) corner lands at (7,−3).
    let i0 = before.iter().position(|a| *a == [0.0, 0.0]).unwrap();
    assert!(scene.rotate_path(id, Rotate90::Cw));
    let a = scene.paths()[0].verts[i0].anchor;
    assert!((a[0] - 7.0).abs() < 1e-9 && (a[1] + 3.0).abs() < 1e-9);

    assert!(!scene.rotate_path(999, Rotate90::Cw));
}

#[test]
fn rotate_path_by_arbitrary_angle_about_pivot() {
    use std::f64::consts::FRAC_PI_2;
    let mut scene = VecScene::new();
    // A single point at (1,0); rotate +90° about the origin → (0,1).
    let id = scene.push_path(VecPath {
        id: 0,
        verts: vec![VecVertex::corner([1.0, 0.0])],
        closed: false,
        fill: None,
        stroke: None,
    });
    assert!(scene.rotate_path_by(id, FRAC_PI_2, [0.0, 0.0]));
    let a = scene.paths()[0].verts[0].anchor;
    assert!(
        (a[0]).abs() < 1e-9 && (a[1] - 1.0).abs() < 1e-9,
        "got {a:?}"
    );

    // Rotating back by −90° returns to the start (handles ride along too).
    assert!(scene.rotate_path_by(id, -FRAC_PI_2, [0.0, 0.0]));
    let a = scene.paths()[0].verts[0].anchor;
    assert!(
        (a[0] - 1.0).abs() < 1e-9 && a[1].abs() < 1e-9,
        "round-trips: {a:?}"
    );

    assert!(!scene.rotate_path_by(999, 1.0, [0.0, 0.0]));
}

#[test]
fn transforms_move_the_gradient_geometry_rigidly() {
    use std::f64::consts::FRAC_PI_2;
    let red = Rgba8::new(255, 0, 0, 255);
    let blue = Rgba8::new(0, 0, 255, 255);
    let ends = |scene: &VecScene, id: VecPathId| match &scene
        .paths()
        .iter()
        .find(|p| p.id == id)
        .unwrap()
        .fill
    {
        Some(Paint::Linear { start, end, .. }) => (*start, *end),
        other => panic!("expected linear, got {other:?}"),
    };
    let close = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9;

    let mut scene = VecScene::new();
    let mut path = rectangle([0.0, 0.0], [10.0, 4.0]);
    // A horizontal ramp across the rect (world-space endpoints).
    path.fill = Some(Paint::Linear {
        stops: vec![GradientStop::new(0.0, red), GradientStop::new(1.0, blue)],
        start: [0.0, 2.0],
        end: [10.0, 2.0],
    });
    let id = scene.push_path(path);

    // Rotate +90° about the bbox center (5,2): the ramp endpoints rotate with it
    // (start→[5,-3], end→[5,7]) — the gradient turns rigidly with the shape.
    scene.rotate_path_by(id, FRAC_PI_2, [5.0, 2.0]);
    let (s, e) = ends(&scene, id);
    assert!(
        close(s, [5.0, -3.0]) && close(e, [5.0, 7.0]),
        "rotated rigidly: {s:?} {e:?}"
    );

    // Rotate back — endpoints return to the original horizontal ramp.
    scene.rotate_path_by(id, -FRAC_PI_2, [5.0, 2.0]);
    let (s, e) = ends(&scene, id);
    assert!(close(s, [0.0, 2.0]) && close(e, [10.0, 2.0]), "round-trips");

    // Translate moves the endpoints too.
    scene.translate_path(id, 3.0, -1.0);
    let (s, e) = ends(&scene, id);
    assert!(close(s, [3.0, 1.0]) && close(e, [13.0, 1.0]), "translated");

    // Flip H mirrors the endpoints about the (translated) shape's bbox center.
    scene.flip_path(id, FlipAxis::Horizontal);
    let (s, e) = ends(&scene, id);
    // bbox x∈[3,13] → center x = 8 → start.x 3→13, end.x 13→3.
    assert!(close(s, [13.0, 1.0]) && close(e, [3.0, 1.0]), "flipped H");

    // Radial radius scales with a uniform scale.
    let mut scene2 = VecScene::new();
    let mut path2 = rectangle([0.0, 0.0], [10.0, 10.0]);
    path2.fill = Some(Paint::Radial {
        stops: vec![GradientStop::new(0.0, red), GradientStop::new(1.0, blue)],
        center: [5.0, 5.0],
        radius: 4.0,
    });
    let id2 = scene2.push_path(path2);
    scene2.scale_path(id2, 2.0, 2.0, [5.0, 5.0]);
    if let Some(Paint::Radial { center, radius, .. }) =
        &scene2.paths().iter().find(|p| p.id == id2).unwrap().fill
    {
        assert!(close(*center, [5.0, 5.0]), "center pinned at pivot");
        assert!((radius - 8.0).abs() < 1e-9, "radius ×2");
    } else {
        panic!("expected radial");
    }
}

#[test]
fn bbox_translate_and_scale_compose() {
    let mut scene = VecScene::new();
    let id = scene.push_path(rectangle([0.0, 0.0], [10.0, 4.0]));
    assert_eq!(scene.path_bbox(id).unwrap(), ([0.0, 0.0], [10.0, 4.0]));

    // Translate: bbox min moves, size unchanged.
    assert!(scene.translate_path(id, 3.0, -2.0));
    let (lo, hi) = scene.path_bbox(id).unwrap();
    assert_eq!((lo, hi), ([3.0, -2.0], [13.0, 2.0]));

    // Scale ×2 in X, ×0.5 in Y about the bbox min (top-left pinned).
    assert!(scene.scale_path(id, 2.0, 0.5, lo));
    let (lo2, hi2) = scene.path_bbox(id).unwrap();
    assert!(
        (lo2[0] - 3.0).abs() < 1e-9 && (lo2[1] + 2.0).abs() < 1e-9,
        "min pinned"
    );
    assert!((hi2[0] - 23.0).abs() < 1e-9, "W 10→20"); // 3 + 10*2
    assert!((hi2[1] - 0.0).abs() < 1e-9, "H 4→2"); // -2 + 4*0.5

    assert!(scene.path_bbox(999).is_none());
    assert!(!scene.translate_path(999, 1.0, 1.0));
    assert!(!scene.scale_path(999, 2.0, 2.0, [0.0, 0.0]));
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

#[test]
fn paint_variants_roundtrip_and_report_primary_color() {
    let red = Rgba8::new(255, 0, 0, 255);
    let blue = Rgba8::new(0, 0, 255, 255);
    let paints = [
        Paint::solid(red),
        Paint::Linear {
            stops: vec![GradientStop::new(0.0, red), GradientStop::new(1.0, blue)],
            start: [0.0, 0.0],
            end: [4.0, 0.0],
        },
        Paint::Radial {
            stops: vec![GradientStop::new(0.0, red), GradientStop::new(1.0, blue)],
            center: [2.0, 2.0],
            radius: 2.0,
        },
        Paint::MultiPoint {
            points: vec![
                GradientPoint::new([0.2, 0.2], red, 1.0),
                GradientPoint::new([0.8, 0.8], blue, 2.0),
            ],
        },
    ];
    // primary_color = solid / first stop / first point.
    for p in &paints {
        assert_eq!(p.primary_color(), red);
    }
    // Each variant survives a full scene save/load (postcard, schema v4).
    let mut scene = VecScene::new();
    for p in &paints {
        let mut path = rectangle([0.0, 0.0], [4.0, 4.0]);
        path.fill = Some(p.clone());
        scene.push_path(path);
    }
    let back = VecScene::from_bytes(&scene.to_bytes().unwrap()).unwrap();
    assert_eq!(scene, back, "all Paint variants round-trip");
    // Rgba8 → Paint::Solid via From.
    assert_eq!(Paint::from(blue), Paint::Solid(blue));
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
    e.fill = Some(Paint::solid(Rgba8::new(90, 150, 230, 255)));
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

#[test]
fn spiral_is_open_grows_from_center_to_edge_and_clamps_turns() {
    let s = spiral([0.0, 0.0], 2.0, 3.0, 3);
    assert!(!s.closed && s.fill.is_none() && s.stroke.is_none());
    // 3 turns × 24 samples + 1 endpoint.
    assert_eq!(s.verts.len(), 3 * 24 + 1);
    // First sample at the center (f = 0).
    assert!(s.verts[0].anchor[0].abs() < 1e-6 && s.verts[0].anchor[1].abs() < 1e-6);
    // Integer turns → the last sample is back at the top of the bbox: (0, −ry).
    let last = s.verts.last().unwrap().anchor;
    assert!(last[0].abs() < 1e-6 && (last[1] + 3.0).abs() < 1e-6);
    // Turns clamp to [1, MAX_SPIRAL_TURNS].
    assert_eq!(spiral([0.0, 0.0], 1.0, 1.0, 0).verts.len(), 24 + 1);
    assert_eq!(
        spiral([0.0, 0.0], 1.0, 1.0, 99).verts.len(),
        MAX_SPIRAL_TURNS as usize * 24 + 1
    );
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

#[test]
fn smooth_path_curves_corners_and_sharpen_restores_straight() {
    let mut scene = VecScene::new();
    // A polygon is all Corner vertices with degenerate (anchor-coincident) handles.
    let id = scene.push_path(regular_polygon([0.0, 0.0], 5.0, 5.0, 5));
    assert!(
        scene
            .paths()
            .iter()
            .find(|p| p.id == id)
            .unwrap()
            .verts
            .iter()
            .all(|v| v.kind == VertexKind::Corner
                && v.in_handle == v.anchor
                && v.out_handle == v.anchor),
        "polygon starts as straight corners"
    );

    // Smooth (1st click): every vertex becomes Smooth with non-degenerate, colinear
    // handles; on a regular polygon the handle length is IDENTICAL at every vertex
    // (consistent — same Catmull-Rom rule per point).
    assert!(scene.smooth_path(id));
    let verts = scene
        .paths()
        .iter()
        .find(|p| p.id == id)
        .unwrap()
        .verts
        .clone();
    let out_len =
        |v: &VecVertex| (v.out_handle[0] - v.anchor[0]).hypot(v.out_handle[1] - v.anchor[1]);
    let len0 = out_len(&verts[0]);
    for v in &verts {
        assert_eq!(v.kind, VertexKind::Smooth);
        let ir = [v.in_handle[0] - v.anchor[0], v.in_handle[1] - v.anchor[1]];
        let or = [v.out_handle[0] - v.anchor[0], v.out_handle[1] - v.anchor[1]];
        assert!(
            ir[0].hypot(ir[1]) > 1e-6 && or[0].hypot(or[1]) > 1e-6,
            "handles synthesized from neighbors"
        );
        // Colinear (opposite directions): cross ≈ 0, dot < 0.
        assert!(
            (ir[0] * or[1] - ir[1] * or[0]).abs() < 1e-9,
            "handles colinear"
        );
        assert!(
            ir[0] * or[0] + ir[1] * or[1] < 0.0,
            "in/out point opposite ways"
        );
        assert!(
            (out_len(v) - len0).abs() < 1e-9,
            "every vertex smoothed by the same amount (consistent)"
        );
    }

    // Incremental: a 2nd click GROWS the handles (rounder); anchors never move.
    assert!(scene.smooth_path(id));
    let len1 = out_len(&scene.paths().iter().find(|p| p.id == id).unwrap().verts[0]);
    assert!(
        len1 > len0 + 1e-9,
        "2nd click grows the smoothing ({len0} → {len1})"
    );
    assert!(
        scene.paths().iter().find(|p| p.id == id).unwrap().verts[0].anchor == verts[0].anchor,
        "anchors fixed under smoothing"
    );

    // Converges: repeated clicks saturate (round) and then change nothing.
    let mut clicks = 0;
    while scene.smooth_path(id) {
        clicks += 1;
        assert!(clicks < 20, "smoothing must saturate, not grow forever");
    }

    // Re-smooth after saturation is a no-op.
    assert!(!scene.smooth_path(id));

    // Sharpen: back to straight corners (anchors preserved).
    assert!(scene.sharpen_path(id));
    let after = scene.paths().iter().find(|p| p.id == id).unwrap();
    assert!(
        after
            .verts
            .iter()
            .zip(&verts)
            .all(|(a, b)| a.anchor == b.anchor),
        "anchors preserved through sharpen"
    );
    assert!(after.verts.iter().all(|v| v.kind == VertexKind::Corner
        && v.in_handle == v.anchor
        && v.out_handle == v.anchor));
    // Idempotent: sharpening again changes nothing.
    assert!(!scene.sharpen_path(id));
    // Missing id.
    assert!(!scene.smooth_path(999));
    assert!(!scene.sharpen_path(999));
}

#[test]
fn simplify_path_drops_redundant_points_and_keeps_the_shape() {
    let mut scene = VecScene::new();
    // Open path: a straight run of colinear points + one real corner. The two
    // interior colinear points are redundant (zero deviation); the corner stays.
    let poly = VecPath {
        id: 0,
        verts: vec![
            VecVertex::corner([0.0, 0.0]),
            VecVertex::corner([3.0, 0.0]), // colinear on [0,0]→[9,0]
            VecVertex::corner([6.0, 0.0]), // colinear
            VecVertex::corner([9.0, 0.0]),
            VecVertex::corner([9.0, 9.0]), // real corner (large deviation)
        ],
        closed: false,
        fill: None,
        stroke: None,
    };
    let id = scene.push_path(poly);
    let count = |s: &VecScene| s.paths().iter().find(|p| p.id == id).unwrap().verts.len();

    // Progressive: each click reduces the count until the 3-point floor, then stops.
    let mut prev = count(&scene);
    let mut clicks = 0;
    while scene.simplify_path(id) {
        let now = count(&scene);
        assert!(
            now < prev,
            "each click removes at least one point ({prev} → {now})"
        );
        prev = now;
        clicks += 1;
        assert!(clicks < 20, "must reach the floor, not loop forever");
    }
    assert_eq!(prev, 3, "simplifies down to the 3-point floor");

    let verts = &scene.paths().iter().find(|p| p.id == id).unwrap().verts;
    // Endpoints preserved, the real corner kept, colinear midpoints gone.
    assert_eq!(
        verts.first().unwrap().anchor,
        [0.0, 0.0],
        "start endpoint kept"
    );
    assert_eq!(
        verts.last().unwrap().anchor,
        [9.0, 9.0],
        "end endpoint kept"
    );
    assert!(
        verts.iter().any(|v| v.anchor == [9.0, 0.0]),
        "the real corner survives"
    );
    assert!(
        !verts
            .iter()
            .any(|v| v.anchor == [3.0, 0.0] || v.anchor == [6.0, 0.0]),
        "colinear midpoints removed"
    );

    // A small closed path (<= 3) is left alone.
    let tri = scene.push_path(regular_polygon([0.0, 0.0], 5.0, 5.0, 3));
    assert!(
        !scene.simplify_path(tri),
        "3-vertex closed path is at the floor"
    );
    assert!(!scene.simplify_path(999));
}

#[test]
fn subdivide_path_doubles_segments_and_preserves_shape() {
    let mut scene = VecScene::new();
    // Smoothed 8-gon → a curved closed path (non-degenerate handles).
    let id = scene.push_path(regular_polygon([0.0, 0.0], 10.0, 10.0, 8));
    scene.smooth_path(id);
    let before = scene.paths().iter().find(|p| p.id == id).unwrap().clone();
    let n0 = before.verts.len();
    let ref_pts = sample_path(&before, 16);

    // Subdivide: one new vertex per segment (closed ⇒ segs == n).
    assert!(scene.subdivide_path(id));
    let after = scene.paths().iter().find(|p| p.id == id).unwrap();
    assert_eq!(
        after.verts.len(),
        n0 * 2,
        "one midpoint inserted per segment"
    );

    // Shape preserved EXACTLY (de Casteljau split): every original sample is on
    // the new curve to numerical precision.
    let new_pts = sample_path(after, 16);
    let mut maxd: f64 = 0.0;
    for rp in &ref_pts {
        let d = new_pts
            .iter()
            .map(|np| (rp[0] - np[0]).hypot(rp[1] - np[1]))
            .fold(f64::INFINITY, f64::min);
        maxd = maxd.max(d);
    }
    assert!(maxd < 1e-6, "subdivision is shape-exact (max dev {maxd})");

    // A single-vertex / missing path can't subdivide.
    let dot = scene.push_path(path_at([0.0, 0.0]));
    assert!(!scene.subdivide_path(dot));
    assert!(!scene.subdivide_path(999));
}

#[test]
fn set_path_closed_toggles_and_guards() {
    let mut scene = VecScene::new();
    // An open 3-point path.
    let id = scene.push_path(VecPath {
        id: 0,
        verts: vec![
            VecVertex::corner([0.0, 0.0]),
            VecVertex::corner([4.0, 0.0]),
            VecVertex::corner([2.0, 3.0]),
        ],
        closed: false,
        fill: None,
        stroke: None,
    });
    assert!(scene.set_path_closed(id, true), "opens → closed");
    assert!(scene.paths()[0].closed);
    // Idempotent: already closed.
    assert!(!scene.set_path_closed(id, true));
    assert!(scene.set_path_closed(id, false), "closed → open");
    assert!(!scene.paths()[0].closed);

    // A single-vertex path can't be closed; missing id is a no-op.
    let dot = scene.push_path(path_at([0.0, 0.0]));
    assert!(!scene.set_path_closed(dot, true), "< 2 verts can't close");
    assert!(!scene.set_path_closed(999, true));
}

/// Sample every cubic segment of a path into a flat list of points.
fn sample_path(p: &VecPath, per_seg: usize) -> Vec<[f64; 2]> {
    let n = p.verts.len();
    let segs = if p.closed { n } else { n.saturating_sub(1) };
    let mut out = Vec::new();
    for s in 0..segs {
        let a = &p.verts[s];
        let b = &p.verts[(s + 1) % n];
        for k in 0..=per_seg {
            let t = k as f64 / per_seg as f64;
            out.push(cubic_at(a.anchor, a.out_handle, b.in_handle, b.anchor, t));
        }
    }
    out
}

#[test]
fn simplify_stays_faithful_to_a_smooth_curve() {
    // A many-sided polygon, smoothed into a near-circular closed curve — a shape
    // with real curvature (non-degenerate handles).
    let mut scene = VecScene::new();
    let id = scene.push_path(regular_polygon([0.0, 0.0], 10.0, 10.0, 16));
    scene.smooth_path(id);
    scene.smooth_path(id); // grow the handles so it's genuinely curved

    let before = scene.paths().iter().find(|p| p.id == id).unwrap().clone();
    let ref_pts = sample_path(&before, 8);

    // One simplify click removes ~20% of the points…
    assert!(scene.simplify_path(id));
    let after = scene.paths().iter().find(|p| p.id == id).unwrap();
    assert!(after.verts.len() < before.verts.len(), "points removed");

    // …and the curve stays FAITHFUL: every original sample is near the new curve.
    let new_pts = sample_path(after, 16);
    let diag = 20.0; // bbox diagonal ≈ 2·radius
    let mut maxd: f64 = 0.0;
    for rp in &ref_pts {
        let d = new_pts
            .iter()
            .map(|np| (rp[0] - np[0]).hypot(rp[1] - np[1]))
            .fold(f64::INFINITY, f64::min);
        maxd = maxd.max(d);
    }
    assert!(
        maxd < 0.05 * diag,
        "simplified curve stays within 5% of the original ({maxd} vs {})",
        0.05 * diag
    );
}

#[test]
fn path_contains_point_is_even_odd_and_closed_only() {
    let mut scene = VecScene::new();
    let closed = scene.push_path(rectangle([0.0, 0.0], [10.0, 10.0]));
    // Inside the square.
    assert!(scene.path_contains_point(closed, [5.0, 5.0]));
    // Outside on every side.
    for p in [[-1.0, 5.0], [11.0, 5.0], [5.0, -1.0], [5.0, 11.0]] {
        assert!(!scene.path_contains_point(closed, p), "outside {p:?}");
    }
    // An OPEN path never contains anything (the gizmo's interior-move needs a region).
    let mut open = rectangle([0.0, 0.0], [10.0, 10.0]);
    open.closed = false;
    let open = scene.push_path(open);
    assert!(!scene.path_contains_point(open, [5.0, 5.0]));
    // A missing id is never inside.
    assert!(!scene.path_contains_point(9999, [5.0, 5.0]));
}
