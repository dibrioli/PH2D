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
//! built-in ramp (Rainbow / Heat / Ice / Grayscale); `Custom` reads a **multi-stop
//! gradient** authored in a TEXT param (doc 32/85 — the `ParamWidget::Gradient` editor),
//! serialized with `ph2d_color::serialize_gradient` / read with `parse_gradient`. Until
//! doc 85 this was two fixed stops behind six raw `0..1` sliders (`a_r`…`b_b`); the
//! gradient string replaces them, so an artist drags coloured stops instead of decoding
//! channels. The ramp is evaluated in linear RGB — the same space the `tint` column and
//! the compositor use — so no colour conversion here. `Effect::Pure`.

use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop, parse_gradient};
use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, LutSpec};
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
/// The custom multi-stop gradient (read from the [`RAMP_KEY`] text param).
const PRESET_CUSTOM: i64 = 4;

/// The text-param key for the Custom multi-stop gradient (doc 32/85), read via
/// [`EvalCtx::text_param`] and sampled by the GPU LUTs ([`LUTS`]). A `ParamWidget::Gradient`
/// hint names it; the panel edits it as a draggable, swatch-per-stop gradient bar.
const RAMP_KEY: &str = "ramp";

/// The Custom gradient's LUT resolution (doc 85, mirror of `value.curve` A1-gpu):
/// samples of the authored ramp over `t ∈ [0,1]`, one table per colour channel.
/// 256 keeps a smooth gradient within a few thousandths of the CPU `eval`.
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
    params: &[
        // 0 Rainbow · 1 Heat · 2 Ice · 3 Grayscale · 4 Custom (the gradient text param).
        ParamSpec {
            name: "preset",
            default: 0.0,
        },
        // 0 Linear · 1 Ease (smoothstep) — governs the PRESETS. A Custom gradient
        // carries its own interp inside the [`RAMP_KEY`] string (the LUT fill can only
        // read the string, doc 85), so this param is inert while `preset == Custom`.
        ParamSpec {
            name: "interp",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

fn rgb(r: f32, g: f32, b: f32) -> [f32; 4] {
    [r, g, b, 1.0]
}

/// Build a PRESET ramp (`interp` Linear or Ease). Custom is NOT built here — it is read
/// from the [`RAMP_KEY`] text param in [`eval`] (and baked into the LUTs on the GPU).
fn preset_ramp(preset: i64, interp: RampInterp) -> ColorRamp {
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
        // Any unknown preset also lands on grayscale (a safe neutral).
        _ => vec![rgb(0.0, 0.0, 0.0), rgb(1.0, 1.0, 1.0)],
    };
    let n = stops.len().max(2);
    let ramp_stops: Vec<RampStop> = stops
        .iter()
        .enumerate()
        .map(|(i, c)| RampStop::new(i as f32 / (n as f32 - 1.0), *c))
        .collect();
    ColorRamp::new(ramp_stops, RampColorMode::Rgb, interp)
}

/// The ramp for a node's `preset` + `interp` + Custom gradient string. Custom parses the
/// text param (empty/malformed → `ColorRamp::default()`, a black→white ramp, the safe
/// drop); a preset is [`preset_ramp`].
fn ramp_for(preset: i64, interp: RampInterp, custom: Option<&str>) -> ColorRamp {
    if preset == PRESET_CUSTOM {
        custom.and_then(parse_gradient).unwrap_or_default()
    } else {
        preset_ramp(preset, interp)
    }
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
/// **Custom rides the LUT channel (doc 85).** A multi-stop gradient is not a fixed
/// set of params, so the Custom branch samples three scalar LUTs (`cr_grad_r/g/b`,
/// [`LUTS`]) the sequencer bakes from the `ramp` text param via
/// [`ColorRamp::bake_into`] — the exact device analog of `value.curve`'s curve LUT.
/// The presets stay inline (their stops are constants); the `ease` flag applies to
/// the presets only, because a Custom gradient's interp is baked INTO its LUT (from
/// the string, the one place the fill can read it).
///
/// **HR-5:** the preset interpolation is this crate's `lerp` (`a + (b − a)·t`), NOT
/// WGSL's `mix`; likewise `smoothstep` is the node's own polynomial. The stop
/// positions are recomputed as `k/(n − 1)` rather than derived from `t·(n − 1)`.
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
        let cr_preset = i32(cr_round(params.preset));\n\
        if (cr_preset == 4) {\n\
        \x20   // Custom: the multi-stop gradient, baked to per-channel LUTs.\n\
        \x20   write_tint(i, vec4<f32>(\n\
        \x20       cr_grad_r_sample(cr_t),\n\
        \x20       cr_grad_g_sample(cr_t),\n\
        \x20       cr_grad_b_sample(cr_t),\n\
        \x20       1.0));\n\
        } else {\n\
        \x20   write_tint(i, cr_eval(cr_preset, i32(cr_round(params.interp)) != 0, cr_t));\n\
        }\n",
    wgsl_lib: "\
        fn cr_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn cr_stop_count(preset: i32) -> i32 {\n\
            if (preset == 0) { return 7; }\n\
            if (preset == 1) { return 5; }\n\
            if (preset == 2) { return 3; }\n\
            return 2;\n\
        }\n\
        fn cr_stop(preset: i32, k: i32) -> vec4<f32> {\n\
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
            // Grayscale (preset 3).\n\
            if (k == 0) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }\n\
            return vec4<f32>(1.0, 1.0, 1.0, 1.0);\n\
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
        fn cr_eval(preset: i32, ease: bool, t: f32) -> vec4<f32> {\n\
            let n = cr_stop_count(preset);\n\
            let first = cr_stop(preset, 0);\n\
            if (n == 1 || t <= cr_pos(0, n)) { return first; }\n\
            let last = cr_stop(preset, n - 1);\n\
            if (t >= cr_pos(n - 1, n)) { return last; }\n\
            // `partition_point(|s| s.pos <= t) - 1`: the last stop at or before t.\n\
            var idx = 0;\n\
            for (var k = 1; k < n; k = k + 1) {\n\
                if (cr_pos(k, n) <= t) { idx = k; }\n\
            }\n\
            let ca = cr_stop(preset, idx);\n\
            let cb = cr_stop(preset, idx + 1);\n\
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
    params: &["preset", "interp"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// The Custom gradient's per-channel LUTs (doc 85): three scalar tables the
/// sequencer bakes from the `ramp` text param, sampled by the WGSL Custom branch
/// as `cr_grad_r_sample(t)` etc. A colour LUT is just three scalar LUTs, so this
/// reuses the existing scalar LUT channel three times — **zero foundational GPU
/// change** (the exact sibling of `value.curve`'s single LUT).
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

/// Sample ONE channel of the authored Custom gradient into `out` at `t = k/(n−1)`
/// — the node-side half of the LUT channel (`ph2d-nodegraph` stays colour-library
/// agnostic; the sampling lives here). An unset/malformed string is the default
/// black→white ramp, matching the CPU `eval`'s fallback.
fn fill_grad_channel(text: &str, out: &mut [f32], channel: usize) {
    let ramp = parse_gradient(text).unwrap_or_default();
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
        let preset = ctx.param("preset").round() as i64;
        let interp = if ctx.param("interp").round() as i64 == 0 {
            RampInterp::Linear
        } else {
            RampInterp::Ease
        };
        let custom = ctx.text_param(RAMP_KEY).map(str::to_owned);
        let ramp = ramp_for(preset, interp, custom.as_deref());
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
    // The Custom multi-stop gradient — a TEXT param (`RAMP_KEY`), not a `ParamSpec`:
    // doc 85's gradient editor (draggable stops + per-stop OKLCH swatch), the colour
    // sibling of `value.curve`'s Curve editor. Inert unless `preset == Custom`.
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
    /// **A length-1 `t` field is a BROADCAST** — the value convention's `0/1/n`
    /// ladder (`motion.look_at::target_at` is the canon). This node used to
    /// take the `_` arm for it: element 0 got the value and every other element
    /// got `t = 0`, so a `value.lfo` driving the ramp coloured exactly one
    /// spark (found porting the `t` path to the GPU, ADR-0136).
    #[test]
    fn a_length_one_t_field_broadcasts_to_every_element() {
        let ramp = super::preset_ramp(3, super::RampInterp::Linear); // grayscale
        let tinted = super::colorize(5, &ramp, &[0.75]);
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

    use super::*;

    /// Grayscale by normalised index: the first element is black, the last is white, and
    /// the middle is mid-grey. FALSIFIED if the ramp were a single solid colour.
    #[test]
    fn grayscale_spreads_black_to_white_by_index() {
        let ramp = preset_ramp(PRESET_GRAYSCALE, RampInterp::Linear);
        let c = colorize(5, &ramp, &[]);
        assert!(c[0][0] < 0.05, "first is black: {:?}", c[0]);
        assert!(c[4][0] > 0.95, "last is white: {:?}", c[4]);
        assert!((c[2][0] - 0.5).abs() < 0.1, "middle is grey: {:?}", c[2]);
    }

    /// The `t` value field overrides the index: two elements both fed `t=1` are both the
    /// ramp's end colour (white), regardless of their index.
    #[test]
    fn the_t_field_overrides_the_index() {
        let ramp = preset_ramp(PRESET_GRAYSCALE, RampInterp::Linear);
        let c = colorize(2, &ramp, &[1.0, 1.0]);
        assert!(c[0][0] > 0.95 && c[1][0] > 0.95, "both white: {c:?}");
    }

    /// The rainbow preset actually spans hues: across the set the colours are not all
    /// equal (the red channel alone varies a lot).
    #[test]
    fn rainbow_spans_many_colours() {
        let ramp = preset_ramp(PRESET_RAINBOW, RampInterp::Linear);
        let c = colorize(12, &ramp, &[]);
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for col in &c {
            lo = lo.min(col[2]); // blue channel sweeps 0→1→0 across the wheel
            hi = hi.max(col[2]);
        }
        assert!(hi - lo > 0.8, "the rainbow spans (blue {lo}..{hi})");
    }

    /// **Custom reads the multi-stop gradient text param** (doc 85). A red→green→blue
    /// gradient laid across the set colours the first element red, the middle green, the
    /// last blue — the multi-stop capability the six raw sliders never had. FALSIFIED if
    /// Custom ignored the string (it would fall back to black→white).
    #[test]
    fn custom_reads_the_multi_stop_gradient_string() {
        let grad = "g1 2 0:1,0,0 0.5:0,1,0 1:0,0,1";
        let ramp = ramp_for(PRESET_CUSTOM, RampInterp::Linear, Some(grad));
        let c = colorize(3, &ramp, &[]);
        assert!(c[0][0] > 0.95 && c[0][1] < 0.05, "first red: {:?}", c[0]);
        assert!(c[1][1] > 0.95 && c[1][0] < 0.05, "middle green: {:?}", c[1]);
        assert!(c[2][2] > 0.95 && c[2][0] < 0.05, "last blue: {:?}", c[2]);
    }

    /// Custom with an unset / malformed string falls back to the default black→white
    /// ramp — never a half-built gradient, never a crash.
    #[test]
    fn custom_falls_back_to_black_to_white() {
        let ramp = ramp_for(PRESET_CUSTOM, RampInterp::Linear, None);
        let c = colorize(2, &ramp, &[]);
        assert!(c[0][0] < 0.05, "first black: {:?}", c[0]);
        assert!(c[1][0] > 0.95, "last white: {:?}", c[1]);
    }

    /// **The GPU LUT fill mirrors the CPU `eval`** (doc 85, the device half). Baking the
    /// red channel of a red→green→blue gradient gives red at t=0 and zero red at t=1 —
    /// the same colour the CPU `colorize` paints. The malformed string bakes the default
    /// ramp (matching the CPU fallback), so the two paths agree on "nothing authored".
    #[test]
    fn the_lut_fill_samples_each_channel_and_falls_back() {
        let grad = "g1 2 0:1,0,0 0.5:0,1,0 1:0,0,1";
        let mut r = [0.0f32; 256];
        fill_grad_r(grad, &mut r);
        assert!(r[0] > 0.95, "red LUT starts at 1.0: {}", r[0]);
        assert!(r[255] < 0.05, "red LUT ends at 0.0: {}", r[255]);
        // Parity with the CPU eval at the endpoints.
        let ramp = parse_gradient(grad).unwrap();
        assert!((r[0] - ramp.eval(0.0)[0]).abs() < 1e-6, "LUT[0] == eval(0)");
        assert!(
            (r[255] - ramp.eval(1.0)[0]).abs() < 1e-6,
            "LUT[255] == eval(1)"
        );
        // Malformed → default black→white: red channel is 0 at t=0, 1 at t=1.
        let mut bad = [9.0f32; 256];
        fill_grad_r("nonsense", &mut bad);
        assert!(bad[0] < 0.05 && bad[255] > 0.95, "fallback ramp baked");
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

    /// Custom cooks through the registry from a `set_text_param` gradient — the
    /// end-to-end path the panel drives. FALSIFIED if the cook ignored the text param.
    #[test]
    fn custom_gradient_cooks_through_the_text_param() {
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
        g.set_param(cr, "preset", PRESET_CUSTOM as f32);
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
