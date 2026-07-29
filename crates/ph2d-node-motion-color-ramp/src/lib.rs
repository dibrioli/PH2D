#![forbid(unsafe_code)]
//! `motion.color_ramp` — **colour instances by a gradient**: the Blender "Color Ramp" /
//! the Houdini ramp parameter (Motion Nodes M1, colour — doc 01 §1.7 / doc 29 / doc 85).
//! Maps a per-instance scalar to a colour along a multi-stop gradient and writes the `tint`
//! column. Until now the only colour node was `motion.tint` (a single solid); this is the
//! continuous one — colour by index / distance / velocity / any value field.
//!
//! **One gradient, always editable (doc 85).** The ramp is a `ph2d-color::ColorRamp`
//! authored in a TEXT param (`RAMP_KEY`, serialized by `serialize_gradient`) — the panel's
//! `ParamWidget::Gradient` editor. There is NO separate "preset vs custom" mode: the presets
//! (Rainbow / Heat / Ice / Grayscale) are one-click **seeds** the editor loads into that same
//! editable ramp (`ph2d_color::GradientPreset`), so a preset's colours appear as draggable,
//! recolourable stops. An unset/malformed string falls back to `default_gradient()` (Rainbow),
//! so a fresh node is colourful from the first frame — the CPU eval, the GPU LUT fill and the
//! panel all agree on that fallback.
//!
//! **Algorithm.** The scalar `t` for element `i` is the `t` value input (`v[i]`, clamped
//! `0..1`) when connected, else the normalised index `i/(n-1)` (a gradient laid across the
//! set). The ramp is evaluated in linear RGB — the same space the `tint` column and the
//! compositor use — so no colour conversion here. `Effect::Pure`.

use ph2d_color::{ColorRamp, default_gradient, parse_gradient};
use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, LutSpec};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `t` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// The text-param key for the gradient (doc 32/85), read via [`EvalCtx::text_param`] and
/// sampled by the GPU LUTs ([`LUTS`]). The `ParamWidget::Gradient` hint names it; the panel
/// edits it as a draggable, swatch-per-stop gradient bar with preset seeds.
const RAMP_KEY: &str = "ramp";

/// The gradient's LUT resolution (doc 85, mirror of `value.curve` A1-gpu): samples of the
/// authored ramp over `t ∈ [0,1]`, one table per colour channel. 256 keeps a smooth gradient
/// within a few thousandths of the CPU `eval`.
const LUT_RESOLUTION: u32 = 256;

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
    // No f32 params: the whole gradient (stops + interp) lives in the `RAMP_KEY` text param
    // (doc 85). Presets are editor seeds, not a mode; interp is a token in the string.
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

/// The ramp to colour with — the `RAMP_KEY` text param, else the default (Rainbow). Shared
/// by [`eval`] so a fresh/malformed node renders the same ramp the panel + LUTs fall back to.
fn ramp_of(custom: Option<&str>) -> ColorRamp {
    custom
        .and_then(parse_gradient)
        .unwrap_or_else(default_gradient)
}

/// Map every instance's scalar through the ramp. `t_field` is the connected value input
/// (empty → normalised index).
fn colorize(n: usize, ramp: &ColorRamp, t_field: &[f32]) -> Vec<[f32; 4]> {
    (0..n)
        .map(|i| {
            // The value-field convention, `motion.look_at::target_at`'s `0/1/n`
            // ladder: absent -> the positional key; **length 1 -> BROADCAST**
            // (one global `t` colours the whole set — a `value.lfo` unconnected
            // is exactly that); else per-element, zero-padded. This node used to
            // skip the broadcast arm and paint everything but element 0 with
            // `t = 0` the moment a global value drove it (found porting the `t`
            // path to the GPU, ADR-0136 — the kernel's broadcast reader IS the
            // ladder, and the CPU had to be the canon it mirrors).
            let t = match t_field.len() {
                0 => {
                    if n <= 1 {
                        0.0
                    } else {
                        i as f32 / (n as f32 - 1.0)
                    }
                }
                1 => t_field[0].clamp(0.0, 1.0), // CLAMP-OK: ramp key
                _ => t_field.get(i).copied().unwrap_or(0.0).clamp(0.0, 1.0),
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

/// GPU compute kernel (GPU/M5 **Fase 3**, ADR-0126 side channel): the WGSL port
/// of [`colorize`] + `ColorRamp::eval`, element for element.
///
/// **The positional key is why this node could not be ported before.** With `t`
/// unconnected the CPU keys the ramp on `i/(n−1)` — a *positional* identity, and
/// `ColumnBinding.identity` is a CONSTANT, which is exactly the gap the Fase 2
/// handoff logged against `motion.tint`'s Gradient. The generated `HAS_<col>`
/// const closes it: the body asks whether the column was really there, and takes
/// the CPU's other branch when it was not (see [`GpuKernel`]).
///
/// **`t` connected rides the broadcast reader** (ADR-0136). `ReadBroadcast` pairs
/// a per-element field positionally and pins a length-1 field to row 0 — the CPU's
/// `0/1/n` ladder — and a field at any OTHER length is REFUSED at cook time
/// (`BroadcastLengthMismatch`).
///
/// **The gradient rides the LUT channel (doc 85).** A multi-stop gradient is not a fixed
/// set of params, so the body samples three scalar LUTs (`cr_grad_r/g/b`, [`LUTS`]) the
/// sequencer bakes from the `ramp` text param via [`ColorRamp::bake_into`] — the exact
/// device analog of `value.curve`'s curve LUT. Presets are the SAME LUT (they are just seeds
/// of the string), so this kernel is ONE branch — no inline preset table.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        // `t` connected → the value field (per-element, or row 0 broadcast);\n\
        // unconnected → the normalised index (the gradient across the set).\n\
        var cr_t = 0.0;\n\
        if (HAS_t_v) {\n\
        \x20   cr_t = clamp(read_t_v(i), 0.0, 1.0);\n\
        } else if (params.count > 1u) {\n\
        \x20   cr_t = f32(i) / (f32(params.count) - 1.0);\n\
        }\n\
        write_tint(i, vec4<f32>(\n\
        \x20   cr_grad_r_sample(cr_t),\n\
        \x20   cr_grad_g_sample(cr_t),\n\
        \x20   cr_grad_b_sample(cr_t),\n\
        \x20   1.0));\n",
    wgsl_lib: "",
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
            // The `t` field (port 1): per-element or a length-1 broadcast — see
            // the doc above. Absent reads the identity, and `HAS_v` routes the
            // body to the positional key instead (the CPU's empty-field arm).
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 1,
        },
    ],
    params: &[],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// The gradient's per-channel LUTs (doc 85): three scalar tables the sequencer bakes from
/// the `ramp` text param, sampled by the WGSL as `cr_grad_r_sample(t)` etc. A colour LUT is
/// just three scalar LUTs, so this reuses the existing scalar LUT channel three times —
/// **zero foundational GPU change** (the exact sibling of `value.curve`'s single LUT).
static LUTS: &[LutSpec] = &[
    LutSpec {
        name: "cr_grad_r",
        text_key: RAMP_KEY,
        resolution: LUT_RESOLUTION,
        fill: fill_grad_r,
    },
    LutSpec {
        name: "cr_grad_g",
        text_key: RAMP_KEY,
        resolution: LUT_RESOLUTION,
        fill: fill_grad_g,
    },
    LutSpec {
        name: "cr_grad_b",
        text_key: RAMP_KEY,
        resolution: LUT_RESOLUTION,
        fill: fill_grad_b,
    },
];

/// Sample ONE channel of the authored gradient into `out` at `t = k/(n−1)` — the node-side
/// half of the LUT channel (`ph2d-nodegraph` stays colour-library agnostic; the sampling
/// lives here). An unset/malformed string is `default_gradient()` (Rainbow), matching the
/// CPU `eval`'s fallback so the two paths agree on "nothing authored".
fn fill_grad_channel(text: &str, out: &mut [f32], channel: usize) {
    let ramp = ramp_of(Some(text));
    let n = out.len();
    for (k, slot) in out.iter_mut().enumerate() {
        let t = if n <= 1 {
            0.0
        } else {
            k as f32 / (n - 1) as f32
        };
        *slot = ramp.eval(t)[channel];
    }
}

fn fill_grad_r(text: &str, out: &mut [f32]) {
    fill_grad_channel(text, out, 0);
}
fn fill_grad_g(text: &str, out: &mut [f32]) {
    fill_grad_channel(text, out, 1);
}
fn fill_grad_b(text: &str, out: &mut [f32]) {
    fill_grad_channel(text, out, 2);
}

struct MotionColorRamp;

impl NodeOp for MotionColorRamp {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let ramp = ramp_of(ctx.text_param(RAMP_KEY));
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
    reg.register_luts(MANIFEST.id, LUTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    // The gradient — a TEXT param (`RAMP_KEY`), not a `ParamSpec` (a multi-stop gradient is
    // not a fixed set of f32). Doc 85's editor: a draggable bar, a swatch per stop, and the
    // preset seeds (Rainbow / Heat / Ice / Grayscale) that load into it.
    ParamUiHint {
        param: RAMP_KEY,
        label: "Gradient",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Gradient,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_color::GradientPreset;

    /// **A length-1 `t` field is a BROADCAST** — the value convention's `0/1/n`
    /// ladder (`motion.look_at::target_at` is the canon). This node used to
    /// take the `_` arm for it: element 0 got the value and every other element
    /// got `t = 0`, so a `value.lfo` driving the ramp coloured exactly one
    /// spark (found porting the `t` path to the GPU, ADR-0136).
    #[test]
    fn a_length_one_t_field_broadcasts_to_every_element() {
        let ramp = GradientPreset::Grayscale.ramp();
        let tinted = colorize(5, &ramp, &[0.75]);
        for (i, c) in tinted.iter().enumerate() {
            assert_eq!(
                c, &tinted[0],
                "element {i} must wear the SAME broadcast colour"
            );
        }
        // …and the broadcast value is the field's, not the positional key: the
        // grayscale ramp at t = 0.75 is 0.75 grey, not black.
        assert!(
            (tinted[0][0] - 0.75).abs() < 1e-6,
            "broadcast t = 0.75 on grayscale: got {:?}",
            tinted[0]
        );
    }

    /// Grayscale by normalised index: the first element is black, the last is white, and
    /// the middle is mid-grey. FALSIFIED if the ramp were a single solid colour.
    #[test]
    fn grayscale_spreads_black_to_white_by_index() {
        let c = colorize(5, &GradientPreset::Grayscale.ramp(), &[]);
        assert!(c[0][0] < 0.05, "first is black: {:?}", c[0]);
        assert!(c[4][0] > 0.95, "last is white: {:?}", c[4]);
        assert!((c[2][0] - 0.5).abs() < 0.1, "middle is grey: {:?}", c[2]);
    }

    /// The `t` value field overrides the index: two elements both fed `t=1` are both the
    /// ramp's end colour (white), regardless of their index.
    #[test]
    fn the_t_field_overrides_the_index() {
        let c = colorize(2, &GradientPreset::Grayscale.ramp(), &[1.0, 1.0]);
        assert!(c[0][0] > 0.95 && c[1][0] > 0.95, "both white: {c:?}");
    }

    /// **The gradient string colours the set** (doc 85). A red→green→blue gradient laid
    /// across the set colours the first element red, the middle green, the last blue.
    /// FALSIFIED if the node ignored the string.
    #[test]
    fn a_gradient_string_colours_the_set() {
        let ramp = ramp_of(Some("g1 2 0:1,0,0 0.5:0,1,0 1:0,0,1"));
        let c = colorize(3, &ramp, &[]);
        assert!(c[0][0] > 0.95 && c[0][1] < 0.05, "first red: {:?}", c[0]);
        assert!(c[1][1] > 0.95 && c[1][0] < 0.05, "middle green: {:?}", c[1]);
        assert!(c[2][2] > 0.95 && c[2][0] < 0.05, "last blue: {:?}", c[2]);
    }

    /// An unset / malformed string falls back to the default gradient (Rainbow) — never a
    /// half-built gradient, never a crash. A fresh node is colourful.
    #[test]
    fn unset_falls_back_to_the_rainbow_default() {
        let none = colorize(7, &ramp_of(None), &[]);
        let bad = colorize(7, &ramp_of(Some("nonsense")), &[]);
        assert_eq!(none, bad, "None and malformed both use the default");
        // Rainbow: first stop is red.
        assert!(
            none[0][0] > 0.95 && none[0][2] < 0.05,
            "first red: {:?}",
            none[0]
        );
        // …and it spans hues (not a flat colour).
        let (lo, hi) = none.iter().fold((f32::MAX, f32::MIN), |(lo, hi), c| {
            (lo.min(c[2]), hi.max(c[2]))
        });
        assert!(hi - lo > 0.8, "the default rainbow spans (blue {lo}..{hi})");
    }

    /// **The GPU LUT fill mirrors the CPU `eval`** (doc 85, the device half). Baking the red
    /// channel of a red→green→blue gradient gives red at t=0 and zero red at t=1 — the same
    /// colour the CPU `colorize` paints. The malformed string bakes the default (Rainbow),
    /// matching the CPU fallback, so the two paths agree on "nothing authored".
    #[test]
    fn the_lut_fill_samples_each_channel_and_falls_back() {
        let grad = "g1 2 0:1,0,0 0.5:0,1,0 1:0,0,1";
        let mut r = [0.0f32; 256];
        fill_grad_r(grad, &mut r);
        assert!(r[0] > 0.95, "red LUT starts at 1.0: {}", r[0]);
        assert!(r[255] < 0.05, "red LUT ends at 0.0: {}", r[255]);
        let ramp = parse_gradient(grad).unwrap();
        assert!((r[0] - ramp.eval(0.0)[0]).abs() < 1e-6, "LUT[0] == eval(0)");
        assert!(
            (r[255] - ramp.eval(1.0)[0]).abs() < 1e-6,
            "LUT[255] == eval(1)"
        );
        // Malformed → the default gradient (Rainbow): red at t=0 (the first stop is red).
        let mut bad = [9.0f32; 256];
        fill_grad_r("nonsense", &mut bad);
        assert!(
            bad[0] > 0.95,
            "fallback rainbow baked (red at 0): {}",
            bad[0]
        );
    }

    /// Deterministic + cooks through the registry: writes the `tint` column at the full
    /// count and passes the geometry columns through. The ramp comes from the text param.
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
        // A grayscale gradient (black→white) so the index sweep is black to white.
        g.set_text_param(cr, RAMP_KEY, "g1 2 0:0,0,0 1:1,1,1".to_string());
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

    /// The gradient cooks through the registry from a `set_text_param` — the end-to-end path
    /// the panel drives. FALSIFIED if the cook ignored the text param.
    #[test]
    fn gradient_cooks_through_the_text_param() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.color_ramp.test.src2"),
            name: "motion.color_ramp.test.src2",
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
                ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]])));
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

        let mut g = Graph::new();
        let src = g.add_node("motion.color_ramp.test.src2");
        let cr = g.add_node("motion.color_ramp");
        g.set_text_param(cr, RAMP_KEY, "g1 2 0:1,0,0 1:0,0,1".to_string());
        g.connect(Edge {
            from: (src, 0),
            to: (cr, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, cr, 0.0).unwrap();
        match out[0].as_stream().get("tint").unwrap() {
            Column::Vec4(v) => {
                assert!(v[0][0] > 0.95 && v[0][2] < 0.05, "first red: {:?}", v[0]);
                assert!(v[1][2] > 0.95 && v[1][0] < 0.05, "last blue: {:?}", v[1]);
            }
            _ => panic!("tint"),
        }
    }
}
