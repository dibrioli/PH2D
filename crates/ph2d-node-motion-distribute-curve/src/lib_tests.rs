//! Gates for [`super`] — split from `lib.rs` at the HR-18 LOC cap. It stays a CHILD
//! module (`#[path]`, not a sibling), so `use super::*` still reaches the private
//! `distribute`/`resolve_count`/`MotionDistributeCurve` these gates are about.

use super::*;

const LINE: [P2; 4] = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]];

/// The exact count is emitted, and on a straight line the points are evenly spaced
/// (equal chords). FALSIFIED by parameter-space sampling, which bunches at the ends.
#[test]
fn count_is_exact_and_evenly_spaced_on_a_line() {
    let pts = distribute(&LINE, 6, 0.0, false).0;
    assert_eq!(pts.len(), 6);
    let gaps: Vec<f32> = pts.windows(2).map(|w| w[1][0] - w[0][0]).collect();
    let g0 = gaps[0];
    for g in &gaps {
        assert!((g - g0).abs() < 2e-2, "even gaps: {gaps:?}");
    }
}

/// Every point lies ON the curve — for the S-curve default, each sample equals a
/// Bézier evaluation (no drift). A crude on-curve check: the y at the sampled x is
/// consistent with the curve.
#[test]
fn points_sit_on_the_curve() {
    let cp = [[-3.0, -1.5], [-1.0, 2.0], [1.0, -2.0], [3.0, 1.5]];
    let pts = distribute(&cp, 10, 0.0, false).0;
    for p in &pts {
        // Find the nearest LUT-sampled curve point; it should be within the sampling
        // resolution (the point came from an eval, so this is tight).
        let near = (0..=64)
            .map(|k| eval(&cp, k as f32 / 64.0))
            .map(|q| {
                let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
                dx * dx + dy * dy
            })
            .fold(f32::MAX, f32::min);
        assert!(near < 0.05, "on the curve (nearest² {near})");
    }
}

/// `offset` slides the points along the arc. A small offset (no wrap-around) shifts
/// every point uniformly along the length-3 line: `0.05 · 3 = +0.15` in x. FALSIFIED
/// by a dead offset (identical sets).
#[test]
fn offset_slides_along_the_arc() {
    let a = distribute(&LINE, 6, 0.0, false).0;
    let b = distribute(&LINE, 6, 0.05, false).0;
    for (pa, pb) in a.iter().zip(&b) {
        assert!(
            (pb[0] - pa[0] - 0.15).abs() < 2e-2,
            "slid +0.15: {pa:?} {pb:?}"
        );
    }
}

/// The S-curve of the defaults — it turns hard both ways, which is what makes it able to
/// tell a per-point heading from a single shared one.
const S: [P2; 4] = [[-3.0, -1.5], [-1.0, 2.0], [1.0, -2.0], [3.0, 1.5]];

/// **The heading is the direction the curve travels there** — and the oracle is the curve's
/// own SHAPE, not the tangent formula: the centred difference `p[i+1] − p[i−1]` approximates
/// the direction at `i` to second order, and it is computed with the standard-library
/// `atan2` in degrees, so it shares no code with the thing under test.
///
/// FALSIFIED by: one shared angle for every point · swapped `atan2` arguments · radians on
/// the wire · the tangent read at the arc fraction `s` instead of the curve parameter `t`.
#[test]
fn the_heading_matches_the_direction_the_curve_travels() {
    let (pos, rot) = distribute(&S, 64, 0.0, true);
    assert_eq!(rot.len(), 64, "one heading per point");

    let mut worst = 0.0f32;
    for i in 1..pos.len() - 1 {
        let (dx, dy) = (pos[i + 1][0] - pos[i - 1][0], pos[i + 1][1] - pos[i - 1][1]);
        let want = dy.atan2(dx).to_degrees();
        let mut err = (rot[i] - want).abs();
        if err > 180.0 {
            err = 360.0 - err; // the seam is a wrap, not a disagreement
        }
        worst = worst.max(err);
    }
    assert!(worst < 1.0, "worst heading error {worst} deg");
}

/// Degrees, and the axis that says so: on a straight line UP every instance reads **+90**.
/// A horizontal line reads 0 — which is also what a dead `align` and a radian wire read,
/// so the vertical half is the one with teeth.
#[test]
fn a_vertical_line_reads_ninety_degrees() {
    let up = [[0.0, 0.0], [0.0, 1.0], [0.0, 2.0], [0.0, 3.0]];
    for r in distribute(&up, 5, 0.0, true).1 {
        assert!((r - 90.0).abs() < 0.2, "up is +90 deg, got {r}");
    }
    for r in distribute(&LINE, 5, 0.0, true).1 {
        assert!(r.abs() < 0.2, "+x is 0 deg, got {r}");
    }
}

/// **Aligning does not move a single point** — byte for byte, on a curve chosen for being
/// hard to sample. The heading is something the node *also* reports, never something that
/// re-decides where an instance goes.
#[test]
fn aligning_does_not_move_a_single_point() {
    let plain = distribute(&S, 33, 0.17, false);
    let aligned = distribute(&S, 33, 0.17, true);
    assert_eq!(plain.0, aligned.0, "positions are untouched by align");
    assert!(plain.1.is_empty(), "align off reports no heading");
}

/// **A graph that never heard of `align` emits `P` and nothing else.** It does not name the
/// default — it exercises it, which is the only way a default is actually under test.
#[test]
fn a_graph_that_never_heard_of_align_emits_no_rotation() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::Graph;

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionDistributeCurve as &dyn NodeOp)
        }
    }
    let mut g = Graph::new();
    let n = g.add_node("motion.distribute_curve");
    g.set_param(n, "count", 8.0);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
    let s = out[0].as_stream();
    assert!(s.get("P").is_some(), "still places points");
    assert!(s.get("rot").is_none(), "and reports no heading");
}

/// Deterministic + cooks through the registry, emitting `P` at the exact count.
#[test]
fn registers_and_cooks() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::Graph;

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionDistributeCurve as &dyn NodeOp)
        }
    }
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());

    let mut g = Graph::new();
    let n = g.add_node("motion.distribute_curve");
    g.set_param(n, "count", 20.0);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
    match out[0].as_stream().get("P").unwrap() {
        Column::Vec2(v) => assert_eq!(v.len(), 20),
        _ => panic!("P"),
    }
}

// ── `mode` : Count | Length ─────────────────────────────────────────────

fn cook_curve(
    set: &dyn Fn(&mut ph2d_nodegraph::graph::Graph, ph2d_nodegraph::graph::NodeId),
) -> Stream {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::Graph;

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionDistributeCurve as &dyn NodeOp)
        }
    }
    let mut g = Graph::new();
    let n = g.add_node("motion.distribute_curve");
    set(&mut g, n);
    let mut cook = Cook::new();
    cook.cook(&g, &Ops, n, 0.0).unwrap()[0].as_stream().clone()
}

/// **`Count` runs the call it always ran.** Not "produces the same numbers" — the same
/// `param_as_count` with the same argument, which is what makes the default structural.
#[test]
fn count_mode_is_the_call_that_always_shipped() {
    for c in [0.0, 1.0, 32.0, 7.5, -4.0, f32::NAN, 1e9] {
        assert_eq!(
            resolve_count(&S, 0.0, c, 0.25),
            param_as_count(c, RECOMMENDED_MAX_ELEMENTS),
            "count {c}",
        );
    }
}

/// **The spacing the artist gets is the spacing they asked for**, to within the half-step a
/// whole number of points allows. The oracle is the DELIVERED geometry: the chord between
/// neighbours on a straight curve (arc == chord), never the count the resolver returned.
///
/// ⚠️ **The fixture is built to CONTAIN the difference between `round` and `floor`.** The
/// first draft used a 9-unit line and the spacings `0.25/0.5/1.0/1.5`, every one of which
/// divides 9 exactly — the two laws agree on all four, so the gate could not have failed for
/// the reason it claims. These four leave a fraction ABOVE a half, where they disagree; and
/// the bar is `round`'s own bound (`0.5·w/n`, half of `floor`'s), so only `round` clears it.
#[test]
fn a_length_spacing_places_them_that_far_apart() {
    let line: [P2; 4] = [
        [0.0, 0.0],
        [10.0 / 3.0, 0.0],
        [20.0 / 3.0, 0.0],
        [10.0, 0.0],
    ];
    for want in [0.35_f32, 0.6, 1.3, 2.2] {
        let frac = (10.0 / want).fract();
        assert!(
            (0.5..1.0).contains(&frac),
            "fixture: {want} leaves fraction {frac}, where round == floor",
        );
        let n = resolve_count(&line, 1.0, 32.0, want);
        let (pos, _) = distribute(&line, n, 0.0, false);
        let step = pos[1][0] - pos[0][0];
        assert!(
            (step - want).abs() <= 0.5 * want / n as f32,
            "asked {want}, got {step} over {n} points",
        );
    }
}

/// Halving the spacing asks for twice the points. The relationship, not one number.
#[test]
fn halving_the_spacing_doubles_the_count() {
    let a = resolve_count(&S, 1.0, 32.0, 0.4);
    let b = resolve_count(&S, 1.0, 32.0, 0.2);
    assert_eq!(b, a * 2, "{a} -> {b}");
}

/// **`Length` ignores `count`, `Count` ignores `spacing`** — each mode reads one number, which
/// is what the `ParamGate` pair promises the artist by hiding the other.
#[test]
fn each_mode_reads_only_its_own_number() {
    assert_eq!(
        resolve_count(&S, 1.0, 32.0, 0.25),
        resolve_count(&S, 1.0, 999.0, 0.25),
        "Length must not read count",
    );
    assert_eq!(
        resolve_count(&S, 0.0, 32.0, 0.25),
        resolve_count(&S, 0.0, 32.0, 9.99),
        "Count must not read spacing",
    );
}

/// A param is an `f32` a graph can carry: zero, NaN and negative spacings land on the clamp,
/// never on a panic and never on a count the loop cannot allocate.
#[test]
fn a_hostile_spacing_lands_on_the_clamp() {
    assert_eq!(resolve_count(&S, 1.0, 32.0, 0.0), RECOMMENDED_MAX_ELEMENTS);
    assert_eq!(resolve_count(&S, 1.0, 32.0, f32::NAN), 1);
    assert_eq!(resolve_count(&S, 1.0, 32.0, -1.0), 1);
    assert_eq!(resolve_count(&S, 1.0, 32.0, 1e9), 1, "one point, not zero");
}

/// **The mode moves no point.** Asked for the spacing that a given count already produces,
/// `Length` lays out the SAME set — because the two only choose a number for one loop.
#[test]
fn the_mode_chooses_a_count_it_does_not_change_the_layout() {
    let total = curve::total_len(&curve::arc_lut(&S));
    let by_count = distribute(&S, 20, 0.13, false).0;
    let n = resolve_count(&S, 1.0, 0.0, total / 20.0);
    assert_eq!(n, 20, "the spacing that 20 points make asks for 20 points");
    assert_eq!(by_count, distribute(&S, n, 0.13, false).0);
}

/// **The seam.** Every gate above calls `resolve_count` directly, so all of them stay green
/// with `ctx.param("mode")` unread. This one authors the params on a graph and cooks.
#[test]
fn the_authored_mode_reaches_the_layout() {
    let by_count = cook_curve(&|g, n| g.set_param(n, "count", 20.0));
    assert_eq!(by_count.count(), 20);

    // 7.30 / 0.5 rounds to 15.
    let by_length = cook_curve(&|g, n| {
        g.set_param(n, "count", 20.0);
        g.set_param(n, "mode", 1.0);
        g.set_param(n, "spacing", 0.5);
    });
    assert_eq!(
        by_length.count(),
        15,
        "the authored mode reaches the layout"
    );
}

/// The `spacing` DEFAULT is measured against the `count` default, not chosen: flipping the
/// mode on an untouched node is a nudge in density, never a jump.
///
/// ⚠️ It names neither number — it reads both off the manifest, so the day someone re-tunes
/// the default curve this fails instead of quietly drifting.
#[test]
fn the_two_defaults_describe_the_same_density() {
    let def = |name: &str| {
        MANIFEST
            .params
            .iter()
            .find(|p| p.name == name)
            .expect("param")
            .default
    };
    let by_count = param_as_count(def("count"), RECOMMENDED_MAX_ELEMENTS);
    let by_length = resolve_count(&S, 1.0, 0.0, def("spacing"));
    let ratio = by_length as f32 / by_count as f32;
    assert!(
        (0.8..=1.25).contains(&ratio),
        "count default {by_count} vs spacing default {by_length}",
    );
}
