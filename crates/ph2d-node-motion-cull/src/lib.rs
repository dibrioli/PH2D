#![forbid(unsafe_code)]
//! `motion.cull` — **prune the instance stream by a predicate**: the Houdini "Blast" /
//! "Delete SOP" (Motion Nodes M3, structure — doc 01 §3 / doc 27). The first node that
//! *shrinks* the count (mirror/kaleidoscope grow it; sort reorders it). Downstream of a
//! `motion.sort`, an animated cull is a reveal: sort **radial** + cull a growing
//! **fraction** wipes a layout in from the centre; sort **random** + cull dissolves it.
//!
//! **Algorithm — keep the elements passing the predicate, filter every column.** Two
//! modes: **Fraction** keeps the first `amount·n` elements (so it reveals in the
//! upstream order — pair with `sort`); **Falloff** keeps the elements whose `falloff`
//! column is ≥ `amount` (a spatial mask — pair with `motion.falloff`, absent column
//! reads as 1). `invert` keeps the complement. The surviving indices gather every column
//! (`P`, `size`, `tint`, …), so the kept instances stay intact at a smaller count. An
//! `amount` **value** input (unconnected → the param) animates the reveal, so a
//! `value.lfo` sweeps it. Transcendental-free (HR-5): counting and comparison only.
//! `Effect::Pure`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, KEEP_FLAG_COL, StreamOp};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `amount` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Prune modes (the `mode` param).
const MODE_FRACTION: i64 = 0;
// MODE_FALLOFF (1) = keep where the `falloff` column ≥ amount.

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.cull"),
    name: "motion.cull",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // The keep fraction / threshold (animatable): unconnected reads the `amount`
        // param. A `value.lfo` sweeps the reveal.
        PortSpec {
            name: "amount",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // 0 Fraction (keep first amount·n) · 1 Falloff (keep falloff ≥ amount).
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
        // The fraction (Fraction mode) or threshold (Falloff mode).
        ParamSpec {
            name: "amount",
            default: 1.0,
        },
        // 0 keep the passing set · 1 keep the complement.
        ParamSpec {
            name: "invert",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// The `amount`: the value input's first element if connected, else the param.
fn amount_of(value: &[f32], param: f32) -> f32 {
    value.first().copied().unwrap_or(param)
}

/// The indices that survive the cull, in their original order.
fn keep_indices(n: usize, mode: i64, amount: f32, invert: bool, falloff: &[f32]) -> Vec<usize> {
    let pass: Box<dyn Fn(usize) -> bool> = match mode {
        MODE_FRACTION => {
            // Round the fraction to a count of leading elements.
            let keep =
                ((amount.clamp(0.0, 1.0) * n as f32).round() as i64).clamp(0, n as i64) as usize;
            Box::new(move |i: usize| i < keep)
        }
        _ => {
            // Falloff: keep where the mask (absent → 1) is at or above the threshold.
            let f = falloff.to_vec();
            Box::new(move |i: usize| f.get(i).copied().unwrap_or(1.0) >= amount)
        }
    };
    (0..n).filter(|&i| pass(i) != invert).collect()
}

/// Gather a column at the kept indices (`out[k] = col[keep[k]]`).
fn gather_col(col: &Column, keep: &[usize]) -> Column {
    fn take<T: Clone>(v: &[T], keep: &[usize]) -> Vec<T> {
        keep.iter().map(|&i| v[i].clone()).collect()
    }
    match col {
        Column::Scalar(v) => Column::Scalar(take(v, keep)),
        Column::Vec2(v) => Column::Vec2(take(v, keep)),
        Column::Vec3(v) => Column::Vec3(take(v, keep)),
        Column::Vec4(v) => Column::Vec4(take(v, keep)),
    }
}

/// The GPU predicate (ADR-0136): one flag per element, exactly
/// [`keep_indices`]'s `pass(i) != invert`, in the same expressions.
///
/// - **Fraction** keeps the first `round(amount·n)` — `floor(x + 0.5)` because
///   WGSL's `round` is half-to-even while Rust's is half-away (the value is
///   non-negative here, where the two agree on `floor(x+0.5)`). The count and
///   the compare both stay under 2²⁴, where `f32` holds integers exactly.
/// - **Falloff** keeps `falloff ≥ amount`; an absent column reads its identity
///   `1.0` (the CPU's `unwrap_or(1.0)`).
/// - `amount` comes from the value input's ROW 0 when connected — the CPU's
///   `value.first()`, which reads element 0 of a field of ANY length — else the
///   param (`HAS_v`).
/// - The mode/invert tests are the CPU casts made explicit: `round(x) as i64 == 0`
///   ⇔ `abs(x) < 0.5`, `round(x) as i64 != 0` ⇔ `abs(x) ≥ 0.5`, for finite x.
///
/// The node's own kernel is [`GpuKernel::PASSTHROUGH`] — the compaction IS the
/// node.
const GPU_PREDICATE: GpuKernel = GpuKernel {
    wgsl: "\
        let cl_amount = select(params.amount, read_amount_v(0u), HAS_amount_v);\n\
        var cl_pass: bool;\n\
        if (abs(params.mode) < 0.5) {\n\
        \x20   let cl_frac = clamp(cl_amount, 0.0, 1.0);\n\
        \x20   let cl_keep = clamp(floor(cl_frac * f32(params.count) + 0.5), 0.0, f32(params.count));\n\
        \x20   cl_pass = f32(i) < cl_keep;\n\
        } else {\n\
        \x20   cl_pass = read_in_falloff(i) >= cl_amount;\n\
        }\n\
        let cl_invert = abs(params.invert) >= 0.5;\n\
        write_cp_keep(i, select(0.0, 1.0, cl_pass != cl_invert));\n",
    wgsl_lib: "",
    bindings: &[
        ColumnBinding {
            // The spatial mask; absent reads 1 (everything passes the threshold
            // at full falloff) — the CPU's `unwrap_or(1.0)`.
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
        ColumnBinding {
            // The animatable amount (port 1). Broadcast: connected at ANY
            // pairable length, only row 0 is read (`amount_of`'s `first()`).
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 1,
        },
        ColumnBinding {
            column: KEEP_FLAG_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &["mode", "amount", "invert"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct MotionCull;

impl NodeOp for MotionCull {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mode = ctx.param("mode").round() as i64;
        let invert = ctx.param("invert").round() as i64 != 0;
        let amount = amount_of(&scalar_col(ctx.input(1), VALUE_COL), ctx.param("amount"));
        let input = ctx.input(0);
        let n = input.count();
        let falloff = scalar_col(input, "falloff");
        let keep = keep_indices(n, mode, amount, invert, &falloff);
        let mut out = Stream::new(keep.len());
        for (name, col) in input.columns() {
            out.set(name.clone(), gather_col(col, &keep));
        }
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionCull))?;
    // ADR-0136: the passthrough kernel makes the plan claim the node; the
    // compaction (the actual filter) is the stream op's machinery. NOT a
    // `register_dense_window` — a cull breaks the dense id window by definition.
    reg.register_gpu_kernel(MANIFEST.id, GpuKernel::PASSTHROUGH);
    reg.register_stream_op(
        MANIFEST.id,
        StreamOp::Compact {
            port: 0,
            predicate: GPU_PREDICATE,
        },
    );
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Cull",
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Fraction", "Falloff"],
        },
    },
    ParamUiHint {
        param: "amount",
        label: "Amount",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "invert",
        label: "Invert",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Keep", "Complement"],
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Fraction mode keeps the FIRST `amount·n` elements. FALSIFIED if it kept a
    /// different count or the wrong (non-leading) ones.
    #[test]
    fn fraction_keeps_the_leading_elements() {
        let keep = keep_indices(10, MODE_FRACTION, 0.3, false, &[]);
        assert_eq!(keep, vec![0, 1, 2], "0.3 * 10 -> the first 3");
    }

    /// Fraction 0 empties the stream; 1 keeps all.
    #[test]
    fn fraction_endpoints() {
        assert!(
            keep_indices(8, MODE_FRACTION, 0.0, false, &[]).is_empty(),
            "0 -> none"
        );
        assert_eq!(
            keep_indices(8, MODE_FRACTION, 1.0, false, &[]).len(),
            8,
            "1 -> all"
        );
    }

    /// Invert keeps the complement (the trailing elements under Fraction).
    #[test]
    fn invert_keeps_the_complement() {
        let keep = keep_indices(10, MODE_FRACTION, 0.3, true, &[]);
        assert_eq!(keep, vec![3, 4, 5, 6, 7, 8, 9], "the other 7");
    }

    /// Falloff mode keeps the elements whose mask is ≥ the threshold.
    #[test]
    fn falloff_threshold_keeps_above() {
        let falloff = vec![0.1, 0.9, 0.5, 1.0, 0.2];
        let keep = keep_indices(5, 1, 0.5, false, &falloff);
        assert_eq!(keep, vec![1, 2, 3], "falloff ≥ 0.5");
    }

    /// Deterministic + cooks through the registry: the `amount` value input drives the
    /// keep count and every column is filtered to the survivors.
    #[test]
    fn registers_and_culls_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.cull.test.src"),
            name: "motion.cull.test.src",
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
                &SRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                let p: Vec<[f32; 2]> = (0..10).map(|i| [i as f32, 0.0]).collect();
                let s: Vec<f32> = (0..10).map(|i| i as f32).collect();
                ctx.emit(
                    Stream::new(10)
                        .with("P", Column::Vec2(p))
                        .with("size", Column::Scalar(s)),
                );
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionCull),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let src = g.add_node("motion.cull.test.src");
        let c = g.add_node("motion.cull");
        g.set_param(c, "amount", 0.4); // keep the first 4
        g.connect(Edge {
            from: (src, 0),
            to: (c, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, c, 0.0).unwrap();
        let st = out[0].as_stream();
        assert_eq!(st.count(), 4, "0.4 × 10 kept");
        match (st.get("P").unwrap(), st.get("size").unwrap()) {
            (Column::Vec2(pv), Column::Scalar(sv)) => {
                assert_eq!(pv.len(), 4, "P filtered");
                assert_eq!(
                    sv,
                    &vec![0.0, 1.0, 2.0, 3.0],
                    "size filtered to the survivors"
                );
            }
            _ => panic!("columns"),
        }
    }
}
