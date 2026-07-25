#![forbid(unsafe_code)]
//! `field.combine` — the **composer** of the `field.*` family: takes TWO field
//! branches and blends their `falloff` masks with an explicit **blend mode**
//! (the C4D Fields layer-blend / MOPs Combine). Fields already compose by
//! *implicit multiply* (each writes `falloff` in sequence), but that is one mode
//! of many; this node makes composition a first-class op with the eight standard
//! modes — **Normal · Add · Subtract · Multiply · Screen · Min · Max · Overlay** —
//! plus a `strength` mix.
//!
//! Two inputs, `a` (port 0, the **base** — the output rides it, carrying its `P`,
//! `size`, colour, …) and `b` (port 1, the **overlay**). Both are branches off the
//! same source, so element `i` of `a` and of `b` are the same instance; the node
//! reads `a.falloff` and `b.falloff` and writes `falloff = lerp(a, blend(a, b,
//! mode), strength)` on the output. `Min` is the intersection (a field masked by
//! another), `Max` the union, `Multiply` the implicit compose made explicit,
//! `Screen`/`Overlay` the soft composites. Pure. **Transcendental-free** (HR-5):
//! only `min`/`max`, `+`, `−`, `*`, so the mask is bit-identical for the replay hash.
//!
//! Params: `mode` (3 Multiply), `strength` (1). `strength = 0` is a true no-op
//! (the base passes through unchanged); `Multiply` with `b`-branch absent (all-1
//! identity) is also the base unchanged — a fresh node does something predictable.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream, par_build};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031). Two inputs; the kernel is
/// side-metadata (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("field.combine"),
    name: "field.combine",
    inputs: &[
        PortSpec {
            name: "a",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "b",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "mode",
            default: 3.0,
        },
        ParamSpec {
            name: "strength",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Blend two masks `a`, `b ∈ [0,1]` by `mode`. The eight standard modes, all
/// transcendental-free (HR-5). Endpoints stay in `[0,1]` for inputs in `[0,1]`
/// (Add/Subtract clamp; the products of two `[0,1]` values stay in range).
/// `0` Normal (replace by `b`) · `1` Add · `2` Subtract · `3` Multiply · `4`
/// Screen · `5` Min · `6` Max · `7` Overlay.
fn blend(a: f32, b: f32, mode: i32) -> f32 {
    match mode {
        1 => (a + b).min(1.0),
        2 => (a - b).max(0.0),
        3 => a * b,
        4 => 1.0 - (1.0 - a) * (1.0 - b),
        5 => a.min(b),
        6 => a.max(b),
        7 => {
            if a < 0.5 {
                2.0 * a * b
            } else {
                1.0 - 2.0 * (1.0 - a) * (1.0 - b)
            }
        }
        _ => b, // 0 Normal (replace)
    }
}

/// GPU compute kernel (ADR-0126): a two-input map. The output rides port `a`
/// (`falloff` `ReadWrite` → `read_a_falloff` + `write_falloff`); port `b`'s
/// `falloff` is a `Read` (`read_b_falloff`). Absent on either port reads the `1.0`
/// identity (an unconnected `b` is the base unchanged under Multiply/Min/Max).
/// Same `min`/`max`/arithmetic as the CPU (HR-5), `mode` via `fc_round`
/// (half-away, matching Rust `f32::round`; WGSL `round` is half-even).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let fc_a = read_a_falloff(i);\n\
        let fc_b = read_b_falloff(i);\n\
        let fc_blended = fc_blend(fc_a, fc_b, i32(fc_round(params.mode)));\n\
        write_falloff(i, fc_a + (fc_blended - fc_a) * params.strength);\n",
    wgsl_lib: "\
        fn fc_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn fc_blend(a: f32, b: f32, mode: i32) -> f32 {\n\
            if (mode == 1) { return min(a + b, 1.0); }\n\
            if (mode == 2) { return max(a - b, 0.0); }\n\
            if (mode == 3) { return a * b; }\n\
            if (mode == 4) { return 1.0 - (1.0 - a) * (1.0 - b); }\n\
            if (mode == 5) { return min(a, b); }\n\
            if (mode == 6) { return max(a, b); }\n\
            if (mode == 7) {\n\
                if (a < 0.5) { return 2.0 * a * b; }\n\
                return 1.0 - 2.0 * (1.0 - a) * (1.0 - b);\n\
            }\n\
            return b;\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [1.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 1,
        },
    ],
    params: &["mode", "strength"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct FieldCombine;

impl NodeOp for FieldCombine {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mode = ctx.param("mode").round() as i32;
        let strength = ctx.param("strength");
        // Read port `b`'s falloff into an owned Vec BEFORE borrowing port `a`
        // (the same order bend uses to avoid aliasing the two input borrows).
        let b_fall: Vec<f32> = match ctx.input(1).get("falloff") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let out = {
            let a = ctx.input(0);
            let n = a.count();
            let a_fall: Option<&[f32]> = match a.get("falloff") {
                Some(Column::Scalar(v)) => Some(v.as_slice()),
                _ => None,
            };
            let fall = par_build(n, |i| {
                let av = a_fall.and_then(|v| v.get(i).copied()).unwrap_or(1.0);
                let bv = b_fall.get(i).copied().unwrap_or(1.0);
                let blended = blend(av, bv, mode);
                av + (blended - av) * strength
            });
            // The output rides port `a` (the base): carry its columns, replace falloff.
            let mut out = Stream::new(n);
            for (name, col) in a.columns() {
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
    reg.register(Box::new(FieldCombine))?;
    // A field op that MERGES two branches → amber (Focus), cigar (merge) silhouette.
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Combine Fields",
            category: ph2d_node_registry::NodeUiCategory::Focus,
            silhouette: ph2d_node_registry::NodeSilhouette::Cigar,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1): a named blend-mode selector + a strength slider.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 7.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "Normal", "Add", "Subtract", "Multiply", "Screen", "Min", "Max", "Overlay",
            ],
        },
    },
    ParamUiHint {
        param: "strength",
        label: "Strength",
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

    // Two sources, each emitting a `falloff` column, so `field.combine` has an
    // `a` and a `b` branch to blend. Same count/order (the composition contract).
    static SRC_A_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("field.combine.test.a"),
        name: "field.combine.test.a",
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
    static SRC_B_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("field.combine.test.b"),
        name: "field.combine.test.b",
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
    struct SrcA;
    impl NodeOp for SrcA {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_A_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(
                Stream::new(4)
                    .with("P", Column::Vec2(vec![[0.0, 0.0]; 4]))
                    .with("falloff", Column::Scalar(vec![0.2, 0.5, 0.8, 1.0])),
            );
        }
    }
    struct SrcB;
    impl NodeOp for SrcB {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_B_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(
                Stream::new(4)
                    .with("P", Column::Vec2(vec![[0.0, 0.0]; 4]))
                    .with("falloff", Column::Scalar(vec![0.5, 0.5, 0.4, 0.0])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_A_MAN.id => Some(&SrcA),
                t if t == SRC_B_MAN.id => Some(&SrcB),
                t if t == MANIFEST.id => Some(&FieldCombine),
                _ => None,
            }
        }
    }

    fn combined(mode: f32, strength: f32) -> Vec<f32> {
        let mut g = Graph::new();
        let a = g.add_node("field.combine.test.a");
        let b = g.add_node("field.combine.test.b");
        let c = g.add_node("field.combine");
        g.connect(Edge {
            from: (a, 0),
            to: (c, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (b, 0),
            to: (c, 1),
            delayed: false,
        })
        .unwrap();
        g.set_param(c, "mode", mode);
        g.set_param(c, "strength", strength);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, c, 0.0).unwrap();
        match out[0].as_stream().get("falloff").unwrap() {
            Column::Scalar(v) => v.clone(),
            _ => panic!("falloff"),
        }
    }

    fn close(actual: &[f32], expected: &[f32]) {
        assert_eq!(actual.len(), expected.len());
        for (i, (x, e)) in actual.iter().zip(expected).enumerate() {
            assert!((x - e).abs() < 1e-6, "at {i}: {x} vs {e}");
        }
    }

    #[test]
    fn multiply_is_the_intersection() {
        // a = [.2 .5 .8 1], b = [.5 .5 .4 0] → a·b.
        close(&combined(3.0, 1.0), &[0.1, 0.25, 0.32, 0.0]);
    }

    #[test]
    fn min_and_max_are_intersection_and_union() {
        close(&combined(5.0, 1.0), &[0.2, 0.5, 0.4, 0.0]); // Min
        close(&combined(6.0, 1.0), &[0.5, 0.5, 0.8, 1.0]); // Max
    }

    #[test]
    fn add_and_subtract_clamp() {
        close(&combined(1.0, 1.0), &[0.7, 1.0, 1.0, 1.0]); // Add: min(a+b,1)
        close(&combined(2.0, 1.0), &[0.0, 0.0, 0.4, 1.0]); // Subtract: max(a-b,0)
    }

    #[test]
    fn normal_replaces_with_b() {
        close(&combined(0.0, 1.0), &[0.5, 0.5, 0.4, 0.0]); // b
    }

    #[test]
    fn screen_lightens() {
        // 1-(1-a)(1-b): .2,.5 → 1-.8·.5=.6 ; .5,.5 → 1-.5·.5=.75 ; .8,.4 → 1-.2·.6=.88 ; 1,0 → 1.
        close(&combined(4.0, 1.0), &[0.6, 0.75, 0.88, 1.0]);
    }

    #[test]
    fn strength_zero_is_the_base_unchanged() {
        // strength 0 ⇒ the base `a` passes through, whatever the mode (D12 neutral).
        close(&combined(6.0, 0.0), &[0.2, 0.5, 0.8, 1.0]);
        close(&combined(0.0, 0.0), &[0.2, 0.5, 0.8, 1.0]);
    }

    #[test]
    fn strength_half_mixes_base_and_blend() {
        // Multiply at strength .5: a + (a·b − a)·.5.
        // [.2 .5 .8 1] blend [.1 .25 .32 0] → midpoints [.15 .375 .56 .5].
        close(&combined(3.0, 0.5), &[0.15, 0.375, 0.56, 0.5]);
    }

    #[test]
    fn an_absent_b_branch_reads_the_identity() {
        // With no `b` connected, `read_b_falloff` / the CPU's fallback is 1.0, so
        // Multiply leaves `a` unchanged and Min leaves `a` (a ≤ 1). The graph is
        // still valid with only `a` wired (b is an optional overlay).
        let mut g = Graph::new();
        let a = g.add_node("field.combine.test.a");
        let c = g.add_node("field.combine");
        g.connect(Edge {
            from: (a, 0),
            to: (c, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(c, "mode", 3.0); // Multiply
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, c, 0.0).unwrap();
        match out[0].as_stream().get("falloff").unwrap() {
            Column::Scalar(v) => close(v, &[0.2, 0.5, 0.8, 1.0]),
            _ => panic!("falloff"),
        }
    }
}
