#![forbid(unsafe_code)]
//! `motion.tint` — a Motion **colour modifier**: sets the `tint` (Vec4 RGBA,
//! linear straight — §1.2) attribute to a target colour, masked per-instance by
//! the multiplicative `falloff` column. The per-instance result is
//! `lerp(existing, target_i, falloff_i)`, so at `falloff = 1` the instance takes
//! the target **exactly** and at `falloff = 0` it keeps its existing tint (absent
//! → opaque white). Every other column passes through unchanged. Pure.
//!
//! Two modes (`mode`): **Solid** (0, the default) applies one uniform colour
//! `(r,g,b,a)`; **Gradient** (1) sweeps from `(r,g,b,a)` (Start) to
//! `(r2,g2,b2,a2)` (End) across the stream, keyed by each instance's normalized
//! identity `Index/(Count−1)` (from the grid; absent → positional `i/(n−1)`), so
//! a grid reads as a colour ramp. The gradient lerp is in the wire's linear space
//! (transcendental-free; OKLab is a future, cbrt-gated refinement).
//!
//! Defaults: `mode` 0 (Solid); Start `r=g=b=a=1` (**opaque white** — the identity,
//! so a fresh Tint is a no-op until the artist picks a colour); End `(0,0,0,1)`
//! (black — so switching to Gradient shows a visible white→black ramp).

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.tint"),
    name: "motion.tint",
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
            name: "mode",
            default: 0.0,
        },
        // Start (Solid = the colour; Gradient = the low end).
        ParamSpec {
            name: "r",
            default: 1.0,
        },
        ParamSpec {
            name: "g",
            default: 1.0,
        },
        ParamSpec {
            name: "b",
            default: 1.0,
        },
        ParamSpec {
            name: "a",
            default: 1.0,
        },
        // End (Gradient only).
        ParamSpec {
            name: "r2",
            default: 0.0,
        },
        ParamSpec {
            name: "g2",
            default: 0.0,
        },
        ParamSpec {
            name: "b2",
            default: 0.0,
        },
        ParamSpec {
            name: "a2",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The multiplicative `falloff` weight for instance `i` (absent → `1.0`).
fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// Read entry `i` of a Scalar column (absent / wrong-typed → `default`).
fn scalar_at(col: Option<&Column>, i: usize, default: f32) -> f32 {
    match col {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(default),
        _ => default,
    }
}

/// Per-channel RGBA lerp `a·(1−t) + b·t` (endpoint-exact; linear-straight space).
fn lerp4(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] * (1.0 - t) + b[0] * t,
        a[1] * (1.0 - t) + b[1] * t,
        a[2] * (1.0 - t) + b[2] * t,
        a[3] * (1.0 - t) + b[3] * t,
    ]
}

/// The falloff-masked colour for one instance: `lerp(existing, target, f)` per
/// RGBA channel via the endpoint-exact form `existing·(1-f) + target·f` — so it
/// returns exactly `existing` at `f = 0` and exactly `target` at `f = 1` (any
/// colour + alpha, no float drift).
fn mixed_tint(existing: [f32; 4], target: [f32; 4], f: f32) -> [f32; 4] {
    let lerp = |e: f32, t: f32| e * (1.0 - f) + t * f;
    [
        lerp(existing[0], target[0]),
        lerp(existing[1], target[1]),
        lerp(existing[2], target[2]),
        lerp(existing[3], target[3]),
    ]
}

/// GPU compute kernel (GPU/M5 Fase 2, ADR-0126): `tint' = lerp(existing, target,
/// falloff)` per RGBA channel — the exact `mixed_tint` form, so parity is
/// bit-exact (no transcendentals). `ReadWrite` on `tint` mirrors the CPU: an
/// absent `tint` column starts from opaque white `[1,1,1,1]` and the column is
/// always written.
///
/// **Gradient runs here too, and the positional key is why it could not before.**
/// The ramp keys off `Index/(Count−1)`, falling back to `i`/`n` when those
/// columns are absent — a *positional* identity, and `ColumnBinding.identity` is
/// a CONSTANT, so `read_Index`'s fallback cannot be `f32(i)`. The generated
/// `HAS_<col>` const closes it exactly as it did for `motion.color_ramp`: the
/// body asks whether the column was really there and takes the CPU's other
/// branch when it was not. This is the gap the Fase 2 handoff logged against
/// this very node, and it is what lets the emitter's documented affordance
/// ("ids ascend oldest-first, so Gradient paints the stream by AGE") survive on
/// the GPU path instead of dropping the whole fountain to the CPU.
///
/// **The short-column hazard `motion.color_ramp` recedes over does not exist
/// here**, and the difference is structural, not luck: its `t` arrives on
/// ANOTHER port, where a chain rooted at a different generator can be a
/// different length, and this engine calls a wrong-length column *absent* — which
/// there would silently mean "use the positional key" while the CPU pads. `Index`
/// and `Count` ride the SAME port as the base, and [`Stream::set`] asserts
/// `col.len() == count`, so present ⟺ exists and a present column is exactly `n`
/// long. The CPU's `unwrap_or(default)` is unreachable for these two — defensive,
/// not a semantic branch — so `HAS_x ? read_x(i) : positional` *is* `scalar_at`.
///
/// **HR-5:** the lerps are written as the CPU writes them (`a·(1−t) + b·t`, the
/// `lerp4`/`mixed_tint` form), never WGSL's `mix` — same value, different
/// expression, different rounding. `mode` is rounded half-away-from-zero
/// (Rust's `f32::round`), not by WGSL's half-even builtin.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let tn_e = read_tint(i);\n\
        var tn_t = vec4<f32>(params.r, params.g, params.b, params.a);\n\
        if (i32(tn_round(params.mode)) == 1) {\n\
        \x20   // Absent Index/Count → the POSITIONAL key, the CPU's own fallback.\n\
        \x20   var tn_idx = f32(i);\n\
        \x20   if (HAS_Index) { tn_idx = read_Index(i); }\n\
        \x20   var tn_cnt = f32(params.count);\n\
        \x20   if (HAS_Count) { tn_cnt = read_Count(i); }\n\
        \x20   var tn_g = 0.0;\n\
        \x20   if (tn_cnt > 1.0) { tn_g = clamp(tn_idx / (tn_cnt - 1.0), 0.0, 1.0); }\n\
        \x20   let tn_end = vec4<f32>(params.r2, params.g2, params.b2, params.a2);\n\
        \x20   tn_t = vec4<f32>(\n\
        \x20       tn_t.x * (1.0 - tn_g) + tn_end.x * tn_g,\n\
        \x20       tn_t.y * (1.0 - tn_g) + tn_end.y * tn_g,\n\
        \x20       tn_t.z * (1.0 - tn_g) + tn_end.z * tn_g,\n\
        \x20       tn_t.w * (1.0 - tn_g) + tn_end.w * tn_g);\n\
        }\n\
        let tn_f = read_falloff(i);\n\
        write_tint(i, vec4<f32>(\n\
            tn_e.x * (1.0 - tn_f) + tn_t.x * tn_f,\n\
            tn_e.y * (1.0 - tn_f) + tn_t.y * tn_f,\n\
            tn_e.z * (1.0 - tn_f) + tn_t.z * tn_f,\n\
            tn_e.w * (1.0 - tn_f) + tn_t.w * tn_f));\n",
    wgsl_lib: "\
        fn tn_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "tint",
            dim: Dim::Vec4,
            access: ColumnAccess::ReadWrite,
            identity: [1.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
        // Read for the ramp key only; the `HAS_` const above decides whether the
        // value or the positional fallback is used, so the identity is inert.
        ColumnBinding {
            column: "Index",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "Count",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &["r", "g", "b", "a", "r2", "g2", "b2", "a2", "mode"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct MotionTint;

impl NodeOp for MotionTint {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let gradient = ctx.param("mode").round() as i32 == 1;
        let start = [
            ctx.param("r"),
            ctx.param("g"),
            ctx.param("b"),
            ctx.param("a"),
        ];
        let end = [
            ctx.param("r2"),
            ctx.param("g2"),
            ctx.param("b2"),
            ctx.param("a2"),
        ];
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            // Base per-instance tint (absent column → opaque white).
            let base: Vec<[f32; 4]> = match input.get("tint") {
                Some(Column::Vec4(v)) => v.clone(),
                _ => vec![[1.0, 1.0, 1.0, 1.0]; n],
            };
            // Identity columns for the gradient ramp (absent → positional).
            let index = input.get("Index");
            let count = input.get("Count");
            let tinted: Vec<[f32; 4]> = (0..n)
                .map(|i| {
                    let target = if gradient {
                        let idx = scalar_at(index, i, i as f32);
                        let cnt = scalar_at(count, i, n as f32);
                        // Normalized position 0..1 across the stream (single
                        // instance → 0, so it just takes the Start colour).
                        let t = if cnt > 1.0 {
                            (idx / (cnt - 1.0)).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        lerp4(start, end, t)
                    } else {
                        start
                    };
                    let e = base.get(i).copied().unwrap_or([1.0, 1.0, 1.0, 1.0]);
                    mixed_tint(e, target, falloff_at(input, i))
                })
                .collect();
            let mut out = Stream::new(n);
            for (name, col) in input.columns() {
                if name != "tint" {
                    out.set(name.clone(), col.clone());
                }
            }
            out.set("tint", Column::Vec4(tinted));
            out
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionTint))?;
    // M1.R1 — UI metadata (a colour effect → magenta Fx, rounded-rect).
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Tint",
            category: ph2d_node_registry::NodeUiCategory::Fx,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    // GPU/M5 Fase 2 (ADR-0126): the WGSL lowering (Solid mode), on the side.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1 → colour authoring): a **named** Solid/Gradient selector,
/// then one canonical colour swatch → OKLCH picker per colour (never raw linear
/// sliders — a raw `0.5` linear reads as light grey). Each [`ParamWidget::Color`]
/// hint anchors on its first channel and names the four params it drives; the
/// shell bridge reads the pick back (sRGB→linear). The `End` swatch is only used
/// in Gradient mode (it paints regardless — v1 has no per-mode row hiding).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Solid", "Gradient"],
        },
    },
    ParamUiHint {
        param: "r",
        label: "Color",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Color {
            channels: ["r", "g", "b", "a"],
        },
    },
    ParamUiHint {
        param: "r2",
        label: "End",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Color {
            channels: ["r2", "g2", "b2", "a2"],
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // Source: 2 white instances with falloff [1, 0.5].
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.tint.test.src"),
        name: "motion.tint.test.src",
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
            ctx.emit(
                Stream::new(2)
                    .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 1.0]]))
                    .with("falloff", Column::Scalar(vec![1.0, 0.5])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionTint),
                _ => None,
            }
        }
    }

    #[test]
    fn tint_sets_target_masked_by_falloff() {
        let mut g = Graph::new();
        let src = g.add_node("motion.tint.test.src");
        let tn = g.add_node("motion.tint");
        g.connect(Edge {
            from: (src, 0),
            to: (tn, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(tn, "r", 1.0);
        g.set_param(tn, "g", 0.0);
        g.set_param(tn, "b", 0.0);
        g.set_param(tn, "a", 0.4);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, tn, 0.0).unwrap();
        match out[0].as_stream().get("tint").unwrap() {
            // existing = white; target = (1,0,0,0.4).
            // i0 f=1: exactly the target ; i1 f=0.5: lerp(white,target,0.5).
            Column::Vec4(v) => {
                assert_eq!(v, &vec![[1.0, 0.0, 0.0, 0.4], [1.0, 0.5, 0.5, 0.7]]);
            }
            _ => panic!("tint"),
        }
    }

    fn default_of(name: &str) -> f32 {
        MANIFEST
            .params
            .iter()
            .find(|p| p.name == name)
            .unwrap()
            .default
    }

    #[test]
    fn default_params_are_opaque_white_and_no_op_on_white() {
        // The colour modifier's identity: Solid mode by default, Start opaque
        // white → a white stream stays white at every falloff (no red/warm
        // dominance from merely dropping in a Tint — the reported-cast fix).
        assert_eq!(default_of("mode"), 0.0, "default mode is Solid");
        let start = [
            default_of("r"),
            default_of("g"),
            default_of("b"),
            default_of("a"),
        ];
        assert_eq!(start, [1.0, 1.0, 1.0, 1.0]);
        let white = [1.0, 1.0, 1.0, 1.0];
        assert_eq!(mixed_tint(white, white, 1.0), white);
        assert_eq!(mixed_tint(white, white, 0.5), white);
        assert_eq!(mixed_tint(white, white, 0.0), white);
    }

    #[test]
    fn lerp4_is_endpoint_exact() {
        let a = [1.0, 1.0, 1.0, 1.0];
        let b = [0.2, 0.4, 0.6, 0.3];
        assert_eq!(lerp4(a, b, 0.0), a);
        assert_eq!(lerp4(a, b, 1.0), b);
        assert_eq!(lerp4(a, b, 0.5), [0.6, 0.7, 0.8, 0.65]);
    }

    #[test]
    fn gradient_mode_ramps_start_to_end_by_normalized_index() {
        // A 3-instance stream carrying Index[0,1,2]+Count[3,3,3]; Gradient mode,
        // default Start=white / End=black → a grayscale ramp keyed by index.
        static GSRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.tint.test.grid"),
            name: "motion.tint.test.grid",
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
        struct GSrc;
        impl NodeOp for GSrc {
            fn manifest(&self) -> &'static NodeManifest {
                &GSRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(
                    Stream::new(3)
                        .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]))
                        .with("Index", Column::Scalar(vec![0.0, 1.0, 2.0]))
                        .with("Count", Column::Scalar(vec![3.0, 3.0, 3.0])),
                );
            }
        }
        struct GOps;
        impl OpResolver for GOps {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == GSRC.id => Some(&GSrc),
                    t if t == MANIFEST.id => Some(&MotionTint),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let src = g.add_node("motion.tint.test.grid");
        let tn = g.add_node("motion.tint");
        g.connect(Edge {
            from: (src, 0),
            to: (tn, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(tn, "mode", 1.0); // Gradient (Start=white, End=black defaults)
        let mut cook = Cook::new();
        let out = cook.cook(&g, &GOps, tn, 0.0).unwrap();
        match out[0].as_stream().get("tint").unwrap() {
            // falloff absent → 1, so tint == target. t = 0, 0.5, 1 across the ramp.
            Column::Vec4(v) => {
                assert_eq!(v[0], [1.0, 1.0, 1.0, 1.0]); // start (white)
                assert_eq!(v[1], [0.5, 0.5, 0.5, 1.0]); // mid grey
                assert_eq!(v[2], [0.0, 0.0, 0.0, 1.0]); // end (black)
            }
            _ => panic!("tint"),
        }
    }

    #[test]
    fn mixed_tint_reaches_any_rgba_at_full_falloff() {
        // f=0 → exactly existing (identity); f=1 → exactly the target (all RGBA).
        assert_eq!(
            mixed_tint([1.0, 1.0, 1.0, 1.0], [0.2, 0.4, 0.6, 0.3], 0.0),
            [1.0, 1.0, 1.0, 1.0]
        );
        assert_eq!(
            mixed_tint([1.0, 1.0, 1.0, 1.0], [0.2, 0.4, 0.6, 0.3], 1.0),
            [0.2, 0.4, 0.6, 0.3]
        );
    }
}
