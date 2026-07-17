#![forbid(unsafe_code)]
//! `motion.color_ramp` — **colour instances by a gradient**: the Blender "Color Ramp" /
//! the Houdini ramp parameter (Motion Nodes M1, colour — doc 01 §1.7 / doc 29). Maps a
//! per-instance scalar to a colour along a multi-stop gradient and writes the `tint`
//! column. Until now the only colour node was `motion.tint` (a single solid); this is
//! the continuous one — colour by index / distance / velocity / any value field.
//!
//! **Algorithm — sample a `ph2d-color::ColorRamp` per instance.** The scalar `t` for
//! element `i` is the `t` value input (`v[i]`, clamped `0..1`) when connected, else the
//! normalised index `i/(n-1)` (a gradient laid across the set). A `preset` picks a
//! built-in ramp (Rainbow / Heat / Ice / Grayscale) or a `Custom` two-stop from the
//! colour params; `interp` is Linear or Ease (smoothstep). The ramp is evaluated in
//! linear RGB — the same space the `tint` column and the compositor use — so no colour
//! conversion here. The foundational `ph2d-color` owns the ramp maths; this node is a
//! thin per-instance map. `Effect::Pure`.

use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `t` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Presets (the `preset` param).
const PRESET_RAINBOW: i64 = 0;
const PRESET_HEAT: i64 = 1;
const PRESET_ICE: i64 = 2;
const PRESET_GRAYSCALE: i64 = 3;
// PRESET_CUSTOM (4) = the two colour params.

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.color_ramp"),
    name: "motion.color_ramp",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // Per-instance scalar to map (animatable). Optional: unconnected → normalised
        // index.
        PortSpec {
            name: "t",
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
        // 0 Rainbow · 1 Heat · 2 Ice · 3 Grayscale · 4 Custom (a→b).
        ParamSpec {
            name: "preset",
            default: 0.0,
        },
        // 0 Linear · 1 Ease (smoothstep).
        ParamSpec {
            name: "interp",
            default: 0.0,
        },
        // Custom two-stop colours (linear RGB).
        ParamSpec {
            name: "a_r",
            default: 0.0,
        },
        ParamSpec {
            name: "a_g",
            default: 0.0,
        },
        ParamSpec {
            name: "a_b",
            default: 0.0,
        },
        ParamSpec {
            name: "b_r",
            default: 1.0,
        },
        ParamSpec {
            name: "b_g",
            default: 1.0,
        },
        ParamSpec {
            name: "b_b",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

fn rgb(r: f32, g: f32, b: f32) -> [f32; 4] {
    [r, g, b, 1.0]
}

/// Build the ramp for a preset (Rgb space; `interp` Linear or Ease). Custom uses the two
/// colour params `a`/`b`.
fn build_ramp(preset: i64, a: [f32; 4], b: [f32; 4], interp: RampInterp) -> ColorRamp {
    let stops: Vec<[f32; 4]> = match preset {
        PRESET_RAINBOW => vec![
            rgb(1.0, 0.0, 0.0),
            rgb(1.0, 1.0, 0.0),
            rgb(0.0, 1.0, 0.0),
            rgb(0.0, 1.0, 1.0),
            rgb(0.0, 0.0, 1.0),
            rgb(1.0, 0.0, 1.0),
            rgb(1.0, 0.0, 0.0),
        ],
        PRESET_HEAT => vec![
            rgb(0.0, 0.0, 0.0),
            rgb(0.7, 0.0, 0.0),
            rgb(1.0, 0.4, 0.0),
            rgb(1.0, 1.0, 0.2),
            rgb(1.0, 1.0, 1.0),
        ],
        PRESET_ICE => vec![
            rgb(0.0, 0.0, 0.25),
            rgb(0.0, 0.55, 1.0),
            rgb(0.75, 1.0, 1.0),
        ],
        PRESET_GRAYSCALE => vec![rgb(0.0, 0.0, 0.0), rgb(1.0, 1.0, 1.0)],
        _ => vec![a, b], // Custom
    };
    let n = stops.len().max(2);
    let ramp_stops: Vec<RampStop> = stops
        .iter()
        .enumerate()
        .map(|(i, c)| RampStop::new(i as f32 / (n as f32 - 1.0), *c))
        .collect();
    ColorRamp::new(ramp_stops, RampColorMode::Rgb, interp)
}

/// Map every instance's scalar through the ramp. `t_field` is the connected value input
/// (empty → normalised index).
fn colorize(n: usize, ramp: &ColorRamp, t_field: &[f32]) -> Vec<[f32; 4]> {
    (0..n)
        .map(|i| {
            let t = if t_field.is_empty() {
                if n <= 1 {
                    0.0
                } else {
                    i as f32 / (n as f32 - 1.0)
                }
            } else {
                t_field.get(i).copied().unwrap_or(0.0).clamp(0.0, 1.0)
            };
            ramp.eval(t)
        })
        .collect()
}

fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// GPU compute kernel (GPU/M5 **Fase 3**, ADR-0122 side channel): the WGSL port
/// of [`colorize`] + `ColorRamp::eval`, element for element.
///
/// **The positional key is why this node could not be ported before.** With `t`
/// unconnected the CPU keys the ramp on `i/(n−1)` — a *positional* identity, and
/// `ColumnBinding.identity` is a CONSTANT, which is exactly the gap the Fase 2
/// handoff logged against `motion.tint`'s Gradient. The generated `HAS_<col>`
/// const closes it: the body asks whether the column was really there, and takes
/// the CPU's other branch when it was not (see [`GpuKernel`]).
///
/// **`t` connected keeps the node on the CPU** (the `v` refusal below). Not
/// because the maths is hard — because the ANSWER would differ: the CPU reads
/// `t_field[i]` and pads a short field with `0.0`, while this engine calls a
/// column of the wrong length absent, which here would silently mean "use the
/// positional key" instead. The plan cannot prove the lengths match (a `t` chain
/// may root at another generator), so it recedes — the default of ADR-0123 D3.
/// It costs nothing today: every `value.*` node is CPU-only, so a connected `t`
/// is already a boundary.
///
/// **HR-5:** the interpolation is this crate's `lerp` (`a + (b − a)·t`), NOT
/// WGSL's `mix` (`a·(1 − t) + b·t`) — same value, different expression, and
/// therefore different rounding; likewise `smoothstep` is the node's own
/// polynomial, not the builtin. The stop positions are recomputed as `k/(n − 1)`
/// rather than derived from `t·(n − 1)`, so the bracket search and the blend
/// factor are the CPU's arithmetic to the letter.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let cr_a = vec4<f32>(params.a_r, params.a_g, params.a_b, 1.0);\n\
        let cr_b = vec4<f32>(params.b_r, params.b_g, params.b_b, 1.0);\n\
        // `t` unconnected → the normalised index (the gradient laid across the set).\n\
        var cr_t = 0.0;\n\
        if (params.count > 1u) {\n\
        \x20   cr_t = f32(i) / (f32(params.count) - 1.0);\n\
        }\n\
        write_tint(i, cr_eval(\n\
        \x20   i32(cr_round(params.preset)),\n\
        \x20   i32(cr_round(params.interp)) != 0,\n\
        \x20   cr_a, cr_b, cr_t));\n",
    wgsl_lib: "\
        fn cr_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn cr_stop_count(preset: i32) -> i32 {\n\
            if (preset == 0) { return 7; }\n\
            if (preset == 1) { return 5; }\n\
            if (preset == 2) { return 3; }\n\
            if (preset == 3) { return 2; }\n\
            return 2;\n\
        }\n\
        fn cr_stop(preset: i32, k: i32, a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {\n\
            if (preset == 0) {\n\
                if (k == 0) { return vec4<f32>(1.0, 0.0, 0.0, 1.0); }\n\
                if (k == 1) { return vec4<f32>(1.0, 1.0, 0.0, 1.0); }\n\
                if (k == 2) { return vec4<f32>(0.0, 1.0, 0.0, 1.0); }\n\
                if (k == 3) { return vec4<f32>(0.0, 1.0, 1.0, 1.0); }\n\
                if (k == 4) { return vec4<f32>(0.0, 0.0, 1.0, 1.0); }\n\
                if (k == 5) { return vec4<f32>(1.0, 0.0, 1.0, 1.0); }\n\
                return vec4<f32>(1.0, 0.0, 0.0, 1.0);\n\
            }\n\
            if (preset == 1) {\n\
                if (k == 0) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }\n\
                if (k == 1) { return vec4<f32>(0.7, 0.0, 0.0, 1.0); }\n\
                if (k == 2) { return vec4<f32>(1.0, 0.4, 0.0, 1.0); }\n\
                if (k == 3) { return vec4<f32>(1.0, 1.0, 0.2, 1.0); }\n\
                return vec4<f32>(1.0, 1.0, 1.0, 1.0);\n\
            }\n\
            if (preset == 2) {\n\
                if (k == 0) { return vec4<f32>(0.0, 0.0, 0.25, 1.0); }\n\
                if (k == 1) { return vec4<f32>(0.0, 0.55, 1.0, 1.0); }\n\
                return vec4<f32>(0.75, 1.0, 1.0, 1.0);\n\
            }\n\
            if (preset == 3) {\n\
                if (k == 0) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }\n\
                return vec4<f32>(1.0, 1.0, 1.0, 1.0);\n\
            }\n\
            if (k == 0) { return a; }\n\
            return b;\n\
        }\n\
        // RampStop::new clamps, and k/(n-1) is already in [0,1].\n\
        fn cr_pos(k: i32, n: i32) -> f32 {\n\
            return f32(k) / (f32(n) - 1.0);\n\
        }\n\
        fn cr_lerp4(a: vec4<f32>, b: vec4<f32>, t: f32) -> vec4<f32> {\n\
            // `lerp(a, b, t) = a + (b - a) * t` — NOT WGSL `mix`, which is\n\
            // `a * (1 - t) + b * t`: same value, different rounding.\n\
            return a + (b - a) * t;\n\
        }\n\
        fn cr_eval(preset: i32, ease: bool, a: vec4<f32>, b: vec4<f32>, t: f32) -> vec4<f32> {\n\
            let n = cr_stop_count(preset);\n\
            let first = cr_stop(preset, 0, a, b);\n\
            if (n == 1 || t <= cr_pos(0, n)) { return first; }\n\
            let last = cr_stop(preset, n - 1, a, b);\n\
            if (t >= cr_pos(n - 1, n)) { return last; }\n\
            // `partition_point(|s| s.pos <= t) - 1`: the last stop at or before t.\n\
            var idx = 0;\n\
            for (var k = 1; k < n; k = k + 1) {\n\
                if (cr_pos(k, n) <= t) { idx = k; }\n\
            }\n\
            let ca = cr_stop(preset, idx, a, b);\n\
            let cb = cr_stop(preset, idx + 1, a, b);\n\
            let span = max(cr_pos(idx + 1, n) - cr_pos(idx, n), 1e-8);\n\
            var fac = clamp((t - cr_pos(idx, n)) / span, 0.0, 1.0);\n\
            if (ease) { fac = fac * fac * (3.0 - 2.0 * fac); }\n\
            return cr_lerp4(ca, cb, fac);\n\
        }\n",
    bindings: &[
        ColumnBinding {
            // Written, never read: the node REPLACES the tint rather than
            // blending onto it (`out.set(\"tint\", …)` after copying the rest).
            column: "tint",
            dim: Dim::Vec4,
            access: ColumnAccess::Write,
            identity: [1.0; 4],
            port: 0,
        },
        ColumnBinding {
            // See the doc above: a connected `t` is a shape this kernel cannot
            // answer for, so it keeps the whole node on the CPU.
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::RefuseIfPresent,
            identity: [0.0; 4],
            port: 1,
        },
    ],
    params: &["preset", "interp", "a_r", "a_g", "a_b", "b_r", "b_g", "b_b"],
    source_count: None,
    applicable: None,
};

struct MotionColorRamp;

impl NodeOp for MotionColorRamp {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let preset = ctx.param("preset").round() as i64;
        let interp = if ctx.param("interp").round() as i64 == 0 {
            RampInterp::Linear
        } else {
            RampInterp::Ease
        };
        let a = rgb(ctx.param("a_r"), ctx.param("a_g"), ctx.param("a_b"));
        let b = rgb(ctx.param("b_r"), ctx.param("b_g"), ctx.param("b_b"));
        let ramp = build_ramp(preset, a, b, interp);
        let t_field = scalar_col(ctx.input(1), VALUE_COL);
        let input = ctx.input(0);
        let n = input.count();
        let tint = colorize(n, &ramp, &t_field);
        let mut out = Stream::new(n);
        for (name, col) in input.columns() {
            if name != "tint" {
                out.set(name.clone(), col.clone());
            }
        }
        out.set("tint", Column::Vec4(tint));
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionColorRamp))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Color Ramp",
            category: ph2d_node_registry::NodeUiCategory::Fx,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "preset",
        label: "Preset",
        min: 0.0,
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Rainbow", "Heat", "Ice", "Grayscale", "Custom"],
        },
    },
    ParamUiHint {
        param: "interp",
        label: "Interp",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Linear", "Ease"],
        },
    },
    chan("a_r", "A R"),
    chan("a_g", "A G"),
    chan("a_b", "A B"),
    chan("b_r", "B R"),
    chan("b_g", "B G"),
    chan("b_b", "B B"),
];

const fn chan(param: &'static str, label: &'static str) -> ParamUiHint {
    ParamUiHint {
        param,
        label,
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Grayscale by normalised index: the first element is black, the last is white, and
    /// the middle is mid-grey. FALSIFIED if the ramp were a single solid colour.
    #[test]
    fn grayscale_spreads_black_to_white_by_index() {
        let ramp = build_ramp(
            PRESET_GRAYSCALE,
            rgb(0.0, 0.0, 0.0),
            rgb(1.0, 1.0, 1.0),
            RampInterp::Linear,
        );
        let c = colorize(5, &ramp, &[]);
        assert!(c[0][0] < 0.05, "first is black: {:?}", c[0]);
        assert!(c[4][0] > 0.95, "last is white: {:?}", c[4]);
        assert!((c[2][0] - 0.5).abs() < 0.1, "middle is grey: {:?}", c[2]);
    }

    /// The `t` value field overrides the index: two elements both fed `t=1` are both the
    /// ramp's end colour (white), regardless of their index.
    #[test]
    fn the_t_field_overrides_the_index() {
        let ramp = build_ramp(
            PRESET_GRAYSCALE,
            rgb(0.0, 0.0, 0.0),
            rgb(1.0, 1.0, 1.0),
            RampInterp::Linear,
        );
        let c = colorize(2, &ramp, &[1.0, 1.0]);
        assert!(c[0][0] > 0.95 && c[1][0] > 0.95, "both white: {c:?}");
    }

    /// The rainbow preset actually spans hues: across the set the colours are not all
    /// equal (the red channel alone varies a lot).
    #[test]
    fn rainbow_spans_many_colours() {
        let ramp = build_ramp(
            PRESET_RAINBOW,
            rgb(0.0, 0.0, 0.0),
            rgb(0.0, 0.0, 0.0),
            RampInterp::Linear,
        );
        let c = colorize(12, &ramp, &[]);
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for col in &c {
            lo = lo.min(col[2]); // blue channel sweeps 0→1→0 across the wheel
            hi = hi.max(col[2]);
        }
        assert!(hi - lo > 0.8, "the rainbow spans (blue {lo}..{hi})");
    }

    /// Deterministic + cooks through the registry: writes the `tint` column at the full
    /// count and passes the geometry columns through.
    #[test]
    fn registers_and_colours_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.color_ramp.test.src"),
            name: "motion.color_ramp.test.src",
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
                ctx.emit(
                    Stream::new(3)
                        .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]])),
                );
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionColorRamp),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let src = g.add_node("motion.color_ramp.test.src");
        let cr = g.add_node("motion.color_ramp");
        g.set_param(cr, "preset", PRESET_GRAYSCALE as f32);
        g.connect(Edge {
            from: (src, 0),
            to: (cr, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, cr, 0.0).unwrap();
        let s = out[0].as_stream();
        assert!(s.get("P").is_some(), "geometry passes through");
        match s.get("tint").unwrap() {
            Column::Vec4(v) => {
                assert_eq!(v.len(), 3, "tint at full count");
                assert!(v[0][0] < 0.05 && v[2][0] > 0.95, "black to white by index");
            }
            _ => panic!("tint"),
        }
    }
}
