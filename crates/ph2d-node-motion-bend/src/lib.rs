//! `motion.bend` — a **bend deformer**: wrap the layout around an arc along the X
//! axis, so a straight row curls into a rainbow and the centre stays put (Motion
//! Nodes M3, deformers — doc 01 §3 / doc 20). This is the classic Maya/Blender/
//! Cinema4D Bend: the object's X extent maps onto a circular arc of total angle
//! `angle`, each element's along-axis distance becoming an arc angle, its
//! perpendicular offset kept as the radial distance. The rim curls up, the pivot
//! column holds — the second deformer of a different family from `motion.twist`
//! (which rotates about a point).
//!
//! **The strength is a value input** (`amount`, the value domain — doc 12), so it
//! can be ANIMATED: wire a `value.lfo` and the layout curls and uncurls (the boot
//! scene does this). `amount` scales `angle`; **unconnected it reads as `1.0`**
//! (full static bend). `amount = 0` (or `angle = 0`) is the identity. Positive
//! curls up, negative curls down. Falloff-masked like every behaviour. `Pure`.
//!
//! Transcendental-free (HR-5): the arc uses the corrected-parabolic `cos/sin`
//! (`trig`, in cycles) and the constant `π` (a literal, not a call); `√` is
//! IEEE-deterministic. Arc length is preserved: an element at X distance `d` from
//! the pivot travels an arc of length `d`, so the bend never stretches the layout.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream, par_build};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, ReduceOp, ReduceSpec};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use std::f32::consts::{PI, TAU};

mod trig;
mod ui;
use trig::cos_sin_cycles;
use ui::{PARAM_HINTS, PARAM_UNITS};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `amount` input (mirror of `ph2d_node_pulse_counter::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

const VALUE_COL: &str = "v";
/// Below this total rim angle (radians) the bend is the identity (no arc to wrap
/// onto), so `motion.bend` never divides by (near-)zero curvature.
const MIN_ANGLE_RAD: f32 = 1e-4;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.bend"),
    name: "motion.bend",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // The 0..1 (or ±) strength multiplier — a value, so it can be animated.
        // Optional: unconnected reads as 1.0 (full static bend).
        PortSpec {
            name: "amount",
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
        // Total arc angle over the layout's X extent, in degrees (the rim's turn).
        ParamSpec {
            name: "angle",
            default: 90.0,
        },
        // **A DIREÇÃO da dobra** — ver [`DIRECTION`]. Apendado; `0` ⇒ o eixo X de sempre.
        ParamSpec {
            name: "direction",
            default: 0.0,
        },
        // **O QUE ACONTECE FORA DA FATIA** — ver [`MODE`]. O default é o nome, não um `0`
        // solto: assim o literal e a escada não podem discordar.
        ParamSpec {
            name: "mode",
            default: MODE_UNLIMITED as f32,
        },
        // **QUAL FATIA DO EIXO DOBRA**, em frações do extent sobre o pivô — ver [`LIMITS`].
        // `−1, +1` é o layout INTEIRO, que é o nó que shipou.
        ParamSpec {
            name: "limit_lo",
            default: -1.0,
        },
        ParamSpec {
            name: "limit_hi",
            default: 1.0,
        },
        ParamSpec {
            name: "pivot_x",
            default: 0.0,
        },
        ParamSpec {
            name: "pivot_y",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The whole-stream reduction this deformer needs: the layout's **X extent**
/// about the pivot (GPU/M5, the deformer channel — `ph2d_nodegraph::reduce_meta`).
///
/// This is the `x_extent` fold at the top of [`bend`], declared so the sequencer
/// can run it on the device before the per-element pass. ⚠️ **`Max` over an
/// expression built only from subtraction and `abs` — so this one is BIT-EXACT
/// against the CPU**: `Max` is associative and exact in any evaluation order, and
/// there is no multiply-add here for a device to contract into an FMA. (Its
/// sibling `motion.twist` folds a radius, which has a product, and is an ε.)
static REDUCES: &[ReduceSpec] = &[ReduceSpec {
    name: "x_extent",
    column: "P",
    dim: Dim::Vec2,
    port: 0,
    op: ReduceOp::Max,
    value: "abs(v.x - params.pivot_x)",
    params: &["pivot_x"],
    // The same identity the `P` binding declares — an absent `P` is materialised
    // as the origin by BOTH paths, so both measure the extent of a layout of
    // origins (which is `|pivot_x|`, not "no extent").
    identity: [0.0; 4],
}];

/// **A DIREÇÃO DA DOBRA** (doc 89 folha 04 — C4D Bend: *"**Angle** defines the direction of
/// deformation. **0° is the deformer's local X axis**"*; Blender *Simple Deform ▸ Bend* tem
/// **Axis**). Nós dobrávamos **só no X**, e o header dizia-o como FACTO.
///
/// A célula media a composição: `motion.orbit(pivot, +θ) → bend → motion.orbit(pivot, −θ)` —
/// **três nós para um knob**, e com uma armadilha nomeada (o `motion.rotate` NÃO serve, porque
/// soma no atributo `rot` e não move `P`).
///
/// ⚠️ **A rotação entra e sai no MESMO ponto**: o deslocamento do pivô é levado ao quadro local
/// da dobra, dobrado ali, e trazido de volta. É a mesma costura que o `field.box` usa para a
/// sua caixa inclinada — e é o que mantém `pivot`, `angle` e `amount` a significarem uma coisa
/// só nas duas.
///
/// ⚠️ **`0` é literal, e a aritmética foi conferida e não assumida:** `cos_sin_cycles(0)` dá
/// `(1, 0)` ao bit, e `dx·1 + dy·0 = dx` em IEEE-754 para todo `dx`/`dy` **finito** — o caminho
/// literal fica escrito na mesma, pelo caso degenerado do `±inf` (onde `0 · inf` é NaN), o
/// mesmo precedente do `axis_angle` do `motion.sort`.
///
/// ⚠️ **O device é RECUSADO quando a direção morde, e a razão é a REDUÇÃO.** O `x_extent` é um
/// `Max` sobre `abs(v.x − pivot_x)` que o sequenciador corre antes do passe por elemento; num
/// quadro rodado ele teria de dobrar sobre `abs(dx·cos + dy·sin)`, e a expressão de um
/// `ReduceSpec` só alcança `params` — o `cos`/`sin` teriam de ser o polinómio do `trig.rs`
/// **inline dentro da string da redução**, escrito uma segunda vez. ⛔ Duas cópias de uma lei
/// de HR-5 é exactamente como as duas metades divergem, e usar o `cos` do WGSL ali seria pior.
/// Manter o extent NÃO-rodado não é opção: ele escala a curvatura, e a dobra sairia com a
/// força errada **em silêncio**.
const DIRECTION: &str = "direction";

/// **QUAL FATIA DO EIXO DOBRA** (doc 89 folha 04 — Blender *Simple Deform ▸ Limits lower/upper*;
/// a caixa do C4D Bend, que é o mesmo controle vestido de gizmo).
///
/// Em frações do extent MEDIDO, sobre o pivô: `−1` é a ponta de um lado, `+1` a do outro, e
/// `0` o pivô. É a coordenada NATIVA deste nó — `angle` já é *"a volta do pivô até a ponta"* e
/// `x_extent` já é `max|dx|` sobre o pivô —, então um artista que entendeu o pivô entendeu os
/// limites. Uma faixa `0..1` como a do Blender pediria uma segunda convenção para o mesmo eixo.
///
/// ⚠️ **A fatia RE-ESCALA a curvatura, e as DUAS referências concordam nisso:** o ângulo
/// inteiro passa a acontecer dentro da fatia, então encolher os limites APERTA a dobra em vez
/// de a revelar aos poucos. É isso que torna o controle poderoso — *"dobre estes 10% em 180°"*
/// é uma DOBRADIÇA, e não há outro jeito de a exprimir com um `angle` só. (A leitura contrária
/// — limites que só escondem — deixaria `angle` a significar a volta sobre um extent que a
/// fatia já não usa.)
///
/// ⚠️ **Os dois limites são um INTERVALO, não um percurso**, e por isso são ordenados antes de
/// serem usados: ao contrário do `from`/`to` do `motion.spline_wrap` — onde `from > to` percorre
/// a curva ao contrário e é legítimo — aqui inverter os dois nomearia a mesma fatia. Sem a
/// ordenação o `clamp` da CPU **entra em pânico** (`f32::clamp` exige `min ≤ max`).
///
/// ⚠️ **A identidade do default é LITERAL, e a conta foi conferida e não assumida:** com
/// `−1, +1` temos `a = −e`, `b = +e` ⇒ `mid = (a + b) · 0,5 = 0,0` e `half = (b − a) · 0,5 = e`
/// **ao bit** (multiplicar e dividir por 2 é exato em IEEE-754). Daí `k = θ/half` é
/// *literalmente* o `θ/x_extent` de sempre, `held − mid` é `dx − 0,0 = dx` (inclusive para
/// `dx = −0,0`), e `run` é `0,0` ⇒ o ramo do arco é a MESMA expressão. Não é um `if` que
/// devolve o mesmo número por outro caminho.
const LIMITS: (&str, &str) = ("limit_lo", "limit_hi");

/// **O QUE ACONTECE COM O QUE FICA DE FORA** (doc 89 folha 04 — C4D Bend *Mode:
/// Limited / Within Box / Unlimited*).
///
/// - `0` **Unlimited** (o default, e o nó que shipou): não há fora — a dobra continua para
///   além da fatia, e o excesso enrola no MESMO círculo. Com os limites no default isto é
///   exatamente o de sempre, porque nenhum elemento está fora.
/// - `1` **Limited**: dentro da fatia, o arco; fora, o layout **acompanha rigidamente** a ponta
///   dobrada — o troço reto sai pela TANGENTE do arco onde ele parou. É a torre cuja base fica
///   a prumo e cujo topo verga inteiro, e a única das três que **não** era exprimível.
/// - `2` **Within Box**: fora da fatia, identidade — o elemento fica onde estava.
///
/// ⚠️ **A célula media que o «Within Box» já era exprimível** (`field.box` → `falloff` → bend,
/// e é verdade), e ele entra na mesma. Duas razões, e nenhuma é conforto: os limites vivem no
/// eixo LOCAL deste nó, então com `direction ≠ 0` a caixa composta teria de ser rodada à mão
/// para o mesmo ângulo — duas fontes para uma orientação só; e um enum que oferecesse dois dos
/// três estados da referência faria o artista procurar o terceiro num sítio onde ele não está.
///
/// ⚠️ **O «Limited» é UM TERMO, e é o que prova que a fatia é geometria e não máscara:**
/// `arco + run · tangente(θ)`, onde `run = dx − clamp(dx, a, b)`. Com `run = 0` o termo não é
/// somado (o ramo é o literal de sempre) — somar `0,0 · c` seria byte-idêntico em quase todo
/// lado e trocaria o sinal de um zero negativo no resto, que é a diferença que um golden vê.
///
/// ⛔ **E aqui morre o `Keep Y-Axis Length`** (`BENDOBJECT_KEEPYAXIS`), a outra célula desta
/// folha: **RECUSADO POR MEDIÇÃO.** Não preservar o comprimento de arco quer dizer preservar a
/// **CORDA** (as pontas ficam onde estavam e o layout estica para arquear — a bandeira presa
/// nos dois cantos), o que é `r = extent / sin(θ)`. Essa expressão **diverge em ±180° e troca
/// de sinal para lá** — o layout atravessa para o outro lado —, e o slider de `angle` deste nó
/// vai a **±270°** ([`PARAM_HINTS`]), medido. ⇒ o modo seria indefinido em mais de metade do
/// curso do knob vizinho, sem porta para o cercar (`ParamHardMax` só ALARGA a caixa de texto
/// para fora do slider — medido na célula `from`/`to` do `motion.spline_wrap`). Preservar o
/// arco é o **default da própria referência**, e o esticado aproxima-se a jusante com um
/// `motion.transform`. *Um knob que faz o layout explodir quando OUTRO knob cruza um número não
/// é um modo, é uma armadilha.*
const MODE: &str = "mode";

/// A dobra continua para além da fatia — o default, e o nó que sempre shipou.
const MODE_UNLIMITED: i32 = 0;
/// Fora da fatia o layout acompanha RIGIDAMENTE a ponta dobrada (pela tangente).
const MODE_LIMITED: i32 = 1;
/// Fora da fatia, identidade — o elemento fica onde estava.
const MODE_WITHIN_BOX: i32 = 2;
/// As PALAVRAS da referência, na ordem dos números acima (C4D Bend ▸ Mode).
const MODE_LABELS: &[&str] = &["Unlimited", "Limited", "Within Box"];

/// A fatia do eixo que dobra, no quadro LOCAL: `(a, b, mid, half)`.
///
/// `a`/`b` são os limites em unidades de mundo, JÁ ORDENADOS (ver [`LIMITS`]); `mid` é o ponto
/// que não se move (o zero do ângulo) e `half` é o meio-curso que o `angle` inteiro atravessa.
fn slice_of(x_extent: f32, lo: f32, hi: f32) -> (f32, f32, f32, f32) {
    let (a, b) = (lo * x_extent, hi * x_extent);
    let (a, b) = if a <= b { (a, b) } else { (b, a) };
    ((a), b, (a + b) * 0.5, (b - a) * 0.5)
}

/// The device form of [`bend`] (GPU/M5). One invocation per element, reading the
/// layout's X extent from the reduction above.
///
/// ⚠️ **The trig is the CPU's polynomial, ported operation for operation** — not
/// WGSL's `sin`/`cos`. The CPU is transcendental-free by HR-5 (the corrected
/// parabolic sine, ~0.09% off true trig), so calling the device's real `sin`
/// here would not be a tighter ε, it would be a *different curve*: the arc would
/// visibly differ from the canonical one wherever the approximation does.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let bd_p = read_in_P(i);\n\
        let bd_dx = bd_p.x - params.pivot_x;\n\
        let bd_dy = bd_p.y - params.pivot_y;\n\
        let bd_theta = params.angle * read_amount_v(i) * 3.1415927 / 180.0;\n\
        let bd_ext = reduce_x_extent();\n\
        // A FATIA, ordenada — o `min`/`max` é a mesma lei da CPU (`slice_of`).\n\
        let bd_a = min(params.limit_lo, params.limit_hi) * bd_ext;\n\
        let bd_b = max(params.limit_lo, params.limit_hi) * bd_ext;\n\
        let bd_mid = (bd_a + bd_b) * 0.5;\n\
        let bd_half = (bd_b - bd_a) * 0.5;\n\
        let bd_mode = i32(bd_round(params.mode));\n\
        var bd_bent = vec2<f32>(bd_dx, bd_dy);\n\
        if (bd_half >= 1e-4 && abs(bd_theta) >= 1e-4) {\n\
        \x20   var bd_held = bd_dx;\n\
        \x20   if (bd_mode != 0) { bd_held = clamp(bd_dx, bd_a, bd_b); }\n\
        \x20   let bd_run = bd_dx - bd_held;\n\
        \x20   if (bd_mode != 2 || bd_run == 0.0) {\n\
        \x20       let bd_k = bd_theta / bd_half;\n\
        \x20       let bd_r = 1.0 / bd_k;\n\
        \x20       let bd_ph = (bd_k * (bd_held - bd_mid)) / 6.2831855;\n\
        \x20       let bd_c = bend_sin_cycles(bd_ph + 0.25);\n\
        \x20       let bd_s = bend_sin_cycles(bd_ph);\n\
        \x20       bd_bent = vec2<f32>((bd_r - bd_dy) * bd_s, bd_r * (1.0 - bd_c) + bd_dy * bd_c);\n\
        \x20       if (bd_run != 0.0) {\n\
        \x20           bd_bent = vec2<f32>(bd_bent.x + bd_run * bd_c, bd_bent.y + bd_run * bd_s);\n\
        \x20       }\n\
        \x20   }\n\
        }\n\
        let bd_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        write_P(i, vec2<f32>(\n\
        \x20   bd_p.x + (params.pivot_x + bd_bent.x - bd_p.x) * bd_f,\n\
        \x20   bd_p.y + (params.pivot_y + bd_bent.y - bd_p.y) * bd_f));\n",
    wgsl_lib: "\
        fn bd_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        // The corrected parabolic sine at `phase` CYCLES — the port of `trig.rs`.\n\
        fn bend_sin_cycles(phase: f32) -> f32 {\n\
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
        }\n",
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            // ReadWrite, not ReadWriteExisting: the CPU materialises an absent
            // `P` from the origin and always emits one (`out.set("P", …)`).
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0, 0.0, 0.0, 0.0],
            port: 0,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            // The `amount_at` rule, declared: absent reads 1.0 (full static
            // bend), length-1 broadcasts, length-N is per element.
            access: ColumnAccess::ReadBroadcast,
            identity: [1.0, 0.0, 0.0, 0.0],
            port: 1,
        },
    ],
    params: &[
        "angle", "mode", "limit_lo", "limit_hi", "pivot_x", "pivot_y",
    ],
    count_law: None,
    variant_by_param: None,
    // ⚠️ Ver [`DIRECTION`]: a redução `x_extent` não roda com o quadro.
    applicable: Some(|p| p("direction") == 0.0),
};

fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// The `amount` multiplier for element `i`: **unconnected (empty) → 1.0**;
/// length-1 broadcasts; length-N is per-element.
fn amount_at(vals: &[f32], i: usize) -> f32 {
    match vals.len() {
        0 => 1.0,
        1 => vals[0],
        _ => vals.get(i).copied().unwrap_or(0.0),
    }
}

/// Bend `base` about `pivot`. The X extent maps onto an arc of total angle
/// `angle_deg · amount`; element `i` at `(dx, dy)` (relative to the pivot) wraps to
/// `((R − dy)·sinθ, R·(1 − cosθ) + dy·cosθ)` where `θ = angle · dx / x_extent` and
/// `R = x_extent / angle` — a rotation-free arc-wrap that preserves arc length.
#[expect(
    clippy::too_many_arguments,
    reason = "a assinatura É o contrato do nó: cada argumento é um param do MANIFEST que a lei lê, e agrupá-los num struct só para contar menos põe uma segunda declaração da mesma lista ao lado do manifesto — que é onde as duas divergem"
)]
fn bend(
    base: &[[f32; 2]],
    pivot: [f32; 2],
    angle_deg: f32,
    direction_deg: f32,
    mode: i32,
    limit_lo: f32,
    limit_hi: f32,
    amount: &[f32],
    falloff: &[f32],
) -> Vec<[f32; 2]> {
    // O quadro LOCAL da dobra — ver [`DIRECTION`]. Em `0` a base é `(1, 0)` ao bit e as duas
    // projecções abaixo devolvem `dx`/`dy` inalterados.
    let (dc, ds) = cos_sin_cycles(direction_deg / 360.0);
    let local = |p: [f32; 2]| {
        let (dx, dy) = (p[0] - pivot[0], p[1] - pivot[1]);
        if direction_deg == 0.0 {
            (dx, dy)
        } else {
            (dx * dc + dy * ds, -dx * ds + dy * dc)
        }
    };
    // The layout's half-extent along the bend's axis (the rim distance) — the arc is
    // scaled to it. ⚠️ Medido no quadro LOCAL: é o eixo em que a dobra corre.
    let x_extent = base
        .iter()
        .map(|p| local(*p).0.abs())
        .fold(0.0_f32, f32::max);
    // `x_extent` is a max-reduction across all instances (kept serial above);
    // given it, output element `i` is a pure per-instance map → parallel above
    // the threshold (bit-identical, no reduction). GPU/M5 Fase 0.
    // A FATIA que dobra — ver [`LIMITS`]. No default `half` é `x_extent` AO BIT e `mid` é `0`,
    // então tudo abaixo reduz literalmente à expressão que sempre shipou.
    let (a, b, mid, half) = slice_of(x_extent, limit_lo, limit_hi);
    par_build(base.len(), |i| {
        let p = base[i];
        let (dx, dy) = local(p);
        let theta_max = angle_deg * amount_at(amount, i) * PI / 180.0;
        // Onde a dobra PARA, e o troço reto que sobra depois dela.
        // ⚠️ Um `mode` fora da lista lê como `Unlimited`, que é o default — nunca como um
        // quarto comportamento sem nome.
        let held = if mode == MODE_LIMITED || mode == MODE_WITHIN_BOX {
            dx.clamp(a, b)
        } else {
            dx
        };
        let run = dx - held;
        // TRÊS razões distintas para a MESMA resposta — o layout intacto: não há fatia, não
        // há ângulo, ou o elemento está fora de uma caixa que não o leva consigo.
        let degenerate = half < MIN_ANGLE_RAD || theta_max.abs() < MIN_ANGLE_RAD;
        let outside_the_box = mode == MODE_WITHIN_BOX && run != 0.0;
        let bent = if degenerate || outside_the_box {
            [dx, dy]
        } else {
            let k = theta_max / half; // curvature (rad per world unit)
            let r = 1.0 / k; // radius of the spine arc
            let (c, s) = cos_sin_cycles((k * (held - mid)) / TAU); // cos/sin of the arc angle
            let arc = [(r - dy) * s, r * (1.0 - c) + dy * c];
            if run == 0.0 {
                arc // a EXPRESSÃO DE SEMPRE, e é por aqui que todo grafo autorado passa
            } else {
                // `Limited`: o que ficou de fora sai pela TANGENTE, rígido.
                [arc[0] + run * c, arc[1] + run * s]
            }
        };
        // De volta ao mundo — a mesma base, no sentido contrário.
        let world = if direction_deg == 0.0 {
            bent
        } else {
            [bent[0] * dc - bent[1] * ds, bent[0] * ds + bent[1] * dc]
        };
        let f = falloff.get(i).copied().unwrap_or(1.0).clamp(0.0, 1.0);
        // Blend from the original toward the bent position by the falloff.
        [
            p[0] + (pivot[0] + world[0] - p[0]) * f,
            p[1] + (pivot[1] + world[1] - p[1]) * f,
        ]
    })
}

struct MotionBend;

impl NodeOp for MotionBend {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let angle = ctx.param("angle");
        let direction = ctx.param(DIRECTION);
        let mode = ctx.param(MODE).round() as i32;
        let (limit_lo, limit_hi) = (ctx.param(LIMITS.0), ctx.param(LIMITS.1));
        let pivot = [ctx.param("pivot_x"), ctx.param("pivot_y")];
        let amount: Vec<f32> = match ctx.input(1).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let input = ctx.input(0);
        let n = input.count();
        let base: Vec<[f32; 2]> = match input.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => vec![[0.0, 0.0]; n],
        };
        // Pure per-instance map → parallel above the threshold
        // (bit-identical, no reduction). GPU/M5 Fase 0.
        let falloff: Vec<f32> = par_build(n, |i| falloff_at(input, i));
        let moved = bend(
            &base, pivot, angle, direction, mode, limit_lo, limit_hi, &amount, &falloff,
        );
        let mut out = Stream::new(n);
        for (name, col) in input.columns() {
            if name != "P" {
                out.set(name.clone(), col.clone());
            }
        }
        out.set("P", Column::Vec2(moved));
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionBend))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Bend",
            // Transform blue: a spatial deformer.
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    // GPU/M5: the kernel and the whole-stream reduction it reads. Side metadata
    // on the registry (ADR-0126) — the frozen node contract is untouched.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_reduces(MANIFEST.id, REDUCES);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A straight horizontal row bends into an arc: the pivot column (dx=0) stays
    /// put, and the rim (max dx) curls UP (+y for a positive angle) and IN (its x
    /// shrinks). FALSIFICATION of "it does nothing" — a dead bend keeps the row flat.
    #[test]
    fn a_straight_row_bends_into_an_arc() {
        let row = [[-2.0, 0.0], [-1.0, 0.0], [0.0, 0.0], [1.0, 0.0], [2.0, 0.0]];
        let out = bend(
            &row,
            [0.0, 0.0],
            90.0,
            0.0,
            MODE_UNLIMITED,
            -1.0,
            1.0,
            &[],
            &[1.0; 5],
        ); // amount empty → 1
        // Centre (dx=0) is unmoved.
        assert!(
            out[2][0].abs() < 1e-4 && out[2][1].abs() < 1e-4,
            "centre holds: {:?}",
            out[2]
        );
        // Rim (+x) curls up and in.
        assert!(out[4][1] > 0.3, "rim curls up (+y): {:?}", out[4]);
        assert!(out[4][0] < 2.0, "rim pulls in (x shrinks): {:?}", out[4]);
        // Symmetry: the -x rim curls up too, mirrored in x.
        assert!(out[0][1] > 0.3, "-x rim curls up: {:?}", out[0]);
        assert!(
            (out[0][0] + out[4][0]).abs() < 1e-3,
            "left/right mirror in x"
        );
    }

    /// Arc length is PRESERVED: the bent rim sits at radius `R` from the arc centre
    /// and its arc length from the centre equals its original X distance. We check a
    /// 180° bend of a unit-extent row folds the two rims onto the SAME point (a
    /// half-circle closes the ends together above the pivot).
    #[test]
    fn the_bend_preserves_arc_length() {
        // A 180° bend wraps the row into a half-circle; the two ±rims meet at the top.
        let row = [[-1.0, 0.0], [1.0, 0.0]];
        let out = bend(
            &row,
            [0.0, 0.0],
            180.0,
            0.0,
            MODE_UNLIMITED,
            -1.0,
            1.0,
            &[],
            &[1.0; 2],
        );
        // Both rims land at the same x (0) and the same height (the arc diameter).
        assert!(
            (out[0][0] - out[1][0]).abs() < 1e-3,
            "rims meet in x: {:?} {:?}",
            out[0],
            out[1]
        );
        assert!((out[0][1] - out[1][1]).abs() < 1e-3, "rims meet in y");
        assert!(
            out[0][1] > 0.5,
            "and they are lifted above the pivot: {}",
            out[0][1]
        );
    }

    /// `amount` scales the bend and `0` is the identity; a negative amount curls the
    /// row DOWN instead of up.
    #[test]
    fn amount_scales_and_signs_the_bend() {
        let row = [[2.0, 0.0]];
        let flat = bend(
            &row,
            [0.0, 0.0],
            90.0,
            0.0,
            MODE_UNLIMITED,
            -1.0,
            1.0,
            &[0.0],
            &[1.0],
        );
        assert!(
            (flat[0][0] - 2.0).abs() < 1e-4 && flat[0][1].abs() < 1e-4,
            "amount 0 = identity"
        );
        let up = bend(
            &row,
            [0.0, 0.0],
            90.0,
            0.0,
            MODE_UNLIMITED,
            -1.0,
            1.0,
            &[1.0],
            &[1.0],
        );
        let down = bend(
            &row,
            [0.0, 0.0],
            90.0,
            0.0,
            MODE_UNLIMITED,
            -1.0,
            1.0,
            &[-1.0],
            &[1.0],
        );
        assert!(up[0][1] > 0.3, "positive curls up: {:?}", up[0]);
        assert!(down[0][1] < -0.3, "negative curls down: {:?}", down[0]);
    }

    /// A `falloff = 0` element is left flat while its neighbour bends — the mask
    /// gates the deform.
    #[test]
    fn falloff_zero_leaves_an_element_flat() {
        let row = [[2.0, 0.0], [2.0, 0.0]];
        let out = bend(
            &row,
            [0.0, 0.0],
            90.0,
            0.0,
            MODE_UNLIMITED,
            -1.0,
            1.0,
            &[],
            &[1.0, 0.0],
        );
        assert!(out[1][1].abs() < 1e-4, "masked stays flat: {:?}", out[1]);
        assert!(out[0][1] > 0.3, "focused bends: {:?}", out[0]);
    }

    #[test]
    fn registers_and_resolves() {
        use ph2d_nodegraph::cook::OpResolver;
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                (ty == MANIFEST.id).then_some(&MotionBend as &dyn NodeOp)
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
        assert!(Ops.resolve(MANIFEST.id).is_some());
    }
}

#[cfg(test)]
#[path = "direction_tests.rs"]
mod direction_tests;

#[cfg(test)]
#[path = "limits_tests.rs"]
mod limits_tests;
