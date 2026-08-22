//! `motion.drive` — the value-domain CONSUMER: route a **value** field onto a
//! transform channel (Motion Nodes M2, the value domain — doc 12). This is the
//! single write-side node that replaces every behaviour that used to bundle its
//! own value math: instead of `motion.step` computing a count AND pushing X, you
//! wire `pulse.counter → motion.drive(channel = X)` — and the same value can fan
//! out to *several* drives (one count → X and Rotation at once), which no bundled
//! node can. It is the Cavalry "connect this value to that attribute" made a
//! first-class node.
//!
//! **The value type** is the continuous per-instance field `(Instances, Scalar,
//! Frame)` on the `v` column — the continuous dual of the pulse (doc 12).
//!
//! **The one broadcast rule (the load-bearing decision, doc 12):** a value field
//! of length 1 is HELD (broadcast) across every instance; a length-N field is
//! applied element-wise; anything else is a mismatch. This is TouchDesigner's
//! "held constant" / Houdini's "detail→point", restricted to `1→N` only so the
//! strict substrate never silently fits a 3-field to a 7-stream. It lives in
//! `channel::value_at`, and is what lets a single global LFO/counter drive many
//! instances without a scalar-vs-field node explosion (the reference
//! convergence — TD/Houdini/vvvv/Faust: a constant is the degenerate field).
//!
//! Params: `channel` (X/Y/Rotation/Size), `scale` (multiplies the value before
//! it hits the channel — the "count · step" that used to live in `motion.step`),
//! and `mode` (Add / Set / Multiply against the existing channel). Falloff-masked
//! like every behaviour. `Pure` (no clock, no state — a straight combinator).

#![forbid(unsafe_code)]

use channel::CH_OPACITY;
use ph2d_node_registry::{NodeRegistry, ParamChannelRange, RegistryError};
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod channel;
/// A aritmética que junta o valor conduzido ao canal — os sete modos.
mod combine;
pub use channel::DRIVE_COL_KEY;
use channel::{
    CH_CUSTOM, CH_FALLOFF, CH_HUE, CH_SAT, CH_SIZE_X, CH_SIZE_Y, CH_VAL, Combine, drive_channel,
    drive_named,
};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of `ph2d_node_pulse_counter::VALUE`; kept local so this stays a leaf
/// drop-crate — the shared vocabulary is the port, not a shared symbol).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value stream's column (the canonical `value`-domain column).
const VALUE_COL: &str = "v";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.drive"),
    name: "motion.drive",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "value",
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
        // 0 X · 1 Y · 2 Rotation · 3 Size · 4 Opacity — the shared channel vocabulary.
        ParamSpec {
            name: "channel",
            default: 0.0,
        },
        // Multiplies the value before it hits the channel (the ex-`step`).
        ParamSpec {
            name: "scale",
            default: 1.0,
        },
        // 0 Add · 1 Set · 2 Multiply — how the value combines with the channel.
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The params every variant declares, in one order — the uniform layout is
/// per-variant, and keeping them identical means a reader never has to ask which
/// variant a `params.scale` belongs to.
const DRIVE_PARAMS: &[&str] = &["channel", "scale", "mode"];

/// The shared prologue: resolve the mode, the scaled value and the falloff mask.
/// Every variant pastes this and then writes ITS column.
///
/// `drive_round` is round-half-away-from-zero to match Rust's `f32::round` —
/// `mode` picks a BRANCH ([[feedback_cpu_gpu_rounding_conventions_diverge]]).
/// The falloff clamp MIRRORS the CPU's; no node writes a falloff outside `[0,1]`
/// today, so it is defensive on both sides rather than load-bearing.
const DRIVE_LIB: &str = "\
    fn drive_round(x: f32) -> f32 {\n\
        // Rust f32::round = half away from zero (WGSL round is half-even).\n\
        return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
    }\n\
    fn drive_combine(cur: f32, v: f32, mode: i32) -> f32 {\n\
        if (mode == 1) { return v; }\n\
        if (mode == 2) { return cur * v; }\n\
        // Apendados (folha 06 linha 40) -- o gemeo de `Combine::apply`.\n\
        if (mode == 3) { return cur - v; }\n\
        if (mode == 4) {\n\
            // A MESMA guarda da CPU, com o MESMO limiar: um `inf` num canal de\n\
            // transform envenena a posicao e todo NaN a jusante vem sem endereco.\n\
            if (abs(v) < 1e-9) { return 0.0; }\n\
            return cur / v;\n\
        }\n\
        if (mode == 5) { return min(cur, v); }\n\
        if (mode == 6) { return max(cur, v); }\n\
        return cur + v;\n\
    }\n";

/// `falloff` and the value port, bound identically by every variant.
macro_rules! drive_common {
    () => {
        [
            ColumnBinding {
                column: "falloff",
                dim: Dim::Scalar,
                access: ColumnAccess::Read,
                // Absent falloff = full effect, the CPU's `falloff_at` fallback.
                identity: [1.0, 0.0, 0.0, 0.0],
                port: 0,
            },
            ColumnBinding {
                column: VALUE_COL,
                dim: Dim::Scalar,
                access: ColumnAccess::ReadBroadcast,
                // Absent value = 0.0, the `0 =>` arm of `value_at`.
                identity: [0.0; 4],
                port: 1,
            },
        ]
    };
}

/// **X / Y** — writes one component of `P`.
const DRIVE_P: GpuKernel = GpuKernel {
    wgsl: "\
        let dr_comp = i32(drive_round(params.channel));\n\
        let dr_mode = i32(drive_round(params.mode));\n\
        let dr_p = read_in_P(i);\n\
        var dr_cur = dr_p.x;\n\
        if (dr_comp == 1) { dr_cur = dr_p.y; }\n\
        let dr_v = read_value_v(i) * params.scale;\n\
        let dr_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        let dr_out = dr_cur + (drive_combine(dr_cur, dr_v, dr_mode) - dr_cur) * dr_f;\n\
        var dr_next = dr_p;\n\
        if (dr_comp == 1) { dr_next.y = dr_out; } else { dr_next.x = dr_out; }\n\
        write_P(i, dr_next);\n",
    wgsl_lib: DRIVE_LIB,
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        drive_common!()[0],
        drive_common!()[1],
    ],
    params: DRIVE_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Rotation** — writes `rot`, in degrees like the CPU's.
const DRIVE_ROT: GpuKernel = GpuKernel {
    wgsl: "\
        let dr_mode = i32(drive_round(params.mode));\n\
        let dr_cur = read_in_rot(i);\n\
        let dr_v = read_value_v(i) * params.scale;\n\
        let dr_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        write_rot(i, dr_cur + (drive_combine(dr_cur, dr_v, dr_mode) - dr_cur) * dr_f);\n",
    wgsl_lib: DRIVE_LIB,
    bindings: &[
        ColumnBinding {
            column: "rot",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        drive_common!()[0],
        drive_common!()[1],
    ],
    params: DRIVE_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Size** — drives BOTH components uniformly, from the unit identity.
const DRIVE_SIZE: GpuKernel = GpuKernel {
    wgsl: "\
        let dr_mode = i32(drive_round(params.mode));\n\
        let dr_s = read_in_size(i);\n\
        let dr_v = read_value_v(i) * params.scale;\n\
        let dr_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        let dr_x = dr_s.x + (drive_combine(dr_s.x, dr_v, dr_mode) - dr_s.x) * dr_f;\n\
        let dr_y = dr_s.y + (drive_combine(dr_s.y, dr_v, dr_mode) - dr_s.y) * dr_f;\n\
        write_size(i, vec2<f32>(dr_x, dr_y));\n",
    wgsl_lib: DRIVE_LIB,
    bindings: &[
        ColumnBinding {
            column: "size",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            // An element with no size starts UNIT, not zero (`base_vec2`).
            identity: [1.0, 1.0, 0.0, 0.0],
            port: 0,
        },
        drive_common!()[0],
        drive_common!()[1],
    ],
    params: DRIVE_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Size X / Size Y** — writes ONE component of `size`, from the unit identity.
///
/// ⚠️ **Um kernel para os dois eixos, ramificando em `params.channel`** — o molde exacto
/// do [`DRIVE_P`], e pelo mesmo motivo: os dois escrevem a MESMA coluna com a MESMA
/// binding, então dois kernels seriam duas cópias de uma lei que não difere.
const DRIVE_SIZE_AXIS: GpuKernel = GpuKernel {
    wgsl: "\
        let dr_comp = i32(drive_round(params.channel)) - 10;\n\
        let dr_mode = i32(drive_round(params.mode));\n\
        let dr_s = read_in_size(i);\n\
        var dr_cur = dr_s.x;\n\
        if (dr_comp == 1) { dr_cur = dr_s.y; }\n\
        let dr_v = read_value_v(i) * params.scale;\n\
        let dr_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        let dr_out = dr_cur + (drive_combine(dr_cur, dr_v, dr_mode) - dr_cur) * dr_f;\n\
        var dr_next = dr_s;\n\
        if (dr_comp == 1) { dr_next.y = dr_out; } else { dr_next.x = dr_out; }\n\
        write_size(i, dr_next);\n",
    wgsl_lib: DRIVE_LIB,
    bindings: &[
        ColumnBinding {
            column: "size",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            // A MESMA identidade unitaria do `DRIVE_SIZE`: uma peca sem tamanho parte de 1.
            identity: [1.0, 1.0, 0.0, 0.0],
            port: 0,
        },
        drive_common!()[0],
        drive_common!()[1],
    ],
    params: DRIVE_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Opacity** — the ALPHA of `tint`, clamped to `[0,1]`. An element with no
/// tint starts from opaque white, so driving the opacity of an uncoloured stream
/// does what it says instead of silently nothing (doc 51).
const DRIVE_TINT: GpuKernel = GpuKernel {
    wgsl: "\
        let dr_mode = i32(drive_round(params.mode));\n\
        let dr_t = read_in_tint(i);\n\
        let dr_v = read_value_v(i) * params.scale;\n\
        let dr_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        let dr_a = dr_t.w + (drive_combine(dr_t.w, dr_v, dr_mode) - dr_t.w) * dr_f;\n\
        write_tint(i, vec4<f32>(dr_t.x, dr_t.y, dr_t.z, clamp(dr_a, 0.0, 1.0)));\n",
    wgsl_lib: DRIVE_LIB,
    bindings: &[
        ColumnBinding {
            column: "tint",
            dim: Dim::Vec4,
            access: ColumnAccess::ReadWrite,
            identity: [1.0, 1.0, 1.0, 1.0],
            port: 0,
        },
        drive_common!()[0],
        drive_common!()[1],
    ],
    params: DRIVE_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// O prólogo mais a ida-e-volta HSV, **verbatim** o que `ph2d_color::rgb_to_hsv` /
/// `hsv_to_rgba` computam em Rust.
///
/// ⚠️ **É a segunda expressão da lei, e é inevitável** — o dispositivo não chama Rust. O que
/// a mantém honesta é o gate de paridade CPU×GPU deste nó, não a disciplina de quem edita.
/// (A `motion.luminance` carrega a metade da IDA pelo mesmo motivo; extrair as duas para um
/// `wgsl_lib` compartilhado é wave própria — o substrato hoje só tem lib POR KERNEL, e a
/// convenção da biblioteca é a mesma que já copia o `falloff_at` nove vezes.)
const DRIVE_LIB_HSV: &str = concat!(
    "\
    fn drive_rgb_to_hsv(c: vec4<f32>) -> vec3<f32> {\n\
        let mx = max(max(c.x, c.y), c.z);\n\
        let mn = min(min(c.x, c.y), c.z);\n\
        let d = mx - mn;\n\
        var h = 0.0;\n\
        if (d > 0.0) {\n\
            if (mx == c.x) { h = (c.y - c.z) / d + select(0.0, 6.0, c.y < c.z); }\n\
            else if (mx == c.y) { h = (c.z - c.x) / d + 2.0; }\n\
            else { h = (c.x - c.y) / d + 4.0; }\n\
            h = h / 6.0;\n\
        }\n\
        var s = 0.0;\n\
        if (mx > 0.0) { s = d / mx; }\n\
        return vec3<f32>(h, s, mx);\n\
    }\n\
    fn drive_hsv_to_rgb(h: f32, s: f32, v: f32) -> vec3<f32> {\n\
        // `rem_euclid(1.0)` do Rust: o matiz envolve AQUI, como na porta de Rust.\n\
        let hw = (h - floor(h)) * 6.0;\n\
        let i = floor(hw);\n\
        let f = hw - i;\n\
        let p = v * (1.0 - s);\n\
        let q = v * (1.0 - s * f);\n\
        let t = v * (1.0 - s * (1.0 - f));\n\
        let k = i32(i) % 6;\n\
        if (k == 0) { return vec3<f32>(v, t, p); }\n\
        if (k == 1) { return vec3<f32>(q, v, p); }\n\
        if (k == 2) { return vec3<f32>(p, v, t); }\n\
        if (k == 3) { return vec3<f32>(p, q, v); }\n\
        if (k == 4) { return vec3<f32>(t, p, v); }\n\
        return vec3<f32>(v, p, q);\n\
    }\n",
    "\
    fn drive_round(x: f32) -> f32 {\n\
        return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
    }\n\
    fn drive_combine(cur: f32, v: f32, mode: i32) -> f32 {\n\
        if (mode == 1) { return v; }\n\
        if (mode == 2) { return cur * v; }\n\
        return cur + v;\n\
    }\n"
);

/// **A COR sobre a cor que já está lá** ([`CH_HUE`]) — matiz, saturação e valor do `tint`.
///
/// ⚠️ **UMA variante para os TRÊS canais, e a régua é a BINDING, não o gosto:** uma variante
/// existe quando a lista de colunas ligadas difere (o módulo gerado só define `write_<col>`
/// para coluna BOUND), e estes três leem e escrevem exatamente o que o [`DRIVE_TINT`] lê e
/// escreve. Três variantes seriam três cópias do mesmo par de bindings esperando divergir; o
/// `channel` é uniforme no dispatch inteiro, então o ramo não diverge entre invocações.
const DRIVE_HSV: GpuKernel = GpuKernel {
    wgsl: "\
        let dr_mode = i32(drive_round(params.mode));\n\
        let dr_ch = i32(drive_round(params.channel));\n\
        let dr_t = read_in_tint(i);\n\
        let dr_v = read_value_v(i) * params.scale;\n\
        let dr_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        let dr_hsv = drive_rgb_to_hsv(dr_t);\n\
        var dr_cur = dr_hsv.z;\n\
        if (dr_ch == 6) { dr_cur = dr_hsv.x; }\n\
        else if (dr_ch == 7) { dr_cur = dr_hsv.y; }\n\
        let dr_next = dr_cur + (drive_combine(dr_cur, dr_v, dr_mode) - dr_cur) * dr_f;\n\
        var dr_h = dr_hsv.x;\n\
        var dr_s = dr_hsv.y;\n\
        var dr_val = dr_hsv.z;\n\
        if (dr_ch == 6) { dr_h = dr_next; }\n\
        else if (dr_ch == 7) { dr_s = clamp(dr_next, 0.0, 1.0); }\n\
        else { dr_val = max(dr_next, 0.0); }\n\
        let dr_rgb = drive_hsv_to_rgb(dr_h, dr_s, dr_val);\n\
        write_tint(i, vec4<f32>(dr_rgb.x, dr_rgb.y, dr_rgb.z, dr_t.w));\n",
    wgsl_lib: DRIVE_LIB_HSV,
    bindings: DRIVE_TINT.bindings,
    params: DRIVE_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **The mask as a target** ([`CH_FALLOFF`]). The scalar template, with ONE difference that
/// carries the whole decision: every other variant binds `falloff` as a `Read` to blend with,
/// so this one — whose target IS `falloff` — binds it once as `ReadWrite` and simply has no
/// common read. The self-mask is not refused by taste; it is **inexpressible** in the binding
/// list, which is the kind of refusal that survives the next person.
///
/// No `dr_f` blend and no clamp, mirroring the CPU arm line for line.
const DRIVE_FALLOFF: GpuKernel = GpuKernel {
    wgsl: "\
        let dr_mode = i32(drive_round(params.mode));\n\
        let dr_cur = read_in_falloff(i);\n\
        let dr_v = read_value_v(i) * params.scale;\n\
        write_falloff(i, drive_combine(dr_cur, dr_v, dr_mode));\n",
    wgsl_lib: DRIVE_LIB,
    bindings: &[
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            // Absent falloff = 1.0 — every reader's fallback, and the CPU's `base_scalar`
            // identity. A writer that started from 0 would disagree with the whole library
            // about what "no mask" means.
            identity: [1.0, 0.0, 0.0, 0.0],
            port: 0,
        },
        drive_common!()[1],
    ],
    params: DRIVE_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// GPU compute kernel (ADR-0126) — the value domain's WRITE side on the device,
/// and the node that named [`GpuKernel::variant_by_param`].
///
/// `drive_channel` writes a DIFFERENT column per channel — `P` for X/Y, `rot`
/// for Rotation, `size` for Size, `tint` for Opacity — and materialises the
/// target from its identity when the stream lacks it. One static shape could not
/// express that: binding all four would emit columns **the CPU's output does not
/// carry** (a different stream SHAPE, not an ε), and binding one meant claiming
/// only the two channels that write `P`. So the node ships four variants and the
/// engine picks by `channel` — the SAME mapping `channel_column` uses, including
/// its `_ => size` catch-all for an out-of-range value.
///
/// The value port BROADCASTS: length 1 is one number held across the field (the
/// `1 => vals[0]` arm of `value_at`), length N is per-element.
const GPU_KERNEL: GpuKernel = GpuKernel {
    // The top-level shape IS the X/Y variant, so a caller that never resolves
    // still sees a real kernel rather than the empty (pass-through) one.
    wgsl: DRIVE_P.wgsl,
    wgsl_lib: DRIVE_P.wgsl_lib,
    bindings: DRIVE_P.bindings,
    params: DRIVE_PARAMS,
    count_law: None,
    // ⚠️ **O CUSTOM RECUSA o device** — ver [`CH_CUSTOM`]: uma `ColumnBinding`
    // carrega o nome como `&'static str`, e o nome que o artista digita só existe
    // em tempo de cook. O sequenciador recua para o `eval` da CPU, que é a porta
    // que a `Median` do `value.reduce` já usa.
    applicable: Some(|param| param("channel").round() as i32 != CH_CUSTOM),
    variant_by_param: Some(|param| {
        // The same rounding and the same mapping as `channel_column`.
        match param("channel").round() as i32 {
            2 => &DRIVE_ROT,
            CH_OPACITY => &DRIVE_TINT,
            CH_FALLOFF => &DRIVE_FALLOFF,
            CH_HUE | CH_SAT | CH_VAL => &DRIVE_HSV,
            0 | 1 => &DRIVE_P,
            CH_SIZE_X | CH_SIZE_Y => &DRIVE_SIZE_AXIS,
            _ => &DRIVE_SIZE,
        }
    }),
};

struct MotionDrive;

impl NodeOp for MotionDrive {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let channel = ctx.param("channel").round() as i32;
        let scale = ctx.param("scale");
        let mode = Combine::from_param(ctx.param("mode"));
        let vals: Vec<f32> = match ctx.input(1).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        // ⚠️ O canal CUSTOM pega o alvo do text param; os nove do enum pegam-no do
        // `channel_column`. Uma porta por pergunta, e a LEI (blend · broadcast ·
        // falloff) é a mesma nas duas.
        let out = if channel == CH_CUSTOM {
            let name = ctx.text_param(DRIVE_COL_KEY).unwrap_or("").to_string();
            drive_named(ctx.input(0), &name, &vals, scale, mode)
        } else {
            drive_channel(ctx.input(0), channel, &vals, scale, mode)
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionDrive))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Drive",
            // Transform blue: it writes a transform channel — a visible behaviour.
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    // ⚠️ A linha do nome só aparece no canal que a LÊ — um campo de texto
    // visível sob "Rotation" seria o controle morto que esta casa recusa.
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_channel_range(MANIFEST.id, PARAM_CHANNEL_RANGE);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_hard_min(MANIFEST.id, PARAM_HARD_MIN);
    Ok(())
}

use ph2d_node_registry::{ParamGate, ParamUiHint, ParamWidget};

/// A linha do NOME pertence ao canal Custom e a mais nenhum.
static PARAM_GATES: &[ParamGate] = &[ParamGate {
    param: DRIVE_COL_KEY,
    when: "channel",
    values: &[CH_CUSTOM],
}];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "channel",
        label: "Channel",
        min: 0.0,
        max: 11.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            // ⚠️ **Apendados**, nunca inseridos: o índice é o que o grafo guarda, então uma
            // ordem "mais bonita" (as três cores ao lado do Opacity) trocaria o canal de todo
            // documento já autorado — em silêncio, porque o param é um `f32` sem versão.
            labels: &[
                "X",
                "Y",
                "Rotation",
                "Size",
                "Opacity",
                "Falloff",
                "Hue",
                "Saturation",
                "Value",
                "Custom…",
                "Size X",
                "Size Y",
            ],
        },
    },
    // ⚠️ **O NOME da coluna é um TEXT param**, não um `ParamSpec`: o manifesto é
    // f32-only por contrato congelado (ADR-0039), e o canal de texto do `Graph` é
    // o padrão canônico para param não-f32 — o mesmo assento que a fórmula do
    // `motion.expression` e a tabela do `value.pattern` ocupam.
    ParamUiHint {
        param: DRIVE_COL_KEY,
        label: "Column",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Text,
    },
    ParamUiHint {
        param: "scale",
        label: "Scale",
        min: -4.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 6.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            // ⚠️ Os quatro últimos são APENDADOS (folha 06 linha 40): `0..2` ficam
            // onde estavam, então todo documento já autorado lê o mesmo modo.
            labels: &["Add", "Set", "Multiply", "Subtract", "Divide", "Min", "Max"],
        },
    },
];
/// **A faixa que estas magnitudes querem quando o canal é ANGULAR** — graus, não
/// unidades de mundo. Uma volta para cada lado, discada em graus inteiros.
///
/// ⚠️ Ela mora AQUI e não numa tabela do shell porque a tabela apodreceu: medida,
/// ela cobria três dos seis nós que precisavam dela, e cada um dos três ausentes
/// esperava o próprio report do artista.
const TURN: f32 = 360.0;
static PARAM_CHANNEL_RANGE: &[ParamChannelRange] = &[ParamChannelRange {
    param: "scale",
    min: -TURN,
    max: TURN,
    step: 1.0,
}];

/// **O teto DIGITÁVEL do `scale`, e o recurso é a PRECISÃO DE REPRESENTAÇÃO.**
///
/// ⚠️ O `scale` é uma multiplicação PURA — nem a CPU nem o WGSL o clampam —, então
/// o teto do slider não era um teto do kernel: medido, o canal Rotation honra
/// `1e7` graus com erro `0.000e0`. Era um número de UI sem recurso nenhum atrás,
/// e a §0 do CLAUDE.md chama isso pelo nome.
///
/// O limite real é o `f32`: **`2^24 = 16 777 216` é o primeiro número em que
/// `x + 1.0 == x`** (medido, não escolhido). Acima dele um passo de UM grau não
/// move o número — o controle deixa de controlar, que é onde *o disfuncional
/// começa*. Abaixo, tudo é honrado exatamente.
const F32_UNIT_STEP_CEILING: f32 = 16_777_216.0;
static PARAM_HARD_MAX: &[ph2d_node_registry::ParamHardMax] = &[ph2d_node_registry::ParamHardMax {
    param: "scale",
    max: F32_UNIT_STEP_CEILING,
}];
/// O piso, espelhado: um `scale` negativo INVERTE o drive, e o slider já o oferece.
static PARAM_HARD_MIN: &[ph2d_node_registry::ParamHardMin] = &[ph2d_node_registry::ParamHardMin {
    param: "scale",
    min: -F32_UNIT_STEP_CEILING,
}];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "custom_tests.rs"]
mod custom_tests;
