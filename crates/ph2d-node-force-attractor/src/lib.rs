#![forbid(unsafe_code)]
//! `force.attractor` — a radial force toward (or away from) a fixed target
//! point. A Motion **force**: a `Pure` node that adds its per-instance
//! contribution into the transient `accel` column (× the multiplicative
//! `falloff` field, plan §1.6); `motion.integrate` turns the accumulated
//! acceleration into motion. It holds no state and never moves anything by
//! itself — wire it inside the integrator's `pre` loop.
//!
//! Reference behaviour (MiniCavalryV2 `forces/attractor.js`, clean-room):
//! magnitude `strength · curve(1 − d/R)` inside radius `R`, zero outside and in
//! the dead-zone at the centre; `repel` flips the sign. The reference's free
//! `(1−d/R)^power` is transcendental for non-integer powers (HR-5), so the
//! shaping is the app's falloff-curve vocabulary instead: Linear / Quad /
//! Smooth / Smoother — deterministic polynomials, endpoint-exact.
//!
//! Params: `target_x`/`target_y` (world), `strength` (accel at the centre,
//! world-units/s²), `radius` (influence extent), `curve` (0 Linear · 1 Quad ·
//! 2 Smooth · 3 Smoother), `repel` (0/1).

use ph2d_node_registry::{NodeRegistry, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::attr::par_build;
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod accum;
use accum::{add_accel, falloff_at, vec2_at, vec2_col};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Dead-zone radius around the target: inside it the direction is numerically
/// meaningless (d → 0), so the force is zero (the reference's `d < 1` px in a
/// ~10-unit world).
const DEAD_ZONE: f32 = 1e-3;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("force.attractor"),
    name: "force.attractor",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // **O ALVO como STREAM** (modo `Stream`). APENDADO — o índice da porta 0 não
        // se mexe, e no modo `Point` ela nem é lida. Ver [`TARGET_MODE`].
        PortSpec {
            name: "target",
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
            name: "target_x",
            default: 0.0,
        },
        ParamSpec {
            name: "target_y",
            default: 0.0,
        },
        ParamSpec {
            name: "strength",
            default: 5.0,
        },
        ParamSpec {
            name: "radius",
            default: 4.0,
        },
        ParamSpec {
            name: "curve",
            default: 2.0, // Smooth
        },
        ParamSpec {
            name: "repel",
            default: 0.0,
        },
        // **DE ONDE VEM O ALVO.** APENDADO, default `0` = os dois params de sempre,
        // ao bit. Ver [`TARGET_MODE`].
        ParamSpec {
            name: "target_mode",
            default: 0.0,
        },
        // **O TETO DA ANTECIPAÇÃO**, em segundos. `0` = mirar onde o alvo ESTÁ, que é
        // o que este nó sempre fez. Ver [`LEAD`].
        ParamSpec {
            name: "lead",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// **DE ONDE VEM O ALVO** (doc 89 folha 02 — POP Attract *"Attraction Type:
/// Particles / Points / Surface Points"*, e o **Goal** do Cavalry).
///
/// - `0` **Point** — os params `target_x`/`target_y`. O que este nó sempre fez.
/// - `1` **Stream** — o ponto **MAIS PRÓXIMO** da porta `target`, por elemento.
///
/// ⚠️ **Isto não era exprimível por composição, e a célula tinha razão:** nada no
/// catálogo entregava *"o ponto mais próximo daquele outro stream"* a uma força. O
/// `motion.pin_constraint` e o `motion.look_at` leem o stream que os atravessa, não um
/// segundo.
///
/// ⚠️ **Empate de distância desempata pelo ÍNDICE MAIS BAIXO** — uma ordem total, para
/// que dois alvos equidistantes deem a mesma resposta em toda plataforma.
///
/// ⚠️ **No modo `Stream` com a porta VAZIA não há força nenhuma**, e é de propósito: um
/// modo que diz *"o alvo é um stream"* e não encontra stream **não tem alvo**. Cair nos
/// params ali daria a impressão de que o modo não faz nada — o knob morto que esta casa
/// acabou de pagar noutro nó. ⛔ E `register_required_inputs` não serve: aquele canal é
/// incondicional, e marcaria todo atrator em modo `Point`.
///
/// ⚠️ **Ligado, ele RECUSA o device** (`applicable`, a porta dos irmãos
/// `motion.combine`/`motion.cull`/`field.index_range`): o vizinho mais próximo dentro de
/// uma porta-template precisa do leitor `ColumnAccess::SourceRead`, que hoje só existe
/// emparelhado com um `StreamOp::SourceRows` — um nó que MUDA a contagem. Este preserva-a.
/// Desligado (o default) nada recua.
const TARGET_MODE: &str = "target_mode";

/// **O TETO DA ANTECIPAÇÃO, em segundos** (doc 89 folha 02 — POP Attract *Force
/// Method: Follow / Predict Intercept*).
///
/// Cada partícula mira onde o alvo dela **vai estar** daqui a `t` segundos, com
/// `t = min(distância / velocidade própria, lead)`. É o intercepto **POR PARTÍCULA** que
/// a célula pedia: o tempo-de-chegada é dela, não um número global — a que está longe e
/// devagar antecipa mais que a que já vai chegando.
///
/// ⚠️ **O `lead` é um TETO e não um multiplicador, e é isso que remove a
/// singularidade.** `distância / velocidade` explode com a partícula parada; um tecto
/// escrito pelo artista corta-o num número que ele próprio escolheu, em vez de numa
/// constante inventada. Parada (velocidade 0) ela mira o tecto inteiro, que é a leitura
/// certa: *não sei quanto vou demorar, então uso o horizonte que me deram.*
///
/// ⚠️ **`lead = 0` é a identidade EXACTA** — `t = 0`, a mira é o próprio alvo, e o nó
/// devolve o que sempre devolveu.
///
/// ⚠️ **Ele só vive no modo `Stream`**, e o painel esconde-o fora dele: a antecipação
/// precisa da VELOCIDADE do alvo, e um par de params não tem velocidade.
const LEAD: &str = "lead";

/// An edge curve on a pre-clamped `s ∈ [0,1]` — the same transcendental-free
/// vocabulary as `motion.falloff` (HR-5). Endpoint-exact (`0→0`, `1→1`).
fn curve(kind: i32, s: f32) -> f32 {
    match kind {
        0 => s,                                         // Linear
        1 => s * s,                                     // Quad
        2 => s * s * (3.0 - 2.0 * s),                   // Smooth (smoothstep)
        _ => s * s * s * (s * (s * 6.0 - 15.0) + 10.0), // Smoother (smootherstep)
    }
}

/// O ALVO de um elemento — o ponto para onde ele é puxado, já LIDERADO.
///
/// `None` quando não há alvo nenhum (modo `Stream`, porta vazia): quem chama não soma
/// contribuição, e nem sequer um zero — ver [`TARGET_MODE`] para o porquê.
///
/// ⚠️ **O vizinho mais próximo escolhe-se pela posição de AGORA, não pela liderada** —
/// é o que a referência faz, e o contrário seria um ponto fixo a resolver por iteração.
fn aim_at(
    p: [f32; 2],
    my_vel: [f32; 2],
    tgt_p: &[[f32; 2]],
    tgt_v: &[[f32; 2]],
    lead: f32,
) -> Option<[f32; 2]> {
    let mut best = usize::MAX;
    let mut best_d2 = f32::INFINITY;
    for (j, q) in tgt_p.iter().enumerate() {
        let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
        let d2 = dx * dx + dy * dy;
        // `<` e não `<=`: o empate fica com o índice MAIS BAIXO, uma ordem total.
        if d2 < best_d2 {
            best_d2 = d2;
            best = j;
        }
    }
    let q = *tgt_p.get(best)?;
    if lead <= 0.0 {
        return Some(q);
    }
    // O tempo-de-chegada DESTA partícula, tectado pelo horizonte que o artista deu.
    let speed = (my_vel[0] * my_vel[0] + my_vel[1] * my_vel[1]).sqrt();
    let t = if speed > 0.0 {
        (best_d2.sqrt() / speed).min(lead)
    } else {
        lead
    };
    let v = tgt_v.get(best).copied().unwrap_or([0.0, 0.0]);
    Some([q[0] + v[0] * t, q[1] + v[1] * t])
}

/// GPU compute kernel (GPU/M5 **Fase 3**, ADR-0126 side channel): the exact
/// per-element map of the CPU `eval` — pull (or push) toward the target, gated
/// by the radius, shaped by the curve, scaled by the focus field.
///
/// Two ports of note. The dead zone and the radius test are the CPU's
/// `if d < DEAD_ZONE || d > radius { return [0,0] }` — the `d` in the
/// denominator is why, and a zero contribution is not the same as "skip the
/// add": the add still happens (of zero), which is what a chain expects. And the
/// `curve` enum goes through `at_round` (half-AWAY, like Rust's `f32::round`)
/// rather than WGSL's `round`, which is half-EVEN and would pick a different
/// branch at `x.5` — [[feedback_cpu_gpu_rounding_conventions_diverge]].
///
/// `sqrt` is IEEE correctly-rounded on both sides (HR-5 bars transcendentals,
/// not exact operations), so no approximation is needed here.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let at_radius = max(params.radius, AT_DEAD_ZONE);\n\
        let at_sign = select(1.0, -1.0, params.repel >= 0.5);\n\
        let at_p = read_P(i);\n\
        let at_dx = params.target_x - at_p.x;\n\
        let at_dy = params.target_y - at_p.y;\n\
        let at_d = sqrt(at_dx * at_dx + at_dy * at_dy);\n\
        var at_c = vec2<f32>(0.0, 0.0);\n\
        if (at_d >= AT_DEAD_ZONE && at_d <= at_radius) {\n\
        \x20   let at_w = at_curve(\n\
        \x20       i32(at_round(params.curve)),\n\
        \x20       clamp(1.0 - at_d / at_radius, 0.0, 1.0));\n\
        \x20   let at_mag = params.strength * at_w * at_sign * read_falloff(i);\n\
        \x20   at_c = vec2<f32>((at_dx / at_d) * at_mag, (at_dy / at_d) * at_mag);\n\
        }\n\
        write_accel(i, read_accel(i) + at_c);\n",
    wgsl_lib: "\
        const AT_DEAD_ZONE: f32 = 1e-3;\n\
        fn at_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn at_curve(kind: i32, s: f32) -> f32 {\n\
            if (kind == 0) { return s; }\n\
            if (kind == 1) { return s * s; }\n\
            if (kind == 2) { return s * s * (3.0 - 2.0 * s); }\n\
            return s * s * s * (s * (s * 6.0 - 15.0) + 10.0);\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "accel",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &[
        "target_x", "target_y", "strength", "radius", "curve", "repel",
    ],
    count_law: None,
    variant_by_param: None,
    // A recusa do modo `Stream` — ver [`TARGET_MODE`] para o mecanismo.
    applicable: Some(|p| p(TARGET_MODE) < 0.5),
};

struct ForceAttractor;

impl NodeOp for ForceAttractor {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let point = [ctx.param("target_x"), ctx.param("target_y")];
        let stream_mode = ctx.param(TARGET_MODE) >= 0.5;
        let lead = ctx.param(LEAD).max(0.0);
        // ⚠️ A porta do alvo é lida ANTES do input 0 e clonada: os dois `ctx.input`
        // não podem coexistir emprestados, e ela só é tocada no modo que a lê.
        let (tgt_p, tgt_v) = if stream_mode {
            (vec2_col(ctx.input(1), "P"), vec2_col(ctx.input(1), "vel"))
        } else {
            (Vec::new(), Vec::new())
        };
        let strength = ctx.param("strength");
        let radius = ctx.param("radius").max(DEAD_ZONE);
        let kind = ctx.param("curve").round() as i32;
        let sign = if ctx.param("repel") >= 0.5 { -1.0 } else { 1.0 };
        let out = {
            let input = ctx.input(0);
            // Pure per-instance map → parallel above the threshold
            // (bit-identical, no reduction). GPU/M5 Fase 0.
            let contrib: Vec<[f32; 2]> = par_build(input.count(), |i| {
                let p = vec2_at(input, "P", i, [0.0, 0.0]);
                let target = if stream_mode {
                    let my_vel = vec2_at(input, "vel", i, [0.0, 0.0]);
                    match aim_at(p, my_vel, &tgt_p, &tgt_v, lead) {
                        Some(t) => t,
                        // Sem alvo não há força — nem um zero somado por engano.
                        None => return [0.0, 0.0],
                    }
                } else {
                    point
                };
                let dx = target[0] - p[0];
                let dy = target[1] - p[1];
                let d = (dx * dx + dy * dy).sqrt();
                if d < DEAD_ZONE || d > radius {
                    return [0.0, 0.0];
                }
                let w = curve(kind, (1.0 - d / radius).clamp(0.0, 1.0));
                let mag = strength * w * sign * falloff_at(input, i);
                [(dx / d) * mag, (dy / d) * mag]
            });
            add_accel(input, &contrib)
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ForceAttractor))?;
    // ADR-0155: a force accumulates `accel`; inert without an integrator downstream.
    reg.register_couplings(
        MANIFEST.id,
        &[ph2d_node_registry::Coupling::Produces("accel")],
    );
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Attractor",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    // ADR-0130: per-element force: accumulates accel, identity preserved.
    reg.register_dense_window(MANIFEST.id);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "target_x",
        label: "Target X",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "target_y",
        label: "Target Y",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "strength",
        label: "Strength",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "radius",
        label: "Radius",
        min: 0.1,
        max: 20.0,
        step: 0.1,
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
        param: "repel",
        label: "Repel",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    // ⚠️ Um Enum NOMEADO: o segundo modo precisa de um FIO na porta `target`, e é o
    // rótulo *"Stream"* que faz o artista procurá-la.
    ParamUiHint {
        param: "target_mode",
        label: "Target",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Point", "Stream"],
        },
    },
    // O tecto da antecipação. ⚠️ A faixa para em 2 s porque acima disso a mira sai do
    // raio de influência antes de a força chegar lá — curso morto, não mais alcance.
    ParamUiHint {
        param: "lead",
        label: "Predict",
        min: 0.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

/// **CADA CONTROLE DO ALVO SÓ APARECE NO MODO QUE O LÊ.**
///
/// ⚠️ **Escrito no mesmo dia em que um smoke pagou a lição** (Enio, 2026-08-21, sobre o
/// `field.remap`: *"Curve offset e outros parâmetros não têm efeito"* — ele estava num
/// modo com dois knobs de outro modo vivos ao lado). Aqui a armadilha é a mesma e
/// simétrica: em `Stream` os dois `Target X/Y` não são lidos, e em `Point` a
/// antecipação não tem velocidade de alvo para ler.
static PARAM_GATES: &[ph2d_node_registry::ParamGate] = &[
    ph2d_node_registry::ParamGate {
        param: "target_x",
        when: "target_mode",
        values: &[0],
    },
    ph2d_node_registry::ParamGate {
        param: "target_y",
        when: "target_mode",
        values: &[0],
    },
    ph2d_node_registry::ParamGate {
        param: "lead",
        when: "target_mode",
        values: &[1],
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
    // A antecipação é um TEMPO — a linha do painel diz o que o número É (doc 88).
    ParamUnitDecl {
        param: "lead",
        unit: ParamUnit::Seconds,
    },
    ParamUnitDecl {
        param: "target_x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "target_y",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "radius",
        unit: ParamUnit::Length,
    },
];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
