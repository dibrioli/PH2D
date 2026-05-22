#![forbid(unsafe_code)]
//! `motion.clone` — a Motion **cloner**: multiplies its input instance stream
//! into `count` copies, each offset on `P` by `copy_index * (step_x, step_y)`.
//! Other columns are replicated unchanged. This is a **stream multiplier**
//! (1 node → N×in instances) — NOT entity spawning; it has no ECS analogue
//! (ADR-0035). Output count = `in_count * count`. Pure.
//!
//! Params (manifest defaults): `count` (3), `step_x` (2.0), `step_y` (0.0).
//! `count` is read as an element count ([`param_as_count`]) and clamped so the
//! output `in_count * count` never overflows the allocation; the minimum is one
//! copy (a cloner is at least a passthrough).

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, RECOMMENDED_MAX_ELEMENTS,
    param_as_count,
};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.clone"),
    name: "motion.clone",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "count",
            default: 3.0,
        },
        ParamSpec {
            name: "step_x",
            default: 2.0,
        },
        ParamSpec {
            name: "step_y",
            default: 0.0,
        },
    ],
    // CPU-only by design (see handoff §9): a cloner *changes the element count*
    // (1 → N×in), which is structural, not a per-element `ph2d-expr` map an
    // `eval_column` could lower; and no Instances-domain WGSL runtime exists.
    lowerings: &[LoweringKind::Cpu],
};

/// A clone param read; `.expect` documents that the name is a literal of this
/// crate's own `MANIFEST` (a typo fails the golden test, never silent `0.0`).
fn param(name: &str) -> f32 {
    MANIFEST
        .param_default(name)
        .expect("clone reads only its own declared params")
}

/// Replicate a column `k` times (copy 0, copy 1, ... — element order within a
/// copy preserved), matching the `P` offset loop in [`clone_stream`].
fn replicate(col: &Column, k: usize) -> Column {
    fn rep<T: Clone>(v: &[T], k: usize) -> Vec<T> {
        let mut out = Vec::with_capacity(v.len() * k);
        for _ in 0..k {
            out.extend_from_slice(v);
        }
        out
    }
    match col {
        Column::Scalar(v) => Column::Scalar(rep(v, k)),
        Column::Vec2(v) => Column::Vec2(rep(v, k)),
        Column::Vec3(v) => Column::Vec3(rep(v, k)),
        Column::Vec4(v) => Column::Vec4(rep(v, k)),
    }
}

/// Clamp the requested copy count so the output `in_count * k` stays within
/// `max` (and so the multiplication can never overflow the allocation). Always
/// at least one copy (a cloner is a passthrough at minimum). With `in_count`
/// already bounded by the same budget upstream, one copy always fits.
fn copies_within_budget(requested: usize, in_count: usize, max: usize) -> usize {
    if in_count == 0 {
        return requested.max(1); // output is empty regardless; avoids /0
    }
    requested.min(max / in_count).max(1)
}

/// Multiply `input` into `k` copies, offsetting each copy's `P` by
/// `copy_index * (sx, sy)` and replicating every other column unchanged. `k`
/// must already be budget-clamped ([`copies_within_budget`]) so `in_count * k`
/// fits the allocation. Pure and isolated so the per-copy offset *and* the
/// column-replication alignment are unit-tested directly (the cook only feeds
/// manifest defaults until per-instance overrides land).
fn clone_stream(input: &Stream, k: usize, sx: f32, sy: f32) -> Stream {
    // The port type guarantees `P` is `Vec2`; any other dim is an upstream
    // node-author bug — assert loudly rather than replicate `P` without the
    // per-copy offset (which would stack every copy on top of the original).
    debug_assert!(
        !matches!(input.get("P"), Some(c) if !matches!(c, Column::Vec2(_))),
        "motion.clone expects `P` to be a Vec2 column (port type guarantees it)"
    );
    let in_count = input.count();
    let mut out = Stream::new(in_count * k);
    for (name, col) in input.columns() {
        match (name.as_str(), col) {
            ("P", Column::Vec2(v)) => {
                let mut nv = Vec::with_capacity(in_count * k);
                for copy in 0..k {
                    let (dx, dy) = (copy as f32 * sx, copy as f32 * sy);
                    for p in v {
                        nv.push([p[0] + dx, p[1] + dy]);
                    }
                }
                out.set("P", Column::Vec2(nv));
            }
            _ => out.set(name.clone(), replicate(col, k)),
        }
    }
    out
}

struct MotionClone;

impl NodeOp for MotionClone {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let (sx, sy) = (param("step_x"), param("step_y"));
        let input = ctx.input(0);
        // `count` from an `f32` param: total conversion (non-finite/negative →
        // 0) then clamped so `in_count * k` cannot overflow the allocation; at
        // least one copy (passthrough).
        let requested = param_as_count(param("count"), RECOMMENDED_MAX_ELEMENTS);
        let k = copies_within_budget(requested, input.count(), RECOMMENDED_MAX_ELEMENTS);
        let out = clone_stream(input, k, sx, sy);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionClone))
}

#[cfg(test)]
mod tests {
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
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionClone),
                _ => None,
            }
        }
    }

    #[test]
    fn multiplies_stream_with_per_copy_offset() {
        // 1 instance × default count 3, step_x 2 → 3 instances at x=0,2,4.
        let mut g = Graph::new();
        let src = g.add_node("motion.clone.test.src");
        let clone = g.add_node("motion.clone");
        g.connect(Edge {
            from: (src, 0),
            to: (clone, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, clone, 0.0).unwrap();
        assert_eq!(out[0].count(), 3);
        match out[0].get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[0.0, 0.0], [2.0, 0.0], [4.0, 0.0]]),
            _ => panic!("P"),
        }
    }

    #[test]
    fn clone_stream_aligns_p_offset_with_replicated_columns() {
        // The riskiest invariant: a *second* column (here `tint`) must stay
        // aligned with `P` element-for-element across copies (copy-major order).
        // 2 input elements × 2 copies, step_x=10.
        let input = Stream::new(2)
            .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]]))
            .with(
                "tint",
                Column::Vec4(vec![[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0]]),
            );
        let out = clone_stream(&input, 2, 10.0, 0.0);
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
}
