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

/// The multiplicative `falloff` weight for instance `i` (absent → `1.0`).
///
/// ⚠️ The four lines below are the tenth copy of this helper in the node library
/// (`motion.tint`, `motion.move`, `motion.rotate`, the three `force.*` accumulators,
/// `motion.noise`/`stagger`/`step`/`wiggle`'s channel modules …), every one of them
/// identical. Collapsing them into one door is a real improvement and a real WAVE —
/// it touches nine crates and none of them is a colour node. Copying it here follows
/// the convention the library already chose; the census is in the handoff so the
/// extraction is a decision somebody makes on purpose rather than a surprise.
fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// The instance's existing colour (absent → opaque white) — the same base
/// `motion.tint` starts from, and the same `identity` the GPU binding declares.
fn existing_tint(stream: &Stream, i: usize) -> [f32; 4] {
    match stream.get("tint") {
        Some(Column::Vec4(v)) => v.get(i).copied().unwrap_or([1.0; 4]),
        _ => [1.0; 4],
    }
}

/// `lerp(existing, target, f)` per RGBA channel, in the endpoint-EXACT form
/// `existing·(1−f) + target·f`.
///
/// The exactness is the whole argument for this wave being safe: at `f = 1` the
/// first term is `existing · 0.0`, which IEEE-754 makes exactly zero for any finite
/// channel, and the second is `target · 1.0`, which is exactly `target`. So a stream
/// with no `falloff` column takes the ramp colour **bit for bit** — the substitution
/// this node has always performed — and only a stream that carries a mask sees
/// anything new. (The same form, and the same reasoning, as `motion.tint::mixed_tint`.)
fn mixed_tint(existing: [f32; 4], target: [f32; 4], f: f32) -> [f32; 4] {
    let lerp = |e: f32, t: f32| e * (1.0 - f) + t * f;
    [
        lerp(existing[0], target[0]),
        lerp(existing[1], target[1]),
        lerp(existing[2], target[2]),
        lerp(existing[3], target[3]),
    ]
}

/// Map every instance's scalar through the ramp, masked by `falloff`. `t_field` is the
/// connected value input (empty → normalised index).
///
/// **The mask is what makes the `field.*` family compose with colour** (doc 89 fam. 9,
/// the C4D effector's *Color group → Use Alpha/Strength*): a `field.box` or a
/// `field.radial_sweep` writes `falloff`, and the ramp now paints only where the field
/// reaches. Before this the node replaced `tint` unconditionally, so a masked gradient
/// was inexpressible — the mixer's `blend` is a global scalar, not a field.
fn colorize(n: usize, ramp: &ColorRamp, t_field: &[f32], input: &Stream) -> Vec<[f32; 4]> {
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
            mixed_tint(existing_tint(input, i), ramp.eval(t), falloff_at(input, i))
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
/// set of params, so the body samples four scalar LUTs (`cr_grad_r/g/b/a`, [`LUTS`]) the
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
        let cr_c = vec4<f32>(\n\
        \x20   cr_grad_r_sample(cr_t),\n\
        \x20   cr_grad_g_sample(cr_t),\n\
        \x20   cr_grad_b_sample(cr_t),\n\
        \x20   cr_grad_a_sample(cr_t));\n\
        // The mask: `lerp(existing, ramp, falloff)` in the endpoint-exact form,\n\
        // so an absent `falloff` (identity 1.0) writes the ramp colour bit for bit.\n\
        // ⚠️ Qualified by PORT: this node has two inputs, so every reader is\n\
        // `read_<port>_<col>` (`accessor_suffix`) — the same rule `read_t_v` above\n\
        // follows. Writers are never qualified; a node has one output.\n\
        let cr_e = read_in_tint(i);\n\
        let cr_f = read_in_falloff(i);\n\
        write_tint(i, vec4<f32>(\n\
        \x20   cr_e.x * (1.0 - cr_f) + cr_c.x * cr_f,\n\
        \x20   cr_e.y * (1.0 - cr_f) + cr_c.y * cr_f,\n\
        \x20   cr_e.z * (1.0 - cr_f) + cr_c.z * cr_f,\n\
        \x20   cr_e.w * (1.0 - cr_f) + cr_c.w * cr_f));\n",
    wgsl_lib: "",
    bindings: &[
        ColumnBinding {
            // ⚠️ `ReadWrite`, not `Write`: since the falloff mask this node BLENDS
            // onto whatever colour is already there, so it has to read it. An absent
            // column reads the identity — opaque white, the same base the CPU's
            // `existing_tint` starts from — and is always written.
            column: "tint",
            dim: Dim::Vec4,
            access: ColumnAccess::ReadWrite,
            identity: [1.0; 4],
            port: 0,
        },
        ColumnBinding {
            // The mask (`field.*` writes it). Identity 1.0 = *no mask*, which is what
            // makes an unmasked graph byte-identical to the day before this existed.
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
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

/// The gradient's per-channel LUTs (doc 85): four scalar tables the sequencer bakes from
/// the `ramp` text param, sampled by the WGSL as `cr_grad_r_sample(t)` etc. An RGBA LUT is
/// just four scalar LUTs, so this reuses the existing scalar LUT channel four times —
/// **zero foundational GPU change** (the exact sibling of `value.curve`'s single LUT).
///
/// **What four costs, measured:** this kernel's module declares 2 column bindings + these
/// LUTs = **6 storage buffers**, against a worst kernel in the registry (`motion.integrate`)
/// that already declares **13** — so the fourth table is not near any device budget. It is
/// measured rather than assumed because the runtime's check counts only the COLUMN bindings
/// (`codegen::storage_bindings`); see the sibling gate that pins the true count.
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
    // The FOURTH channel, and the reason it is a fourth scalar LUT rather than a
    // widened one: doc 85 already weighed `LutSpec` carrying a vec4 and rejected it
    // (*three scalar LUTs give the same thing reusing the existing channel*), so alpha
    // is that decision applied one more time — zero foundational GPU change.
    //
    // ⚠️ An OPAQUE gradient bakes 1.0 into every entry, which is exactly the literal
    // this replaced ⇒ every ramp authored before per-stop alpha existed renders
    // byte-identically. What changes is only the ramp that has something else to say.
    LutSpec {
        name: "cr_grad_a",
        text_key: RAMP_KEY,
        resolution: LUT_RESOLUTION,
        fill: fill_grad_a,
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
fn fill_grad_a(text: &str, out: &mut [f32]) {
    fill_grad_channel(text, out, 3);
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
        let tint = colorize(n, &ramp, &t_field, input);
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
#[path = "tests.rs"]
mod tests;
