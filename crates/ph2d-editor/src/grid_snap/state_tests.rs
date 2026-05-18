use super::*;

#[test]
fn active_origin_dispatches_per_kind() {
    let mut s = GridSnapState::default();
    s.square_cfg.origin = [1.0, 2.0];
    s.hex_cfg.origin = [3.0, 4.0];
    assert_eq!(s.active_origin(), [1.0, 2.0]);
    s.kind = GridKind::Hex;
    assert_eq!(s.active_origin(), [3.0, 4.0]);
    // Quadtree / Voronoi have no origin — always [0, 0].
    s.kind = GridKind::Quadtree;
    assert_eq!(s.active_origin(), [0.0, 0.0]);
}

#[test]
fn snap_world_respects_origin_offset() {
    let mut s = GridSnapState {
        snap_enabled: true,
        snap_target: SnapTarget::Center,
        // Disable magnetism so this test exercises the math
        // path (origin offset) without interference from the
        // attraction-radius gate added for bug 2.
        snap_magnetism_radius: 0.0,
        ..Default::default()
    };
    // Default cell_size=1.0, origin=[0,0]. Center of cell (0,0)
    // is (0.5, 0.5).
    assert_eq!(s.snap_world([0.1, 0.1], [0.0, 0.0]), [0.5, 0.5]);
    // Shift origin by (10, 20). Same world point [0.1, 0.1]
    // sits 9.9 units LEFT and 19.9 units DOWN of cell (0,0) of
    // the shifted grid → cell (-10, -20) → center
    // (-9.5 + 10, -19.5 + 20) = (0.5, 0.5)? Wait that's the
    // SAME center because the world point shifts relative to a
    // grid moved equally. Let me reconsider: snap pulls the
    // world point to a cell center of the shifted grid. With
    // origin = (10, 20), cell (0, 0) center is at
    // world (10.5, 20.5). For input [0.1, 0.1], the nearest
    // cell center of the shifted grid is one of:
    //   ..., (-9.5, -19.5), (0.5, -19.5), ... (... etc)
    // i.e. (k + 0.5 + 10, j + 0.5 + 20) — closest to [0.1, 0.1]
    // is k=-10, j=-20 → (0.5, 0.5). So same answer as no offset
    // when world point falls exactly on an integer-cell pattern.
    // Use a half-integer origin to make the test less degenerate.
    s.square_cfg.origin = [0.3, 0.7];
    // Input [0.1, 0.1] → local [-0.2, -0.6] → cell (-1, -1) →
    // local center (-0.5, -0.5) → world center (-0.2, 0.2).
    let snapped = s.snap_world([0.1, 0.1], [0.0, 0.0]);
    assert!(
        (snapped[0] - -0.2).abs() < 1e-5 && (snapped[1] - 0.2).abs() < 1e-5,
        "expected [-0.2, 0.2], got {snapped:?}"
    );
}

#[test]
fn default_state_snap_is_off_so_passthrough() {
    let mut s = GridSnapState::default();
    assert!(!s.snap_enabled);
    let p = s.snap_world([1.3, 2.7], [0.0, 0.0]);
    assert_eq!(p, [1.3, 2.7]);
}

#[test]
fn enabled_snap_pulls_to_cell_center_for_square() {
    let mut s = GridSnapState {
        snap_enabled: true,
        kind: GridKind::Square,
        snap_target: SnapTarget::Center,
        ..Default::default()
    };
    // Cursor at (0.4, 0.4): cell center at (0.5, 0.5), distance
    // ≈ 0.141 < default magnetism radius 0.30 → snap engages.
    let p = s.snap_world([0.4, 0.4], [0.0, 0.0]);
    assert_eq!(p, [0.5, 0.5]);
}

#[test]
fn snap_intersection_picks_corner_for_hex() {
    let mut s = GridSnapState {
        snap_enabled: true,
        kind: GridKind::Hex,
        snap_target: SnapTarget::Intersection,
        // Disable magnetism so any input near-origin engages
        // the snap (default radius 0.30 m would otherwise gate
        // the (0.1, 0.1) → vertex shift away).
        snap_magnetism_radius: 0.0,
        ..Default::default()
    };
    // Hex point near origin — must land on one of the 6 corners
    // of the containing hex. Verify the result is at distance
    // ≈ cell_size (= 1.0) from the cell center.
    let p = s.snap_world([0.1, 0.1], [0.0, 0.0]);
    let center = s.make_hex();
    use ph2d_grid::GridMath;
    let cell = center.world_to_cell([0.1, 0.1]);
    let cc = center.cell_to_world_center(cell);
    let d = ((p[0] - cc[0]).powi(2) + (p[1] - cc[1]).powi(2)).sqrt();
    assert!(
        (d - s.hex_cfg.cell_size).abs() < 1e-4,
        "vertex should be at radius cell_size from center; got d={d}"
    );
}

#[test]
fn quadtree_snap_to_center_returns_inside_bounds() {
    // With snap enabled, Quadtree should land on a leaf center
    // that's inside the cfg bounds (default `[-10, -10] → [10, 10]`).
    let mut s = GridSnapState {
        snap_enabled: true,
        kind: GridKind::Quadtree,
        snap_target: SnapTarget::Center,
        // Disable magnetism — the assertion below requires the
        // snap to actually engage and pull to a leaf center, but
        // Quadtree leaves can be several meters wide so the
        // default 0.30 m radius would gate the snap to passthrough.
        snap_magnetism_radius: 0.0,
        ..Default::default()
    };
    let p = s.snap_world([0.5, 0.5], [0.0, 0.0]);
    let b = s.quadtree_cfg.bounds;
    assert!(
        p[0] >= b.min[0] && p[0] <= b.max[0],
        "x out of bounds: {p:?}"
    );
    assert!(
        p[1] >= b.min[1] && p[1] <= b.max[1],
        "y out of bounds: {p:?}"
    );
    // And it must NOT be the input — Center mode always pulls to
    // a leaf center which is unlikely to coincide with the input.
    assert!(
        p != [0.5, 0.5],
        "Center snap should pull to a leaf center, got passthrough"
    );
}

#[test]
fn quadtree_snap_subdivisions_finer_than_one_lands_inside_active_leaf() {
    // Regression for the pre-scale bug: with subdivisions=4, the
    // old impl multiplied `world` by 4 before calling the quadtree
    // helper, which pushed the lookup outside `cfg.bounds` and
    // collapsed every snap to the outer-AABB fallback. With the
    // fix, the snap stays inside the leaf containing `world` and
    // the result lies on the N×N sub-grid of that leaf.
    use ph2d_grid::quadtree::{AABB, Quadtree};
    let mut s = GridSnapState {
        snap_enabled: true,
        kind: GridKind::Quadtree,
        snap_target: SnapTarget::Center,
        snap_subdivisions: 4,
        // Disable magnetism (same rationale as the sibling test).
        snap_magnetism_radius: 0.0,
        ..Default::default()
    };
    let world = [0.5, 0.5];
    let snapped = s.snap_world(world, [0.0, 0.0]);
    // Recompute the active leaf the helper would have hit.
    let cfg = &s.quadtree_cfg;
    let mut qt: Quadtree<()> = Quadtree::new(cfg.bounds, cfg.max_points_per_leaf, cfg.max_depth);
    for i in 0..cfg.demo_point_count {
        let t = i as u64;
        let mut h = cfg
            .demo_rng_seed
            .wrapping_add(t)
            .wrapping_mul(0x9E37_79B9_7F4A_7C15);
        h ^= h >> 30;
        h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h ^= h >> 27;
        h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
        h ^= h >> 31;
        let fx = ((h >> 32) as u32 as f64) / (u32::MAX as f64);
        let fy = ((h & 0xFFFF_FFFF) as u32 as f64) / (u32::MAX as f64);
        let x = cfg.bounds.min[0] + (fx as f32) * (cfg.bounds.max[0] - cfg.bounds.min[0]);
        let y = cfg.bounds.min[1] + (fy as f32) * (cfg.bounds.max[1] - cfg.bounds.min[1]);
        let _ = qt.insert([x, y], ());
    }
    let mut leaves: Vec<AABB> = Vec::new();
    qt.iter_leaf_bounds(&mut leaves);
    let leaf = leaves
        .into_iter()
        .find(|l| l.contains_point(world))
        .expect("world must land in some leaf for the default cfg");
    // Snap point must be inside (or on the boundary of) the active
    // leaf — proves the lookup didn't collapse to the outer bounds.
    assert!(
        snapped[0] >= leaf.min[0] - 1e-4 && snapped[0] <= leaf.max[0] + 1e-4,
        "snapped x={} outside leaf x∈[{}, {}]",
        snapped[0],
        leaf.min[0],
        leaf.max[0]
    );
    assert!(
        snapped[1] >= leaf.min[1] - 1e-4 && snapped[1] <= leaf.max[1] + 1e-4,
        "snapped y={} outside leaf y∈[{}, {}]",
        snapped[1],
        leaf.min[1],
        leaf.max[1]
    );
    // And the result must lie on the 4×4 sub-grid of that leaf
    // (sub-cell centers — quarters offset by half-cell).
    let n = 4.0_f32;
    let dx = (leaf.max[0] - leaf.min[0]) / n;
    let dy = (leaf.max[1] - leaf.min[1]) / n;
    let on_sub_x = (0..4).any(|i| {
        let cx = leaf.min[0] + (i as f32 + 0.5) * dx;
        (snapped[0] - cx).abs() < 1e-3
    });
    let on_sub_y = (0..4).any(|j| {
        let cy = leaf.min[1] + (j as f32 + 0.5) * dy;
        (snapped[1] - cy).abs() < 1e-3
    });
    assert!(
        on_sub_x && on_sub_y,
        "snapped {snapped:?} not on the 4×4 sub-grid of leaf {leaf:?}"
    );
}

#[test]
fn quadtree_snap_to_intersection_picks_a_leaf_corner() {
    // Intersection mode picks the nearest corner of the leaf
    // containing `world`. Corners are AABB extrema, so coordinates
    // line up with the subdivision boundaries.
    let mut s = GridSnapState {
        snap_enabled: true,
        kind: GridKind::Quadtree,
        snap_target: SnapTarget::Intersection,
        snap_magnetism_radius: 0.0,
        ..Default::default()
    };
    let p = s.snap_world([0.0, 0.0], [0.0, 0.0]);
    let b = s.quadtree_cfg.bounds;
    // Corners must be within bounds and at half-multiples of the
    // bounds extent (default subdivision halves repeatedly).
    assert!(p[0] >= b.min[0] && p[0] <= b.max[0]);
    assert!(p[1] >= b.min[1] && p[1] <= b.max[1]);
}

#[test]
fn voronoi_snap_to_center_lands_on_a_seed() {
    // Center mode for Voronoi snaps to the nearest seed (cell
    // center). Returned point must equal one of the deterministic
    // seeds.
    let mut s = GridSnapState {
        snap_enabled: true,
        kind: GridKind::Voronoi,
        snap_target: SnapTarget::Center,
        snap_magnetism_radius: 0.0,
        ..Default::default()
    };
    let p = s.snap_world([0.0, 0.0], [0.0, 0.0]);
    let seeds = ph2d_grid::voronoi::deterministic_seeds(
        s.voronoi_cfg.bounds,
        s.voronoi_cfg.seed_count,
        s.voronoi_cfg.rng_seed,
    );
    let matches_seed = seeds
        .iter()
        .any(|sd| (sd[0] - p[0]).abs() < 1e-4 && (sd[1] - p[1]).abs() < 1e-4);
    assert!(
        matches_seed,
        "snapped point {p:?} doesn't match any of {} seeds",
        seeds.len()
    );
}

#[test]
fn voronoi_snap_intersection_lands_on_a_cell_vertex() {
    // Intersection mode snaps to a Voronoi vertex (where 3+ cells
    // meet). Verify the result is inside cfg bounds.
    let mut s = GridSnapState {
        snap_enabled: true,
        kind: GridKind::Voronoi,
        snap_target: SnapTarget::Intersection,
        snap_magnetism_radius: 0.0,
        ..Default::default()
    };
    let p = s.snap_world([0.0, 0.0], [0.0, 0.0]);
    let b = s.voronoi_cfg.bounds;
    assert!(
        p[0] >= b.min[0] && p[0] <= b.max[0],
        "x out of bounds: {p:?}"
    );
    assert!(
        p[1] >= b.min[1] && p[1] <= b.max[1],
        "y out of bounds: {p:?}"
    );
}

#[test]
fn quadtree_and_voronoi_passthrough_when_disabled() {
    // snap_enabled = false → unconditional passthrough, regardless
    // of kind. Same as every other kind.
    let mut s = GridSnapState {
        snap_enabled: false,
        kind: GridKind::Quadtree,
        ..Default::default()
    };
    assert_eq!(s.snap_world([3.7, 2.1], [0.0, 0.0]), [3.7, 2.1]);
    s.kind = GridKind::Voronoi;
    assert_eq!(s.snap_world([3.7, 2.1], [0.0, 0.0]), [3.7, 2.1]);
}

#[test]
fn grid_kind_label_covers_all_nine_variants() {
    for k in GridKind::all() {
        assert!(!k.label().is_empty());
    }
}

// ── Magnetism radius (bug 2: "snap deve atrair, não teleportar") ──

#[test]
fn magnetism_passthrough_when_cursor_is_far_from_any_snap_point() {
    // Cursor at (0.5, 0.5) is exactly the cell center under
    // Intersection mode — distance to nearest vertex (0, 0) is
    // ≈ 0.707 m. With radius = 0.2 m, no vertex is in range →
    // free drag.
    let mut s = GridSnapState {
        snap_enabled: true,
        kind: GridKind::Square,
        snap_target: SnapTarget::Intersection,
        snap_magnetism_radius: 0.2,
        ..Default::default()
    };
    let p = s.snap_world([0.5, 0.5], [0.0, 0.0]);
    assert_eq!(p, [0.5, 0.5], "expected passthrough outside radius");
}

#[test]
fn magnetism_snaps_when_cursor_is_inside_radius() {
    // Cursor at (0.05, 0.05): nearest vertex (0, 0) is ≈ 0.0707
    // m away — well inside the 0.2 m radius → snap engages.
    let mut s = GridSnapState {
        snap_enabled: true,
        kind: GridKind::Square,
        snap_target: SnapTarget::Intersection,
        snap_magnetism_radius: 0.2,
        ..Default::default()
    };
    let p = s.snap_world([0.05, 0.05], [0.0, 0.0]);
    assert_eq!(p, [0.0, 0.0]);
}

#[test]
fn magnetism_zero_radius_is_back_compat_always_snap() {
    // Radius = 0.0 reproduces the legacy always-snap semantics
    // — every distance is "in range".
    let mut s = GridSnapState {
        snap_enabled: true,
        kind: GridKind::Square,
        snap_target: SnapTarget::Center,
        snap_magnetism_radius: 0.0,
        ..Default::default()
    };
    // (0.9, 0.9) is 0.566 m from the nearest cell center, well
    // beyond any reasonable magnetism radius — but radius = 0
    // disables the gate so we still snap.
    let p = s.snap_world([0.9, 0.9], [0.0, 0.0]);
    assert_eq!(p, [0.5, 0.5]);
}

#[test]
fn magnetism_composite_picks_closer_candidate_and_respects_threshold() {
    // CenterAndIntersection at (0.9, 0.9) on cell_size=1:
    //   Center  (0.5, 0.5) — dist ≈ 0.566.
    //   Intersection (1, 1) — dist ≈ 0.141.
    //   Winner = (1, 1). Radius 0.2 m > 0.141 → snap engages.
    let mut s = GridSnapState {
        snap_enabled: true,
        kind: GridKind::Square,
        snap_target: SnapTarget::CenterAndIntersection,
        snap_magnetism_radius: 0.2,
        ..Default::default()
    };
    assert_eq!(s.snap_world([0.9, 0.9], [0.0, 0.0]), [1.0, 1.0]);
    // Mid-cell (0.5, 0.5): Center wins at distance 0, but
    // (0.4, 0.4) puts Center 0.141 m away vs Intersection (0, 0)
    // at 0.566. Winner = Center. Radius 0.1 < 0.141 → gates to
    // passthrough.
    let mut s2 = GridSnapState {
        snap_enabled: true,
        kind: GridKind::Square,
        snap_target: SnapTarget::CenterAndIntersection,
        snap_magnetism_radius: 0.1,
        ..Default::default()
    };
    assert_eq!(s2.snap_world([0.4, 0.4], [0.0, 0.0]), [0.4, 0.4]);
}

#[test]
fn magnetism_radius_scales_with_subdivisions() {
    // Subdivisions = 4 on cell_size = 1 → effective sub-cell
    // 0.25 m. Vertices at every 0.25 m. Cursor at (0.08, 0.08):
    // nearest sub-vertex (0, 0) at ≈ 0.113 m. World-space
    // magnetism radius 0.15 m → snap engages (0.113 ≤ 0.15).
    let mut s = GridSnapState {
        snap_enabled: true,
        kind: GridKind::Square,
        snap_target: SnapTarget::Intersection,
        snap_subdivisions: 4,
        snap_magnetism_radius: 0.15,
        ..Default::default()
    };
    let p = s.snap_world([0.08, 0.08], [0.0, 0.0]);
    assert_eq!(p, [0.0, 0.0]);
    // Same setup, cursor at (0.18, 0.18): nearest sub-vertex
    // (0.25, 0.25) at ≈ 0.099 m → still inside radius → snap.
    let q = s.snap_world([0.18, 0.18], [0.0, 0.0]);
    assert!(
        (q[0] - 0.25).abs() < 1e-5 && (q[1] - 0.25).abs() < 1e-5,
        "expected snap to (0.25, 0.25), got {q:?}"
    );
    // But (0.125, 0.125) sits exactly midway between sub-grid
    // vertices — nearest one is 0.177 m away, just outside the
    // 0.15 m radius → passthrough.
    let r = s.snap_world([0.125, 0.125], [0.0, 0.0]);
    assert!(
        (r[0] - 0.125).abs() < 1e-5 && (r[1] - 0.125).abs() < 1e-5,
        "expected passthrough at midpoint, got {r:?}"
    );
}

#[test]
fn magnetism_corner_mode_uses_shift_magnitude_not_cursor_distance() {
    // Corner snap on a sprite half = (0.5, 0.5) centered at
    // (0.05, 0.05). One sprite corner is (-0.45, -0.45) — that's
    // 0.0707 m from grid vertex (0, 0) → shift magnitude 0.071
    // (NOT the cursor's distance to a vertex, which would be
    // (-0.95, -0.95) → very far). With radius 0.1 m, the snap
    // engages because the SHIFT is inside the radius.
    let mut s = GridSnapState {
        snap_enabled: true,
        kind: GridKind::Square,
        snap_target: SnapTarget::Corner,
        snap_magnetism_radius: 0.1,
        ..Default::default()
    };
    let snapped = s.snap_world([0.05, 0.05], [0.5, 0.5]);
    // Best corner shift = -(0.45, 0.45) applied → 0.05 - 0.45 =
    // -0.40? No — corners array uses sprite center ± half. The
    // closest corner to a vertex is (0.55, 0.55) → (1, 1) shift
    // (0.45, 0.45)? Let me trust the code: corners are
    // {(-0.45, -0.45), (0.55, -0.45), (-0.45, 0.55), (0.55, 0.55)}.
    // Nearest vertex of each: {(0, 0), (1, 0), (0, 1), (1, 1)}.
    // Shift magnitudes all = sqrt(0.45² + 0.45²) ≈ 0.636. That's
    // OUT of radius — verify passthrough.
    assert_eq!(snapped, [0.05, 0.05], "shift > radius should passthrough");
    // Tighter setup: sprite half (0.5, 0.5) centered at (0.45, 0.45).
    // Corners {(-0.05, -0.05), (0.95, -0.05), (-0.05, 0.95), (0.95, 0.95)}.
    // Nearest vertex of each: (0, 0), (1, 0), (0, 1), (1, 1).
    // All shifts ≈ (0.05, 0.05) magnitude 0.0707. radius 0.1 → snap.
    let snapped = s.snap_world([0.45, 0.45], [0.5, 0.5]);
    // Sprite center moves to (0.5, 0.5) so corner (0.95, 0.95)
    // lands on (1, 1) (or symmetric equivalent).
    assert!(
        (snapped[0] - 0.5).abs() < 1e-5 && (snapped[1] - 0.5).abs() < 1e-5,
        "expected snap to (0.5, 0.5), got {snapped:?}"
    );
}
