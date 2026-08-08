//! `motion.trail` — an echo trail: every element leaves N fading, shrinking
//! copies of where it was on the previous ticks (Motion Nodes M2, plan §1.3).
//!
//! Comet tails, motion smears, cursor sparkle wakes. The reference catalogue's
//! `trail`: *"Rastro de cometa, trail de cursor, partícula com cauda colorida"*.
//!
//! **The ring buffer IS the value of the self-loop.** The plan reaches for a
//! `History { slots, head, len }` carried as an opaque value; it turns out the
//! stream itself is already that ring. Each tick the node emits
//!
//! ```text
//! out = carried(previous out) ++ live
//! ```
//!
//! where `carried` drops the generation that just aged past `length` and
//! multiplies the survivors' alpha by `fade` and their size by `shrink`. The
//! decay is therefore **geometric** — applied once per tick to the rows carried
//! forward — so it can never double-count, and the state needs no pristine copy
//! of the original colour. The `state` port (the substrate's sequential-node
//! convention) gets its `pre` self-loop wired by the editor, exactly like
//! `spring` and `integrate`.
//!
//! Carried rows draw FIRST so the live head paints on top of its own tail.
//!
//! ## O ESPAÇAMENTO (doc 88 §B3 — a varredura PRO da família ECHO)
//!
//! Um fantasma por TICK é um rastro contínuo: a 60 fps ele lê como borrão, e o eco
//! discreto — o *sprite echo* que o catálogo de referência nomeia, com `spacing 2` no
//! default DELE — era **inexprimível**, porque o motor promovia a cabeça a fantasma em
//! todo tick e não havia knob que dissesse o contrário.
//!
//! ⚠️ **E ele não precisou de estado novo, o que é o desenho inteiro:** a coluna
//! `trail_age` já carrega tudo. A cabeça (`age 0`) só é PROMOVIDA a fantasma quando
//! **nenhuma linha ocupa a faixa `1..spacing`** — ou seja, quando o fantasma mais novo
//! já tem a idade do espaçamento. Simulando `spacing = 2`, as idades vivas caminham
//! `1 · 3 · 5 …`: exatamente um eco a cada dois ticks. Com `spacing = 1` a faixa
//! `1..1` é VAZIA, a promoção acontece sempre e a janela de descarte volta a ser `k` —
//! a expressão que já shipava, **byte a byte**.
//!
//! ## O QUE UM KNOB DE DECAIMENTO SIGNIFICA (doc 88 §B3, report do Enio de 2026-08-08)
//!
//! ⚠️ Os cinco knobs de decaimento são **ALVOS na ponta da cauda**, nunca taxas por tick.
//! *"Saturate To 0.5"* quer dizer *o eco mais velho tem metade da saturação*, e continua
//! querendo dizer isso quando o Length e o Spacing mudam.
//!
//! O motor continua **geométrico** — uma aplicação por tick sobre as linhas carregadas —,
//! o que muda é de onde vem a taxa: ela é DERIVADA do alvo (`rate^span == target`, com
//! `span = (length − 1) × spacing`, a idade do eco mais velho). O smoke reprovou a forma
//! anterior, em que o knob **era** a taxa, e a medição diz por quê:
//!
//! - a resposta era **exponencial no slider** — na esteira do smoke (`length 6`,
//!   `spacing 4`, span 20) `saturation = 0.90` produzia **0.17** na ponta, e a faixa útil
//!   inteira do controle cabia em **5,2%** do curso dele (1,9% a `spacing 8`);
//! - e o **`spacing` MULTIPLICAVA todo decaimento**: `fade = 0.80` dava 0.21 no default e
//!   **0.0010** em `length 32` — cauda invisível sem nada no controle dizendo isso. A nota
//!   que esta seção trazia (*"um eco de `n` ticks recebeu o operador `n` vezes, e a
//!   semântica «por eco» cai de graça"*) só era verdade em `spacing = 1`.
//!
//! É o modelo `satMin/satMax` do catálogo de referência (um ESTADO FINAL, não uma taxa) —
//! que a wave anterior tinha citado e não seguido.
//!
//! Two things to know when placing it:
//! - It **duplicates `id`s** (an echo shares its source's identity), so it
//!   belongs *downstream* of anything that pairs state by id — put it after
//!   `motion.integrate`, never before.
//! - It is **sequential** (it reads a `pre`), so it may not sit upstream of a
//!   `motion.time_remap`: the editor refuses that wire, and the cook would
//!   refuse it too (`CookError::SequentialInTimeScope`).

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, SIZE_IDENTITY, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod carry;
mod colour;
use carry::{add_scalar, ages, concat, fade_alpha, gather, scale_vec2, tint_op};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The column holding how many ticks ago a row was live. `0` = the live head.
/// Namespaced away from the emitter's own `age` (a particle's lifetime), which
/// means something else entirely and must survive untouched.
const AGE: &str = "trail_age";

/// A cor que a lowering assume quando não há coluna `tint` — branco opaco
/// (`ph2d-eval-motion::lower`, `vec4_at(tint, i, [1,1,1,1])`).
const TINT_IDENTITY: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

/// **Materializa as colunas que o rastro DESBOTA.**
///
/// ⚠️ Este é o defeito que o smoke de 08/08 reportou como *"Fade e Shrink não têm efeito
/// algum"*, e o mecanismo é exato: `fade_alpha`/`scale_vec2` multiplicam uma coluna que
/// **precisa existir**, e um stream posicional puro — um `motion.grid`, o caso mais comum
/// que existe — não carrega `tint` nem `size`. Medido na cena do smoke: as colunas eram
/// `["Count", "Index", "P", "trail_age"]`. Os dois knobs eram no-ops silenciosos, com a
/// lowering desenhando todo fantasma opaco e do mesmo tamanho.
///
/// A cura é a que o `motion.scale` já usa para o `size`: **começar da identidade que a
/// própria lowering assume**, o que torna o primeiro tick byte-idêntico no render (a
/// coluna passa a existir carregando exatamente o valor que a ausência dela significava)
/// e dá aos ticks seguintes o que multiplicar.
fn materialize_render_columns(s: &mut Stream) {
    let n = s.count();
    if s.get("size").is_none() {
        s.set("size", Column::Vec2(vec![SIZE_IDENTITY; n]));
    }
    if s.get("tint").is_none() {
        s.set("tint", Column::Vec4(vec![TINT_IDENTITY; n]));
    }
}

/// Hard ceiling on the echo count, independent of the `length` param's own
/// slider range — a document loaded from disk (or authored over MCP) can carry
/// any `f32`.
const MAX_LENGTH: usize = 32;

/// Hard ceiling on the emitted element count. `length × live` is an allocation
/// size driven by an untrusted param times an untrusted upstream: 4096 live
/// particles × 32 echoes is already 131k quads. The trail is CLAMPED (fewer
/// generations), never truncated mid-generation — a half-drawn echo reads as a
/// bug, a shorter tail reads as a setting.
const MAX_INSTANCES: usize = 65_536;

/// Teto do espaçamento: `length × spacing` é a janela de IDADE que o nó carrega, e um
/// `f32` vindo de um documento é intocado. 16 ticks entre ecos já é um rastro de flip-book.
const MAX_SPACING: usize = 16;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.trail"),
    name: "motion.trail",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // The feedback port: last tick's output IS the ring buffer. The editor
        // plumbs its `pre` self-loop on drop (the `state` convention).
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
        // A dropped Trail must SHOW something (an inert default reads as a
        // broken node), so it lands at 8 echoes. `length = 1` is the identity.
        ParamSpec {
            name: "length",
            default: 8.0,
        },
        // ⚠️ Os defaults de `fade`/`shrink` NÃO foram escolhidos: eles são o que as taxas
        // que já shipavam (0.72 e 0.94 por tick) PRODUZIAM na ponta no default do nó
        // (`length 8`, `spacing 1`, span 7) — `0.72^7 = 0.1003` e `0.94^7 = 0.6485`. O
        // rastro no default é o mesmo de antes; o que muda é que agora ele CONTINUA o
        // mesmo quando o Length ou o Spacing se movem.
        // Alfa do eco mais VELHO, relativa à cabeça viva.
        ParamSpec {
            name: "fade",
            default: 0.10,
        },
        // Tamanho do eco mais VELHO, relativo à cabeça viva.
        ParamSpec {
            name: "shrink",
            default: 0.65,
        },
        // Um eco a cada N ticks. `1` = um por tick, o rastro contínuo que sempre shipou.
        ParamSpec {
            name: "spacing",
            default: 1.0,
        },
        // Graus de MATIZ que a cauda INTEIRA percorre — a "cauda colorida" da referência.
        ParamSpec {
            name: "hue_shift",
            default: 0.0,
        },
        // Saturação do eco mais VELHO. `1` = identidade; `0` desbota a cinza; `>1` satura.
        ParamSpec {
            name: "saturation",
            default: 1.0,
        },
        // Graus de GIRO que a cauda INTEIRA percorre — o irmão rotacional do `shrink`.
        // Nenhuma das referências tem: é o "e mais alguns" da ordem de 2026-08-08.
        ParamSpec {
            name: "spin",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// How many generations to keep, clamped by both the hard ceiling and the
/// instance budget. Non-finite / negative → 1 (the identity), never 0 rows.
fn generations(length: f32, live: usize) -> usize {
    let requested = if length.is_finite() && length >= 1.0 {
        (length.round() as usize).min(MAX_LENGTH)
    } else {
        1
    };
    if live == 0 {
        return requested;
    }
    requested.max(1).min(MAX_INSTANCES / live.max(1)).max(1)
}

/// Quantos ticks entre um eco e o seguinte. Não-finito / abaixo de 1 → `1` (o rastro
/// contínuo), e o teto existe porque `length × spacing` é a janela de idade: sem ele um
/// documento carregado do disco pediria uma cauda de milhares de ticks de memória.
fn spacing_of(spacing: f32) -> usize {
    if spacing.is_finite() && spacing >= 1.0 {
        (spacing.round() as usize).min(MAX_SPACING)
    } else {
        1
    }
}

/// **A cabeça do tick anterior vira fantasma?** Sim quando **nenhum fantasma já promovido
/// ocupa a faixa de idade `1..s`** — isto é, quando o mais novo deles já alcançou o
/// espaçamento (ou quando ainda não há nenhum).
///
/// ⚠️ É a porta ÚNICA do espaçamento, e ela pergunta ao ESTADO em vez de a um contador: a
/// coluna `trail_age` já sabe há quantos ticks o último eco foi deixado, então não há
/// segundo lugar onde a resposta possa divergir — nem estado novo a costurar no ciclo de
/// vida do nó. Com `s = 1` a faixa `1..1` é VAZIA, a resposta é sempre `true`, e o motor
/// é o que sempre shipou.
///
/// As idades aqui são as do estado (antes do bump), e `a >= 1.0` é o que separa um
/// fantasma da cabeça: a cabeça é sempre `age 0`.
fn promotes_head(prev_ages: &[f32], s: usize) -> bool {
    !prev_ages
        .iter()
        .any(|&a| a >= 1.0 && (a as usize) < s.max(1))
}

/// **O PISO de um alvo multiplicativo.**
///
/// ⚠️ Um alvo de exatamente zero faria a taxa ser zero, e a cauda inteira colapsaria no
/// PRIMEIRO eco — um penhasco onde o artista pediu uma rampa. O piso é **um nível de 8
/// bits**, abaixo do qual a ponta é indistinguível de zero na tela: o número é do
/// RENDERER, não escolhido. Com ele `Fade To 0` é uma rampa rápida e lisa até o invisível,
/// que é o que a palavra quer dizer.
const TARGET_FLOOR: f32 = 1.0 / 255.0;

/// A raiz `span`-ésima do alvo — a taxa por tick que o alcança na ponta.
///
/// ⚠️ `powf(1.0, _)` é `1.0` EXATO em IEEE-754 (a norma o especifica para todo expoente),
/// então o ponto neutro atravessa esta função **ao bit**, e é isso que mantém um rastro
/// sem knobs byte-idêntico ao que já shipava. Um alvo não-finito vindo de um documento
/// cai na identidade em vez de envenenar a cauda com `NaN`.
fn rate_for(target: f32, inv_span: f32) -> f32 {
    if !target.is_finite() {
        return 1.0;
    }
    libm::powf(target.max(TARGET_FLOOR), inv_span)
}

/// O passo angular por tick: o TOTAL que a cauda percorre, dividido pelo vão dela.
fn step_for(total: f32, span: u32) -> f32 {
    if !total.is_finite() {
        return 0.0;
    }
    total / span as f32
}

/// **Tudo o que um eco sofre ao longo da CAUDA INTEIRA**, num lugar só.
///
/// ⚠️ Os cinco campos são o que o artista AUTORA — o estado do eco mais VELHO, relativo à
/// cabeça viva (os multiplicativos) e o total percorrido (os angulares). Eles **não** são
/// taxas por tick: quem as deriva é [`Decay::per_tick`], e é essa derivação que torna o
/// número do slider independente do Length e do Spacing. Um knob no neutro é a identidade
/// **ao bit** — nenhum deles toca um byte no default.
#[derive(Copy, Clone, Debug)]
pub struct Decay {
    /// Alfa do eco mais velho, relativa à cabeça viva.
    pub fade: f32,
    /// Tamanho do eco mais velho, relativo à cabeça viva.
    pub shrink: f32,
    /// Graus de matiz que a cauda inteira percorre (rotação luma-preservante, RGB linear).
    pub hue_shift: f32,
    /// Saturação do eco mais velho, relativa à cabeça viva.
    pub saturation: f32,
    /// Graus de giro que a cauda inteira percorre.
    pub spin: f32,
}

impl Decay {
    /// O ponto neutro — todo operador na identidade.
    pub const NEUTRAL: Self = Self {
        fade: 1.0,
        shrink: 1.0,
        hue_shift: 0.0,
        saturation: 1.0,
        spin: 0.0,
    };

    /// Só o par que sempre existiu (para as fixtures que não falam de cor).
    #[must_use]
    pub fn new(fade: f32, shrink: f32) -> Self {
        Self {
            fade,
            shrink,
            ..Self::NEUTRAL
        }
    }

    /// **Converte os alvos AUTORADOS nas taxas POR TICK que os alcançam.**
    ///
    /// `span` é a idade do eco mais VELHO — `(length − 1) × spacing` —, então
    /// `rate^span == target` por construção: o que o artista digita é o que ele vê na
    /// ponta da cauda, e o número **não se move** quando o Length ou o Spacing mudam.
    ///
    /// ⚠️ O `span` sai do `k` **CLAMPADO** (o orçamento de instâncias pode encurtar a
    /// cauda), porque o alvo pertence à cauda que de fato existe. Se o `k` mudar no meio
    /// de um traço, as linhas já carregadas guardam o que a taxa anterior lhes deu e as
    /// novas seguem a nova — transitório, e ele se cura sozinho quando as velhas saem.
    #[must_use]
    fn per_tick(self, span: u32) -> Self {
        if span == 0 {
            return Self::NEUTRAL;
        }
        let inv = 1.0 / span as f32;
        Self {
            fade: rate_for(self.fade, inv),
            shrink: rate_for(self.shrink, inv),
            saturation: rate_for(self.saturation, inv),
            hue_shift: step_for(self.hue_shift, span),
            spin: step_for(self.spin, span),
        }
    }

    /// Envelhece um conjunto de linhas carregadas em UM tick. **Recebe as taxas já
    /// derivadas** — chamá-la com os alvos autorados é o defeito que o smoke de
    /// 2026-08-08 reportou.
    fn apply(self, carried: &mut Stream) {
        fade_alpha(carried, "tint", self.fade);
        scale_vec2(carried, "size", self.shrink);
        // ⚠️ UMA matriz para os dois operadores de cor: compor antes do laço deixa o
        // caminho por-linha com nove multiplicações, sejam zero, um ou dois knobs armados
        // — e no neutro a matriz É a identidade, então o `tint` não se move.
        let m = colour::compose(
            colour::hue_rotation(self.hue_shift),
            colour::saturation(self.saturation),
        );
        if m != colour::IDENTITY {
            tint_op(carried, "tint", m);
        }
        // ⚠️ Gateado em `!= 0`, ao contrário dos outros: o `rot` é MATERIALIZADO quando
        // ausente, e materializá-lo sem o artista ter pedido giro acrescentaria uma coluna
        // que ninguém pediu a todo rastro do app.
        if self.spin != 0.0 {
            add_scalar(carried, "rot", self.spin);
        }
    }
}

/// One tick of the echo: age last tick's rows, drop the generation that fell
/// off the end, decay the survivors, and put the live head in front.
fn step(live: &Stream, state: &Stream, length: f32, decay: Decay, spacing: f32) -> Stream {
    let k = generations(length, live.count());
    if k <= 1 {
        // A length-1 trail is the identity — forward the live stream untouched,
        // with no `trail_age` column to confuse a downstream reader.
        return live.clone();
    }
    let s = spacing_of(spacing);

    // Survivors: rows whose age, once bumped, still fits inside the age window.
    //
    // ⚠️ `length` conta LINHAS (a cabeça viva mais os fantasmas), e é o que ele sempre
    // contou — o gate `the_echo_holds_the_last_n_positions_oldest_first` pina três
    // posições para `length = 3`. Com espaçamento `s` os fantasmas pousam nas idades
    // `s−1, 2s−1, …`, então caber `k−1` deles é a janela `(k−1)·s + 1`; um `k·s` ingênuo
    // deixaria passar um fantasma A MAIS e `length` passaria a significar duas coisas
    // diferentes conforme o espaçamento. Em `s = 1` a expressão vale exatamente `k`.
    let prev_ages = ages(state, AGE);
    let window = (k - 1).saturating_mul(s) + 1;
    let promote = promotes_head(&prev_ages, s);
    let keep: Vec<usize> = (0..state.count())
        .filter(|&i| {
            let bumped = (prev_ages[i] as usize) + 1;
            // A cabeça do tick anterior (`age 0`) só sobrevive se for PROMOVIDA; os
            // fantasmas já promovidos apenas envelhecem.
            bumped < window && (prev_ages[i] >= 1.0 || promote)
        })
        .collect();
    let mut carried = gather(state, &keep);
    let bumped: Vec<f32> = keep.iter().map(|&i| prev_ages[i] + 1.0).collect();
    carried.set(AGE, Column::Scalar(bumped));
    // Geometric decay: applied once per tick to the rows carried forward, so a
    // row `n` ticks old has had it applied exactly `n` times. Nothing to undo.
    //
    // ⚠️ O VÃO é `window − 1` — a idade que o eco mais velho alcança —, e é ele que
    // converte os alvos autorados em taxas. Derivá-lo aqui, e não no `eval`, é o que faz
    // o alvo pertencer à cauda REAL: o `k` já passou pelo teto de instâncias.
    let span = u32::try_from(window - 1).unwrap_or(u32::MAX);
    decay.per_tick(span).apply(&mut carried);

    let mut head = live.clone();
    head.set(AGE, Column::Scalar(vec![0.0; live.count()]));
    // ⚠️ ANTES do concat: o head deste tick é o estado do próximo, então é aqui que as
    // colunas nascem — materializá-las só nos `carried` deixaria o primeiro fantasma de
    // cada geração sem nada a desbotar.
    materialize_render_columns(&mut head);
    // Tail first, head last: the live element paints over its own echoes.
    concat(&carried, &head)
}

struct MotionTrail;

impl NodeOp for MotionTrail {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let decay = Decay {
            fade: ctx.param("fade"),
            shrink: ctx.param("shrink"),
            hue_shift: ctx.param("hue_shift"),
            saturation: ctx.param("saturation"),
            spin: ctx.param("spin"),
        };
        let out = step(
            ctx.input(0),
            ctx.input(1),
            ctx.param("length"),
            decay,
            ctx.param("spacing"),
        );
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionTrail))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Trail",
            category: ph2d_node_registry::NodeUiCategory::Fx,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_groups(MANIFEST.id, PARAM_GROUPS);
    Ok(())
}

use ph2d_node_registry::{ParamGroup, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "length",
        label: "Length",
        min: 1.0,
        max: MAX_LENGTH as f32,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    // ⚠️ Os cinco rótulos abaixo dizem **"Tail"** porque o número é o estado da PONTA da
    // cauda, não uma taxa: é a diferença entre um slider linear no que se vê e o que o
    // smoke de 2026-08-08 reprovou. Um rótulo que dissesse só "Fade" deixaria o artista
    // adivinhar se 0.9 é por tick, por eco ou no fim — e as três respostas dão desenhos
    // que diferem por ordens de grandeza.
    ParamUiHint {
        param: "fade",
        label: "Tail Alpha",
        // Fechado pelo SIGNIFICADO, não por orçamento: a alfa é uma fração da cabeça viva,
        // e acima de 1 o fantasma ficaria mais opaco que a fonte dele.
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "shrink",
        label: "Tail Size",
        // ⚠️ Passa de 1 de propósito: abaixo é o cometa (a cauda afina), acima é a baforada
        // (a cauda ABRE). A lei antiga também permitia, mas exponencialmente — `1.1` por
        // tick virava 6,7× em 20 ticks; agora `2.0` é exatamente o dobro na ponta.
        min: 0.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "hue_shift",
        label: "Tail Hue Shift",
        // Uma volta INTEIRA para cada lado — o total que a cauda percorre. Além de 360°
        // ela repete matizes que já tem, então é onde a grandeza se fecha.
        min: -360.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "saturation",
        label: "Tail Saturation",
        // Abaixo de 1 a cauda desbota a cinza; acima ela satura — as duas direções são
        // usadas (fumaça × brasa), então a faixa não pode parar na identidade.
        min: 0.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "spin",
        label: "Tail Spin",
        // O mesmo fecho do matiz: a 360° a cauda completou uma revolução.
        min: -360.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "spacing",
        label: "Spacing",
        min: 1.0,
        // O teto do slider É o teto do recurso: acima dele a janela de idade cresceria
        // sem o eco aparecer, entao nao ha faixa confortavel a separar da legal.
        max: MAX_SPACING as f32,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
];

/// O espaçamento é uma contagem de TICKS, os dois ângulos são GRAUS, e os três alvos
/// multiplicativos são FRAÇÕES da cabeça viva — a unidade que o painel declara é a que o
/// número É.
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "spacing",
        unit: ParamUnit::Count,
    },
    ParamUnitDecl {
        param: "fade",
        unit: ParamUnit::Ratio,
    },
    ParamUnitDecl {
        param: "shrink",
        unit: ParamUnit::Ratio,
    },
    ParamUnitDecl {
        param: "saturation",
        unit: ParamUnit::Ratio,
    },
    ParamUnitDecl {
        param: "length",
        unit: ParamUnit::Count,
    },
    ParamUnitDecl {
        param: "hue_shift",
        unit: ParamUnit::Angle,
    },
    ParamUnitDecl {
        param: "spin",
        unit: ParamUnit::Angle,
    },
];

/// **As seções** (doc 88 §B3, a metade visual). Sete knobs numa coluna são uma parede;
/// agrupados eles viram três perguntas — *que forma tem a cauda* (solto, no topo), *como
/// ela morre* e *que cor ela toma*. ⚠️ `length`/`spacing` ficam FORA de seção de
/// propósito: um param sem entrada é pintado ANTES de tudo, que é onde os essenciais
/// devem estar (a lei do `ParamGroup`, e o padrão do Blender).
static PARAM_GROUPS: &[ParamGroup] = &[
    ParamGroup::new("fade", "Decay"),
    ParamGroup::new("shrink", "Decay"),
    ParamGroup::new("spin", "Decay"),
    ParamGroup::new("hue_shift", "Colour"),
    ParamGroup::new("saturation", "Colour"),
];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tail_target_tests.rs"]
mod tail_target_tests;
