#![forbid(unsafe_code)]
//! `field.remap` — the Motion **remapper** of a `falloff` mask: the downstream node
//! the whole `field.*` family DEFERS its remapping to (D1). Where a spatial field
//! (`field.box`/`radial_sweep`) shapes a raw mask by position, this one TRANSFORMS an
//! existing mask value-by-value — it reads `falloff`, rewrites `falloff` (it does NOT
//! multiply into it; a remap is not a field, it is a transfer function). It is the
//! C4D **Remapping** tab as a node: `inner_offset` (the solid-core plateau), a
//! `contour` transfer (None/Quadratic/Step/Quantize/**Curve**), an output `[min, max]`
//! range with a `multiplier`, a `clamp`, an `invert`, and a `strength` that blends
//! input→remapped.
//!
//! **Transcendental-free (HR-5).** Every transfer is `min`/`max`/`clamp`/`floor`/`round`
//! and polynomials, so the mask is bit-identical CPU↔GPU (within FMA ULPs). The GPU
//! kernel is a straight WGSL port; there is no `P` to read — a remap is position-blind,
//! it reads the mask alone.
//!
//! **The Curve contour (mode 4)** is the one transfer whose shape is not a scalar: it
//! is a `ph2d-curve` authored in a **text param** (`CURVE_KEY`). On the CPU it is
//! `eval`'d directly; on the GPU the sequencer bakes it to a LUT (A1-gpu, see [`LUTS`])
//! and the kernel samples `rm_curve_sample(t)` — so ALL five contour modes cook on the
//! device, no CPU fallback. An unset curve is the identity, so a Curve contour with
//! nothing authored is a passthrough on both.
//!
//! **The neutral (D12).** `strength = 0` is a byte-identical passthrough (the mask is
//! returned unchanged) — the "off" that never mutates the scene. A fresh node, however,
//! does something VISIBLE: it is born `contour = Quadratic` with a gentle `curvature`,
//! so dropping it on a soft field sharpens the ramp — the sign that a remap is working,
//! trivially reset to `None`/Linear.

use ph2d_curve::Curve;
use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream, par_build};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, LutSpec};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The text-param key carrying the Curve contour's shape (a `ph2d-curve`
/// serialized string, read via `EvalCtx::text_param`). NOT a `ParamSpec` — a
/// curve is not one number (the `motion.expression` precedent).
const CURVE_KEY: &str = "curve";

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("field.remap"),
    name: "field.remap",
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
            name: "inner_offset",
            default: 0.0,
        },
        ParamSpec {
            // 0 None/Linear · 1 Quadratic · 2 Step · 3 Quantize · 4 Curve (shape in
            // the `curve` text param, CPU-evaluated; A1-gpu bakes its LUT).
            name: "contour",
            default: 1.0,
        },
        ParamSpec {
            // The Quadratic bend, [-1, 1]: >0 ease-in (t²), <0 ease-out. Inert on the
            // other contour modes.
            name: "curvature",
            default: 0.5,
        },
        ParamSpec {
            // The level count for Step/Quantize (≥ 2). Inert on the other modes.
            name: "steps",
            default: 4.0,
        },
        ParamSpec {
            name: "min",
            default: 0.0,
        },
        ParamSpec {
            name: "max",
            default: 1.0,
        },
        ParamSpec {
            name: "multiplier",
            default: 1.0,
        },
        ParamSpec {
            name: "clamp",
            default: 1.0,
        },
        ParamSpec {
            name: "invert",
            default: 0.0,
        },
        ParamSpec {
            name: "strength",
            default: 1.0,
        },
        // **O DESLOCAMENTO DA CURVA** — o C4D *Spline Offset* / *Spline Animation
        // Speed* (doc 89 folha 10; o único item daquele dump com `!`). APENDADO,
        // default `0` = a curva de hoje ao bit. Ver [`shifted`].
        ParamSpec {
            name: "curve_offset",
            default: 0.0,
        },
        ParamSpec {
            // Field → binary mask: each instance survives with this probability, decided
            // by a stable hash of its index. `1` (default) keeps everyone (neutral).
            name: "probability",
            default: 1.0,
        },
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// A stable per-instance hash in `[0, 1)` from the instance index + seed — the
/// probability gate's randomness. Integer ops only (`wrapping_mul`/`xor`/`shift`), so it
/// is **bit-identical** CPU↔GPU (not just ε): `x >> 8` is `≤ 2²⁴−1` and exactly
/// representable, and `/ 2²⁴` is exact. Because it is strictly `< 1`, `probability = 1`
/// keeps EVERY instance with no special case (the gate is `hash < probability`).
fn hash01(id: u32, seed: u32) -> f32 {
    let mut x = id
        .wrapping_mul(0x9E37_79B9)
        .wrapping_add(seed.wrapping_mul(0x85EB_CA6B));
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    (x >> 8) as f32 / 16_777_216.0
}

/// Round half-away-from-zero — matching Rust's `f32::round` (the WGSL `round` is
/// half-even), so Step/Quantize land on the same level on both devices.
fn round_haz(x: f32) -> f32 {
    if x >= 0.0 {
        (x + 0.5).floor()
    } else {
        (x - 0.5).ceil()
    }
}

/// **O deslocamento da curva ao longo do X, com WRAP** — o C4D *Spline Offset*, e o
/// mecanismo do *Spline Animation Speed* (uma expressão a somar tempo a este número faz
/// a forma desfilar).
///
/// ⚠️ **`offset == 0.0` devolve `t` ANTES de qualquer aritmética, e essa linha é
/// load-bearing.** O wrap natural (`x − floor(x)`) leva `1.0` a `0.0`, e `t = 1.0` não é
/// um caso de canto aqui: é o que TODA peça a máscara cheia entrega. Sem a guarda, ligar
/// o nó (sem tocar no knob) trocaria `curve.eval(1)` por `curve.eval(0)` em metade da
/// cena. *O ponto neutro de um knob tem de ser a identidade, e o `rem_euclid` não é a
/// identidade no topo do intervalo.*
///
/// ⚠️ **A costura é da CURVA, não do deslocamento.** Com o knob fora de zero há um salto
/// onde `curve(1) ≠ curve(0)` — é o mesmo salto que qualquer marquise tem, e some
/// sozinho numa curva cujos extremos concordam. Recortar o salto seria decidir pelo
/// artista o que a curva dele diz.
fn shifted(t: f32, offset: f32) -> f32 {
    if offset == 0.0 {
        return t;
    }
    let u = t + offset;
    u - u.floor()
}

/// The **contour** transfer on `t ∈ [0,1]` — the C4D Remapping shapes, HR-5. `None`
/// (0) is the identity; `Quadratic` (1) bends by `curvature` (`>0` ease-in `t²`, `<0`
/// ease-out, `0` linear); `Step` (2) is a `steps`-level FLOOR staircase (holds, hits 0
/// and 1); `Quantize` (3) is a `steps`-level ROUND (nearest); `Curve` (4) evaluates the
/// authored `curve` at [`shifted`]`(t, curve_offset)` (an unset/`None` curve is the
/// identity — an exact passthrough).
///
/// ⚠️ O `curve_offset` só entra no modo `Curve`, e é o único modo em que ele tem
/// referente: as outras quatro formas são fórmulas, não uma tabela a deslizar. O nó não
/// esconde o controle nos outros modos porque **nenhum** dos seus params inertes é
/// escondido (o `curvature` e o `steps` também não) — uma excepção só para o mais novo
/// seria a incoerência, e gatear os quatro esconderia o editor de curva no modo default.
fn contour(
    mode: i32,
    t: f32,
    curvature: f32,
    steps: f32,
    curve: Option<&Curve>,
    curve_offset: f32,
) -> f32 {
    match mode {
        1 => {
            let ease_in = t * t;
            let ease_out = 1.0 - (1.0 - t) * (1.0 - t);
            if curvature >= 0.0 {
                t + (ease_in - t) * curvature
            } else {
                t + (ease_out - t) * (-curvature)
            }
        }
        2 => {
            let n = steps.max(2.0);
            let k = (t * n).floor().min(n - 1.0);
            k / (n - 1.0)
        }
        3 => {
            let n = steps.max(2.0);
            round_haz(t * (n - 1.0)) / (n - 1.0)
        }
        // ⚠️ **O deslocamento corre ANTES de consultar a curva, e a ordem é o bug.**
        // A primeira versão era `curve.map_or(t, |c| c.eval(shifted(t, off)))`: com
        // NADA autorado o `map_or` devolvia `t` e o `shifted` nunca corria — o knob
        // ficava morto exactamente no estado em que o nó nasce. Pior, o device fazia o
        // contrário (a LUT assa a IDENTIDADE e o WGSL amostra-a já deslocada), então os
        // dois caminhos discordavam sem nada dizer. *Uma curva ausente é a identidade,
        // e a identidade deslocada é um dente de serra — que é uma resposta, não um
        // no-op.* Achado por um smoke do Enio (2026-08-21).
        4 => {
            let u = shifted(t, curve_offset);
            curve.map_or(u, |c| c.eval(u))
        }
        _ => t, // None/Linear (0) passes through.
    }
}

/// The full remap of one mask value `t_in` — the SAME pipeline the WGSL kernel runs.
/// Invert flips the input; `inner_offset` expands the solid core (values above the
/// offset saturate to 1); the contour shapes it; the `[min, max]` range + `multiplier`
/// scale it; `clamp` bounds it to `[0,1]`; and `strength` blends input→remapped (so
/// `strength = 0` is an exact passthrough).
// The 12 args are the C4D Remapping pipeline; a struct would be a second model of the
// same knobs the `NodeManifest` already lists (mirrors `field_gizmo`'s `view_from_params`).
#[allow(clippy::too_many_arguments)]
fn remap(
    t_in: f32,
    inner_offset: f32,
    mode: i32,
    curvature: f32,
    steps: f32,
    curve: Option<&Curve>,
    curve_offset: f32,
    lo: f32,
    hi: f32,
    multiplier: f32,
    clamp: bool,
    invert: bool,
    strength: f32,
) -> f32 {
    let mut t = if invert { 1.0 - t_in } else { t_in };
    // Inner offset: `1 − io` is the input level that reaches full effect; above it the
    // core is a flat 1. Clamped below 1 so the denominator is never 0.
    let io = inner_offset.clamp(0.0, 0.999);
    t = (t / (1.0 - io)).min(1.0);
    t = contour(mode, t, curvature, steps, curve, curve_offset);
    let mut mapped = (lo + t * (hi - lo)) * multiplier;
    if clamp {
        mapped = mapped.clamp(0.0, 1.0);
    }
    t_in + (mapped - t_in) * strength
}

/// GPU compute kernel (ADR-0126): a straight WGSL port of [`remap`] × [`contour`] over
/// the existing `falloff` — same `min`/`max`/`clamp`/`floor` + polynomials (HR-5), so
/// parity holds within float ULPs. There is no `P` binding — a remap is position-blind.
/// `falloff` `ReadWrite` from the `1.0` identity mirrors the CPU (an absent mask reads
/// full effect); it is REWRITTEN, not multiplied (a remap is a transfer function).
///
/// **All five contour modes cook on the DEVICE.** Modes 0–3 are pure WGSL; the Curve
/// contour (mode 4) reads its shape from a `ph2d-curve` text param the uniform layout
/// cannot carry (a curve is not a scalar), so A1-gpu bakes it to a LUT (see [`LUTS`])
/// and `rm_contour` samples `rm_curve_sample(t)`. No `applicable` gate — the sequencer
/// never falls back to the CPU for this node, which is what the GPU-resident field
/// pipeline (the "maximize GPU" north) wants. Parity for mode 4 holds within a small ε
/// (the LUT's resolution ÷ its slope), wider than the ULP parity of modes 0–3.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let rm_in = read_falloff(i);\n\
        var rm_t = rm_in;\n\
        if (params.invert >= 0.5) { rm_t = 1.0 - rm_t; }\n\
        let rm_io = clamp(params.inner_offset, 0.0, 0.999);\n\
        rm_t = min(rm_t / (1.0 - rm_io), 1.0);\n\
        rm_t = rm_contour(\n\
            i32(rm_round(params.contour)), rm_t,\n\
            params.curvature, params.steps, params.curve_offset);\n\
        var rm_mapped = (params.min + rm_t * (params.max - params.min)) * params.multiplier;\n\
        if (params.clamp >= 0.5) { rm_mapped = clamp(rm_mapped, 0.0, 1.0); }\n\
        let rm_out = rm_in + (rm_mapped - rm_in) * params.strength;\n\
        // The probability gate — the SAME integer hash as the CPU (bit-identical).\n\
        let rm_gate = select(0.0, 1.0, rm_hash01(i, u32(max(params.seed, 0.0))) < params.probability);\n\
        write_falloff(i, rm_out * rm_gate);\n",
    wgsl_lib: "\
        fn rm_round(x: f32) -> f32 {\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn rm_hash01(id: u32, seed: u32) -> f32 {\n\
            var x = id * 0x9E3779B9u + seed * 0x85EBCA6Bu;\n\
            x = x ^ (x >> 16u);\n\
            x = x * 0x7FEB352Du;\n\
            x = x ^ (x >> 15u);\n\
            x = x * 0x846CA68Bu;\n\
            x = x ^ (x >> 16u);\n\
            return f32(x >> 8u) / 16777216.0;\n\
        }\n\
        fn rm_shifted(t: f32, offset: f32) -> f32 {\n\
            // A guarda do zero e\x27 a identidade EXACTA no topo do intervalo:\n\
            // `x - floor(x)` leva 1.0 a 0.0, e t = 1 e\x27 o que toda peca a\n\
            // mascara cheia entrega. Ver o doc da `shifted` na CPU.\n\
            if (offset == 0.0) { return t; }\n\
            let u = t + offset;\n\
            return u - floor(u);\n\
        }\n\
        fn rm_contour(mode: i32, t: f32, curvature: f32, steps: f32, offset: f32) -> f32 {\n\
            if (mode == 1) {\n\
                let ease_in = t * t;\n\
                let ease_out = 1.0 - (1.0 - t) * (1.0 - t);\n\
                if (curvature >= 0.0) { return t + (ease_in - t) * curvature; }\n\
                return t + (ease_out - t) * (-curvature);\n\
            }\n\
            if (mode == 2) {\n\
                let n = max(steps, 2.0);\n\
                let k = min(floor(t * n), n - 1.0);\n\
                return k / (n - 1.0);\n\
            }\n\
            if (mode == 3) {\n\
                let n = max(steps, 2.0);\n\
                return rm_round(t * (n - 1.0)) / (n - 1.0);\n\
            }\n\
            if (mode == 4) { return rm_curve_sample(rm_shifted(t, offset)); }\n\
            return t;\n\
        }\n",
    bindings: &[ColumnBinding {
        column: "falloff",
        dim: Dim::Scalar,
        access: ColumnAccess::ReadWrite,
        identity: [1.0; 4],
        port: 0,
    }],
    params: &[
        "inner_offset",
        "contour",
        "curvature",
        "steps",
        "min",
        "max",
        "multiplier",
        "clamp",
        "invert",
        "strength",
        "curve_offset",
        "probability",
        "seed",
    ],
    count_law: None,
    variant_by_param: None,
    // No fallback: the Curve contour (mode 4) samples the LUT (A1-gpu), so every
    // mode is device-resident. The gate that declined mode 4 is gone.
    applicable: None,
};

/// The Curve contour's LUT resolution (A1-gpu) — samples of the authored curve over
/// `t ∈ [0,1]`. 256 keeps a smooth transfer within a few thousandths of the CPU `eval`
/// while costing 1 KiB the sequencer uploads per cook.
const LUT_RESOLUTION: u32 = 256;

/// The LUT channel this node registers (A1-gpu): the sequencer samples the authored
/// curve (the `CURVE_KEY` text param) to [`LUT_RESOLUTION`] floats and binds them, so
/// `rm_contour`'s `mode == 4` branch reads `rm_curve_sample(t)` on the DEVICE instead of
/// the whole node dropping to the CPU. Naming it `rm_curve` makes the accessor
/// `rm_curve_sample`, matching the WGSL call.
static LUTS: &[LutSpec] = &[LutSpec {
    name: "rm_curve",
    text_key: CURVE_KEY,
    resolution: LUT_RESOLUTION,
    fill: fill_curve_lut,
}];

/// Sample the authored curve into `out` at `t = k/(n−1)` — the NODE-side half of the LUT
/// channel (`ph2d-nodegraph` stays curve-library-agnostic; the sampling lives here).
/// Mirrors the CPU `contour(4, t, …)`: an unset or malformed string is the identity
/// [`Curve`] (`eval(t) = t`), which is exactly the passthrough the CPU takes on `None`.
fn fill_curve_lut(text: &str, out: &mut [f32]) {
    let curve = ph2d_curve::parse(text).unwrap_or_else(Curve::identity);
    let n = out.len();
    for (k, slot) in out.iter_mut().enumerate() {
        let t = if n <= 1 {
            0.0
        } else {
            k as f32 / (n - 1) as f32
        };
        *slot = curve.eval(t);
    }
}

struct FieldRemap;

impl NodeOp for FieldRemap {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let inner_offset = ctx.param("inner_offset");
        let mode = ctx.param("contour").round() as i32;
        let curvature = ctx.param("curvature");
        let steps = ctx.param("steps");
        let lo = ctx.param("min");
        let hi = ctx.param("max");
        let multiplier = ctx.param("multiplier");
        let clamp = ctx.param("clamp") >= 0.5;
        let invert = ctx.param("invert") >= 0.5;
        let strength = ctx.param("strength");
        let curve_offset = ctx.param("curve_offset");
        let probability = ctx.param("probability");
        let seed = ctx.param("seed").max(0.0) as u32;
        // The Curve contour's shape lives in the text channel — parse it ONCE per cook
        // (not per instance). Only mode 4 reads it; an unset or malformed string yields
        // `None`, which `contour` treats as the identity (a passthrough).
        let curve: Option<Curve> = if mode == 4 {
            ctx.text_param(CURVE_KEY).and_then(ph2d_curve::parse)
        } else {
            None
        };
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            // The mask being remapped; absent → 1.0 (full effect), the CPU/GPU identity.
            let prev: Option<&[f32]> = match input.get("falloff") {
                Some(Column::Scalar(v)) => Some(v.as_slice()),
                _ => None,
            };
            let fall = par_build(n, |i| {
                let t_in = prev.and_then(|v| v.get(i).copied()).unwrap_or(1.0);
                let m = remap(
                    t_in,
                    inner_offset,
                    mode,
                    curvature,
                    steps,
                    curve.as_ref(),
                    curve_offset,
                    lo,
                    hi,
                    multiplier,
                    clamp,
                    invert,
                    strength,
                );
                // The probability gate: keep or zero the WHOLE instance by a stable hash
                // of its index. `probability = 1` keeps everyone (hash < 1 always).
                let gate = if hash01(i as u32, seed) < probability {
                    1.0
                } else {
                    0.0
                };
                m * gate
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
    reg.register(Box::new(FieldRemap))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Remap",
            category: ph2d_node_registry::NodeUiCategory::Focus,
            silhouette: ph2d_node_registry::NodeSilhouette::Diamond,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_groups(MANIFEST.id, PARAM_GROUPS);
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_luts(MANIFEST.id, LUTS);
    Ok(())
}

use ph2d_node_registry::{ParamGroup, ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1): the C4D Remapping controls. `contour` is the transfer
/// selector; `curve` is the A1 curve editor (a text param, live when contour = Curve);
/// `curvature` bends the Quadratic; `steps` counts the Step/Quantize levels; `strength`
/// is the input→remapped blend (0 = passthrough).
/// As SEÇÕES deste nó (doc 88 B3). Treze controles numa lista plana são uma parede; em dois
/// grupos, com os essenciais soltos na frente, eles viram três perguntas.
///
/// ⚠️ **A TRANSFERÊNCIA inteira fica de FORA de propósito** — `contour` (qual curva),
/// `CURVE_KEY` (a curva), `curvature` e `steps` (as formas das outras contours) e
/// `inner_offset`. Param sem grupo pinta antes de toda seção, e é ali que os essenciais devem
/// estar: a transferência é a razão de existir deste nó, e pô-la numa seção seria escondê-la
/// atrás de um clique.
///
/// ⚠️ E isto foi corrigido pelo gate `selected_field_remap_yields_an_interactive_curve_row`,
/// que já afirmava que a curva é a PRIMEIRA row: a primeira versão desta tabela punha o
/// `CURVE_KEY` num grupo "Shape" — contradizendo, duas linhas acima, o comentário que dizia
/// para não a sepultar.
static PARAM_GROUPS: &[ParamGroup] = &[
    // A FAIXA de saída: entre que números o resultado vive.
    ParamGroup::new("min", "Range"),
    ParamGroup::new("max", "Range"),
    ParamGroup::new("multiplier", "Range"),
    ParamGroup::new("clamp", "Range"),
    ParamGroup::new("invert", "Range"),
    // E quanto do resultado chega ao mundo.
    ParamGroup::new("strength", "Output"),
    ParamGroup::new("probability", "Output"),
    ParamGroup::new("seed", "Output"),
];

/// **UM NÚMERO POR CONTORNO, E SÓ O DO CONTORNO ESCOLHIDO APARECE.**
///
/// ⚠️ **Isto nasceu de um smoke** (Enio, 2026-08-21: *"Curve offset e outros parâmetros
/// não têm efeito"*). Ele estava com o contorno em `Curve` e tinha ao lado, vivos no
/// painel, o `curvature` (que é do Quadratic) e o `steps` (que é do Step/Quantize) —
/// dois knobs que ali não fazem nada **por desenho**. Um controle vivo que não muda
/// nada é indistinguível de um bug, e foi lido como um.
///
/// ⛔ **O `curve` (o editor de curva) NÃO é gateado, de propósito.** O gate
/// `selected_field_remap_yields_an_interactive_curve_row` afirma que ele é a PRIMEIRA
/// linha e é interactivo; gateá-lo por `contour == 4` esconderia-o no modo default
/// (Quadratic) e reprovaria aquele gate. Se um dia a decisão for escondê-lo, é aquele
/// gate que se reconcilia primeiro — não este.
static PARAM_GATES: &[ph2d_node_registry::ParamGate] = &[
    ph2d_node_registry::ParamGate {
        param: "curvature",
        when: "contour",
        values: &[1],
    },
    ph2d_node_registry::ParamGate {
        param: "steps",
        when: "contour",
        values: &[2, 3],
    },
    ph2d_node_registry::ParamGate {
        param: "curve_offset",
        when: "contour",
        values: &[4],
    },
];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "inner_offset",
        label: "Inner Offset",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "contour",
        label: "Contour",
        min: 0.0,
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["None", "Quadratic", "Step", "Quantize", "Curve"],
        },
    },
    ParamUiHint {
        // The Curve contour's shape — a TEXT param (`CURVE_KEY`), not a `ParamSpec`
        // (the `motion.expression` Text precedent). The panel draws the curve editor;
        // inert unless `contour = Curve`. `min/max/step` are inert for a curve widget.
        param: CURVE_KEY,
        label: "Curve",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Curve,
    },
    ParamUiHint {
        param: "curvature",
        label: "Curvature",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "steps",
        label: "Steps",
        min: 2.0,
        max: 32.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "min",
        label: "Min",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "max",
        label: "Max",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "multiplier",
        label: "Multiplier",
        min: 0.0,
        max: 4.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "clamp",
        label: "Clamp",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    ParamUiHint {
        param: "invert",
        label: "Invert",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    // ⚠️ **A faixa é UMA VOLTA, e ela é a unidade natural do knob**: o deslocamento
    // dá a volta em 1, então `−1..1` cobre o percurso inteiro nos dois sentidos e
    // nada além disso é alcançável (`1.5` desenha o mesmo que `0.5`). Um slider mais
    // largo seria curso morto.
    ParamUiHint {
        param: "curve_offset",
        label: "Curve Offset",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "strength",
        label: "Strength",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "probability",
        label: "Probability",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 9999.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
