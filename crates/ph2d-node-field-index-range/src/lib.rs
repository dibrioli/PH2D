#![forbid(unsafe_code)]
//! `field.index_range` — a Motion **focus field keyed by INDEX, not position**
//! (Cavalry's Range Falloff). It writes the same multiplicative `falloff` column
//! `motion.falloff` does — the sister-of-`accel` mask contract (§1.2) that every
//! downstream modifier scales its effect by — but the mask is a function of an
//! instance's *ordinal* `i / (n-1)` within the count, never its `(x,y)`. That is
//! the one field a spatial shape cannot express: "the first third of the clones",
//! "instances 40 %..60 %", the stagger-by-rank a grid+falloff can't reach.
//!
//! It is a **soft band** `[start, end]` (fractions of the count, `0..1`) with a
//! ramp of width `soft` at each edge, shaped by the same 4-curve as `motion.falloff`
//! (Linear/Quad/Smooth/Smoother — HR-5, polynomials only). `start > end` auto-swaps
//! (drag either handle past the other). It **multiplies** into any existing `falloff`
//! so fields compose (the MOPs contract), and passes every other column through
//! unchanged (count preserved). Pure. **Transcendental-free** (HR-5): the mask is
//! bit-identical across platforms for the replay hash.
//!
//! Params (read via `ctx.param`): `start` (0.25), `end` (0.75), `soft` (0.10 —
//! a *living* default: a newly-dropped node masks a visible middle band, never the
//! inert full range, D12), `curve` (2 Smooth), `invert` (0/1 — flips to `1 − f`).
//! The neutral is `start=0, end=1, soft=0, invert=0` ⇒ mask `1` everywhere ⇒ the
//! `falloff` column is multiplied by the identity, byte-for-byte unchanged.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{par_build, Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126, `register_gpu_kernel`); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("field.index_range"),
    name: "field.index_range",
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
            name: "start",
            default: 0.25,
        },
        ParamSpec {
            name: "end",
            default: 0.75,
        },
        ParamSpec {
            name: "soft",
            default: 0.10,
        },
        ParamSpec {
            name: "curve",
            default: 2.0,
        },
        ParamSpec {
            name: "invert",
            default: 0.0,
        },
    ],
    // Same as `motion.falloff`: the frozen manifest declares only the CPU
    // lowering; the WGSL kernel below is the ADR-0126 side channel.
    lowerings: &[LoweringKind::Cpu],
};

/// An edge curve on a pre-clamped `s ∈ [0,1]` — the SAME transcendental-free set as
/// `motion.falloff` (HR-5). `0` Linear · `1` Quad · `2` Smooth (smoothstep) · `3`
/// Smoother (smootherstep). Every curve is monotone and endpoint-exact (`0→0`, `1→1`).
fn curve(kind: i32, s: f32) -> f32 {
    match kind {
        1 => s * s,                                     // Quad
        2 => s * s * (3.0 - 2.0 * s),                   // Smooth (smoothstep)
        3 => s * s * s * (s * (s * 6.0 - 15.0) + 10.0), // Smoother (smootherstep)
        _ => s,                                         // Linear
    }
}

/// The band mask at normalized ordinal `s ∈ [0,1]`. `[start, end]` auto-swap into
/// `[lo, hi]`; `soft` is the ramp width at each edge, clamped so the two ramps meet
/// at most at the middle (a wide `soft` degenerates the trapezoid to a triangle,
/// never past). Because every `curve` is monotone, `curve(min(rise, fall))` equals
/// `min(curve(rise), curve(fall))` — one eval, shaping both edges. `soft = 0` gives
/// a hard rectangle; `[0,1]` gives the constant `1` (the identity a field multiplies
/// by, so an untouched full range never darkens the scene).
fn band_mask(s: f32, start: f32, end: f32, soft: f32, curve_kind: i32) -> f32 {
    let lo = start.min(end);
    let hi = start.max(end);
    let w = soft.max(0.0).min((hi - lo) * 0.5);
    let rise = if w > 0.0 {
        ((s - lo) / w).clamp(0.0, 1.0)
    } else if s >= lo {
        1.0
    } else {
        0.0
    };
    let fall = if w > 0.0 {
        ((hi - s) / w).clamp(0.0, 1.0)
    } else if s <= hi {
        1.0
    } else {
        0.0
    };
    curve(curve_kind, rise.min(fall))
}

/// GPU compute kernel (ADR-0126): a straight WGSL port of [`band_mask`] × [`curve`]
/// multiplied into the existing `falloff` — same polynomials, same `min`/`max`/
/// `clamp` (HR-5), so parity holds within float ULPs. The `curve` enum is routed
/// through `ir_round` (round-half-AWAY, matching Rust's `f32::round`; WGSL's builtin
/// `round` is half-even and would pick a different branch at `x.5` —
/// [[feedback_cpu_gpu_rounding_conventions_diverge]]) and `invert` through the
/// CPU's own `>= 0.5` threshold. The ordinal `s` reads `params.count` (the dispatch
/// width, = port 0's element count = the CPU's `input.count()`) so the two `s`
/// arithmetics are bit-identical. `ReadWrite` on `falloff` mirrors the CPU: an absent
/// column starts from the `1.0` identity (fields multiply) and is always written.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let ir_denom = f32(max(params.count, 2u) - 1u);\n\
        let ir_s = select(0.0, f32(i) / ir_denom, params.count > 1u);\n\
        let ir_m = ir_band_mask(\n\
            ir_s,\n\
            params.start,\n\
            params.end,\n\
            params.soft,\n\
            i32(ir_round(params.curve)));\n\
        var ir_f = ir_m;\n\
        if (params.invert >= 0.5) { ir_f = 1.0 - ir_m; }\n\
        write_falloff(i, read_falloff(i) * ir_f);\n",
    wgsl_lib: "\
        fn ir_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn ir_curve(kind: i32, s: f32) -> f32 {\n\
            if (kind == 1) { return s * s; }\n\
            if (kind == 2) { return s * s * (3.0 - 2.0 * s); }\n\
            if (kind == 3) { return s * s * s * (s * (s * 6.0 - 15.0) + 10.0); }\n\
            return s;\n\
        }\n\
        fn ir_band_mask(s: f32, start: f32, end: f32, soft: f32, curve_kind: i32) -> f32 {\n\
            let lo = min(start, end);\n\
            let hi = max(start, end);\n\
            let w = min(max(soft, 0.0), (hi - lo) * 0.5);\n\
            var rise: f32;\n\
            if (w > 0.0) { rise = clamp((s - lo) / w, 0.0, 1.0); }\n\
            else { rise = select(0.0, 1.0, s >= lo); }\n\
            var fall: f32;\n\
            if (w > 0.0) { fall = clamp((hi - s) / w, 0.0, 1.0); }\n\
            else { fall = select(0.0, 1.0, s <= hi); }\n\
            return ir_curve(curve_kind, min(rise, fall));\n\
        }\n",
    bindings: &[ColumnBinding {
        column: "falloff",
        dim: Dim::Scalar,
        access: ColumnAccess::ReadWrite,
        identity: [1.0; 4],
        port: 0,
    }],
    params: &["start", "end", "soft", "curve", "invert"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct FieldIndexRange;

impl NodeOp for FieldIndexRange {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let start = ctx.param("start");
        let end = ctx.param("end");
        let soft = ctx.param("soft");
        let curve_kind = ctx.param("curve").round() as i32;
        let invert = ctx.param("invert") >= 0.5;
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            // Existing per-instance falloff (fields multiply); absent → 1.
            let prev: Option<&[f32]> = match input.get("falloff") {
                Some(Column::Scalar(v)) => Some(v.as_slice()),
                _ => None,
            };
            // `n.max(2)` keeps the denominator ≥ 1 (a single element reads s = 0,
            // guarded below); the GPU's `max(count, 2u) - 1u` is the same integer.
            let denom = (n.max(2) - 1) as f32;
            let fall = par_build(n, |i| {
                let s = if n > 1 { i as f32 / denom } else { 0.0 };
                let m = band_mask(s, start, end, soft, curve_kind);
                let f = if invert { 1.0 - m } else { m };
                let base = prev.and_then(|v| v.get(i).copied()).unwrap_or(1.0);
                base * f
            });
            let mut out = Stream::new(n);
            for (name, col) in input.columns() {
                if name != "falloff" {
                    out.set(name.clone(), col.clone());
                }
            }
            out.set("falloff", Column::Scalar(fall));
            out
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via `ph2d-node-sync`
/// codegen) from `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(FieldIndexRange))?;
    // A focus field → amber, diamond value silhouette (same class as Falloff).
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Index Range",
            category: ph2d_node_registry::NodeUiCategory::Focus,
            silhouette: ph2d_node_registry::NodeSilhouette::Diamond,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    // The WGSL lowering, registered on the side (ADR-0126).
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1): fractional Start/End/Softness sliders (0..1), a named
/// Curve selector, an Invert checkbox — never number sliders for the enum.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "start",
        label: "Start",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "end",
        label: "End",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "soft",
        label: "Softness",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "curve",
        label: "Curve",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Linear", "Quad", "Smooth", "Smoother"],
        },
    },
    ParamUiHint {
        param: "invert",
        label: "Invert",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

    // Source: 11 instances (ordinals s = 0.0, 0.1, …, 1.0). Positions are inert
    // here — this field never reads them — but the stream must carry a column so
    // the count is real.
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("field.index_range.test.src"),
        name: "field.index_range.test.src",
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
    struct Src(usize);
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            let n = self.0;
            let p = par_build(n, |i| [i as f32, 0.0]);
            ctx.emit(Stream::new(n).with("P", Column::Vec2(p)));
        }
    }
    // Holds the source so the resolver hands out a `&self`-lifetime op keyed by
    // the count of THIS `Ops` — no shared static (which would pin the first n).
    struct Ops {
        src: Src,
    }
    impl Ops {
        fn new(n: usize) -> Self {
            Ops { src: Src(n) }
        }
    }
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&self.src),
                t if t == MANIFEST.id => Some(&FieldIndexRange),
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

    /// A ramp value is inherently f32-inexact (`(0.3 − 0.25)/0.1` computes to
    /// `0.5000001`, not `0.5`), so the mask SHAPE is asserted within a tolerance.
    /// The neutral/passthrough tests below stay `assert_eq!` on purpose — an
    /// identity that is off must be off AT THE BIT (D12), which is a stronger claim.
    fn assert_close(actual: &[f32], expected: &[f32]) {
        assert_eq!(actual.len(), expected.len(), "length");
        for (i, (a, e)) in actual.iter().zip(expected).enumerate() {
            assert!((a - e).abs() < 1e-5, "at {i}: {a} vs {e}");
        }
    }

    fn chain() -> (Graph, NodeId) {
        let mut g = Graph::new();
        let src = g.add_node("field.index_range.test.src");
        let foc = g.add_node("field.index_range");
        g.connect(Edge {
            from: (src, 0),
            to: (foc, 0),
            delayed: false,
        })
        .unwrap();
        (g, foc)
    }

    #[test]
    fn default_middle_band_is_a_clean_trapezoid() {
        let (mut g, foc) = chain();
        // Linear curve so the ramp math is transparent; defaults start .25/end .75/soft .1.
        g.set_param(foc, "curve", 0.0);
        // s: .0 .1 .2 .3 .4 .5 .6 .7 .8 .9 1.0 — band [.25,.75], ramp width .1:
        // rise 0 until .25, 1 by .35; fall 1 until .65, 0 by .75.
        assert_close(
            &falloff_of(&g, &Ops::new(11), foc),
            &[0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0, 0.5, 0.0, 0.0, 0.0],
        );
    }

    #[test]
    fn full_range_no_softness_is_the_identity() {
        // The neutral: start 0, end 1, soft 0 ⇒ mask 1 everywhere ⇒ the falloff
        // column is multiplied by the identity (D12 — off is exactly off).
        let (mut g, foc) = chain();
        g.set_param(foc, "start", 0.0);
        g.set_param(foc, "end", 1.0);
        g.set_param(foc, "soft", 0.0);
        assert_eq!(falloff_of(&g, &Ops::new(7), foc), vec![1.0; 7]);
    }

    #[test]
    fn invert_flips_the_mask() {
        let (mut g, foc) = chain();
        g.set_param(foc, "curve", 0.0);
        g.set_param(foc, "invert", 1.0);
        // 1 − trapezoid.
        assert_close(
            &falloff_of(&g, &Ops::new(11), foc),
            &[1.0, 1.0, 1.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0],
        );
    }

    #[test]
    fn start_after_end_auto_swaps() {
        // Dragging Start past End yields the SAME band as End..Start — the mask is
        // a function of the interval, not of which handle names which bound.
        let a = band_mask(0.5, 0.25, 0.75, 0.1, 0);
        let b = band_mask(0.5, 0.75, 0.25, 0.1, 0);
        assert_eq!(a, b);
        assert_eq!(a, 1.0);
    }

    #[test]
    fn a_prior_falloff_column_is_multiplied_not_overwritten() {
        // Fields COMPOSE multiplicatively (the MOPs contract): a carried `falloff`
        // is scaled by this band, never replaced.
        static FSRC_MAN: NodeManifest = NodeManifest {
            id: NodeTypeId::of("field.index_range.test.fsrc"),
            name: "field.index_range.test.fsrc",
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
                // 3 instances, prior falloff [0.5, 0.9, 0.4].
                ctx.emit(
                    Stream::new(3)
                        .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]))
                        .with("falloff", Column::Scalar(vec![0.5, 0.9, 0.4])),
                );
            }
        }
        struct FOps;
        impl OpResolver for FOps {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == FSRC_MAN.id => Some(&FSrc),
                    t if t == MANIFEST.id => Some(&FieldIndexRange),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let src = g.add_node("field.index_range.test.fsrc");
        let foc = g.add_node("field.index_range");
        g.connect(Edge {
            from: (src, 0),
            to: (foc, 0),
            delayed: false,
        })
        .unwrap();
        // 3 instances ⇒ s = 0.0, 0.5, 1.0. Full range so mask = 1 everywhere ⇒
        // the carried column survives unchanged.
        g.set_param(foc, "start", 0.0);
        g.set_param(foc, "end", 1.0);
        g.set_param(foc, "soft", 0.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &FOps, foc, 0.0).unwrap();
        match out[0].as_stream().get("falloff").unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![0.5, 0.9, 0.4]),
            _ => panic!("falloff"),
        }
    }

    #[test]
    fn curves_are_monotone_and_endpoint_exact() {
        for k in 0..=3 {
            assert_eq!(curve(k, 0.0), 0.0, "curve {k} at 0");
            assert_eq!(curve(k, 1.0), 1.0, "curve {k} at 1");
        }
        assert_eq!(curve(0, 0.5), 0.5); // Linear
        assert_eq!(curve(1, 0.5), 0.25); // Quad
        assert_eq!(curve(2, 0.5), 0.5); // Smoothstep symmetric
        assert!((curve(3, 0.5) - 0.5).abs() < 1e-6); // Smootherstep symmetric
    }

    #[test]
    fn degenerate_empty_band_masks_almost_everything() {
        // start == end ⇒ the interval has no width ⇒ the mask is 0 everywhere
        // except the single ordinal exactly on the point.
        assert_eq!(band_mask(0.3, 0.5, 0.5, 0.1, 2), 0.0);
        assert_eq!(band_mask(0.5, 0.5, 0.5, 0.1, 2), 1.0); // exactly on it
        assert_eq!(band_mask(0.7, 0.5, 0.5, 0.1, 2), 0.0);
    }

    #[test]
    fn single_element_reads_the_band_start_ordinal() {
        // n == 1 ⇒ s = 0 (no division). With the default band starting at .25 the
        // lone element sits below it ⇒ mask 0; a band containing 0 lights it.
        let (g, foc) = chain();
        assert_eq!(falloff_of(&g, &Ops::new(1), foc), vec![0.0]);
    }
}
