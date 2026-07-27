#![forbid(unsafe_code)]
//! `value.median` — the value-domain NON-LINEAR filter: replace each element with
//! the MEDIAN of its index-window, an order-statistic filter over the instance
//! order (Motion Nodes M2, the value domain — doc 12/82). It is the sibling of
//! `value.smooth` that `value.smooth` cannot be: where the box blur is LINEAR (it
//! averages, so a single outlier bleeds into its neighbours and every edge
//! softens), the median is NON-LINEAR (it picks the middle value, so a spike is
//! DROPPED and an edge is KEPT). It is the classic salt-and-pepper / impulse-noise
//! remover — the Median of image processing, the de-spike of a signal filter.
//!
//! **The one thing smooth cannot do, and why both exist:** feed a field with a lone
//! SPIKE (a bad sample, a `value.noise` glitch). `value.smooth` spreads the spike
//! into a bump and rounds every edge; `value.median` deletes the spike and leaves
//! the edges razor-sharp. Linear vs order-statistic — the reason a toolset ships
//! both a Blur and a Median.
//!
//! **Like `value.smooth`, element `i` reads its NEIGHBOURS** `v[i−r] … v[i+r]` (the
//! edges **extend** — a clamped index repeats the boundary), and returns the
//! `r`-th order statistic of that `2r+1`-sample window (the middle when sorted).
//! The order is meaningful when the instances are laid out in sequence — the common
//! case for a per-instance driver.
//!
//! **`radius`** is the half-window (`0` = a bit-exact passthrough, the neutral
//! default; `1` = the classic median-of-3). It is **capped at `16`** (a window of
//! `33`): the GPU selection runs on a fixed register array, and the cost is `O(w²)`
//! per element — a median is a SMALL-window spike-remover by nature, and a
//! wide-window rank filter is a different tool. The output is always an EXISTING
//! sample value (never an average), so it is bit-exact CPU↔GPU: both select the
//! same `r`-th smallest value of the same edge-clamped multiset.
//!
//! **The value type** is the continuous per-instance scalar field `(Instances,
//! Scalar, Frame)` on the `v` column (doc 12). `Pure` (no clock, no state); length
//! preserved. The GPU kernel reads the window off the input buffer and selects on
//! the device, so it is **device-resident** (no CPU fallback) with the existing
//! kernel channel — no reduction, no scan; transcendental-free (comparisons only).

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of the sibling value nodes; kept local so this stays a leaf drop-crate
/// — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value column, in and out (the canonical `value`-domain column).
const VALUE_COL: &str = "v";
/// The largest half-window. A window of `2·16+1 = 33` fits the GPU's fixed
/// selection array, and the `O(w²)` selection stays cheap — a median is a
/// small-window tool. Spelled here AND as the literal `16`/`33` in the WGSL (a
/// `&'static str` cannot interpolate a Rust constant); the parity gate straddles
/// the cap so the two spellings cannot drift.
const MAX_RADIUS: usize = 16;

/// The median of `field` over an index-window of half-width `radius` (capped at
/// [`MAX_RADIUS`]). `radius = 0` is a passthrough (bit-exact). Each window is
/// gathered with edge-clamped indices — exactly the WGSL's window — and the
/// `r`-th smallest value is returned. The output is always an EXISTING sample, so
/// it agrees with the GPU's rank selection to the byte (both pick the same order
/// statistic of the same multiset). `total_cmp` is a total order (no panic; for the
/// finite fields the value nodes produce it agrees with `<`, the GPU's comparison).
fn median(field: &[f32], radius: usize) -> Vec<f32> {
    let n = field.len();
    let r = radius.min(MAX_RADIUS);
    if r == 0 || n == 0 {
        return field.to_vec();
    }
    let last = n as isize - 1;
    let w = 2 * r + 1;
    (0..n)
        .map(|i| {
            let mut win: Vec<f32> = (0..w)
                .map(|k| {
                    let idx = (i as isize + k as isize - r as isize).clamp(0, last) as usize;
                    field[idx]
                })
                .collect();
            win.sort_by(|a, b| a.total_cmp(b));
            win[r] // the middle of the sorted window — the r-th order statistic
        })
        .collect()
}

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.median"),
    name: "value.median",
    inputs: &[PortSpec {
        name: "in",
        ty: VALUE,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[ParamSpec {
        name: "radius",
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (ADR-0126) — the WGSL port of [`median`], **fully
/// device-resident**. No `applicable` gate — the sequencer never falls back to the
/// CPU. VALUE in, VALUE out; the binding is `ReadWrite` because the kernel READS
/// the input (its window, off `in_v`) and WRITES a fresh `out_v` — separate
/// buffers, so a write never corrupts a window read (the `value.smooth` pattern).
///
/// **The selection is a counting rank, no sort.** The window is gathered into a
/// fixed register array (`MAX_RADIUS` bounds it at 33), then each candidate's rank
/// is the number of window elements strictly below it, ties broken by window
/// POSITION (`< vmd_win[a] || (== && b < a)`) so exactly one candidate has each
/// rank. The candidate whose rank equals `r` is the median — the SAME `r`-th order
/// statistic the CPU's sort selects, so the written value is bit-identical (a
/// selected existing sample, no arithmetic to diverge). `vmd_round` matches Rust's
/// `f32::round` (radius picks a WINDOW WIDTH, an integer — a half-even disagreement
/// would size a different window).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vmd_r = clamp(i32(vmd_round(params.radius)), 0, 16);\n\
        if (vmd_r <= 0) {\n\
            write_v(i, read_v(i));\n\
        } else {\n\
            let vmd_last = i32(params.count) - 1;\n\
            let vmd_w = 2 * vmd_r + 1;\n\
            var vmd_win: array<f32, 33>;\n\
            var vmd_k = 0;\n\
            loop {\n\
                if (vmd_k >= vmd_w) { break; }\n\
                let vmd_idx = clamp(i32(i) + vmd_k - vmd_r, 0, vmd_last);\n\
                vmd_win[vmd_k] = read_v(u32(vmd_idx));\n\
                vmd_k = vmd_k + 1;\n\
            }\n\
            var vmd_out = vmd_win[0];\n\
            var vmd_a = 0;\n\
            loop {\n\
                if (vmd_a >= vmd_w) { break; }\n\
                var vmd_rank = 0;\n\
                var vmd_b = 0;\n\
                loop {\n\
                    if (vmd_b >= vmd_w) { break; }\n\
                    let vmd_lt = vmd_win[vmd_b] < vmd_win[vmd_a];\n\
                    let vmd_tie = (vmd_win[vmd_b] == vmd_win[vmd_a]) && (vmd_b < vmd_a);\n\
                    if (vmd_lt || vmd_tie) { vmd_rank = vmd_rank + 1; }\n\
                    vmd_b = vmd_b + 1;\n\
                }\n\
                if (vmd_rank == vmd_r) { vmd_out = vmd_win[vmd_a]; }\n\
                vmd_a = vmd_a + 1;\n\
            }\n\
            write_v(i, vmd_out);\n\
        }\n",
    wgsl_lib: "\
        fn vmd_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n",
    bindings: &[ColumnBinding {
        column: VALUE_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::ReadWrite,
        identity: [0.0; 4],
        port: 0,
    }],
    params: &["radius"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct ValueMedian;

impl NodeOp for ValueMedian {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // Radius is a non-negative half-window; round, clamp at 0, cap at MAX.
        let radius = (ctx.param("radius").round().max(0.0) as usize).min(MAX_RADIUS);
        let input: Vec<f32> = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let n = input.len();
        let out = median(&input, radius);
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueMedian))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Median",
            // Utility grey: a value->value transformer, plumbing (not a transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    // The half-window. `0` is a passthrough; `1` is the classic median-of-3. The
    // max is the fixed selection window (a median is a small-window de-spiker).
    param: "radius",
    label: "Radius",
    min: 0.0,
    max: 16.0,
    step: 1.0,
    widget: ParamWidget::Slider,
}];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// **Radius 0 is a bit-exact passthrough** — the neutral default.
    #[test]
    fn radius_zero_is_the_identity() {
        let f = vec![3.0, -1.0, 7.5, 2.0];
        assert_eq!(median(&f, 0), f);
    }

    /// **A lone spike is DELETED, and this is the whole point** — a single tall
    /// value surrounded by a flat field vanishes (the median of the window is a
    /// neighbour, not the outlier). `[0,0,9,0,0]` at radius 1 → all zeros: the
    /// spike is gone, no bump left behind (a box blur would spread it to `[0,3,3,3,0]`).
    #[test]
    fn a_lone_spike_is_deleted_not_spread() {
        assert_eq!(
            median(&[0.0, 0.0, 9.0, 0.0, 0.0], 1),
            vec![0.0, 0.0, 0.0, 0.0, 0.0]
        );
        // Salt-and-pepper: isolated outliers on a constant field, all removed.
        assert_eq!(median(&[5.0, 99.0, 5.0, 5.0, -99.0, 5.0], 1), vec![5.0; 6]);
    }

    /// **An EDGE is KEPT razor-sharp** — the median does not soften a step the way
    /// the linear blur does. `[0,0,0,1,1,1]` at radius 1 stays `[0,0,0,1,1,1]`: the
    /// transition is untouched (a box blur would ramp it). This is the property that
    /// makes the median an edge-preserving de-noiser.
    #[test]
    fn an_edge_is_kept_sharp() {
        assert_eq!(
            median(&[0.0, 0.0, 0.0, 1.0, 1.0, 1.0], 1),
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            "the step is preserved exactly"
        );
    }

    /// **A constant field is unchanged at any radius** — the median of equal values
    /// is that value; the edge-extend keeps it constant.
    #[test]
    fn a_constant_field_is_unchanged() {
        let f = vec![4.0; 7];
        for r in [0usize, 1, 3, 20] {
            assert_eq!(median(&f, r), f, "constant survives radius {r}");
        }
    }

    /// **The output is always an EXISTING sample, finite, length-preserving** — the
    /// median never invents a value (unlike the average), including a radius larger
    /// than the field (edge-extend clamps) and past the cap (`MAX_RADIUS`).
    #[test]
    fn output_is_an_existing_sample_finite_and_length_preserving() {
        let f = vec![-3.0, 100.0, -50.0, 0.0, 8.0];
        for r in [0usize, 1, 2, 5, 100] {
            let out = median(&f, r);
            assert_eq!(out.len(), f.len(), "length preserved at radius {r}");
            for o in &out {
                assert!(o.is_finite(), "finite at radius {r}");
                assert!(
                    f.contains(o),
                    "the median is an existing sample at radius {r}"
                );
            }
        }
        // The cap: radius 100 behaves exactly as radius MAX_RADIUS (16).
        assert_eq!(
            median(&f, 100),
            median(&f, MAX_RADIUS),
            "radius is capped at MAX_RADIUS"
        );
    }

    /// A value source emitting a fixed field, so `value.median` can be driven
    /// through a real cook (the whole-chain proof, not just the math).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.median.test.src"),
        name: "value.median.test.src",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: VALUE,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Src(Vec<f32>);
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(self.0.len()).with(VALUE_COL, Column::Scalar(self.0.clone())));
        }
    }

    /// End-to-end through the cook: a field with a lone spike `[2, 2, 8, 2, 2]`
    /// through radius 1 becomes `[2, 2, 2, 2, 2]` (the spike is a minority in every
    /// window it touches), length preserved (the unary contract).
    #[test]
    fn de_spikes_a_field_through_the_cook() {
        struct Ops(Vec<f32>);
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC_MAN.id => {
                        Some(Box::leak(Box::new(Src(self.0.clone()))) as &dyn NodeOp)
                    }
                    t if t == MANIFEST.id => Some(&ValueMedian),
                    _ => None,
                }
            }
        }
        let ops = Ops(vec![2.0, 2.0, 8.0, 2.0, 2.0]);
        let mut g = Graph::new();
        let src = g.add_node("value.median.test.src");
        let vm = g.add_node("value.median");
        g.set_param(vm, "radius", 1.0);
        g.connect(Edge {
            from: (src, 0),
            to: (vm, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &ops, vm, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![2.0, 2.0, 2.0, 2.0, 2.0], "the spike is gone"),
            _ => panic!("v"),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
