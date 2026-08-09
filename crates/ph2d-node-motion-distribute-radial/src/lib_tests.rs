//! Gates for [`super`] — split from `lib.rs` at the HR-18 LOC cap. It stays a CHILD
//! module (`#[path]`, not a sibling), so `use super::*` still reaches the private
//! `radial`/`Wedge`/`MotionDistributeRadial` these gates are about.

use super::*;

/// The positions alone. Every gate written before `align` existed asks only about those, and
/// threading a `false` through each call would have buried the one place the flag is the
/// subject — so the flag has its OWN gates and these keep reading as they read.
fn radial_pos(
    count: usize,
    rings: usize,
    radius: f32,
    inner: f32,
    spin_cycles: f32,
    wedge: Wedge,
) -> Vec<[f32; 2]> {
    radial(count, rings, radius, inner, spin_cycles, wedge, false).0
}

fn radius_of(p: [f32; 2]) -> f32 {
    (p[0] * p[0] + p[1] * p[1]).sqrt()
}

/// Degrees CCW from +x, folded to `[0, 360)` — the standard-library `atan2`, so the oracle
/// shares nothing with the parabolic `cos_sin_cycles` it judges.
fn angle_of(p: [f32; 2]) -> f32 {
    p[1].atan2(p[0]).to_degrees().rem_euclid(360.0)
}

/// **The layout as it stood before the wedge existed**, frozen verbatim under `cfg(test)`.
/// A `pub` copy with no caller would be a second answer waiting for someone to call it; this
/// one exists only to be disagreed with.
fn radial_before_the_wedge(
    count: usize,
    rings: usize,
    radius: f32,
    inner: f32,
    spin_cycles: f32,
) -> Vec<[f32; 2]> {
    let mut out = Vec::with_capacity(count);
    let per = ring_counts(count, rings);
    for (r, &n_ring) in per.iter().enumerate() {
        let rr = if rings > 1 {
            inner + (radius - inner) * r as f32 / (rings as f32 - 1.0)
        } else {
            radius
        };
        for k in 0..n_ring {
            let cycles = k as f32 / n_ring.max(1) as f32 + spin_cycles;
            let (c, s) = cos_sin_cycles(cycles);
            out.push([rr * c, rr * s]);
        }
    }
    out
}

/// **The circle that always shipped is BYTE-identical** — every point, every bit, across a
/// spread of counts, rings and spins. This is what makes `0 .. 360` a default that costs
/// nothing rather than a default that is merely close.
#[test]
fn the_full_circle_is_byte_identical_to_the_law_that_shipped() {
    for &(count, rings, spin) in &[
        (60usize, 3usize, 0.0f32),
        (8, 1, 0.0),
        (61, 4, 0.137),
        (1, 1, -0.4),
        (255, 7, 1.75),
    ] {
        let before = radial_before_the_wedge(count, rings, 3.0, 0.6, spin);
        let now = radial_pos(count, rings, 3.0, 0.6, spin, Wedge::FULL);
        assert_eq!(before, now, "count {count} rings {rings} spin {spin}");
        let wired = radial_pos(
            count,
            rings,
            3.0,
            0.6,
            spin,
            Wedge::from_degrees(0.0, 360.0),
        );
        assert_eq!(before, wired, "0..360 is FULL: {count}/{rings}");
    }
}

/// **The wedge PACKS, it does not CULL** — the distinction the whole param exists for. The
/// composition it replaces (`field.radial_sweep → motion.cull`) would have returned FEWER
/// than `count` points; this returns all of them, inside the window.
#[test]
fn the_wedge_packs_the_points_it_does_not_cull_them() {
    let pts = radial_pos(8, 1, 2.0, 0.0, 0.0, Wedge::from_degrees(0.0, 180.0));
    assert_eq!(pts.len(), 8, "every point asked for is placed");
    for p in &pts {
        let a = angle_of(*p);
        assert!((-0.2..=180.2).contains(&a), "inside the wedge: {a} deg");
    }
}

/// **An open wedge lands on both of its ends** — first on `start`, last on `end`. This is the
/// half that separates a fan from a ring, and the half an artist checks first.
#[test]
fn an_open_wedge_lands_on_both_of_its_ends() {
    let pts = radial_pos(5, 1, 2.0, 0.0, 0.0, Wedge::from_degrees(20.0, 200.0));
    assert!((angle_of(pts[0]) - 20.0).abs() < 0.3, "{:?}", pts[0]);
    assert!((angle_of(pts[4]) - 200.0).abs() < 0.3, "{:?}", pts[4]);
    // And evenly, in between: 45 deg of wedge per step.
    for k in 0..4 {
        let step = angle_of(pts[k + 1]) - angle_of(pts[k]);
        assert!((step - 45.0).abs() < 0.3, "even step {step}");
    }
}

/// **A closed wedge does not double up its seam** — the price of the inclusive law, and the
/// reason the node reads `wraps` off the geometry instead of offering it as a mode. Eight
/// points over a full turn step by 45 deg and the last does NOT sit on the first.
#[test]
fn a_closed_wedge_does_not_double_up_its_seam() {
    let pts = radial_pos(8, 1, 2.0, 0.0, 0.0, Wedge::from_degrees(0.0, 360.0));
    assert!((angle_of(pts[7]) - 315.0).abs() < 0.3, "{:?}", pts[7]);
    let (dx, dy) = (pts[7][0] - pts[0][0], pts[7][1] - pts[0][1]);
    assert!(dx * dx + dy * dy > 1.0, "the seam is not a stack");
}

/// `spin` carries the wedge with it: the fan is an EXTENT and the spin is a ROTATION, so
/// they compose instead of being two doors onto the same number.
#[test]
fn the_spin_carries_the_wedge_with_it() {
    let wedge = Wedge::from_degrees(0.0, 90.0);
    let still = radial_pos(4, 1, 2.0, 0.0, 0.0, wedge);
    let spun = radial_pos(4, 1, 2.0, 0.0, 0.25, wedge);
    for (a, b) in still.iter().zip(&spun) {
        let d = (angle_of(*b) - angle_of(*a)).rem_euclid(360.0);
        assert!(
            (d - 90.0).abs() < 0.3,
            "the whole fan turned 90 deg, got {d}"
        );
    }
}

/// **A graph that never heard of the wedge draws the circle it always drew.** It does not
/// name the defaults — it cooks through the registry without touching them.
#[test]
fn a_graph_that_never_heard_of_the_wedge_draws_the_circle() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::Graph;

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionDistributeRadial as &dyn NodeOp)
        }
    }
    let mut g = Graph::new();
    let n = g.add_node("motion.distribute_radial");
    g.set_param(n, "count", 24.0);
    g.set_param(n, "rings", 2.0);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
    let Some(Column::Vec2(got)) = out[0].as_stream().get("P") else {
        panic!("P")
    };
    assert_eq!(*got, radial_before_the_wedge(24, 2, 3.0, 0.6, 0.0));
}

/// **The authored wedge REACHES the layout.** Every other wedge gate calls `radial` directly,
/// so all of them stay green with the two params unread — this is the one that walks the seam
/// from `set_param` to a placed point.
#[test]
fn the_authored_wedge_reaches_the_layout() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::Graph;

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionDistributeRadial as &dyn NodeOp)
        }
    }
    let mut g = Graph::new();
    let n = g.add_node("motion.distribute_radial");
    g.set_param(n, "count", 8.0);
    g.set_param(n, "rings", 1.0);
    g.set_param(n, "start_angle", 0.0);
    g.set_param(n, "end_angle", 90.0);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
    let Some(Column::Vec2(got)) = out[0].as_stream().get("P") else {
        panic!("P")
    };
    assert_eq!(got.len(), 8, "the count survives the wedge");
    for p in got {
        let a = angle_of(*p);
        assert!((-0.2..=90.2).contains(&a), "inside the quarter: {a} deg");
    }
}

/// A single ring: every point sits on the outer radius, equally spaced. FALSIFIED
/// if they landed at mixed radii (that would be a spiral, not a ring).
#[test]
fn a_single_ring_is_evenly_spaced_on_the_radius() {
    let pts = radial_pos(8, 1, 3.0, 0.6, 0.0, Wedge::FULL);
    assert_eq!(pts.len(), 8);
    for p in &pts {
        // ~1e-2 tolerance: the parabolic cos_sin_cycles is ~0.09% off unit.
        assert!((radius_of(*p) - 3.0).abs() < 1e-2, "on the radius: {p:?}");
    }
    // Equal spacing: consecutive points differ by 1/8 turn — the same chord length.
    let chord = |a: [f32; 2], b: [f32; 2]| {
        let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
        (dx * dx + dy * dy).sqrt()
    };
    let d0 = chord(pts[0], pts[1]);
    for k in 1..8 {
        assert!(
            (chord(pts[k], pts[(k + 1) % 8]) - d0).abs() < 2e-2,
            "equal chords"
        );
    }
}

/// `rings` concentric rings: every point lands between `inner` and `radius`, and
/// more than one distinct radius appears.
#[test]
fn rings_are_concentric_between_inner_and_radius() {
    let pts = radial_pos(60, 3, 3.0, 1.0, 0.0, Wedge::FULL);
    assert_eq!(pts.len(), 60);
    let mut radii: Vec<f32> = pts.iter().map(|p| radius_of(*p)).collect();
    for r in &radii {
        assert!(
            *r >= 1.0 - 1e-2 && *r <= 3.0 + 1e-2,
            "within [inner, radius]: {r}"
        );
    }
    radii.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!(radii[59] - radii[0] > 1.0, "spans multiple rings");
}

/// The exact count is honoured even when it doesn't divide evenly across rings.
#[test]
fn count_is_exact_across_uneven_rings() {
    assert_eq!(radial_pos(61, 4, 3.0, 0.5, 0.0, Wedge::FULL).len(), 61);
    assert_eq!(ring_counts(61, 4), vec![16, 15, 15, 15]);
}

/// `spin` rotates the whole array: a quarter-turn spin moves a point that was on
/// +x onto +y. FALSIFIED by a dead spin (the point stays on +x).
#[test]
fn spin_rotates_the_array() {
    let base = radial_pos(4, 1, 2.0, 0.0, 0.0, Wedge::FULL); // points at 0°, 90°, 180°, 270°
    let spun = radial_pos(4, 1, 2.0, 0.0, 0.25, Wedge::FULL); // +90°
    assert!(
        base[0][0] > 1.9 && base[0][1].abs() < 1e-3,
        "base point on +x"
    );
    assert!(
        spun[0][1] > 1.9 && spun[0][0].abs() < 1e-3,
        "spun point on +y: {:?}",
        spun[0]
    );
}

/// Deterministic + cooks through the registry, emitting `P` at the exact count.
#[test]
fn registers_and_cooks() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::Graph;

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionDistributeRadial as &dyn NodeOp)
        }
    }
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());

    let mut g = Graph::new();
    let n = g.add_node("motion.distribute_radial");
    g.set_param(n, "count", 24.0);
    g.set_param(n, "rings", 2.0);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
    match out[0].as_stream().get("P").unwrap() {
        Column::Vec2(v) => assert_eq!(v.len(), 24),
        _ => panic!("P"),
    }
}

// ── `align` ─────────────────────────────────────────────────────────────

/// Cook the node through the REGISTRY with whatever params the caller authors — the only
/// path that walks `ctx.param` to an emitted column.
fn cook_radial(set: &dyn Fn(&mut ph2d_nodegraph::graph::Graph, NodeId)) -> Stream {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::Graph;

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionDistributeRadial as &dyn NodeOp)
        }
    }
    let mut g = Graph::new();
    let n = g.add_node("motion.distribute_radial");
    set(&mut g, n);
    let mut cook = Cook::new();
    cook.cook(&g, &Ops, n, 0.0).unwrap()[0].as_stream().clone()
}

use ph2d_nodegraph::graph::NodeId;

fn rots(s: &Stream) -> Vec<f32> {
    match s.get("rot") {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// **A graph that never heard of `align` emits `P` and nothing else.**
///
/// The param is APPENDED, so a saved graph reads it as absent, takes the `0.0` default and
/// lays out the stream it always laid out. This is the byte-identity claim, asserted through
/// the real cook rather than promised in a comment.
#[test]
fn a_graph_that_never_heard_of_align_emits_no_rotation() {
    let s = cook_radial(&|_, _| {});
    assert!(s.get("P").is_some(), "the layout still happens");
    assert!(s.get("rot").is_none(), "and it reports no heading");
}

/// Turning `align` on **moves no point**. The heading is a second column, never a nudge.
#[test]
fn aligning_does_not_move_a_single_point() {
    let plain = radial(37, 3, 3.0, 0.6, 0.13, Wedge::FULL, false);
    let aligned = radial(37, 3, 3.0, 0.6, 0.13, Wedge::FULL, true);
    assert_eq!(plain.0, aligned.0, "positions are untouched by align");
    assert!(plain.1.is_empty(), "align off reports no heading");
    assert_eq!(aligned.1.len(), 37, "align on reports one per point");
}

/// **The clone faces outward**, checked against an INDEPENDENT oracle: `atan2` of the
/// position the layout produced. The node never calls `atan2` (module docs) — this gate does,
/// which is exactly what makes it an oracle instead of a mirror.
#[test]
fn the_clone_faces_outward_along_its_own_radius() {
    let (pos, rot) = radial(12, 1, 2.0, 0.0, 0.0, Wedge::FULL, true);
    for (p, r) in pos.iter().zip(&rot) {
        let want = p[1].atan2(p[0]).to_degrees();
        let got = ((r - want + 180.0).rem_euclid(360.0)) - 180.0;
        assert!(got.abs() < 0.5, "heading {r} vs outward {want} at {p:?}");
    }
}

/// **A clone at the centre still has a heading** — the case an `atan2` of the position
/// cannot answer (`atan2(0, 0)` is `0` for every one of them).
///
/// With `inner = 0` the innermost ring sits at radius zero, so all its points share a
/// position; their LAYOUT angles are still the `k/n` turns that spaced them, and that is
/// what they report.
#[test]
fn a_clone_at_the_centre_still_reports_the_angle_that_spaced_it() {
    let (pos, rot) = radial(8, 2, 2.0, 0.0, 0.0, Wedge::FULL, true);
    // Ring 0 is the degenerate one: four points, all at the origin.
    let ring0 = &pos[..4];
    assert!(
        ring0.iter().all(|p| p[0].abs() < 1e-6 && p[1].abs() < 1e-6),
        "the fixture must CONTAIN the phenomenon: ring 0 is at the centre, got {ring0:?}",
    );
    let mut seen = rot[..4].to_vec();
    seen.sort_by(f32::total_cmp);
    seen.dedup();
    assert_eq!(
        seen.len(),
        4,
        "four distinct headings, not four zeroes: {seen:?}"
    );
}

/// `spin` rotates the array, so it rotates the headings with it — they are the same number.
#[test]
fn spin_carries_the_heading_with_it() {
    let still = radial(6, 1, 2.0, 0.0, 0.0, Wedge::FULL, true).1;
    let spun = radial(6, 1, 2.0, 0.0, 0.25, Wedge::FULL, true).1;
    for (a, b) in still.iter().zip(&spun) {
        assert!((b - a - 90.0).abs() < 1e-3, "{a} -> {b} is not +90 deg");
    }
}

/// **The heading is NOT wrapped**, and that is deliberate (module docs): this node knows the
/// winding, and an `atan2` would throw it away. A spin past a full turn reads past 360.
#[test]
fn the_heading_keeps_its_winding_instead_of_wrapping() {
    let rot = radial(4, 1, 2.0, 0.0, 2.5, Wedge::FULL, true).1;
    assert!(rot[0] > 890.0, "2.5 turns of spin reads {} deg", rot[0]);
}

/// **The seam.** Every other gate builds the layout by calling `radial` directly, so all of
/// them stay green with `ctx.param("align")` unread. This one authors the param on a graph
/// and cooks, which is the only path that walks param -> layout -> emitted column.
#[test]
fn the_authored_align_reaches_the_layout() {
    let off = cook_radial(&|g, n| g.set_param(n, "count", 9.0));
    assert!(rots(&off).is_empty(), "unauthored stays off");

    let on = cook_radial(&|g, n| {
        g.set_param(n, "count", 9.0);
        g.set_param(n, "align", 1.0);
    });
    assert_eq!(rots(&on).len(), 9, "the authored align reaches the layout");
}
