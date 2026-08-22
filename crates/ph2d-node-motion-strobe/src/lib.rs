//! `motion.strobe` — a PULSE fires a decaying flash on the stream it passes
//! through (Motion Nodes M2; decision doc `06_pulse_*`, §4).
//!
//! This is the first CONSUMER of the pulse type, and the wave's visible payoff:
//! every pulse "lights up" the element (a size boost + a colour flash) that then
//! **decays geometrically** over the next ticks — attack-instant, decay-exp, the
//! minimal ADSR. It is the TouchDesigner Trigger CHOP's *"audio-style ADSR
//! envelope"* and Unreal's Notify-State *begin/tick/end*, reduced to the shape a
//! motion element actually needs.
//!
//! **The envelope IS the value of the `pre` self-loop.** A per-instance `glow`
//! (0..1) rides the `state` port: a pulse sets it to 1.0, and each tick with no
//! pulse multiplies it by a rate DERIVED from the authored `Decay` (see
//! `decay_per_tick`). Applied ONCE per tick to the carried glow (so
//! a glow `n` ticks old has decayed exactly `n` times), never recomputed from an
//! age — the same geometric discipline as `motion.trail`. The lit look is
//! applied to the FRESH upstream `in` each tick (size/tint), so the boost never
//! compounds into the geometry; only the state persists.
//!
//! ## O envelope tem FORMA (Attack · Hold · Decay · Curve)
//!
//! O nó se auto-declarava *"the minimal ADSR"*: subida instantânea e queda
//! exponencial fixa. Faltavam os três trechos que o **Trigger** do Houdini/MOPs
//! nomeia — *Attack (len+shape) → Peak → Decay* — e a folha 07 marcava cada um.
//!
//! ⚠️ **E é a FASE que os torna exprimíveis, não mais um multiplicador.** Um
//! decaimento geométrico é *sem memória* (o `glow` de agora determina o do tick
//! seguinte), mas *"estou subindo ou caindo, e há quanto tempo?"* não sai de um
//! número só: um `glow` de 0,5 pode ser uma subida a meio caminho ou uma queda já
//! avançada. O estado ganha a **idade** ([`AGE_COL`]) e o envelope passa a ser
//! escolhido por ela:
//!
//! ```text
//! age < attack          ⇒  age / attack        (a rampa, 0 no tick do pulso)
//! age <= attack + hold  ⇒  1.0                 (o platô)
//! senão                 ⇒  glow_anterior · rate (a queda, a recorrência de sempre)
//! ```
//!
//! ⚠️ **Com `attack = 0` e `hold = 0` a escada reduz LITERALMENTE:** `0 < 0` é
//! falso, `0 <= 0` é verdadeiro ⇒ o tick do pulso emite `1.0`, e o seguinte cai no
//! ramo da recorrência com `glow_anterior = 1.0`. **Byte a byte** o que já
//! shipava, sem um `powf` no meio — derivar a queda da idade (`rate^age`) daria a
//! MESMA curva matemática e não os mesmos bits, porque `n` multiplicações não são
//! uma potência.
//!
//! ⚠️ **A CURVA mora no LOOK, e a coluna `glow` continua a ser o envelope CRU** —
//! ao lado do `size_boost` e do `flash_amount`, que são os outros dois controles
//! de aparência que não tocam a memória. É a distinção que este nó já faz com o
//! `falloff` (a máscara pesa o que é APLICADO, nunca o que é lembrado), e é ela
//! que mantém a recorrência livre de composição: se a curva entrasse no estado,
//! aplicá-la `n` vezes seria `c(c(c(…)))` — o **produto sobre a lista** que esta
//! casa já curou quatro vezes no relevo do Painter. Curva não-setada = identidade
//! ⇒ nem sequer é construída, e o caminho é o de antes.
//!
//! ⚠️ E a ORDEM é `curva(envelope) × máscara`, nunca o contrário: o `falloff` é um
//! peso ESPACIAL e a curva é a forma do envelope no TEMPO. Mascarar primeiro faria
//! um ponto a meia influência ler um ponto DIFERENTE da curva desenhada.
//!
//! ## Nem todo pulso acende (`probability`)
//!
//! O **Strobe Light** da AE tem *Random Strobe Probability*, e a folha mediu a
//! cadeia que a tentava: `pulse.sample_hold → pulse.compare` é de **BORDA com
//! histerese**, então dois sorteios altos seguidos disparam UMA vez e a
//! probabilidade sai errada. Aqui o sorteio é do próprio nó: cada pulso que chega
//! avança a pista ([`SEQ_COL`]) e acende só se `rand01(linha, ordinal) <
//! probability`.
//!
//! ⚠️ **A pista avança em todo pulso que CHEGA, não em todo pulso que ACENDE** —
//! se ela só andasse nos aceitos, um sorteio recusado re-sortearia **o mesmo
//! número** no pulso seguinte e a instância nunca mais acenderia. Determinístico e
//! travado é pior que aleatório.
//!
//! ⚠️ **Sem `seed`, e a ausência é deliberada:** dentro de UM nó a pista
//! `(linha, ordinal)` já decorrelaciona toda instância e todo pulso, que é a
//! feature; um seed serve para separar DOIS nós, e a resposta canônica desta casa
//! a essa pergunta é o `unique_per_node` do `value.instance_field` — derivado da
//! identidade do nó, sem param. Um `seed` aqui seria um controle **inerte no
//! default** (`probability = 1` nunca sorteia). O preço fica nomeado: dois strobes
//! com a mesma probabilidade sobre o mesmo stream acendem as MESMAS instâncias.
//!
//! The multiplicative `falloff` column masks the APPLIED look (size boost +
//! flash), like every behaviour — a focus field chooses which dots flash. The
//! glow memory itself is unmasked, so an animated field fades a live flash
//! without retriggering it.
//!
//! Positional per-instance (v1), matching `pulse.threshold`: `in`/`pulse`/`state`
//! pair by row order — e é a MESMA linha que serve de identidade ao sorteio.
//! The focus rig has a stable count.

#![forbid(unsafe_code)]

use ph2d_curve::Curve;
use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod hash;
use hash::rand01;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The pulse type (mirror of `ph2d_node_pulse_threshold::PULSE`; kept local so
/// this crate stays a leaf drop-crate — the shared vocabulary is the port
/// `(Instances, Scalar, Event)`, not a shared symbol).
const PULSE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event);

/// The pulse stream's fire column (`1.0` on a fired tick).
const PULSE_COL: &str = "pulse";
/// The per-instance envelope carried on the `pre` self-loop. **O nível CRU** — a
/// curva é do LOOK (ver o cabeçalho).
const GLOW_COL: &str = "glow";
/// **A FASE do envelope**: quantos ticks desde o pulso que acendeu esta
/// instância. É ela que torna *attack* e *hold* exprimíveis — um `glow` de 0,5
/// não diz se está a subir ou a cair.
const AGE_COL: &str = "glow_age";
/// **Quantos pulsos esta instância já VIU** — a pista do sorteio da
/// probabilidade. Avança em todo pulso que chega (ver o cabeçalho).
const SEQ_COL: &str = "glow_seq";
/// **A idade de quem NUNCA acendeu.**
///
/// ⚠️ Um estado recém-nascido é `0` em toda coluna, e `age = 0` significa *"um
/// pulso acabou de acender esta instância"* — indistinguível de *"nenhum pulso
/// jamais chegou"*. Com `attack = 0` os dois colapsam (a idade 1 já cai na
/// recorrência, sobre um brilho zero), e é por isso que o defeito não existia
/// antes desta wave; com `attack > 0` a fileira **INCHAVA SOZINHA no nascimento**,
/// sem que nada a tivesse disparado. Foi a cena `=46` que o mostrou, antes do
/// smoke.
///
/// `f32::MAX` é a sentinela porque ela **satura**: `MAX + 1.0` é `MAX` (o `1.0`
/// está abaixo do ULP dali), então quem nunca acendeu fica para sempre no ramo da
/// queda, sobre um brilho zero — que é exatamente o que *"não há flash"* é.
const NEVER: f32 = f32::MAX;
/// The text-param key carrying the envelope's shape (a `ph2d-curve` serialized
/// string, authored by the `ParamWidget::Curve` editor). NOT a `ParamSpec` — a
/// curve is not one number (the `value.curve` / `field.remap` precedent).
const CURVE_KEY: &str = "curve";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.strobe"),
    name: "motion.strobe",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "pulse",
            ty: PULSE,
        },
        // The envelope feedback; `state` → editor-plumbed `pre` self-loop.
        PortSpec {
            name: "state",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Pure: the tick enters the fingerprint through the consumed `pre` edge.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // ⚠️ A DURAÇÃO do flash em TICKS, não a taxa por tick — ver `decay_per_tick`.
        // O default NÃO foi escolhido: `34` ticks é o que a taxa `0.85` que já shipava
        // produzia, então o flash no default é o mesmo de antes.
        //
        // ⚠️ O param mantém o nome de fio `decay` de propósito: renomeá-lo faria o
        // `validate` RECUSAR todo grafo salvo que o sobrescreve (params que o manifesto
        // não declara mais derrubam o documento inteiro — a cicatriz do `motion.color_ramp`
        // na integração de 2026-07-30). Quem o artista lê é o RÓTULO.
        ParamSpec {
            name: "decay",
            default: 34.0,
        },
        // Peak size multiplier at full glow: size *= 1 + size_boost·glow.
        ParamSpec {
            name: "size_boost",
            default: 0.8,
        },
        // Flash colour + how much of it to mix at full glow (0 = size-only).
        ParamSpec {
            name: "flash_r",
            default: 1.0,
        },
        ParamSpec {
            name: "flash_g",
            default: 1.0,
        },
        ParamSpec {
            name: "flash_b",
            default: 1.0,
        },
        ParamSpec {
            name: "flash_amount",
            default: 0.9,
        },
        // ⚠️ Os três abaixo são APENDADOS e nascem NEUTROS — um grafo salvo que
        // nunca os escreveu coza byte a byte o que cozia (ver o cabeçalho).
        //
        // Quantos ticks o envelope leva a SUBIR. `0` = a subida instantânea de
        // sempre; o tick do pulso já é o pico.
        ParamSpec {
            name: "attack",
            default: 0.0,
        },
        // Quantos ticks ele fica no PICO antes de começar a cair. `0` = nenhum.
        ParamSpec {
            name: "hold",
            default: 0.0,
        },
        // A chance de um pulso ACENDER. `1` = todos, o de sempre — e o sorteio
        // nem é computado.
        ParamSpec {
            name: "probability",
            default: 1.0,
        },
        // **O OPERADOR DO FLASH** (doc 89 folha 07 — o *Strobe Operator* do AE): como cada
        // linha compõe enquanto acende. Mesma escada e mesmo vocabulário do
        // `motion.trail::ECHO_BLEND`, e pela mesma razão — a célula da folha dizia
        // *"um conserto serve os dois nós"*. `0` = o modo do sink ⇒ nenhuma coluna ⇒ o
        // lerp de sempre, byte-idêntico.
        ParamSpec {
            name: FLASH_BLEND,
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct Params {
    decay: f32,
    attack: f32,
    hold: f32,
    probability: f32,
    /// A forma do envelope. `None` = identidade ⇒ o caminho de antes, ao bit.
    curve: Option<Curve>,
    size_boost: f32,
    flash: [f32; 3],
    flash_amount: f32,
}

/// **O PISO do brilho** — um nível de 8 bits. Abaixo dele o `glow` não levanta um pixel
/// de tamanho nem mistura uma cor visível, então é ali que o flash ACABOU. Mesmo número
/// que o `motion.trail` usa para a ponta da cauda: uma lei, dois nós.
const GLOW_FLOOR: f32 = 1.0 / 255.0;

/// **A taxa por tick que apaga o brilho em `ticks` ticks.**
///
/// ⚠️ O knob é a DURAÇÃO do flash, nunca a taxa — o report de 2026-08-08 mediu o preço de
/// expor a taxa: no curso `0..0.99` do controle antigo os primeiros **86%** cobriam
/// 5..34 ticks e os últimos **14%** cobriam 34..551. *Catorze por cento do slider
/// carregava noventa e quatro por cento da faixa.* Agora ele é linear no que se vê.
///
/// ⚠️ **TICKS e não segundos, e a escolha é sobre RISCO.** Segundos exigiriam `ctx.dt()`,
/// que é `0.0` dentro de um time scope e cuja unidade depende do relógio que o chamador
/// passa ao cook — e um `dt` de zero faria a taxa virar `1.0`, isto é **o flash nunca
/// apagaria**. Uma regressão dessas é muito pior que um slider mal escalado, e o
/// vocabulário do módulo já é o tick: o `motion.trail` mede a cauda dele em ticks
/// (`length`, `spacing`) pela mesma razão.
fn decay_per_tick(ticks: f32) -> f32 {
    if !ticks.is_finite() || ticks <= 0.0 {
        // O degenerado que o artista quer dizer com zero: um flash de UM tick.
        return 0.0;
    }
    libm::powf(GLOW_FLOOR, 1.0 / ticks)
}

/// **O envelope de UMA instância neste tick** — o nível CRU e a fase que ela
/// passa a ter.
///
/// `age` é a idade JÁ AVANÇADA (0 no tick em que o pulso acendeu). Os três ramos
/// são os três trechos do Trigger, e com `attack = hold = 0` a escada reduz à
/// lei antiga ao bit — ver o cabeçalho.
fn glow_of(fires: bool, prev_glow: f32, prev_age: f32, p: &Params) -> (f32, f32) {
    let age = if fires { 0.0 } else { prev_age + 1.0 };
    let g = if age < p.attack {
        age / p.attack
    } else if age <= p.attack + p.hold {
        1.0
    } else {
        prev_glow * p.decay
    };
    (g, age)
}

/// Este pulso ACENDE? `probability >= 1` responde sim sem sortear — é o default,
/// e é por isso que ele não custa uma avalanche por instância.
fn fires(pulse: f32, p: &Params, row: usize, prev_seq: f32) -> bool {
    pulse > 0.5
        && (p.probability >= 1.0 || rand01(row as u32, prev_seq.max(0.0) as u32) < p.probability)
}

fn step(input: &Stream, pulse: &Stream, state: &Stream, p: &Params) -> Stream {
    let n = input.count();
    let pulses = scalar_col(pulse, PULSE_COL, n, 0.0);
    let prev_glow = scalar_col(state, GLOW_COL, n, 0.0);
    // A fase e a pista do sorteio. Um estado ausente (o 1º tick, ou um `pre` que
    // ninguém plumbou) nasce em [`NEVER`] — ver lá por que zero seria errado.
    let prev_age = scalar_col(state, AGE_COL, n, NEVER);
    let prev_seq = scalar_col(state, SEQ_COL, n, 0.0);
    // The focus field masks the APPLICATION (like every behaviour): a dot at
    // falloff 0 never visibly flashes. The envelope itself stays unmasked so a
    // falloff animated over a live glow fades the look, not the memory.
    let falloff = scalar_col(input, "falloff", n, 1.0);

    let mut glow = Vec::with_capacity(n);
    let mut age = Vec::with_capacity(n);
    let mut seq = Vec::with_capacity(n);
    for i in 0..n {
        let arrived = pulses[i] > 0.5;
        let lit = fires(pulses[i], p, i, prev_seq[i]);
        let (g, a) = glow_of(lit, prev_glow[i], prev_age[i], p);
        glow.push(g.clamp(0.0, 1.0));
        age.push(a);
        seq.push(prev_seq[i] + f32::from(arrived));
    }

    // Apply the lit look to the FRESH upstream geometry (never the state), so the
    // boost cannot compound; copy every other column through untouched.
    let mut size = vec2_col(input, "size", n, [1.0, 1.0]);
    let mut tint = vec4_col(input, "tint", n, [1.0, 1.0, 1.0, 1.0]);
    for i in 0..n {
        // A curva PRIMEIRO, a máscara depois: a curva é a forma do envelope no
        // tempo, o `falloff` é um peso espacial (ver o cabeçalho).
        let shaped = p.curve.as_ref().map_or(glow[i], |c| c.eval(glow[i]));
        let g = shaped * falloff[i].clamp(0.0, 1.0);
        let k = 1.0 + p.size_boost * g;
        size[i] = [size[i][0] * k, size[i][1] * k];
        // Lerp RGB toward the flash colour by amount·glow; alpha untouched (a
        // flash brightens, it does not change opacity).
        let a = p.flash_amount * g;
        for (channel, &target) in tint[i].iter_mut().zip(p.flash.iter()) {
            *channel += (target - *channel) * a;
        }
    }

    let mut out = Stream::new(n);
    for (name, col) in input.columns() {
        if name != "size" && name != "tint" {
            out.set(name.clone(), col.clone());
        }
    }
    out.set("size", Column::Vec2(size));
    out.set("tint", Column::Vec4(tint));
    out.set(GLOW_COL, Column::Scalar(glow));
    out.set(AGE_COL, Column::Scalar(age));
    out.set(SEQ_COL, Column::Scalar(seq));
    out
}

fn scalar_col(s: &Stream, name: &str, n: usize, id: f32) -> Vec<f32> {
    let mut v = match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, id);
    v
}
fn vec2_col(s: &Stream, name: &str, n: usize, id: [f32; 2]) -> Vec<[f32; 2]> {
    let mut v = match s.get(name) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, id);
    v
}
fn vec4_col(s: &Stream, name: &str, n: usize, id: [f32; 4]) -> Vec<[f32; 4]> {
    let mut v = match s.get(name) {
        Some(Column::Vec4(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, id);
    v
}

struct MotionStrobe;

impl NodeOp for MotionStrobe {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let p = Params {
            decay: decay_per_tick(ctx.param("decay")),
            // Negativos são lixo de documento, não um pedido: um attack de −3
            // faria `age / attack` inverter o sinal do envelope inteiro.
            attack: ctx.param("attack").max(0.0),
            hold: ctx.param("hold").max(0.0),
            probability: ctx.param("probability").clamp(0.0, 1.0), // CLAMP-OK: uma chance vive em [0,1]
            curve: ctx.text_param(CURVE_KEY).and_then(ph2d_curve::parse),
            size_boost: ctx.param("size_boost"),
            flash: [
                ctx.param("flash_r"),
                ctx.param("flash_g"),
                ctx.param("flash_b"),
            ],
            flash_amount: ctx.param("flash_amount"),
        };
        let mut out = step(ctx.input(0), ctx.input(1), ctx.input(2), &p);
        // ⚠️ **Só escreve a coluna quando o artista escolheu** — `0` é *o modo do sink*, e
        // não escrever é o que mantém o default byte-idêntico.
        if let Some(tag) = flash_blend_tag(ctx.param(FLASH_BLEND)) {
            let n = out.count();
            out = out.with(BLEND_COLUMN, Column::Scalar(vec![tag; n]));
        }
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionStrobe))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Strobe",
            // FX magenta: a stylistic flash effect.
            category: ph2d_node_registry::NodeUiCategory::Fx,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_groups(MANIFEST.id, PARAM_GROUPS);
    // CPU-only: o strobe lê `falloff` só no `eval` (não há kernel de GPU de onde
    // uma `ColumnBinding` pudesse ser derivada) — declare (ADR-0155).
    reg.register_couplings(
        MANIFEST.id,
        &[ph2d_node_registry::Coupling::Consumes("falloff")],
    );
    Ok(())
}

use ph2d_node_registry::{ParamGroup, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget};

/// Os três trechos do envelope são contagens de TICKS — a unidade que o painel declara é
/// a que o número É, e é a mesma do `length`/`spacing` do `motion.trail`.
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "attack",
        unit: ParamUnit::Count,
    },
    ParamUnitDecl {
        param: "hold",
        unit: ParamUnit::Count,
    },
    ParamUnitDecl {
        param: "decay",
        unit: ParamUnit::Count,
    },
];

/// **Duas perguntas, duas seções** (doc 88): *que forma tem o flash no tempo?* e
/// *com que aparência ele acende?* — a `probability` fica com a primeira porque
/// ela decide QUANDO há flash, não como ele se parece.
static PARAM_GROUPS: &[ParamGroup] = &[
    ParamGroup::new("attack", "Envelope"),
    ParamGroup::new("hold", "Envelope"),
    ParamGroup::new("decay", "Envelope"),
    ParamGroup::new(CURVE_KEY, "Envelope"),
    ParamGroup::new("probability", "Envelope"),
    ParamGroup::new("size_boost", "Look"),
    ParamGroup::new("flash_r", "Look"),
];

/// **O OPERADOR DO FLASH** — irmão do `motion.trail::ECHO_BLEND`, e a célula da folha 07
/// pediu-os juntos: *"mesma causa do Echo Operator do trail (o blend é do renderer). Um
/// conserto serve os dois nós"*.
pub const FLASH_BLEND: &str = "flash_blend";

/// A coluna de convenção que o `ph2d_eval_motion` lê por LINHA (a mesma do rastro).
pub const BLEND_COLUMN: &str = "blend";

/// Os modos, na ordem das tags. ⚠️ **Copiados e não importados** — um nó é uma folha e não
/// alcança o `ph2d-ecs` nem o nó irmão; quem impede as listas de divergir é um gate na
/// shell, que vê os dois lados.
pub const FLASH_BLEND_LABELS: [&str; 7] = [
    // O primeiro NÃO é um modo: é a ausência de escolha.
    "Sink",
    "Normal",
    "Add",
    "Subtract",
    "Multiply",
    "Screen",
    "Premultiplied",
];

/// O valor da coluna para um `flash_blend` autorado — `None` quando ele é *o do sink*.
/// O índice do dropdown JÁ é `modo + 1` (o `Sink` ocupa o `0`).
#[must_use]
fn flash_blend_tag(v: f32) -> Option<f32> {
    if !v.is_finite() || v < 0.5 {
        return None;
    }
    #[expect(clippy::cast_precision_loss, reason = "sete rotulos")]
    let top = (FLASH_BLEND_LABELS.len() - 1) as f32;
    Some(v.round().clamp(1.0, top))
}

static PARAM_HINTS: &[ParamUiHint] = &[
    // ⚠️ O RÓTULO mudou de "Flash Length" para "Decay", e é correção: ele
    // descrevia o flash INTEIRO quando a queda era o flash inteiro. Com attack e
    // hold ao lado, o flash mede `attack + hold + decay` e este número é só a
    // cauda. O NOME de fio continua `decay` — renomeá-lo faria o `validate`
    // RECUSAR todo grafo salvo que o sobrescreve.
    ParamUiHint {
        param: "attack",
        label: "Attack",
        // Ticks até o pico. Mesma régua do Decay: o que fecha a faixa é a
        // PALAVRA (acima de dois segundos deixou de ser um strobe), não recurso.
        min: 0.0,
        max: 120.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "hold",
        label: "Hold",
        min: 0.0,
        max: 120.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "decay",
        label: "Decay",
        // Ticks. `0` é o flash de um tick; a 120 (dois segundos a 60 Hz) a coisa deixou
        // de ser um strobe e virou um fade — não há recurso a limitar aqui, o que se
        // fecha é a PALAVRA. Um documento pode carregar mais, e a derivação o honra.
        min: 0.0,
        max: 120.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    // A FORMA do envelope — um TEXT param (`CURVE_KEY`), não um `ParamSpec`: o
    // editor de curva arrastável. Não-setada = identidade (a lei do `value.curve`).
    ParamUiHint {
        param: CURVE_KEY,
        label: "Shape",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Curve,
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
        param: "size_boost",
        label: "Size Boost",
        min: 0.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    // ⚠️ `Enum`, nunca slider: uma tag é um NOME (a mesma lei do `Blend` do sink).
    ParamUiHint {
        param: FLASH_BLEND,
        label: "Flash Operator",
        min: 0.0,
        #[expect(clippy::cast_precision_loss, reason = "sete rotulos")]
        max: (FLASH_BLEND_LABELS.len() - 1) as f32,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &FLASH_BLEND_LABELS,
        },
    },
    // The flash colour authored as one swatch → OKLCH picker (the canonical
    // colour UI, like `motion.tint`), driving the three linear channels with
    // `flash_amount` riding the picker's 4th (alpha) slot as the intensity.
    // No standalone `flash_amount` row: the params bridge SUPPRESSES the row of
    // any param folded into a Color group (`motion_bridge_params.rs`,
    // `consumed`), so a dedicated Slider hint here would be dead code.
    ParamUiHint {
        param: "flash_r",
        label: "Flash",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Color {
            channels: ["flash_r", "flash_g", "flash_b", "flash_amount"],
        },
    },
];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
