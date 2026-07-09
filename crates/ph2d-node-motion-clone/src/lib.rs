#![forbid(unsafe_code)]
//! `motion.clone` — a Motion **cloner**: multiplies its input instance stream
//! into `count` copies laid along a **polar step axis** — each copy offset on
//! `P` by `copy_base · (distance·cos θ, distance·sin θ)`, where θ (`angle`, in
//! turns) frees the row from the X/Y grid so the artist dials any direction with
//! one control, and `copy_base` is the copy's signed rank. With `center` off the
//! ranks are `0,1,2,…` (the row grows away from the original); with `center` on
//! they are balanced about zero (`… −1, 0, 1 …`) so the queue straddles the
//! original instead of trailing from it. Other columns replicate unchanged —
//! **except** `Index`/`Count`, which are renumbered **continuous across copies**
//! so downstream index-driven effects (colour ramps, staggers) flow seamlessly
//! over the whole multiplied set rather than restarting per copy.
//!
//! This is a **stream multiplier** (1 node → N×in instances) — NOT entity
//! spawning; it has no ECS analogue (ADR-0035). Output count = `in_count *
//! count`. Pure; the polar direction is transcendental-free (HR-5, see [`trig`]).
//!
//! Params (read via `ctx.param` — per-instance override else the manifest
//! default shown): `count` (3), `distance` (2.0), `angle` (0 turns → +X, so the
//! default reproduces the old `step = (2, 0)` row), `center` (0 = off).
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

mod trig;
use trig::cos_sin_cycles;

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
            name: "distance",
            default: 2.0,
        },
        ParamSpec {
            name: "angle",
            default: 0.0,
        },
        ParamSpec {
            name: "center",
            default: 0.0,
        },
    ],
    // CPU-only by design (see handoff §9): a cloner *changes the element count*
    // (1 → N×in), which is structural, not a per-element `ph2d-expr` map an
    // `eval_column` could lower; and no Instances-domain WGSL runtime exists.
    lowerings: &[LoweringKind::Cpu],
};

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

/// The signed rank of copy `copy` (of `k`): `0,1,2,…` when `center` is off (the
/// row trails from the original), or balanced about zero when on
/// (`…, −1, 0, 1, …` for odd `k`; `…, −0.5, 0.5, …` for even) so the queue
/// straddles the original. Multiplying this by the step vector gives the copy's
/// `P` offset (and the offset of copy 0 with `center` off is exactly zero — the
/// cloner is a passthrough of the input at rank 0).
fn copy_rank(copy: usize, k: usize, center: bool) -> f32 {
    let r = copy as f32;
    if center {
        r - (k as f32 - 1.0) * 0.5
    } else {
        r
    }
}

/// Multiply `input` into `k` copies, offsetting each copy's `P` by
/// `copy_rank · (sx, sy)` (see [`copy_rank`] for `center`) and replicating every
/// other column unchanged — **except** `Index`/`Count`, renumbered continuous
/// across copies (copy `c` gets `Index += c·in_count`, `Count = in_count·k`) so
/// index-driven downstream effects span the whole multiplied set. `k` must
/// already be budget-clamped ([`copies_within_budget`]) so `in_count * k` fits
/// the allocation. Pure and isolated so the per-copy offset, the global
/// renumbering, *and* the column-replication alignment are unit-tested directly,
/// alongside the end-to-end cook test that drives the params via overrides.
fn clone_stream(input: &Stream, k: usize, sx: f32, sy: f32, center: bool) -> Stream {
    // The port type guarantees `P` is `Vec2`; any other dim is an upstream
    // node-author bug — assert loudly rather than replicate `P` without the
    // per-copy offset (which would stack every copy on top of the original).
    debug_assert!(
        !matches!(input.get("P"), Some(c) if !matches!(c, Column::Vec2(_))),
        "motion.clone expects `P` to be a Vec2 column (port type guarantees it)"
    );
    let in_count = input.count();
    let total = in_count * k;
    let mut out = Stream::new(total);
    for (name, col) in input.columns() {
        match (name.as_str(), col) {
            ("P", Column::Vec2(v)) => {
                let mut nv = Vec::with_capacity(total);
                for copy in 0..k {
                    let rank = copy_rank(copy, k, center);
                    let (dx, dy) = (rank * sx, rank * sy);
                    for p in v {
                        nv.push([p[0] + dx, p[1] + dy]);
                    }
                }
                out.set("P", Column::Vec2(nv));
            }
            // Continuous global index so a colour ramp / stagger reads one
            // uninterrupted `0..total` sequence over the multiplied set.
            ("Index", Column::Scalar(v)) => {
                let mut nv = Vec::with_capacity(total);
                for copy in 0..k {
                    let off = (copy * in_count) as f32;
                    nv.extend(v.iter().map(|&i| i + off));
                }
                out.set("Index", Column::Scalar(nv));
            }
            ("Count", Column::Scalar(_)) => {
                out.set("Count", Column::Scalar(vec![total as f32; total]));
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
        // Polar step axis: distance along a free direction θ (turns). θ = 0 → +X,
        // so `distance = 2` reproduces the old `(step_x, step_y) = (2, 0)` row.
        let distance = ctx.param("distance");
        let (c, s) = cos_sin_cycles(ctx.param("angle"));
        let (sx, sy) = (distance * c, distance * s);
        let center = ctx.param("center") >= 0.5; // Toggle: ≥0.5 → centred queue
        // `count` from an `f32` param: total conversion (non-finite/negative →
        // 0) then clamped so `in_count * k` cannot overflow the allocation; at
        // least one copy (passthrough).
        let requested = param_as_count(ctx.param("count"), RECOMMENDED_MAX_ELEMENTS);
        let input = ctx.input(0);
        let k = copies_within_budget(requested, input.count(), RECOMMENDED_MAX_ELEMENTS);
        let out = clone_stream(input, k, sx, sy, center);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionClone))?;
    // M1.R1 — UI metadata for the card (a stream multiplier → muted-green
    // distribute, rounded-rect silhouette).
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Clone",
            category: ph2d_node_registry::NodeUiCategory::Distribute,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    // M1.P1 — param rows: whole-number copy count + signed per-copy step.
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{AngleUnit, ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1) for the clone rows: whole-number copy count, a polar
/// step (distance + free angle), and a centre toggle. The angle is stored in
/// turns (the cycle-based trig in [`trig`]) and shown in degrees.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "count",
        label: "Count",
        min: 1.0,
        max: 32.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "distance",
        label: "Distance",
        min: 0.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "angle",
        label: "Angle",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Angle {
            unit: AngleUnit::Turns,
        },
    },
    ParamUiHint {
        param: "center",
        label: "Center",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
];

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

    /// Cook `motion.clone` on the 1-element source, applying `setup` to its
    /// params, and return the output `P` column.
    fn clone_p(setup: impl FnOnce(&mut Graph, ph2d_nodegraph::graph::NodeId)) -> Vec<[f32; 2]> {
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
        // angle ¼ turn → step direction +Y: count 3, distance 2 → y = 0, 2, 4.
        let p = clone_p(|g, clone| {
            g.set_param(clone, "angle", 0.25);
            g.set_param(clone, "distance", 2.0);
        });
        for (i, expected) in [0.0f32, 2.0, 4.0].into_iter().enumerate() {
            assert!(p[i][0].abs() < 1e-5, "x stays ~0 (pure +Y step)");
            assert!((p[i][1] - expected).abs() < 1e-5, "y = {expected}");
        }
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
        let out = clone_stream(&input, 3, 5.0, 0.0, false);
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
        let out = clone_stream(&input, 2, 10.0, 0.0, false);
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
