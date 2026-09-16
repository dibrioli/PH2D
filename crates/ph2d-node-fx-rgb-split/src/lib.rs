#![forbid(unsafe_code)]
//! `fx.rgb_split` — **chromatic aberration / RGB split** as a per-instance stream
//! FX (Motion Nodes M4, doc 01 §3). The colour-fringed edge of a cheap lens, or of
//! a glitching monitor: the look every motion package ships (Cavalry's *RGB Split*,
//! AE's *Shift Channels* + offset preset, the *Chromatic Aberration* of every game
//! post-process stack).
//!
//! **Two looks, and they are genuinely different** (`mode`):
//!
//! - **Split** — a UNIFORM displacement `(x, y)`. The datamosh / glitch stylisation:
//!   the whole image's channels slide apart by the same amount.
//! - **Aberration** — a RADIAL displacement, `strength × (P − centroid)`. This is the
//!   physical one: *lateral* chromatic aberration is a wavelength-dependent
//!   **magnification**, so the fringe is zero at the optical axis and grows LINEARLY
//!   with the distance from it. It is what Unity/Unreal/Godot's post-process actually
//!   does (they sample the R/B channels at UVs scaled in and out from the screen
//!   centre) — the centre stays clean and the corners smear.
//!
//! **Why the shader's three channels become two ghosts here.** A pass FX splits the
//! image ADDITIVELY into R, G and B and slides them apart; the three sum back to
//! white where they overlap. This node runs on the *instance stream*, and the
//! renderer ALPHA-blends the quads (`premultiplied = 0`, standard over): three
//! stacked opaque copies would show only the topmost, not their sum — the centre
//! would come out solid blue instead of untouched. So the honest per-instance form
//! is the pair of **complementary ghosts BEHIND the untouched element**:
//!
//! ```text
//!   out = [ R ghost (+offset) ] ++ [ GB ghost (−offset) ] ++ [ the element itself ]
//!            (behind)                  (behind)                  (on top, verbatim)
//! ```
//!
//! On an opaque element that reproduces exactly the edges the additive split
//! produces — the `+` side shows R with G,B missing (a **red** fringe) and the `−`
//! side shows G,B with R missing (a **cyan** fringe) — while the body stays the
//! colour the artist authored.
//!
//! **Channel isolation is a MULTIPLY.** The `tint` column multiplies into the
//! element's colour, so masking it with `[1,0,0,·]` *is* the red channel of whatever
//! that element happened to be — a blue element throws no red fringe, exactly as it
//! should. Nothing about this node assumes white.
//!
//! Transcendental-free (HR-5): offsets and tints are add/multiply. `Effect::Pure`.
//!
//! **Place it downstream.** Like `motion.trail`, it duplicates `id`s (a ghost shares
//! its source's identity), so it belongs *after* anything that pairs state by id
//! (`motion.integrate`, `motion.spring`) — conventionally just before the Output.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod copies;
use copies::{falloff_at, positions, tile, tints};
mod kernel;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// `mode` ≥ this → the radial (physical) aberration; below → the uniform split.
const MODE_ABERRATION: f32 = 0.5;

/// How many rows one element becomes: two ghosts + itself.
const COPIES: usize = 3;

/// Hard ceiling on the emitted element count — `3 × count` is an allocation driven
/// by an untrusted upstream. Over budget the FX **turns itself off** (the input,
/// verbatim) rather than half-drawing: a scene missing a third of its fringes reads
/// as a bug, an un-aberrated scene reads as "the effect is off".
///
/// ⚠️⚠️ **O RECURSO É O TEMPO DO DISPOSITIVO, e o número é MEDIDO lá** (ciclo 7, doc 112 §4 —
/// `CLAUDE.md` §0.0: *nunca deixe o fallback definir o produto*). O tecto anterior, `262 144`,
/// foi medido no caminho de CPU (`~10–15 ns` por linha emitida) quando este nó só corria na CPU,
/// e **desligava o efeito em silêncio a partir de `87 382` objectos**. Com o kernel (`kernel.rs`)
/// o recurso mudou; a sonda `fx_row_ceiling_probe` (`ph2d-gpu-cook/tests/it/gpu_cpu_parity_fx.rs`,
/// `--release`, 3 corridas a `load 5,6–8,1`, a mediana de 20 quadros de `grid → oscillator → fx →
/// output`, o menor dos três):
///
/// ```text
///   nó             │ linhas     │ disp. ms │ CPU ms │ memória da descida (188 B × linhas)
///   fx.rgb_split   │    786 432 │     1,52 │  17,2  │   141 MiB
///   fx.rgb_split   │  3 145 728 │     5,66 │  71,0  │   564 MiB   ← o tecto
///   fx.rgb_split   │  6 290 112 │    11,15 │ 146,2  │ 1 127 MiB
///   fx.drop_shadow │  2 097 152 │     3,79 │  44,3  │   376 MiB
///   fx.drop_shadow │  4 193 408 │     7,44 │  94,5  │   751 MiB
///   fx.drop_shadow │  8 388 608 │    15,14 │ 221,6  │ 1 504 MiB
/// ```
///
/// ⇒ **~1,8 ns por linha no dispositivo**, contra `~22–27 ns` na CPU do mesmo binário. O tecto é o
/// ponto em que a cadeia ocupa **cerca de um terço de um quadro de 60 fps** — o MESMO critério que
/// decidiu o de CPU —, medido: `3 145 728` linhas a `5,66–5,89 ms` (`34–35 %`). Os limites duros
/// do dispositivo ficam longe: a ligação de armazenamento do adaptador (`2 047 MiB` ⇒ `11,4 M`
/// linhas nesta máquina; acima dela o cozimento RECUSA com `BindingTooLarge`), o despacho
/// (`65 535 × 256`) e o `ID_WRAP` (`2²⁴`).
///
/// ⚠️ **A CPU computa a MESMA resposta e paga o dela** (`~71 ms` no tecto): ela é a referência e o
/// recurso de quem não tem adaptador, nunca quem decide o tecto.
///
/// ⚠️ **Os dois `fx.*` andam juntos, e já NÃO andam com o `motion.trail`** — gate
/// `the_instance_ceilings_agree_per_resource` na `ph2d-node-registry-init`, a única crate que vê
/// os quatro. Drop-crates não podem depender umas das outras (ADR-0075), então a const é copiada;
/// o que a mantém honesta é o gate.
pub const MAX_INSTANCES: usize = 3_145_728;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("fx.rgb_split"),
    name: "fx.rgb_split",
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
        // 0 = Split (uniform), 1 = Aberration (radial).
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
        // Split: the uniform displacement, in world units.
        ParamSpec {
            name: "x",
            default: 0.06,
        },
        ParamSpec {
            name: "y",
            default: 0.0,
        },
        // Aberration: the displacement per unit of distance from the centroid.
        ParamSpec {
            name: "strength",
            default: 0.06,
        },
        // How present the fringes are. 1 = full channel isolation.
        ParamSpec {
            name: "opacity",
            default: 1.0,
        },
        // Apendados (doc 89 folha 11). `(0, 0)` e `0` ⇒ o nó de sempre, ao bit.
        ParamSpec {
            name: CENTER_X,
            default: 0.0,
        },
        ParamSpec {
            name: CENTER_Y,
            default: 0.0,
        },
        ParamSpec {
            name: START,
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// **O DESLOCAMENTO DO EIXO** — de quanto o eixo óptico sai do centroide (doc 89 folha 11).
///
/// ⚠️ **O centroide continua a ser a ORIGEM, e isso é uma decisão medida** (doc 38 §2): o
/// efeito segue o layout, então mover a arte não faz a aberração ficar para trás — o que o
/// post-process de jogo, ancorado no centro da TELA, não consegue. O que faltava não era
/// *"usar o centro da tela"*: era **não haver como deslocar o eixo de todo**, e uma lente cujo
/// eixo não se move é uma lente que só sabe estar centrada.
///
/// `(0, 0)` é o centroide de sempre, **ao bit** — a soma de zero não muda um `f32`.
pub const CENTER_X: &str = "center_x";
/// O gémeo em Y de [`CENTER_X`].
pub const CENTER_Y: &str = "center_y";

/// **O RAIO INTERNO** — a franja só começa para além dele (o *Start Offset* do Unreal).
///
/// ⚠️ **A célula tinha uma cadeia PARCIAL medida e disse porque ela não chega:**
/// `field.radial → field.remap → fx.rgb_split` modula a coluna `falloff`, e o nó multiplica por
/// ela **o ALFA de cada fantasma**. Dentro de `r₀` a franja fica *transparente*, não
/// *inexistente* — e o deslocamento continua a crescer linearmente desde o centro, então as
/// cópias já estão separadas ali, à espera de que alguém suba a opacidade. O que a referência
/// faz é outra coisa: **o miolo fica perfeitamente limpo porque as cópias não se afastaram**.
///
/// A lei: `off = (q − eixo) · strength · max(0, r − start) / r`, com `r` a distância ao eixo.
/// Dentro de `start` o deslocamento é **zero exacto**; fora, ele cresce como
/// `(r − start) · strength`, ou seja a mesma rampa linear com a origem empurrada para fora.
pub const START: &str = "start";

/// **A LENTE** — o eixo óptico e o raio limpo em torno dele, os dois números que só o modo
/// `Aberration` lê.
///
/// ⚠️ **Juntos numa estrutura, e não como mais dois `f32` na lista de argumentos**: eles
/// respondem à mesma pergunta (*onde é o centro, e onde ele começa a doer*), e uma função de
/// oito parâmetros posicionais é onde um `x` troca de lugar com um `y` sem o compilador
/// reparar. E ela dá de graça o NOME do caso neutro — ver [`Lens::CENTRED`].
#[derive(Clone, Copy)]
struct Lens {
    /// O deslocamento do eixo **em relação ao centroide** ([`CENTER_X`]/[`CENTER_Y`]).
    axis: [f32; 2],
    /// O raio dentro do qual não há franja nenhuma ([`START`]).
    start: f32,
}

impl Lens {
    /// **A lente que este nó sempre teve** — eixo no centroide, sem raio limpo.
    ///
    /// ⚠️ Nomeada em vez de herdada por um default, do mesmo jeito que o `unlimited` do
    /// `sim.step`: os gates que já existiam continuam a perguntar **exactamente** o que sempre
    /// perguntaram, e uma premissa herdada em silêncio inverteria de sentido no dia em que
    /// alguém mexesse no default.
    /// `cfg(test)` porque só os gates a nomeiam: no produto o `eval` monta a lente a partir dos
    /// params, e uma const que só o teste lê não pertence ao binário (o precedente é o `COPIES`
    /// do `fx.drop_shadow`).
    #[cfg(test)]
    const CENTRED: Self = Self {
        axis: [0.0, 0.0],
        start: 0.0,
    };
}

/// The fringe displacement of every element.
///
/// Split → the same `(x, y)` for all. Aberration → `strength × (P − centroid)`: zero
/// at the layout's own optical axis, growing linearly outward. The centroid (not the
/// world origin) is the axis, so the effect follows the layout wherever it is moved.
fn offsets(p: &[[f32; 2]], mode: f32, x: f32, y: f32, strength: f32, lens: Lens) -> Vec<[f32; 2]> {
    let (axis, start) = (lens.axis, lens.start);
    let n = p.len();
    if mode < MODE_ABERRATION {
        return vec![[x, y]; n];
    }
    let sum = p
        .iter()
        .fold([0.0f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]]);
    // ⚠️ **O EIXO é o centroide MAIS o deslocamento autorado** (doc 89 folha 11). A soma de
    // `(0, 0)` não muda um `f32`, então o default é o centroide de sempre ao bit.
    let c = [
        sum[0] / n.max(1) as f32 + axis[0],
        sum[1] / n.max(1) as f32 + axis[1],
    ];
    p.iter()
        .map(|q| radial([q[0] - c[0], q[1] - c[1]], strength, start))
        .collect()
}

/// O deslocamento radial de UM elemento, dado o vector dele ao eixo.
///
/// ⚠️ **O `start` desligado é um braço LITERAL, não `× 1.0`** — a mesma lei do `space` do
/// `motion.move`. Uma divisão por `r` daria o mesmo número para quase todo elemento e **não**
/// para os que estão em cima do eixo, onde `r` é pequeno e o piso do denominador morde: o
/// default deixaria de ser byte-idêntico exactamente nas peças que a referência promete não
/// tocar.
fn radial(d: [f32; 2], strength: f32, start: f32) -> [f32; 2] {
    if !(start.is_finite() && start > 0.0) {
        return [d[0] * strength, d[1] * strength];
    }
    // `sqrt`, e não `hypot`: o segundo é uma chamada de libm com precisão por plataforma, e
    // esta casa quer o mesmo número em toda a matriz (HR-5 — o precedente é a normal do
    // `force.buoyancy`).
    let r = (d[0] * d[0] + d[1] * d[1]).sqrt();
    if r <= start {
        // ⚠️ ZERO EXACTO, e é a diferença que a célula mede: a cadeia por `falloff` deixava as
        // cópias separadas e transparentes; aqui elas simplesmente não se afastaram.
        return [0.0, 0.0];
    }
    let k = (r - start) / r;
    [d[0] * strength * k, d[1] * strength * k]
}

/// One evaluation: the two complementary ghosts, then the untouched element.
fn split(
    input: &Stream,
    mode: f32,
    x: f32,
    y: f32,
    strength: f32,
    opacity: f32,
    lens: Lens,
) -> Stream {
    let n = input.count();
    // Nothing to fringe, no budget for the ghosts, or the fringes were dialled to
    // nothing: forward the input verbatim rather than paying for invisible quads. A
    // junk `opacity` (NaN / ∞ — a loaded document, an MCP edit) counts as "off": it
    // would otherwise poison every fringe's alpha.
    let dead = !opacity.is_finite() || opacity <= 0.0;
    if n == 0 || n.saturating_mul(COPIES) > MAX_INSTANCES || dead {
        return input.clone();
    }
    let p = positions(input);
    let base = tints(input);
    let off = offsets(&p, mode, x, y, strength, lens);

    let mut pos = Vec::with_capacity(n * COPIES);
    let mut tint = Vec::with_capacity(n * COPIES);
    // The R ghost, displaced `+off`.
    for i in 0..n {
        pos.push([p[i][0] + off[i][0], p[i][1] + off[i][1]]);
        let a = base[i][3] * opacity * falloff_at(input, i);
        tint.push([base[i][0], 0.0, 0.0, a]);
    }
    // The G+B ghost — the complement — displaced `−off`.
    for i in 0..n {
        pos.push([p[i][0] - off[i][0], p[i][1] - off[i][1]]);
        let a = base[i][3] * opacity * falloff_at(input, i);
        tint.push([0.0, base[i][1], base[i][2], a]);
    }
    // The element itself, verbatim and LAST, so it paints over its own fringes.
    for i in 0..n {
        pos.push(p[i]);
        tint.push(base[i]);
    }

    let mut out = Stream::new(n * COPIES);
    for (name, col) in input.columns() {
        if name != "P" && name != "tint" {
            out.set(name.clone(), tile(col, COPIES));
        }
    }
    out.set("P", Column::Vec2(pos));
    out.set("tint", Column::Vec4(tint));
    out
}

struct FxRgbSplit;

impl NodeOp for FxRgbSplit {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let (mode, x, y) = (ctx.param("mode"), ctx.param("x"), ctx.param("y"));
        let (strength, opacity) = (ctx.param("strength"), ctx.param("opacity"));
        let lens = Lens {
            axis: [ctx.param(CENTER_X), ctx.param(CENTER_Y)],
            start: ctx.param(START),
        };
        let out = split(ctx.input(0), mode, x, y, strength, opacity, lens);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(FxRgbSplit))?;
    kernel::regista(reg); // o dispositivo (ciclo 7, W1b) — ver o cabeçalho de `kernel`
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "RGB Split",
            category: ph2d_node_registry::NodeUiCategory::Fx,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_hard_min(MANIFEST.id, PARAM_HARD_MIN);
    // O `falloff` declarado à mão (ADR-0155) — o kernel também o lê, e a declaração fica.
    reg.register_couplings(
        MANIFEST.id,
        &[ph2d_node_registry::Coupling::Consumes("falloff")],
    );
    Ok(())
}

use ph2d_node_registry::{
    ParamGate, ParamHardMax, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget,
};

/// **Os dois modos leem números DIFERENTES** (doc 90 §2, caça aos knobs mortos).
///
/// ⚠️ `offsets` devolve o deslocamento uniforme `(x, y)` **antes** de olhar para `strength`, que
/// só existe no braço `Aberration`; e em `Aberration` é `(x, y)` que ele não olha. Com `mode`
/// a nascer em `Split`, o painel de cinco linhas pintava **Strength** logo no estado inicial, e
/// arrastá-la de `0` a `0,5` não movia um pixel.
///
/// ⚠️ **Os três têm de ser gateados JUNTOS.** Esconder só o `strength` trocaria um par de knobs
/// mortos por outro — `x` e `y` são o espelho exacto dele do outro lado do seletor.
static PARAM_GATES: &[ParamGate] = &[
    ParamGate {
        param: "strength",
        when: "mode",
        // `0 = Split` · `1 = Aberration`.
        values: &[1],
    },
    ParamGate {
        param: "x",
        when: "mode",
        values: &[0],
    },
    ParamGate {
        param: "y",
        when: "mode",
        values: &[0],
    },
    // ⚠️ **A LENTE inteira só existe em `Aberration`** (doc 89 folha 11), e pela MESMA razão
    // que já gateava os três acima: no modo `Split` o deslocamento é uniforme e não há eixo
    // nenhum — um `Axis X` vivo ali seria exactamente o knob morto que este trio curou.
    ParamGate {
        param: CENTER_X,
        when: "mode",
        values: &[1],
    },
    ParamGate {
        param: CENTER_Y,
        when: "mode",
        values: &[1],
    },
    ParamGate {
        param: START,
        when: "mode",
        values: &[1],
    },
];

/// **O que cada número É** (doc 88, Wave A · doc 89 folha 11) — o irmão exacto da declaração
/// do `fx.drop_shadow`, e pela mesma medição: a família `fx.*` não declarava nenhum dos quatro
/// canais de side-metadata.
///
/// `x`/`y` são o deslocamento das cópias em MUNDO. A `strength` é a fração que o modo
/// *Aberration* escala com a distância ao centro — um `Ratio`, que é o que impede alguém de a
/// ler como pixels.
///
/// ⚠️ **A `opacity` fica de fora**, e não por esquecimento: ela multiplica o alfa das cópias,
/// e o vocabulário do `ParamUnit` já tem `Ratio` a significar *fração de uma grandeza*.
/// Rotular um alfa assim não diz nada que o rótulo «Opacity» já não diga — e *uma unidade
/// errada é pior que uma ausente*.
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "y",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "strength",
        unit: ParamUnit::Ratio,
    },
    // A LENTE: os três são comprimentos de MUNDO (o eixo é uma posição relativa, o raio é uma
    // distância), e o painel resolve a face do `ProjectSettings::display_unit`.
    ParamUnitDecl {
        param: CENTER_X,
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: CENTER_Y,
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: START,
        unit: ParamUnit::Length,
    },
];

/// O curso da MÃO fica onde estava; o da MÁQUINA alcança-se por digitação (doc 88 §11).
/// Uma aberração de `±1` mundo é um efeito discreto — o *datamosh* pede uma ordem de
/// grandeza a mais, e o custo é o mesmo (três cópias, contagem fixa).
static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "x",
        max: 20.0,
    },
    ParamHardMax {
        param: "y",
        max: 20.0,
    },
    // ⚠️ **Os três da LENTE nascem já com o teto DERIVADO** (bloco Z, doc 91): nada nas leis
    // deles satura, então o recurso é a precisão — acima de `2²⁰` somar o `step` do slider
    // (0,05) já não move o `f32`. Um param novo sem teto é o defeito que aquele bloco curou 25
    // vezes, e o gate `every_precision_bound_param_types_to_the_measured_ceiling` conta-os.
    ParamHardMax {
        param: CENTER_X,
        max: 1_048_576.0 - 0.0625,
    },
    ParamHardMax {
        param: CENTER_Y,
        max: 1_048_576.0 - 0.0625,
    },
    ParamHardMax {
        param: START,
        max: 1_048_576.0 - 0.0625,
    },
];

/// O piso do eixo — ele tem SINAL, e as duas pontas ou nenhuma (bloco Z, doc 91).
///
/// ⚠️ O `start` fica de fora: um raio negativo não é um raio, e o piso dele é do DESENHO.
static PARAM_HARD_MIN: &[ph2d_node_registry::ParamHardMin] = &[
    ph2d_node_registry::ParamHardMin {
        param: CENTER_X,
        min: -(1_048_576.0 - 0.0625),
    },
    ph2d_node_registry::ParamHardMin {
        param: CENTER_Y,
        min: -(1_048_576.0 - 0.0625),
    },
];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Split", "Aberration"],
        },
    },
    ParamUiHint {
        param: "x",
        label: "Offset X",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "y",
        label: "Offset Y",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "strength",
        label: "Strength",
        min: 0.0,
        max: 0.5,
        step: 0.005,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "opacity",
        label: "Opacity",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // A LENTE (doc 89 folha 11) — os três só aparecem no modo `Aberration`, ver [`PARAM_GATES`].
    ParamUiHint {
        param: CENTER_X,
        label: "Axis X",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: CENTER_Y,
        label: "Axis Y",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: START,
        label: "Start Radius",
        min: 0.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
];

/// OS FANTASMAS — os gates do que este nó emite, em arquivo próprio.
#[cfg(test)]
#[path = "ghost_tests.rs"]
mod tests;

/// A LENTE — assunto próprio, arquivo próprio (o corte do irmão `fx.drop_shadow`).
#[cfg(test)]
#[path = "lens_tests.rs"]
mod lens_tests;
