#![forbid(unsafe_code)]
//! `value.percentile` — the value-domain MORPHOLOGICAL / rank filter: replace each
//! element with the `p`-th order statistic of its index-window (Motion Nodes M2,
//! the value domain — doc 12/83). Where `value.median` picks the MIDDLE of the
//! window (`p = 0.5`), this picks any rank — and the ends of the range are the
//! genuinely new operations, not a median with a knob:
//! - **`p = 0` → the MINIMUM filter → grayscale EROSION** (each element becomes the
//!   smallest of its neighbourhood, so high regions SHRINK) — Photoshop's *Minimum*.
//! - **`p = 1` → the MAXIMUM filter → grayscale DILATION** (the largest, so high
//!   regions GROW) — Photoshop's *Maximum*.
//! - **`p = 0.5` → the median** (the de-spike; `value.median` is the dedicated,
//!   knob-free shortcut for this most-common case).
//!
//! Erosion and dilation are the morphological primitives (open/close are their
//! compositions), and a windowed min/max is NOT expressible from the existing
//! nodes (`value.reduce` Min/Max is the GLOBAL aggregate, not a per-element WINDOW).
//!
//! **Like `value.median`/`value.smooth`, element `i` reads its NEIGHBOURS**
//! `v[i−r] … v[i+r]` (edges **extend** — a clamped index repeats the boundary), and
//! returns the `rank`-th smallest of that `2r+1`-sample window, where
//! `rank = round(p · (w−1))` — `0` at `p=0` (min), `w−1` at `p=1` (max), `(w−1)/2`
//! at `p=0.5` (median). The order is meaningful when the instances are laid out in
//! sequence — the common case for a per-instance driver.
//!
//! **The output is always an EXISTING sample** (an order statistic, never an
//! average), so it is bit-exact CPU↔GPU: both select the same rank of the same
//! edge-clamped multiset. This agrees with `value.median` at `p=0.5` by the
//! MATHEMATICS (an order statistic is unique), not by shared code — no drift.
//!
//! **`radius`** is the half-window (`0` = a bit-exact passthrough — any `p` of a
//! one-sample window is that sample), **capped at `16`** (a window of `33`): the
//! GPU selection runs on a fixed register array, `O(w²)` per element — a rank
//! filter is a small-window tool. **The value type** is the continuous per-instance
//! scalar field `(Instances, Scalar, Frame)` on the `v` column (doc 12). `Pure`;
//! length preserved; **device-resident** (no CPU fallback); transcendental-free.

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
/// The largest half-window (window `2·16+1 = 33`, the GPU's fixed selection array).
/// Spelled here AND as the literal `16`/`33` in the WGSL; the parity gate straddles
/// the cap so the two spellings cannot drift.
const MAX_RADIUS: usize = 16;

/// The `rank`-th order statistic (`rank = round(p·(w−1))`) of `field` over an
/// index-window of half-width `radius` (capped at [`MAX_RADIUS`]). `radius = 0` is
/// a passthrough. Each window is gathered with edge-clamped indices — exactly the
/// WGSL's — and the selected value is an EXISTING sample, so it agrees with the
/// device's rank selection to the byte. The rank uses round-half-away (`.round()`),
/// matching the WGSL, so both devices target the same index; it is clamped so a
/// `p` at either end lands on a valid rank.
fn percentile(field: &[f32], radius: usize, p: f32) -> Vec<f32> {
    let n = field.len();
    let r = radius.min(MAX_RADIUS);
    if r == 0 || n == 0 {
        return field.to_vec();
    }
    let last = n as isize - 1;
    let w = 2 * r + 1;
    // rank in [0, w−1]: 0 = min, w−1 = max, (w−1)/2 = median.
    let rank = (p * (w - 1) as f32).round().clamp(0.0, (w - 1) as f32) as usize;
    (0..n)
        .map(|i| {
            let mut win: Vec<f32> = (0..w)
                .map(|k| {
                    let idx = (i as isize + k as isize - r as isize).clamp(0, last) as usize;
                    field[idx]
                })
                .collect();
            win.sort_by(|a, b| a.total_cmp(b));
            win[rank]
        })
        .collect()
}

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.percentile"),
    name: "value.percentile",
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
    params: &[
        ParamSpec {
            name: "radius",
            default: 0.0,
        },
        // 0 = min (erosion) · 0.5 = median · 1 = max (dilation).
        ParamSpec {
            name: "percentile",
            default: 0.5,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (ADR-0126) — the WGSL port of [`percentile`], **fully
/// device-resident**. No `applicable` gate. VALUE in, VALUE out; the binding is
/// `ReadWrite` (reads the window off `in_v`, writes a fresh `out_v` — the
/// `value.median` pattern). The window is gathered into a fixed register array
/// (`MAX_RADIUS` bounds it at 33), then the candidate whose counting rank (elements
/// strictly below, ties broken by window POSITION) equals `vpc_rank` is selected —
/// the SAME order statistic the CPU's sort takes, so the written value is
/// bit-identical (a selected existing sample). `vpc_round` matches Rust's
/// `f32::round` for BOTH the radius (a window width) and the rank (`p·(w−1)`).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vpc_r = clamp(i32(vpc_round(params.radius)), 0, 16);\n\
        if (vpc_r <= 0) {\n\
            write_v(i, read_v(i));\n\
        } else {\n\
            let vpc_last = i32(params.count) - 1;\n\
            let vpc_w = 2 * vpc_r + 1;\n\
            let vpc_rank = clamp(\n\
            \x20   i32(vpc_round(params.percentile * f32(vpc_w - 1))), 0, vpc_w - 1);\n\
            var vpc_win: array<f32, 33>;\n\
            var vpc_k = 0;\n\
            loop {\n\
                if (vpc_k >= vpc_w) { break; }\n\
                let vpc_idx = clamp(i32(i) + vpc_k - vpc_r, 0, vpc_last);\n\
                vpc_win[vpc_k] = read_v(u32(vpc_idx));\n\
                vpc_k = vpc_k + 1;\n\
            }\n\
            var vpc_out = vpc_win[0];\n\
            var vpc_a = 0;\n\
            loop {\n\
                if (vpc_a >= vpc_w) { break; }\n\
                var vpc_cnt = 0;\n\
                var vpc_b = 0;\n\
                loop {\n\
                    if (vpc_b >= vpc_w) { break; }\n\
                    let vpc_lt = vpc_win[vpc_b] < vpc_win[vpc_a];\n\
                    let vpc_tie = (vpc_win[vpc_b] == vpc_win[vpc_a]) && (vpc_b < vpc_a);\n\
                    if (vpc_lt || vpc_tie) { vpc_cnt = vpc_cnt + 1; }\n\
                    vpc_b = vpc_b + 1;\n\
                }\n\
                if (vpc_cnt == vpc_rank) { vpc_out = vpc_win[vpc_a]; }\n\
                vpc_a = vpc_a + 1;\n\
            }\n\
            write_v(i, vpc_out);\n\
        }\n",
    wgsl_lib: "\
        fn vpc_round(x: f32) -> f32 {\n\
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
    params: &["radius", "percentile"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct ValuePercentile;

impl NodeOp for ValuePercentile {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let radius = (ctx.param("radius").round().max(0.0) as usize).min(MAX_RADIUS);
        let p = ctx.param("percentile").clamp(0.0, 1.0);
        let input: Vec<f32> = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let n = input.len();
        let out = percentile(&input, radius, p);
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValuePercentile))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Percentile",
            // Utility grey: a value->value transformer, plumbing (not a transform).
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
        // The half-window. `0` is a passthrough; the max is the fixed selection window.
        param: "radius",
        label: "Radius",
        min: 0.0,
        max: 16.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        // 0 = min (erosion) · 0.5 = median · 1 = max (dilation).
        param: "percentile",
        label: "Percentile",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// **Radius 0 is a bit-exact passthrough** at any percentile — a one-sample
    /// window is that sample.
    #[test]
    fn radius_zero_is_the_identity() {
        let f = vec![3.0, -1.0, 7.5, 2.0];
        for &p in &[0.0, 0.5, 1.0] {
            assert_eq!(percentile(&f, 0, p), f, "radius 0 passthrough at p={p}");
        }
    }

    /// **`p = 0` is the MINIMUM filter (erosion)** — each element becomes the
    /// smallest of its window, so a high plateau SHRINKS at its edges and a lone
    /// LOW dip SPREADS. `[5,5,5,1,5,5]` at radius 1, p=0 → the `1` pulls its
    /// neighbours down to `1`.
    #[test]
    fn p_zero_is_the_minimum_filter_erosion() {
        assert_eq!(
            percentile(&[5.0, 5.0, 5.0, 1.0, 5.0, 5.0], 1, 0.0),
            vec![5.0, 5.0, 1.0, 1.0, 1.0, 5.0],
            "the low value erodes into its neighbours"
        );
    }

    /// **`p = 1` is the MAXIMUM filter (dilation)** — each element becomes the
    /// largest of its window, so a lone HIGH spike GROWS into its neighbours.
    /// `[0,0,0,9,0,0]` at radius 1, p=1 → the `9` dilates to a 3-wide plateau.
    #[test]
    fn p_one_is_the_maximum_filter_dilation() {
        assert_eq!(
            percentile(&[0.0, 0.0, 0.0, 9.0, 0.0, 0.0], 1, 1.0),
            vec![0.0, 0.0, 9.0, 9.0, 9.0, 0.0],
            "the high value dilates into its neighbours"
        );
    }

    /// **`p = 0.5` is the median** — it agrees with `value.median` by definition
    /// (both take the middle order statistic). A lone spike is deleted, not spread.
    #[test]
    fn p_half_is_the_median() {
        assert_eq!(
            percentile(&[0.0, 0.0, 9.0, 0.0, 0.0], 1, 0.5),
            vec![0.0, 0.0, 0.0, 0.0, 0.0],
            "the spike is deleted (median)"
        );
    }

    /// **Erosion and dilation are DUAL and ORDERED** — for any window, `min ≤
    /// median ≤ max`, so across a field `p=0 ≤ p=0.5 ≤ p=1` element-wise. A
    /// constant field is a fixed point of all three.
    #[test]
    fn the_ranks_are_ordered_and_a_constant_is_fixed() {
        let f = vec![2.0, 8.0, 1.0, 9.0, 4.0, 7.0, 3.0];
        let lo = percentile(&f, 2, 0.0);
        let mid = percentile(&f, 2, 0.5);
        let hi = percentile(&f, 2, 1.0);
        for k in 0..f.len() {
            assert!(lo[k] <= mid[k], "min <= median at {k}");
            assert!(mid[k] <= hi[k], "median <= max at {k}");
        }
        let c = vec![4.0; 6];
        for &p in &[0.0, 0.35, 0.5, 0.8, 1.0] {
            assert_eq!(percentile(&c, 2, p), c, "constant is fixed at p={p}");
        }
    }

    /// **The output is always an EXISTING sample, finite, length-preserving** — the
    /// rank filter never invents a value, at any radius (capped) or percentile.
    #[test]
    fn output_is_an_existing_sample_finite_and_length_preserving() {
        let f = vec![-3.0, 100.0, -50.0, 0.0, 8.0];
        for r in [0usize, 1, 2, 100] {
            for &p in &[0.0, 0.25, 0.5, 0.75, 1.0] {
                let out = percentile(&f, r, p);
                assert_eq!(out.len(), f.len(), "length preserved (r={r}, p={p})");
                for o in &out {
                    assert!(o.is_finite(), "finite (r={r}, p={p})");
                    assert!(f.contains(o), "an existing sample (r={r}, p={p})");
                }
            }
        }
        // The cap: radius 100 behaves as MAX_RADIUS.
        assert_eq!(percentile(&f, 100, 0.5), percentile(&f, MAX_RADIUS, 0.5), "capped");
    }

    /// A value source emitting a fixed field, so `value.percentile` can be driven
    /// through a real cook (the whole-chain proof, not just the math).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.percentile.test.src"),
        name: "value.percentile.test.src",
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

    /// End-to-end through the cook: dilation (`p = 1`, radius 1) grows a lone spike
    /// `[0, 0, 5, 0, 0]` into `[0, 5, 5, 5, 0]`, length preserved (the unary contract).
    #[test]
    fn dilates_a_field_through_the_cook() {
        struct Ops(Vec<f32>);
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC_MAN.id => {
                        Some(Box::leak(Box::new(Src(self.0.clone()))) as &dyn NodeOp)
                    }
                    t if t == MANIFEST.id => Some(&ValuePercentile),
                    _ => None,
                }
            }
        }
        let ops = Ops(vec![0.0, 0.0, 5.0, 0.0, 0.0]);
        let mut g = Graph::new();
        let src = g.add_node("value.percentile.test.src");
        let vp = g.add_node("value.percentile");
        g.set_param(vp, "radius", 1.0);
        g.set_param(vp, "percentile", 1.0); // max filter = dilation
        g.connect(Edge {
            from: (src, 0),
            to: (vp, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &ops, vp, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![0.0, 5.0, 5.0, 5.0, 0.0], "the spike dilated"),
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
