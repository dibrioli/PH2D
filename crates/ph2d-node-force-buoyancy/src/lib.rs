#![forbid(unsafe_code)]
//! `force.buoyancy` — **Archimedes, and a sea to float on.** A Motion force (marked
//! `Temporal`: the wave reads the playhead) that adds its contribution into the transient
//! `accel` column × the multiplicative `falloff` field; `motion.integrate` / `sim.step`
//! turns the accumulated acceleration into motion (Motion Nodes M2 forces — doc 01 §3 /
//! doc 60).
//!
//! ## The model
//!
//! The reference is the 2D one every game engine converged on — Unity's
//! `BuoyancyEffector2D` (a *surface level*, a *density*, a *linear drag*) — with the
//! surface promoted from a level to a **travelling wave**, because a flat sea does not
//! bob and bobbing is the entire reason to reach for this node.
//!
//! ```text
//! surface(x, t) = level + amplitude · sin( (x − speed·t) / wavelength )   [phase in cycles]
//! submersion    = clamp( (surface − y) / depth , 0 , 1 )        0 = dry, 1 = fully under
//! a  =  density · submersion · n            n = the surface NORMAL at x
//!    −  drag    · submersion · velocity
//! ```
//!
//! Three things fall out of that, and each is a gate below:
//!
//! - **It floats.** The buoyant force grows with how deep the thing is, so with gravity
//!   pulling `g` and this pushing `density` the object settles at the depth where they
//!   cancel — `submersion = g/density`. It is not a floor you sit on: push it under and
//!   it springs back, drop it from high and it plunges and rises. (`density > g` or it
//!   sinks — as it should. The demo's water is `12` against a gravity of `4`.)
//! - **It bobs sideways too.** The buoyant force is normal to the *surface*, not straight
//!   up (pressure is normal to the isobar), so on the flank of a wave it has a horizontal
//!   component pointing **downhill** — a float drifts into the trough and rides the swell
//!   instead of pumping up and down on the spot. The slope comes free: it is the `cos`
//!   the same parabolic-sine pair already computes.
//! - **Water is thick.** `drag`, applied only to the submerged fraction, is what stops the
//!   float oscillating forever; it is why a thing that hits the water *slows* rather than
//!   bouncing, and it is the same `−k·v` the standalone `force.drag` applies (this one is
//!   gated by submersion, so a thing in the air is untouched by it).
//!
//! A horizontal current is *not* a param here: that is `force.wind` with `angle = 0`, the
//! same argument by which there is no separate gravity node.
//!
//! HR-5: the wave is the corrected parabolic sine (sibling `trig.rs`), the normal is a
//! `sqrt` — no transcendentals.

use ph2d_node_registry::{
    NodeRegistry, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget, RegistryError,
};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod accum;
mod trig;
use accum::{add_accel, falloff_at, vec2_at};

/// **A DENSIDADE POR-INSTÂNCIA** (doc 89 folha 02) — a coluna que multiplica o param
/// global. Ausente ⇒ `1.0` ⇒ o nó que sempre shipou, ao bit.
///
/// ⚠️ **O doc-comment deste nó CONFESSAVA a lacuna** (*"This is what `size` would be
/// if the substrate had one true notion of it"*), e a célula media o preço dela: uma
/// **rolha e uma pedra na mesma água** é o caso de uso, não o exótico, e a única rota
/// era `field.index_range → buoyancy → field.index_range → buoyancy` — **quatro nós
/// por material**, particionando por RANK ordinal em vez de por *"que objeto é este"*.
/// E o `falloff` não servia: ele escala empuxo **e** arrasto juntos, então não separa
/// *quão denso* de *quanto está submerso*.
///
/// ⚠️ **É uma ESCALA e não um valor absoluto**, e a distinção é o que faz o neutro
/// existir: um `density` por-instância absoluto teria de valer `0` quando ausente, e
/// aí a coluna ausente afundaria tudo. Multiplicativa, a ausência é `1`.
///
/// ⚠️ **Quem a escreve já existe:** `motion.drive` no canal **Custom**, que escreve a
/// coluna que o artista nomeia. *Um canal que ninguém consegue escrever não existe* —
/// a lei que a folha 05 pagou —, e aqui o escritor já estava no catálogo.
const DENSITY_COL: &str = "density";

/// A escala de densidade de `i` (coluna ausente ou curta → `1.0`).
fn scale_at(stream: &ph2d_nodegraph::attr::Stream, i: usize) -> f32 {
    match stream.get(DENSITY_COL) {
        Some(ph2d_nodegraph::attr::Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}
use trig::cos_sin_cycles;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Zero draft would divide by zero: an object of no depth is submerged infinitely fast.
const MIN_DEPTH: f32 = 1e-3;
/// Zero wavelength is a wave of infinite frequency — spatial aliasing, and a division by
/// zero in the slope.
const MIN_WAVELENGTH: f32 = 1e-3;

/// A chave do param **do número de ondas somadas** — o espectro do mar.
pub const WAVES: &str = "waves";

/// O tecto de ondas somadas. ⚠️ **É o mesmo `4` do `octaves` do `force.wind`**, e pela
/// mesma razão: cada camada é uma avaliação de seno por elemento por quadro, e a quarta já
/// tem `1/8` da amplitude da primeira — abaixo do que o olho separa de uma linha.
const MAX_WAVES: i32 = 4;

/// Cada onda seguinte é **metade da altura e metade do comprimento** da anterior.
///
/// ⚠️ **Não são números soltos: são o `lacunarity = 2` e o `roughness = 0,5` que o
/// `force.wind` já declara como default**, escritos aqui como constantes porque este nó
/// soma SENOS e não ruído — o que se partilha é a razão entre camadas, e ela é a mesma.
/// Expô-los seria um par de knobs a mais num nó que já tem sete; o gatilho para o fazer é
/// alguém pedir um mar com uma razão que não é esta.
const WAVE_LACUNARITY: f32 = 2.0;
const WAVE_GAIN: f32 = 0.5;

/// **O desencontro de FASE entre camadas** — o deslocamento áureo, o mesmo espalhamento que
/// o `decorrelate` do `value.instance_field` usa.
///
/// ⛔ **A justificação que eu tinha escrito aqui foi REFUTADA POR MEDIÇÃO, e ao contrário.**
/// Ela dizia: *«com a fase de todas a ser `(x − vt)/λ_k`, em `x = vt` todas as cristas
/// coincidem e o mar ganha um pico viajante»*. **Falso duas vezes.** Primeiro, `fase = 0` é
/// o cruzamento por ZERO de um seno e não a crista. Segundo — e é o que importa — com
/// comprimentos **harmónicos** (`λ, λ/2, λ/4, λ/8`) as camadas completam números de ciclos
/// diferentes, então elas **nunca** cristam juntas, com ou sem deslocamento: o pico da soma
/// em fase mede `0,7563` de um empilhamento total de `1,1250`.
///
/// ⭐ **O que o deslocamento de facto compra está medido, e é o oposto do que eu disse:** o
/// pico sobe para **`1,0251`**, ou seja as cristas ficam **mais variadas em altura** para a
/// mesma energia — e é exactamente isso que separa um mar de um padrão. A constante fica,
/// com a razão certa ao lado dela. *Uma mutação que a apagava sobreviveu ao gate que a
/// defendia, porque o gate media a premissa errada.*
const PHASE_STEP: f32 = 0.618_034;

/// Quantas ondas o param pede — totalizado, e nunca menos que uma.
fn wave_count(v: f32) -> i32 {
    if v.is_finite() {
        (v.round() as i32).clamp(1, MAX_WAVES)
    } else {
        1
    }
}

/// **O MAR em `x` no instante `t`** — a altura da superfície e a inclinação dela.
///
/// ⚠️ **Com `waves = 1` esta função devolve a expressão LITERAL do nó de sempre**, por
/// ramo: uma soma de um termo é o termo, mas `0 + a` não é `a` para todo `a` em `f32`
/// (o `-0.0` muda de sinal), e o default de um nó que já shipou reduz ao bit ou não reduz.
fn sea_at(x: f32, t: f32, level: f32, amp: f32, lambda: f32, speed: f32, waves: i32) -> (f32, f32) {
    let one = |amp: f32, lambda: f32, phase_off: f32| {
        let phase = (x - speed * t) / lambda + phase_off;
        let (cos, sin) = cos_sin_cycles(phase);
        (amp * sin, amp * (std::f32::consts::TAU / lambda) * cos)
    };
    if waves <= 1 {
        let (h, s) = one(amp, lambda, 0.0);
        return (level + h, s);
    }
    let (mut h, mut s) = (0.0, 0.0);
    let (mut a, mut l) = (amp, lambda);
    for k in 0..waves {
        let (dh, ds) = one(a, l, k as f32 * PHASE_STEP);
        h += dh;
        s += ds;
        a *= WAVE_GAIN;
        l /= WAVE_LACUNARITY;
    }
    (level + h, s)
}

/// **A ALTURA DA SUPERFÍCIE em `x` no instante `t`** — a metade pública de [`sea_at`].
///
/// ⚠️ **Existe porque uma cena que DESENHA este mar tem de poder afirmar que as boias
/// assentam nele**, e a forma da onda só é calculável aqui. Sem isto, o gate de uma cena de
/// smoke só consegue medir dispersões e derivas — grandezas que um mar e uma nuvem lançada
/// partilham. *Uma lei que só o autor consegue avaliar não é verificável por quem a mostra.*
///
/// Devolve só a altura; a INCLINAÇÃO fica privada de propósito (ela é do modelo de força,
/// não da superfície que se vê).
#[must_use]
pub fn surface_at(
    x: f32,
    t: f32,
    level: f32,
    amp: f32,
    lambda: f32,
    speed: f32,
    waves: f32,
) -> f32 {
    sea_at(
        x,
        t,
        level,
        amp,
        lambda.max(MIN_WAVELENGTH),
        speed,
        wave_count(waves),
    )
    .0
}

/// **O COMPRIMENTO DA ONDA MAIS FINA** que este mar contém, dado o comprimento base e
/// quantas camadas foram pedidas.
///
/// ⚠️ **Existe porque quem AMOSTRA este mar precisa de saber o que tem de resolver.** A
/// razão entre camadas ([`WAVE_LACUNARITY`]) e o tecto ([`MAX_WAVES`]) são decisões deste
/// nó; uma cena que espalhe boias sobre a superfície e queira garantir que a onda mais fina
/// se lê **não pode adivinhá-los** — e adivinhar errado não dá erro nenhum, dá ruído com
/// cara de onda. *Abaixo de dois pontos por período um seno não é sub-amostrado: ele é
/// irreconhecível.*
#[must_use]
pub fn finest_wavelength(lambda: f32, waves: f32) -> f32 {
    // ⚠️ A escada é PERCORRIDA e não elevada a potência: é a mesma divisão sucessiva que o
    // `sea_at` faz, então as duas respostas não podem divergir por arredondamento (e HR-5
    // fica fora de discussão neste arquivo, mesmo para uma função que o `eval` não chama).
    let mut l = lambda.max(MIN_WAVELENGTH);
    for _ in 1..wave_count(waves) {
        l /= WAVE_LACUNARITY;
    }
    l
}

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("force.buoyancy"),
    name: "force.buoyancy",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // The wave reads the playhead. (Convention: reads playhead ⇒ Temporal — only a
    // Temporal manifest folds the playhead into the memo fingerprint, so a same-tick
    // re-cook at a moved playhead returns the sea where it now is, not where it was.)
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        // The still-water line (world Y): where the sea sits with no wave.
        ParamSpec {
            name: "level",
            default: 0.0,
        },
        // The buoyant acceleration at FULL submersion. Beat gravity and it floats; lose
        // to gravity and it sinks (a stone is not a bug).
        ParamSpec {
            name: "density",
            default: 12.0,
        },
        // The object's draft: how far under the surface it must be to be fully submerged.
        // This is what `size` would be if the substrate had one true notion of it.
        ParamSpec {
            name: "depth",
            default: 0.3,
        },
        // Viscous damping, applied to the submerged fraction only.
        ParamSpec {
            name: "drag",
            default: 2.0,
        },
        // The swell. Amplitude 0 = a flat sea (and the node is then a pure float line).
        ParamSpec {
            name: "wave_amplitude",
            default: 0.12,
        },
        ParamSpec {
            name: "wave_length",
            default: 2.5,
        },
        // World units per second the crests travel in +X (negative = the other way).
        // **O ESPECTRO** (doc 89, folha 02) — `1` é a senoide única de sempre, ao bit.
        ParamSpec {
            name: WAVES,
            default: 1.0,
        },
        ParamSpec {
            name: "wave_speed",
            default: 0.4,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct ForceBuoyancy;

impl NodeOp for ForceBuoyancy {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let level = ctx.param("level");
        let density = ctx.param("density").max(0.0);
        let depth = ctx.param("depth").max(MIN_DEPTH);
        let drag = ctx.param("drag").max(0.0);
        let amp = ctx.param("wave_amplitude");
        let lambda = ctx.param("wave_length").max(MIN_WAVELENGTH);
        let speed = ctx.param("wave_speed");
        let t = ctx.playhead() as f32;
        let waves = wave_count(ctx.param(WAVES));

        let out = {
            let input = ctx.input(0);
            let contrib: Vec<[f32; 2]> = (0..input.count())
                .map(|i| {
                    let p = vec2_at(input, "P", i, [0.0, 0.0]);
                    let vel = vec2_at(input, "vel", i, [0.0, 0.0]);

                    // The sea at this instance's x, right now.
                    let (surface, slope) = sea_at(p[0], t, level, amp, lambda, speed, waves);

                    // How much of it is under water: 0 dry, 1 fully submerged.
                    let sub = ((surface - p[1]) / depth).clamp(0.0, 1.0);
                    let w = sub * falloff_at(input, i);

                    // Buoyancy is normal to the surface: n = normalize(−slope, 1). On the
                    // flank of a wave that tilts the push downhill, into the trough.
                    let inv_len = 1.0 / (slope * slope + 1.0).sqrt();
                    // ⚠️ A densidade DESTE elemento — ver [`DENSITY_COL`]. Coluna
                    // ausente ⇒ o param global, e a expressão é a de antes ao bit.
                    let dens = density * scale_at(input, i);
                    [
                        (dens * -slope * inv_len - drag * vel[0]) * w,
                        (dens * inv_len - drag * vel[1]) * w,
                    ]
                })
                .collect();
            add_accel(input, &contrib)
        };
        ctx.emit(out);
    }
}

/// The GPU kernel (ADR-0126): **side metadata**, not a lowering — the manifest above is
/// untouched and `eval` stays the canonical answer. This one exists because a force with no
/// kernel is not "one node slower": a single uncovered node **inside the loop** leaves a
/// boundary, and the boundary makes `plan` refuse the whole simulation (the two-sims rule).
/// Five forces on the GPU are worth nothing in the graph that drops a buoy in the water.
///
/// The wave is the sibling `trig.rs` ported literally — the same corrected parabolic sine
/// `force.wind` already carries, so the sea has one shape on both sides (HR-5). `sqrt` and
/// `/` are *not* bit-exact by the Vulkan guarantee (3 / 2.5 ULP), which is why this is
/// gated at the ε the sim parity already budgets, and why `force.attractor` — sqrt and a
/// divide in the same breath — is the precedent that says the budget holds.
///
/// **The param clamps are part of the model, not hygiene** — `eval` reads `depth`/`wave_length`
/// through `.max(1e-3)` and `density`/`drag` through `.max(0.0)`, and a kernel taking the raw
/// uniform would answer a different node. They are not equally *observable*, and the gates say
/// which is which rather than implying they all are:
///
/// - `wave_length` is gated (`a_sea_of_no_wavelength_matches_the_cpu`): drop the clamp and
///   `phase = x/0 = inf`, `frac(inf) = NaN`, and the whole field NaNs while the CPU sails on.
/// - `depth` **cannot be caught by any fixture**, and that is worth knowing rather than
///   papering over: the downstream `clamp(sub, 0, 1)` already tames the ±inf, so the two paths
///   differ only for an instance sitting *exactly* on the waterline — where the CPU reads
///   `0/1e-3 = 0` (dry, does not move) and an unclamped GPU reads `0/0 = NaN`, which
///   `motion.integrate`'s own finiteness guard rejects (so it also does not move). The
///   integrator's guard converges them. It stays because it mirrors `eval`, which is the
///   contract — not because a red gate is holding it here.
/// - `density`/`drag` `.max(0.0)` is likewise below the resolution of a parity gate (a
///   negative density is a well-defined, if odd, sea on both sides).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let by_p = read_P(i);\n\
        let by_vel = read_vel(i);\n\
        let by_lambda = max(params.wave_length, 1e-3);\n\
        // O ESPECTRO: N senos, cada um metade da altura e metade do comprimento do\n\
        // anterior, desencontrados pelo aureo para as cristas nunca coincidirem.\n\
        let by_n = i32(clamp(by_round(params.waves), 1.0, 4.0));\n\
        let by_x = by_p.x - params.wave_speed * params.playhead;\n\
        var by_h = 0.0;\n\
        var by_slope = 0.0;\n\
        var by_a = params.wave_amplitude;\n\
        var by_l = by_lambda;\n\
        if (by_n <= 1) {\n\
        \x20   let by_cs = by_cos_sin_cycles(by_x / by_lambda);\n\
        \x20   by_h = params.wave_amplitude * by_cs.y;\n\
        \x20   by_slope = params.wave_amplitude * (6.2831855 / by_lambda) * by_cs.x;\n\
        } else {\n\
        \x20   for (var k = 0; k < by_n; k = k + 1) {\n\
        \x20       let by_cs = by_cos_sin_cycles(by_x / by_l + f32(k) * 0.618034);\n\
        \x20       by_h = by_h + by_a * by_cs.y;\n\
        \x20       by_slope = by_slope + by_a * (6.2831855 / by_l) * by_cs.x;\n\
        \x20       by_a = by_a * 0.5;\n\
        \x20       by_l = by_l / 2.0;\n\
        \x20   }\n\
        }\n\
        let by_surface = params.level + by_h;\n\
        let by_sub = clamp((by_surface - by_p.y) / max(params.depth, 1e-3), 0.0, 1.0);\n\
        let by_w = by_sub * read_falloff(i);\n\
        // Buoyancy is normal to the surface: n = normalize(-slope, 1).\n\
        let by_inv_len = 1.0 / sqrt(by_slope * by_slope + 1.0);\n\
        let by_dens = max(params.density, 0.0) * read_density(i);\n\
        let by_drag = max(params.drag, 0.0);\n\
        write_accel(i, read_accel(i) + vec2<f32>(\n\
        \x20   (by_dens * -by_slope * by_inv_len - by_drag * by_vel.x) * by_w,\n\
        \x20   (by_dens * by_inv_len - by_drag * by_vel.y) * by_w));\n",
    wgsl_lib: "\
        fn by_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (o `round` do WGSL e' half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn by_sin_cycles(phase: f32) -> f32 {\n\
            let f = phase - floor(phase);\n\
            var p: f32;\n\
            if (f < 0.5) {\n\
                let u = f * 2.0;\n\
                p = 4.0 * u * (1.0 - u);\n\
            } else {\n\
                let u = (f - 0.5) * 2.0;\n\
                p = -4.0 * u * (1.0 - u);\n\
            }\n\
            return 0.225 * (p * abs(p) - p) + p;\n\
        }\n\
        fn by_cos_sin_cycles(phase: f32) -> vec2<f32> {\n\
            return vec2<f32>(by_sin_cycles(phase + 0.25), by_sin_cycles(phase));\n\
        }\n",
    bindings: &[
        // ⚠️ A escala de densidade POR ELEMENTO — `Read` com `identity: 1.0`, que é
        // exactamente o que a CPU devolve quando a coluna falta (ver [`DENSITY_COL`]).
        // As duas portas resolvem a mesma expressão, não uma parecida.
        ColumnBinding {
            column: DENSITY_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "accel",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
        // The drag term reads velocity. A field with no `vel` is a still sea acting on
        // still things: identity 0 makes `−drag·v` vanish, exactly as `vec2_at`'s default
        // does on the CPU.
        ColumnBinding {
            column: "vel",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
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
    ],
    params: &[
        "level",
        "density",
        "depth",
        "drag",
        "wave_amplitude",
        "wave_length",
        "wave_speed",
        WAVES,
    ],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ForceBuoyancy))?;
    // ADR-0155: a force accumulates `accel`; inert without an integrator downstream.
    reg.register_couplings(
        MANIFEST.id,
        &[ph2d_node_registry::Coupling::Produces("accel")],
    );
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    // ADR-0130: per-element force: accumulates accel, identity preserved.
    reg.register_dense_window(MANIFEST.id);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Buoyancy",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

/// **O teto DIGITÁVEL da profundidade, MEDIDO** — bloco Z, doc 91.
///
/// ⚠️ **A cena `=4` (o MAR) autora `depth = 4` e o campo digitava até `3`.** Sem entrada aqui o
/// digitado para no fim do ARRASTO (`ui.rs:206`), então o próprio demo do nó publicava uma
/// profundidade que o artista não conseguia escrever — acusação da sonda
/// `what_the_corpus_authors_and_no_one_can_type`.
///
/// **O recurso é a PRECISÃO** (`CLAUDE.md` §0.0): a profundidade não satura, então o que acaba é
/// o `f32` — acima daqui somar o `step` do slider (0,01) **não move o número**. Derivado a cada
/// corrida pelo gate `every_precision_bound_param_types_to_the_measured_ceiling`.
static PARAM_HARD_MAX: &[ph2d_node_registry::ParamHardMax] = &[ph2d_node_registry::ParamHardMax {
    param: "depth",
    max: 262_144.0 - 0.015625,
}];

/// Param UI hints (M1.P1).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "level",
        label: "Level",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "density",
        label: "Density",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "depth",
        label: "Depth",
        min: 0.01,
        max: 3.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "drag",
        label: "Drag",
        min: 0.0,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "wave_amplitude",
        label: "Wave Amplitude",
        min: 0.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "wave_length",
        label: "Wave Length",
        min: 0.05,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "wave_speed",
        label: "Wave Speed",
        min: -5.0,
        max: 5.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: WAVES,
        label: "Waves",
        min: 1.0,
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::Slider,
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
        param: "level",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "depth",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "wave_amplitude",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "wave_length",
        unit: ParamUnit::Length,
    },
];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "density_tests.rs"]
mod density_tests;

#[cfg(test)]
#[path = "spectrum_tests.rs"]
mod spectrum_tests;
