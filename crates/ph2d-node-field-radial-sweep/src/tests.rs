//! Tests for `field.radial_sweep` — the pseudo-angle sector math, the repetition
//! fold, the full-circle neutral, the radial clip, and the falloff composition.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("field.radial_sweep.test.src"),
    name: "field.radial_sweep.test.src",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
struct Src(Vec<[f32; 2]>);
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(self.0.len()).with("P", Column::Vec2(self.0.clone())));
    }
}
struct Ops {
    src: Src,
}
impl Ops {
    fn new(pts: Vec<[f32; 2]>) -> Self {
        Ops { src: Src(pts) }
    }
}
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&self.src),
            t if t == MANIFEST.id => Some(&FieldRadialSweep),
            _ => None,
        }
    }
}

fn falloff_of(g: &Graph, ops: &Ops, target: NodeId) -> Vec<f32> {
    let mut cook = Cook::new();
    let out = cook.cook(g, ops, target, 0.0).unwrap();
    match out[0].as_stream().get("falloff").unwrap() {
        Column::Scalar(v) => v.clone(),
        _ => panic!("falloff must be a Scalar column"),
    }
}

fn chain() -> (Graph, NodeId) {
    let mut g = Graph::new();
    let src = g.add_node("field.radial_sweep.test.src");
    let sw = g.add_node("field.radial_sweep");
    g.connect(Edge {
        from: (src, 0),
        to: (sw, 0),
        delayed: false,
    })
    .unwrap();
    (g, sw)
}

// A point on the ray at `deg` degrees, at distance `r` from the origin — the honest
// input for an angular test (via true trig, NOT the node's parabolic one). Kept well
// inside the default `radius = 10` so the radial ramp is 1 and we isolate the angle.
fn ray(deg: f32, r: f32) -> [f32; 2] {
    let a = deg.to_radians();
    [r * a.cos(), r * a.sin()]
}

#[test]
fn default_wedge_is_one_inside_zero_outside() {
    // Default sector [0°, 60°], radius 10, soft 0.15, curve Smooth. A ray at 30°
    // (mid-sector) well inside the radius is fully in → 1; a ray at 120° is outside
    // the wedge → 0; a ray at 30° but BEYOND the radius (r = 20) is outside the disk
    // → 0.
    let (g, sw) = chain();
    let ops = Ops::new(vec![ray(30.0, 5.0), ray(120.0, 5.0), ray(30.0, 20.0)]);
    let got = falloff_of(&g, &ops, sw);
    assert!((got[0] - 1.0).abs() < 1e-4, "mid-sector inside: {}", got[0]);
    assert_eq!(got[1], 0.0, "outside the wedge");
    assert_eq!(got[2], 0.0, "beyond the radius");
}

#[test]
fn rotation_spins_the_wedge() {
    // Rotate the field +90°: the [0,60] wedge now covers [90,150]. A ray at 120° was
    // OUT (default) and is now IN; a ray at 30° was IN and is now OUT.
    let (mut g, sw) = chain();
    g.set_param(sw, "soft", 0.0); // hard edges, so the membership test is crisp
    g.set_param(sw, "rotation", 90.0);
    let ops = Ops::new(vec![ray(120.0, 5.0), ray(30.0, 5.0)]);
    let got = falloff_of(&g, &ops, sw);
    assert_eq!(got[0], 1.0, "120° now inside the rotated wedge");
    assert_eq!(got[1], 0.0, "30° now outside");
}

#[test]
fn repetitions_tile_the_sector_around_the_circle() {
    // 4 repetitions of a [0,30] wedge ⇒ wedges at [0,30], [90,120], [180,210],
    // [270,300]. A ray at 100° falls in the second copy → 1; a ray at 60° is in a GAP
    // between copies → 0.
    let (mut g, sw) = chain();
    g.set_param(sw, "soft", 0.0);
    g.set_param(sw, "end_angle", 30.0);
    g.set_param(sw, "repetitions", 4.0);
    let ops = Ops::new(vec![ray(100.0, 5.0), ray(60.0, 5.0), ray(15.0, 5.0)]);
    let got = falloff_of(&g, &ops, sw);
    assert_eq!(got[0], 1.0, "100° in the 2nd copy");
    assert_eq!(got[1], 0.0, "60° in a gap");
    assert_eq!(got[2], 1.0, "15° in the 1st copy");
}

#[test]
fn full_circle_is_the_identity_disk() {
    // The neutral (D12): end − start ≥ 360 ⇒ a full disk. With a radius larger than
    // the points and soft 0, the mask is 1 everywhere in the disk — including the
    // ANTIPODE of the sector mid (180° from where a wedge would point), which a
    // distance-from-mid model would seam. Fields multiply, so falloff ← 1·falloff.
    let (mut g, sw) = chain();
    g.set_param(sw, "start_angle", 0.0);
    g.set_param(sw, "end_angle", 360.0);
    g.set_param(sw, "radius", 100.0);
    g.set_param(sw, "soft", 0.0);
    // Rays all around the circle, all inside the huge radius.
    let ops = Ops::new(vec![
        ray(0.0, 5.0),
        ray(90.0, 5.0),
        ray(180.0, 5.0),
        ray(270.0, 5.0),
        ray(210.0, 5.0),
    ]);
    assert_eq!(falloff_of(&g, &ops, sw), vec![1.0; 5]);
}

#[test]
fn a_wider_wedge_covers_a_wider_arc() {
    // A 180° sector [0,180] with hard edges: rays in the upper half-plane are in,
    // rays in the lower half-plane are out — the pseudo-angle membership is EXACT
    // across the octant boundaries (0/90/180 all handled by the branch table).
    let (mut g, sw) = chain();
    g.set_param(sw, "soft", 0.0);
    g.set_param(sw, "end_angle", 180.0);
    let ops = Ops::new(vec![
        ray(45.0, 5.0),
        ray(135.0, 5.0),
        ray(225.0, 5.0),
        ray(315.0, 5.0),
    ]);
    assert_eq!(falloff_of(&g, &ops, sw), vec![1.0, 1.0, 0.0, 0.0]);
}

#[test]
fn radius_clips_the_disk() {
    // Along the mid-ray (30°, always inside the wedge), the mask is a radial edge_ramp:
    // 1 deep inside, 0 at/after the radius. soft 0 ⇒ hard edge at r = radius.
    let (mut g, sw) = chain();
    g.set_param(sw, "soft", 0.0);
    g.set_param(sw, "radius", 6.0);
    let ops = Ops::new(vec![ray(30.0, 1.0), ray(30.0, 5.9), ray(30.0, 6.1)]);
    let got = falloff_of(&g, &ops, sw);
    assert_eq!(got[0], 1.0);
    assert_eq!(got[1], 1.0);
    assert_eq!(got[2], 0.0);
}

#[test]
fn soft_ramps_both_edges_between_zero_and_one() {
    // With soft > 0 the wedge and the disk both feather. A ray just inside the angular
    // edge of the default [0,60] wedge (say 58°) sits on the ramp → strictly between
    // 0 and 1; a ray near the radial edge does too. Curve Linear keeps the ramp plain.
    let (mut g, sw) = chain();
    g.set_param(sw, "curve", 0.0); // Linear, so we can reason about the ramp
    g.set_param(sw, "soft", 0.3);
    let ops = Ops::new(vec![ray(58.0, 5.0), ray(30.0, 9.6)]);
    let got = falloff_of(&g, &ops, sw);
    assert!(got[0] > 0.0 && got[0] < 1.0, "angular ramp: {}", got[0]);
    assert!(got[1] > 0.0 && got[1] < 1.0, "radial ramp: {}", got[1]);
}

#[test]
fn invert_flips_the_mask() {
    // Default wedge: a mid-sector inside point → 1-1 = 0; an outside point → 1-0 = 1.
    let (mut g, sw) = chain();
    g.set_param(sw, "soft", 0.0);
    g.set_param(sw, "invert", 1.0);
    let ops = Ops::new(vec![ray(30.0, 5.0), ray(200.0, 5.0)]);
    assert_eq!(falloff_of(&g, &ops, sw), vec![0.0, 1.0]);
}

#[test]
fn center_moves_the_pivot() {
    // Shift the centre to (5, 0): the sweep now pivots there. A point at (10, 0) has
    // local offset (5, 0) — angle 0°, inside [0,60], within radius 10 → 1. A point at
    // world (0,0) has local (−5,0) — angle 180°, outside the wedge → 0.
    let (mut g, sw) = chain();
    g.set_param(sw, "soft", 0.0);
    g.set_param(sw, "center_x", 5.0);
    let ops = Ops::new(vec![[10.0, 0.0], [0.0, 0.0]]);
    assert_eq!(falloff_of(&g, &ops, sw), vec![1.0, 0.0]);
}

#[test]
fn a_prior_falloff_column_is_multiplied_not_overwritten() {
    static FSRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("field.radial_sweep.test.fsrc"),
        name: "field.radial_sweep.test.fsrc",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: INST_VEC2,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct FSrc;
    impl NodeOp for FSrc {
        fn manifest(&self) -> &'static NodeManifest {
            &FSRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            // 30° inside the default wedge, 200° outside; a prior falloff on each.
            ctx.emit(
                Stream::new(2)
                    .with("P", Column::Vec2(vec![[4.33, 2.5], [-4.7, -1.7]]))
                    .with("falloff", Column::Scalar(vec![0.5, 0.9])),
            );
        }
    }
    struct FOps;
    impl OpResolver for FOps {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == FSRC_MAN.id => Some(&FSrc),
                t if t == MANIFEST.id => Some(&FieldRadialSweep),
                _ => None,
            }
        }
    }
    let mut g = Graph::new();
    let src = g.add_node("field.radial_sweep.test.fsrc");
    let sw = g.add_node("field.radial_sweep");
    g.set_param(sw, "soft", 0.0);
    g.connect(Edge {
        from: (src, 0),
        to: (sw, 0),
        delayed: false,
    })
    .unwrap();
    // (4.33, 2.5) ≈ 30° at r=5 → mask 1; (−4.7,−1.7) ≈ 200° → mask 0. Composed:
    // 0.5·1, 0.9·0.
    let mut cook = Cook::new();
    let out = cook.cook(&g, &FOps, sw, 0.0).unwrap();
    match out[0].as_stream().get("falloff").unwrap() {
        Column::Scalar(v) => assert_eq!(v, &vec![0.5, 0.0]),
        _ => panic!("falloff"),
    }
}

#[test]
fn pseudo_angle_is_monotone_with_true_angle() {
    // The spine of the HR-5 angular test: the diamond pseudo-angle preserves the
    // ORDER of the true angle over a full turn, so sector membership is exact. Sweep
    // 0..360° and assert the pseudo-angle strictly increases, staying in [0,4).
    let mut prev = -1.0;
    for deg in 0..360 {
        let a = (deg as f32).to_radians();
        let pa = pseudo_angle(a.cos(), a.sin());
        assert!((0.0..4.0).contains(&pa), "pa in range at {deg}: {pa}");
        assert!(pa > prev, "monotone at {deg}: {pa} !> {prev}");
        prev = pa;
    }
    // Anchors: the octant corners land on the integers.
    assert!((pseudo_angle(1.0, 0.0) - 0.0).abs() < 1e-6);
    assert!((pseudo_angle(0.0, 1.0) - 1.0).abs() < 1e-6);
    assert!((pseudo_angle(-1.0, 0.0) - 2.0).abs() < 1e-6);
    assert!((pseudo_angle(0.0, -1.0) - 3.0).abs() < 1e-6);
}

#[test]
fn empty_sector_masks_nothing() {
    // start == end ⇒ a zero-width wedge ⇒ the mask is 0 everywhere but the exact ray.
    let (mut g, sw) = chain();
    g.set_param(sw, "soft", 0.0);
    g.set_param(sw, "end_angle", 0.0); // == start_angle
    let ops = Ops::new(vec![ray(0.0, 5.0), ray(1.0, 5.0), ray(30.0, 5.0)]);
    let got = falloff_of(&g, &ops, sw);
    // On the ray (0°) it is 1; a degree off it is 0.
    assert_eq!(got[0], 1.0);
    assert_eq!(got[1], 0.0);
    assert_eq!(got[2], 0.0);
}

#[test]
fn curves_are_monotone_and_endpoint_exact() {
    for k in 0..=3 {
        assert_eq!(curve(k, 0.0), 0.0, "curve {k} at 0");
        assert_eq!(curve(k, 1.0), 1.0, "curve {k} at 1");
    }
    assert_eq!(curve(1, 0.5), 0.25);
    assert_eq!(curve(2, 0.5), 0.5);
}
