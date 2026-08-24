//! Os gates de FONTE do `motion.clone` — a multiplicação, o posto assinado e a renumeração.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o corte é o
//! mesmo dos irmãos: o `lib.rs` responde *como o cloner funciona* e os `*_tests.rs` provam-no.
//! O módulo continua a chamar-se `tests`, então o `crate::tests::{Ops, clone_p}` que os
//! `radial_tests` importam não se mexe.

use super::*;
use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.clone.test.src"),
    name: "motion.clone.test.src",
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
struct Src;
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[0.0, 0.0]])));
    }
}
pub(crate) struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&Src),
            t if t == MANIFEST.id => Some(&MotionClone),
            _ => None,
        }
    }
}

/// Cook `motion.clone` on the 1-element source, applying `setup` to its
/// params, and return the output `P` column.
pub(crate) fn clone_p(
    setup: impl FnOnce(&mut Graph, ph2d_nodegraph::graph::NodeId),
) -> Vec<[f32; 2]> {
    let mut g = Graph::new();
    let src = g.add_node("motion.clone.test.src");
    let clone = g.add_node("motion.clone");
    g.connect(Edge {
        from: (src, 0),
        to: (clone, 0),
        delayed: false,
    })
    .unwrap();
    setup(&mut g, clone);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, clone, 0.0).unwrap();
    match out[0].as_stream().get("P").unwrap() {
        Column::Vec2(v) => v.clone(),
        _ => panic!("P"),
    }
}

#[test]
fn multiplies_stream_with_per_copy_offset() {
    // 1 instance × default count 3, distance 2 (angle 0 → +X) → x=0,2,4.
    let p = clone_p(|_, _| {});
    assert_eq!(p, vec![[0.0, 0.0], [2.0, 0.0], [4.0, 0.0]]);
}

#[test]
fn per_instance_overrides_drive_clone_through_the_cook() {
    // Authoring path: override count → 2, distance → 5, on a 1-element source
    // → 2 copies at x = 0, 5 (vs the default count 3, distance 2).
    let p = clone_p(|g, clone| {
        g.set_param(clone, "count", 2.0);
        g.set_param(clone, "distance", 5.0);
    });
    assert_eq!(p, vec![[0.0, 0.0], [5.0, 0.0]]);
}

#[test]
fn centered_queue_balances_copies_on_the_original() {
    // count 3, distance 2, centre on → ranks −1,0,1 → x = −2, 0, 2 (the
    // original element sits at rank 0, unmoved, with a copy each side).
    let p = clone_p(|g, clone| {
        g.set_param(clone, "count", 3.0);
        g.set_param(clone, "distance", 2.0);
        g.set_param(clone, "center", 1.0);
    });
    assert_eq!(p, vec![[-2.0, 0.0], [0.0, 0.0], [2.0, 0.0]]);
}

#[test]
fn polar_angle_rotates_the_step_axis() {
    // angle 90° → step direction +Y: count 3, distance 2 → y = 0, 2, 4.
    let p = clone_p(|g, clone| {
        g.set_param(clone, "angle", 90.0);
        g.set_param(clone, "distance", 2.0);
    });
    for (i, expected) in [0.0f32, 2.0, 4.0].into_iter().enumerate() {
        assert!(p[i][0].abs() < 1e-5, "x stays ~0 (pure +Y step)");
        assert!((p[i][1] - expected).abs() < 1e-5, "y = {expected}");
    }
}

#[test]
fn a_360_degree_angle_is_the_plus_x_axis_again() {
    // The degrees→cycles edge is exact: 360° wraps to a whole cycle, so the
    // step axis returns to +X (guards the `deg / 360` divisor).
    let p = clone_p(|g, clone| {
        g.set_param(clone, "angle", 360.0);
        g.set_param(clone, "distance", 2.0);
    });
    assert!(
        (p[1][0] - 2.0).abs() < 1e-5 && p[1][1].abs() < 1e-5,
        "back to +X"
    );
}

#[test]
fn copy_rank_is_balanced_when_centered() {
    // off: 0,1,2 ; on (k=3): −1,0,1 ; on (k=4): −1.5,−0.5,0.5,1.5.
    assert_eq!([0, 1, 2].map(|c| copy_rank(c, 3, false)), [0.0, 1.0, 2.0]);
    assert_eq!([0, 1, 2].map(|c| copy_rank(c, 3, true)), [-1.0, 0.0, 1.0]);
    assert_eq!(
        [0, 1, 2, 3].map(|c| copy_rank(c, 4, true)),
        [-1.5, -0.5, 0.5, 1.5]
    );
}

#[test]
fn index_and_count_are_renumbered_continuous_across_copies() {
    // 2 input elements (Index 0,1 / Count 2) × 3 copies → one uninterrupted
    // Index 0..5 and a Count of 6 everywhere, so a downstream ramp spans the
    // whole set instead of restarting per copy.
    let input = Stream::new(2)
        .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]]))
        .with("Index", Column::Scalar(vec![0.0, 1.0]))
        .with("Count", Column::Scalar(vec![2.0, 2.0]));
    let out = clone_row(&input, 3, 5.0, 0.0, false, 1.0, 0.0);
    match out.get("Index").unwrap() {
        Column::Scalar(v) => assert_eq!(v, &vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0]),
        _ => panic!("Index"),
    }
    match out.get("Count").unwrap() {
        Column::Scalar(v) => assert_eq!(v, &vec![6.0; 6]),
        _ => panic!("Count"),
    }
}

#[test]
fn clone_stream_aligns_p_offset_with_replicated_columns() {
    // The riskiest invariant: a *second* column (here `tint`) must stay
    // aligned with `P` element-for-element across copies (copy-major order).
    // 2 input elements × 2 copies, step (10, 0), centre off.
    let input = Stream::new(2)
        .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]]))
        .with(
            "tint",
            Column::Vec4(vec![[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0]]),
        );
    let out = clone_row(&input, 2, 10.0, 0.0, false, 1.0, 0.0);
    assert_eq!(out.count(), 4);
    // copy 0: elements at x=0,1 ; copy 1: same elements + (10,0).
    match out.get("P").unwrap() {
        Column::Vec2(v) => {
            assert_eq!(v, &vec![[0.0, 0.0], [1.0, 0.0], [10.0, 0.0], [11.0, 0.0]]);
        }
        _ => panic!("P"),
    }
    // tint of element e in copy c sits at index c*in_count + e, with the
    // SAME color it had in the input — proving offset and replicate share
    // copy-major order (a mismatch here is the silent-misalignment bug).
    match out.get("tint").unwrap() {
        Column::Vec4(v) => assert_eq!(
            v,
            &vec![
                [1.0, 0.0, 0.0, 1.0],
                [0.0, 1.0, 0.0, 1.0],
                [1.0, 0.0, 0.0, 1.0],
                [0.0, 1.0, 0.0, 1.0],
            ]
        ),
        _ => panic!("tint"),
    }
}

#[test]
fn copies_within_budget_caps_and_floors() {
    // floors at 1 copy even if 0 requested (cloner is ≥ passthrough).
    assert_eq!(copies_within_budget(0, 10, 1000), 1);
    // honors the request when it fits.
    assert_eq!(copies_within_budget(3, 10, 1000), 3);
    // clamps so in_count * k ≤ max: 10 elements, max 25 → at most 2 copies.
    assert_eq!(copies_within_budget(99, 10, 25), 2);
    // empty input: no division by zero, output will be empty anyway.
    assert_eq!(copies_within_budget(5, 0, 1000), 5);
    // input ALREADY over budget (in_count > max): still ≥ 1 copy
    // (passthrough), never 0 — the cloner does not drop a stream it cannot
    // grow. `in_count * 1` does not overflow (no multiplication grows it).
    assert_eq!(copies_within_budget(5, 1001, 1000), 1);
}
