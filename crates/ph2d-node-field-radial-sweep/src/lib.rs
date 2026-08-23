#![forbid(unsafe_code)]
//! `field.radial_sweep` — a Motion **focus field: a bounded angular SECTOR** (a
//! radar / pie wedge) keyed by POSITION. It writes the same multiplicative `falloff`
//! mask `motion.falloff`, `field.index_range` and `field.box` do (the sister-of-`accel`
//! contract, §1.2), so fields compose. Where `field.box` is an axis-aligned rectangle,
//! this is the family's **angular** primitive: a point's weight depends on its ANGLE
//! about `center` (inside `[start_angle, end_angle]`) AND its distance (inside `radius`).
//!
//! **The wedge REPEATS.** `repetitions = N` tiles the sector `N` times evenly around the
//! circle — a fan, a star of beams, a clock (Cavalry Sweep). `repetitions = 1` is a
//! single wedge. Both angular and radial edges get a **soft** ramp (a fraction `[0,1]`
//! of the extent — one dimensionless knob, because the two edges live in different units,
//! degrees vs world), shaped by the same 4-curve (Linear/Quad/Smooth/Smoother) as the box.
//!
//! **Transcendental-free (HR-5).** The angular test never calls `atan2` **per instance**:
//! a monotone **pseudo-angle** (the "diamond angle" — octant-linear, only `abs`/`+`/`/`)
//! maps a direction to `[0, 4)` preserving order, so membership in a sector is exact and
//! the soft ramp is linear in pseudo-space. The sector's pseudo-bounds are computed from
//! `start`/`end` via the SAME parabolic `cos_sin_cycles` the box uses for its rotation
//! (a per-cook constant), so the CPU and GPU kernels share one polynomial and parity
//! holds within ULPs. `sqrt` (radial distance) is IEEE-correctly-rounded on both, so it
//! is bit-safe too. This is a **spatial** field: it has Coordinates (`center`/`rotation`/
//! the gizmo-driven `radius`) and is the second field type the canvas gizmo drives (D9).
//!
//! **Remapping is a DOWNSTREAM node** (D1): inner-offset / contour / clamp / invert-graph
//! belong to `field.remap`, not here — this node only shapes the raw mask, then `min`s the
//! angular and radial ramps (the intersection of the wedge and the disk, exactly as the
//! box `min`s its two axis bands). The neutral is `end − start ≥ 360` (a full disk) with a
//! `radius` larger than the scene and `soft = 0` ⇒ mask `1` everywhere ⇒ the `falloff`
//! column is multiplied by the identity, byte-unchanged (D12).

use ph2d_node_registry::{NodeRegistry, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream, par_build};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod trig;
use trig::cos_sin_cycles;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("field.radial_sweep"),
    name: "field.radial_sweep",
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
            name: "radius",
            default: 10.0,
        },
        ParamSpec {
            name: "start_angle",
            default: 0.0,
        },
        ParamSpec {
            name: "end_angle",
            default: 60.0,
        },
        ParamSpec {
            name: "repetitions",
            default: 1.0,
        },
        // **O RAIO INTERNO** — ver [`INNER_RADIUS`]. Apendado; `0` ⇒ o disco de sempre.
        ParamSpec {
            name: "inner_radius",
            default: 0.0,
        },
        ParamSpec {
            name: "soft",
            default: 0.15,
        },
        ParamSpec {
            name: "center_x",
            default: 0.0,
        },
        ParamSpec {
            name: "center_y",
            default: 0.0,
        },
        ParamSpec {
            name: "rotation",
            default: 0.0,
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
    lowerings: &[LoweringKind::Cpu],
};

/// An edge curve on a pre-clamped `s ∈ [0,1]` — the SAME set as the other fields
/// (HR-5). `0` Linear · `1` Quad · `2` Smooth · `3` Smoother. Monotone, endpoint-exact.
fn curve(kind: i32, s: f32) -> f32 {
    match kind {
        1 => s * s,                                     // Quad
        2 => s * s * (3.0 - 2.0 * s),                   // Smooth (smoothstep)
        3 => s * s * s * (s * (s * 6.0 - 15.0) + 10.0), // Smoother (smootherstep)
        _ => s,                                         // Linear
    }
}

/// The plateau-and-ramp: `1` for `|d| ≤ half − soft`, ramping to `0` at `|d| = half`;
/// `soft = 0` is a hard edge; `soft` is clamped so it can never exceed `half`. `half ≤ 0`
/// degenerates to empty (only `d = 0` is inside). Identical to `field.box`'s `edge_ramp`,
/// used here for BOTH the radial edge (`d = r`, world units) and the angular edge
/// (`d = pseudo-distance`, dimensionless).
fn edge_ramp(d: f32, half: f32, soft: f32) -> f32 {
    let a = d.abs();
    let s = soft.max(0.0).min(half);
    if s > 0.0 {
        ((half - a) / s).clamp(0.0, 1.0)
    } else if a <= half {
        1.0
    } else {
        0.0
    }
}

/// The **diamond pseudo-angle** of `(x, y)` in `[0, 4)`, monotone with the true angle
/// (CCW from +X): `0`→`0`, `+X`→`0`, `+Y`→`1`, `−X`→`2`, `−Y`→`3`. Only `abs`/`+`/`/`
/// and comparisons — no `atan2`, so it is transcendental-free (HR-5) and bit-identical
/// CPU↔GPU. The origin (`|x|+|y| = 0`) returns `0` (guards the `0/0`); it is a single
/// degenerate texel a sweep has no defined angle for.
fn pseudo_angle(x: f32, y: f32) -> f32 {
    let denom = x.abs() + y.abs();
    if denom == 0.0 {
        return 0.0;
    }
    let p = y / denom; // [-1, 1]
    if x < 0.0 {
        2.0 - p
    } else if y < 0.0 {
        4.0 + p
    } else {
        p
    }
}

/// The pseudo-angle of a DEGREE angle, via the parabolic trig (a per-cook constant).
/// The `(cos, sin)` are the corrected-parabolic ones the box uses, so CPU and GPU agree;
/// the ~0.09% trig error only nudges the sector edge <0.1° (invisible), and it nudges it
/// the SAME way on both devices.
fn pseudo_of_deg(deg: f32) -> f32 {
    let (c, s) = cos_sin_cycles(deg / 360.0);
    pseudo_angle(c, s)
}

/// `x mod 4` into `[0, 4)` (floor-based, so bit-identical to the WGSL `floor`).
fn wrap04(x: f32) -> f32 {
    x - 4.0 * (x * 0.25).floor()
}

/// `x` folded into `[-period/2, period/2)` (the nearest-repetition distance). Floor-based
/// ⇒ bit-identical CPU↔GPU. `period = 0` never happens (repetitions ≥ 1 ⇒ period ≤ 4).
fn wrap_sym(x: f32, period: f32) -> f32 {
    x - period * (x / period + 0.5).floor()
}

/// The per-cook sector constants derived from the raw params (so the CPU computes them
/// once outside the hot loop; the GPU recomputes the same formula per instance — same
/// inputs, same result, parity holds). `full` is a genuine full disk (`end − start ≥
/// 360`) whose angular ramp is bypassed to `1` (a distance-from-mid model always seams at
/// the antipode; a full circle must skip it — this is what makes the neutral EXACT).
struct Sector {
    pa_mid: f32,
    pa_half: f32,
    period: f32,
    full: bool,
}

fn sector(start_angle: f32, end_angle: f32, repetitions: f32) -> Sector {
    let pa_start = pseudo_of_deg(start_angle);
    let pa_end = pseudo_of_deg(end_angle);
    let span = wrap04(pa_end - pa_start);
    let pa_half = span * 0.5;
    let pa_mid = wrap04(pa_start + pa_half);
    let reps = repetitions.round().max(1.0);
    Sector {
        pa_mid,
        pa_half,
        period: 4.0 / reps,
        full: (end_angle - start_angle).abs() >= 360.0,
    }
}

/// **O RAIO INTERNO — o ANEL** (doc 89 folha 10 — C4D §B4 field **Torus**; MOPs Shape
/// Falloff: *"**inner/outer** (zona cheia→zero)"*).
///
/// A célula mediu a composição que já existia: `sweep(r = 10) → field.combine(Subtract,
/// b = sweep(r = 6))` — **três nós para um knob**, que é o critério de `P1` verbatim da §7 do
/// plano 89.
///
/// ⚠️ **`0` é o disco de hoje AO BIT, e não «quase»**: [`inner_rise`] devolve `1.0` exacto
/// para todo `r ≥ 0` quando `inner = 0`, e `min(rad, 1.0)` é `rad` para qualquer `rad ≤ 1`,
/// que é toda a imagem da rampa. Nenhum caminho novo é tomado no default.
///
/// ⚠️ **A banda macia do anel come para DENTRO nos dois lados**, como a de fora já fazia: a
/// externa consome `[radius − soft·radius, radius]` e a interna `[inner, inner + soft·inner]`.
/// Um `soft` medido em fracção da **própria** extensão é o que mantém as duas bordas com o
/// mesmo carácter quando o anel é fino — um `soft` absoluto engoliria o anel inteiro.
const INNER_RADIUS: &str = "inner_radius";

/// A rampa que SOBE de `0` em `inner` até `1` em `inner + soft` — o buraco do anel.
///
/// ⚠️ **Não é `1 − edge_ramp(r, inner, soft)`.** Aquela põe a banda macia em
/// `[inner − soft, inner]`, isto é, **fora** do anel; esta põe-na dentro, que é onde a borda
/// externa também vive. As duas dão o mesmo valor nos extremos e desenham anéis diferentes.
fn inner_rise(r: f32, inner: f32, soft: f32) -> f32 {
    if inner <= 0.0 {
        return 1.0;
    }
    let s = soft.max(0.0);
    if s > 0.0 {
        ((r - inner) / s).clamp(0.0, 1.0)
    } else if r >= inner {
        1.0
    } else {
        0.0
    }
}

/// The raw sweep mask (before the `curve` and `invert`) at LOCAL offset `(lx, ly)` from
/// the centre — i.e. the offset already un-rotated into the field's frame. It is the
/// `min` of the radial ramp (inside the disk of `radius`) and the angular ramp (inside the
/// nearest repetition of the sector), each softened by `soft` (a fraction of its extent).
fn sweep_mask(lx: f32, ly: f32, radius: f32, inner: f32, soft: f32, sec: &Sector) -> f32 {
    let r = (lx * lx + ly * ly).sqrt();
    let rad = edge_ramp(r, radius, soft * radius).min(inner_rise(r, inner, soft * inner));
    let ang = if sec.full {
        1.0
    } else {
        let pa = pseudo_angle(lx, ly);
        let d = wrap_sym(pa - sec.pa_mid, sec.period);
        edge_ramp(d, sec.pa_half, soft * sec.pa_half)
    };
    rad.min(ang)
}

/// GPU compute kernel (ADR-0126): a straight WGSL port of [`sweep_mask`] × [`curve`]
/// multiplied into the existing `falloff` — same `min`/`max`/`clamp`/`sqrt`/`floor` +
/// polynomials (HR-5), so parity holds within float ULPs. The sector constants are
/// recomputed per instance from the raw params (the CPU hoists them; the device redoes
/// the arithmetic — same formula, same result). `P` reads its `0` identity when absent
/// (the CPU's `unwrap_or([0,0])`); `falloff` `ReadWrite` from the `1.0` identity mirrors
/// the CPU (fields multiply, the column is always written).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let rs_p = read_P(i);\n\
        let rs_dx = rs_p.x - params.center_x;\n\
        let rs_dy = rs_p.y - params.center_y;\n\
        // Rotate the offset by -rotation into the field's local frame.\n\
        let rs_b = rs_cos_sin(params.rotation / 360.0);\n\
        let rs_lx =  rs_dx * rs_b.x + rs_dy * rs_b.y;\n\
        let rs_ly = -rs_dx * rs_b.y + rs_dy * rs_b.x;\n\
        let rs_soft = clamp(params.soft, 0.0, 1.0);\n\
        // Radial ramp: inside the disk of `radius`.\n\
        let rs_r = sqrt(rs_lx * rs_lx + rs_ly * rs_ly);\n\
        var rs_rad = rs_edge_ramp(rs_r, params.radius, rs_soft * params.radius);\n\
        rs_rad = min(rs_rad, rs_inner_rise(rs_r, params.inner_radius, rs_soft * params.inner_radius));\n\
        // Angular ramp: inside the nearest repetition of the sector.\n\
        let rs_pa_start = rs_pseudo_of_deg(params.start_angle);\n\
        let rs_pa_end = rs_pseudo_of_deg(params.end_angle);\n\
        let rs_span = rs_wrap04(rs_pa_end - rs_pa_start);\n\
        let rs_pa_half = rs_span * 0.5;\n\
        let rs_pa_mid = rs_wrap04(rs_pa_start + rs_pa_half);\n\
        let rs_reps = max(rs_round(params.repetitions), 1.0);\n\
        let rs_period = 4.0 / rs_reps;\n\
        var rs_ang = 1.0;\n\
        if (abs(params.end_angle - params.start_angle) < 360.0) {\n\
            let rs_pa = rs_pseudo_angle(rs_lx, rs_ly);\n\
            let rs_d = rs_wrap_sym(rs_pa - rs_pa_mid, rs_period);\n\
            rs_ang = rs_edge_ramp(rs_d, rs_pa_half, rs_soft * rs_pa_half);\n\
        }\n\
        let rs_m = rs_curve(i32(rs_round(params.curve)), min(rs_rad, rs_ang));\n\
        var rs_f = rs_m;\n\
        if (params.invert >= 0.5) { rs_f = 1.0 - rs_m; }\n\
        write_falloff(i, read_falloff(i) * rs_f);\n",
    wgsl_lib: "\
        fn rs_inner_rise(r: f32, inner: f32, soft: f32) -> f32 {\n\
            if (inner <= 0.0) { return 1.0; }\n\
            let s = max(soft, 0.0);\n\
            if (s > 0.0) { return clamp((r - inner) / s, 0.0, 1.0); }\n\
            if (r >= inner) { return 1.0; }\n\
            return 0.0;\n\
        }\n\
        fn rs_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn rs_sin_cycles(phase: f32) -> f32 {\n\
            let ff = phase - floor(phase);\n\
            var p: f32;\n\
            if (ff < 0.5) { let u = ff * 2.0; p = 4.0 * u * (1.0 - u); }\n\
            else { let u = (ff - 0.5) * 2.0; p = -4.0 * u * (1.0 - u); }\n\
            return 0.225 * (p * abs(p) - p) + p;\n\
        }\n\
        fn rs_cos_sin(phase: f32) -> vec2<f32> {\n\
            return vec2<f32>(rs_sin_cycles(phase + 0.25), rs_sin_cycles(phase));\n\
        }\n\
        fn rs_curve(kind: i32, s: f32) -> f32 {\n\
            if (kind == 1) { return s * s; }\n\
            if (kind == 2) { return s * s * (3.0 - 2.0 * s); }\n\
            if (kind == 3) { return s * s * s * (s * (s * 6.0 - 15.0) + 10.0); }\n\
            return s;\n\
        }\n\
        fn rs_edge_ramp(d: f32, half: f32, soft: f32) -> f32 {\n\
            let a = abs(d);\n\
            let s = min(max(soft, 0.0), half);\n\
            if (s > 0.0) { return clamp((half - a) / s, 0.0, 1.0); }\n\
            return select(0.0, 1.0, a <= half);\n\
        }\n\
        fn rs_pseudo_angle(x: f32, y: f32) -> f32 {\n\
            let denom = abs(x) + abs(y);\n\
            if (denom == 0.0) { return 0.0; }\n\
            let p = y / denom;\n\
            if (x < 0.0) { return 2.0 - p; }\n\
            if (y < 0.0) { return 4.0 + p; }\n\
            return p;\n\
        }\n\
        fn rs_pseudo_of_deg(deg: f32) -> f32 {\n\
            let b = rs_cos_sin(deg / 360.0);\n\
            return rs_pseudo_angle(b.x, b.y);\n\
        }\n\
        fn rs_wrap04(x: f32) -> f32 {\n\
            return x - 4.0 * floor(x * 0.25);\n\
        }\n\
        fn rs_wrap_sym(x: f32, period: f32) -> f32 {\n\
            return x - period * floor(x / period + 0.5);\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [1.0; 4],
            port: 0,
        },
    ],
    params: &[
        "radius",
        "inner_radius",
        "start_angle",
        "end_angle",
        "repetitions",
        "soft",
        "center_x",
        "center_y",
        "rotation",
        "curve",
        "invert",
    ],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct FieldRadialSweep;

impl NodeOp for FieldRadialSweep {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let radius = ctx.param("radius");
        let inner = ctx.param(INNER_RADIUS);
        let soft = ctx.param("soft").clamp(0.0, 1.0);
        let (cx, cy) = (ctx.param("center_x"), ctx.param("center_y"));
        // The rotation basis and the sector's pseudo-bounds, computed ONCE per cook (a
        // per-cook constant): rotating a world offset by −rotation brings it into the
        // field's local frame, where `start`/`end` define the sector.
        let (rc, rs) = cos_sin_cycles(ctx.param("rotation") / 360.0);
        let sec = sector(
            ctx.param("start_angle"),
            ctx.param("end_angle"),
            ctx.param("repetitions"),
        );
        let curve_kind = ctx.param("curve").round() as i32;
        let invert = ctx.param("invert") >= 0.5;
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            let prev: Option<&[f32]> = match input.get("falloff") {
                Some(Column::Scalar(v)) => Some(v.as_slice()),
                _ => None,
            };
            let positions: &[[f32; 2]] = match input.get("P") {
                Some(Column::Vec2(v)) => v.as_slice(),
                _ => &[],
            };
            let fall = par_build(n, |i| {
                let p = positions.get(i).copied().unwrap_or([0.0, 0.0]);
                let (dx, dy) = (p[0] - cx, p[1] - cy);
                // Rotate the offset by −rotation into the field's local frame.
                let (lx, ly) = (dx * rc + dy * rs, -dx * rs + dy * rc);
                let m = curve(curve_kind, sweep_mask(lx, ly, radius, inner, soft, &sec));
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
    reg.register(Box::new(FieldRadialSweep))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Radial Sweep",
            category: ph2d_node_registry::NodeUiCategory::Focus,
            silhouette: ph2d_node_registry::NodeSilhouette::Diamond,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_groups(MANIFEST.id, PARAM_GROUPS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

use ph2d_node_registry::{ParamGroup, ParamHardMax, ParamUiHint, ParamWidget};

/// **O teto DIGITÁVEL dos dois raios, MEDIDO** — o irmão exacto do `field.box`.
///
/// ⚠️ **O neutro que o doc-comment deste módulo promete era inalcançável:** ele diz
/// *"`radius` larger than the scene"* ⇒ máscara `1` em toda a parte, e sem entrada aqui o
/// digitado parava em **40** (o fim do arrasto — `ui.rs:206`).
///
/// **O recurso é a PRECISÃO** (`CLAUDE.md` §0.0): nada nesta lei satura — um raio maior que a
/// cena É o neutro, e o nó honra-o. O que acaba é o `f32`: acima de `2²¹` somar o `step` do
/// slider (0,1) **não move o número**, então dois raios autoráveis vizinhos são o mesmo campo.
/// O valor é `2²¹ − 1 ulp`, derivado a cada corrida pelo gate
/// `every_precision_bound_param_types_to_the_measured_ceiling` (`ph2d-node-registry-init`).
///
/// ⚠️ **Os dois raios levam o MESMO teto, pela mesma razão que já governa a faixa do arrasto:**
/// o anel só existe enquanto `inner < radius`, e um teto menor no interno esconderia metade dos
/// anéis que o externo alcança.
static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "radius",
        max: 2_097_152.0 - 0.125,
    },
    ParamHardMax {
        param: INNER_RADIUS,
        max: 2_097_152.0 - 0.125,
    },
];

/// As SEÇÕES deste nó (doc 88 B3).
///
/// ⚠️ A VARREDURA fica solta inteira (`radius`, `start_angle`, `end_angle`, `repetitions`):
/// ela é a razão de existir do nó, e sepultá-la atrás de um clique é o erro que o gate do
/// `field.remap` já pegou uma vez nesta linha.
///
/// ⚠️ O `curve` aqui é o SELETOR de contorno do falloff (um `Enum`), não um editor de curva —
/// por isso ele agrupa com `soft`/`invert` em vez de ficar solto como o do `field.remap`.
static PARAM_GROUPS: &[ParamGroup] = &[
    // Onde o radar está plantado, e para onde ele aponta.
    ParamGroup::new("center_x", "Placement"),
    ParamGroup::new("center_y", "Placement"),
    ParamGroup::new("inner_radius", "Placement"),
    ParamGroup::new("rotation", "Placement"),
    // Como a borda do feixe desvanece.
    ParamGroup::new("soft", "Falloff"),
    ParamGroup::new("curve", "Falloff"),
    ParamGroup::new("invert", "Falloff"),
];

/// Param UI hints (M1.P1): the sweep's radius (gizmo-driven), the angular sector
/// (start/end in degrees), the repetition count, a normalized softness, a signed centre,
/// a named Curve selector, an Invert checkbox. `soft` is a FRACTION `[0,1]` of the extent
/// (dimensionless — it softens both the angular and the radial edge; see the module doc).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "radius",
        label: "Radius",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **A mesma faixa do `radius`, de propósito**: o anel só existe enquanto
    // `inner < radius`, e um teto menor esconderia metade dos anéis que o raio externo
    // alcança. Acima do externo o campo fica vazio — que é uma resposta, não um erro.
    ParamUiHint {
        param: INNER_RADIUS,
        label: "Inner Radius",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "start_angle",
        label: "Start Angle",
        min: -360.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "end_angle",
        label: "End Angle",
        min: -360.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "repetitions",
        label: "Repetitions",
        min: 1.0,
        max: 32.0,
        step: 1.0,
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
        param: "center_x",
        label: "Center X",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "center_y",
        label: "Center Y",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "rotation",
        label: "Rotation",
        min: -180.0,
        max: 180.0,
        step: 1.0,
        widget: ParamWidget::Angle,
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

/// **What each of this node's numbers IS** (doc 88, Wave A) — never how it is
/// shown. A `Length` is stored in world METRES and the panel resolves the face
/// the artist reads (`px` or `m`) from `ProjectSettings::display_unit`; a node
/// that could pin one would be overriding a setting it does not own.
///
/// Only params whose value is a world COORDINATE or a world DISTANCE are declared
/// here. A weight, a fraction, a rate and a count are left bare on purpose: a unit
/// that is wrong is worse than a unit that is missing, because the artist can read
/// a bare number but a mislabelled one teaches them something false.
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "radius",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "center_x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "center_y",
        unit: ParamUnit::Length,
    },
];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "ring_tests.rs"]
mod ring_tests;
