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
//! ⚠️ **NÃO construído nesta wave, com o motivo:** o `hueShift` (a *cauda colorida* do
//! mesmo catálogo). Ele é barato como aritmética e caro como DECISÃO: girar matiz em
//! RGB linear com uma matriz YIQ é o atalho que todo motor de partícula usa e que este
//! app **não** usa em lugar nenhum — a cor aqui passa por OKLCH (`ph2d-color`, o picker,
//! o gradiente, a paleta). Um segundo modelo de matiz dentro de um nó de rastro é uma
//! segunda resposta a *"o que é girar uma cor"*, e ela divergiria no único lugar onde
//! ninguém lê um número. Quando entrar, entra pela porta que já existe.
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
        ParamSpec {
            name: "fade",
            default: 0.72,
        },
        ParamSpec {
            name: "shrink",
            default: 0.94,
        },
        // Um eco a cada N ticks. `1` = um por tick, o rastro contínuo que sempre shipou.
        ParamSpec {
            name: "spacing",
            default: 1.0,
        },
        // Graus de MATIZ por eco — a "cauda colorida" da referência. `0` = identidade.
        ParamSpec {
            name: "hue_shift",
            default: 0.0,
        },
        // Multiplicador de SATURAÇÃO por eco. `1` = identidade; `0` desbota a cinza.
        ParamSpec {
            name: "saturation",
            default: 1.0,
        },
        // Graus de GIRO por eco — o irmão rotacional do `shrink`. Nenhuma das referências
        // tem: é o "e mais alguns" da ordem de 2026-08-08.
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

/// **Tudo o que um eco sofre ao envelhecer UM tick**, num lugar só.
///
/// ⚠️ Todos são operadores **por TICK**, a forma que o `fade` e o `shrink` já tinham, e é
/// ela que dá de graça a semântica *"por eco"* que as referências descrevem: um eco de
/// `n` ticks recebeu cada operador exatamente `n` vezes. Um knob no neutro é a identidade
/// **ao bit** — nenhum deles toca um byte no default.
#[derive(Copy, Clone, Debug)]
pub struct Decay {
    /// Multiplicador de alfa por tick.
    pub fade: f32,
    /// Multiplicador de tamanho por tick.
    pub shrink: f32,
    /// Graus de matiz por tick (rotação que preserva a luma, em RGB linear).
    pub hue_shift: f32,
    /// Multiplicador de saturação por tick.
    pub saturation: f32,
    /// Graus somados ao `rot` por tick.
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

    /// Envelhece um conjunto de linhas carregadas em UM tick.
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
    decay.apply(&mut carried);

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
    ParamUiHint {
        param: "fade",
        label: "Fade",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "shrink",
        label: "Shrink",
        min: 0.5,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "hue_shift",
        label: "Hue Shift",
        // Uma volta inteira para cada lado: o eco pode dar a volta no círculo de matiz,
        // que é exatamente o arco-íris da "cauda colorida" da referência.
        min: -180.0,
        max: 180.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "saturation",
        label: "Saturation",
        // Abaixo de 1 a cauda desbota a cinza; acima ela satura — as duas direções são
        // usadas (fumaça × brasa), então a faixa não pode parar na identidade.
        min: 0.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "spin",
        label: "Spin",
        min: -45.0,
        max: 45.0,
        step: 0.5,
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

/// O espaçamento é uma contagem de TICKS e os dois ângulos são GRAUS — a unidade que o
/// painel declara é a que o número É.
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "spacing",
        unit: ParamUnit::Count,
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
mod tests {
    use super::*;

    fn dot(x: f32, alpha: f32) -> Stream {
        Stream::new(1)
            .with("P", Column::Vec2(vec![[x, 0.0]]))
            .with("tint", Column::Vec4(vec![[1.0, 1.0, 1.0, alpha]]))
            .with("size", Column::Vec2(vec![[1.0, 1.0]]))
    }

    fn xs(s: &Stream) -> Vec<f32> {
        match s.get("P").unwrap() {
            Column::Vec2(v) => v.iter().map(|p| p[0]).collect(),
            _ => panic!(),
        }
    }
    fn alphas(s: &Stream) -> Vec<f32> {
        match s.get("tint").unwrap() {
            Column::Vec4(v) => v.iter().map(|c| c[3]).collect(),
            _ => panic!(),
        }
    }

    fn trail_ages(s: &Stream) -> Vec<f32> {
        match s.get(AGE) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => vec![],
        }
    }

    /// Roda `ticks` ticks de um ponto que anda em x, devolvendo a saída final.
    fn run(ticks: usize, length: f32, spacing: f32) -> Stream {
        let mut state = Stream::new(0);
        for t in 0..ticks {
            state = step(
                &dot(t as f32, 1.0),
                &state,
                length,
                Decay::new(1.0, 1.0),
                spacing,
            );
        }
        state
    }

    /// **O ECO FICA ESPAÇADO** (doc 88 §B3 — a família ECHO).
    ///
    /// ⚠️ Nasceu VERMELHO: o motor promovia a cabeça a fantasma em TODO tick, então um
    /// rastro discreto — o *sprite echo* que a referência traz com `spacing 2` no default
    /// dela — não era difícil, era **inexprimível**. O oráculo é a CADÊNCIA (as idades
    /// vivas), não a contagem: é ela que o param nomeia, e um espaçamento descartado a
    /// deixa em `1,2,3…` para sempre.
    #[test]
    fn the_spacing_lays_one_ghost_every_n_ticks() {
        // 3 ecos com espaçamento 2: as idades vivas caminham 1 · 3 · 5, e a cabeça é 0.
        let out = run(8, 3.0, 2.0);
        let mut ages = trail_ages(&out);
        ages.sort_by(f32::total_cmp);
        assert_eq!(
            ages,
            vec![0.0, 1.0, 3.0],
            "as idades tinham de andar de dois em dois — em 1,2 o espaçamento foi \
             descartado"
        );
        // ⚠️ TRES linhas, como em `length = 3` sem espaçamento: o que muda é o ARCO que
        // elas cobrem (os ticks 4 e 6 em vez de 5 e 6), nunca quantas são.
        assert_eq!(xs(&out), vec![4.0, 6.0, 7.0]);
    }

    /// **`spacing = 1` É O MOTOR QUE JÁ SHIPAVA, AO BIT.**
    ///
    /// A regressão que importa: todo grafo autorado antes desta wave não declara
    /// `spacing`, e o `ctx.param` devolve o default. Com `s = 1` a faixa `1..1` é vazia,
    /// a promoção acontece sempre e a janela de descarte volta a ser `k`.
    #[test]
    fn a_spacing_of_one_is_the_continuous_trail_to_the_bit() {
        let out = run(6, 3.0, 1.0);
        assert_eq!(xs(&out), vec![3.0, 4.0, 5.0]);
        assert_eq!(trail_ages(&out), vec![2.0, 1.0, 0.0]);
        // A faixa vazia: nenhuma idade impede a promoção.
        assert!(promotes_head(&[1.0, 2.0, 7.0], 1));
        assert!(promotes_head(&[], 4));
        assert!(
            !promotes_head(&[2.0], 4),
            "um fantasma novo demais SEGURA a promocao"
        );
        assert!(promotes_head(&[4.0], 4), "alcancado o espacamento, promove");
    }

    /// **O PARAM AUTORADO ATRAVESSA O COOK.**
    ///
    /// ⚠️ Toda a suíte acima dirige `step` direto — ela prova a LEI e é **cega** a um
    /// `ctx.param` que ninguém chamou. Uma capacidade sem porta passa em todo gate que só
    /// olha para a função pura, então este dirige o grafo REAL, com o `pre` self-loop que
    /// o editor plumba, e mede a cadência na saída do cook.
    #[test]
    fn the_authored_spacing_reaches_the_node_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.trail.test.src"),
            name: "motion.trail.test.src",
            inputs: &[],
            outputs: &[PortSpec {
                name: "out",
                ty: INST_VEC2,
            }],
            effect: Effect::Pure,
            clock: Clock::Frame,
            params: &[],
            lowerings: &[LoweringKind::Cpu],
        };
        struct Src;
        impl NodeOp for Src {
            fn manifest(&self) -> &'static NodeManifest {
                &SRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                // Um ponto que anda com o relógio: o x de cada eco DIZ de que tick ele é.
                let x = ctx.playhead() as f32;
                ctx.emit(dot(x, 1.0));
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionTrail),
                    _ => None,
                }
            }
        }

        let mut g = Graph::new();
        let src = g.add_node("motion.trail.test.src");
        let tr = g.add_node("motion.trail");
        g.connect(Edge {
            from: (src, 0),
            to: (tr, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (tr, 0),
            to: (tr, 1),
            delayed: true,
        })
        .unwrap();
        g.set_param(tr, "length", 3.0);
        g.set_param(tr, "fade", 1.0);
        g.set_param(tr, "shrink", 1.0);
        g.set_param(tr, "spacing", 2.0);

        let mut cook = Cook::new();
        let mut last = Stream::new(0);
        for t in 0..8 {
            let out = cook.cook(&g, &Ops, tr, f64::from(t)).unwrap();
            last = out[0].as_stream().clone();
            cook.advance_tick(&g, &Ops, f64::from(t)).unwrap();
        }
        let mut ages = trail_ages(&last);
        ages.sort_by(f32::total_cmp);
        assert_eq!(
            ages,
            vec![0.0, 1.0, 3.0],
            "o `spacing` autorado no grafo não chegou ao nó"
        );
    }

    /// **CADA knob autorado atravessa o cook** — a porta, não a lei.
    ///
    /// ⚠️ O gate acima dirige só o `spacing`, e uma capacidade sem porta passa em todo
    /// gate que olha para a função pura: bastaria um `ctx.param` esquecido para um knob
    /// nascer morto com a suíte verde. Este dirige os SETE pelo grafo REAL, um a um,
    /// exigindo que mover cada um mude a saída — e que o conjunto no default não a mude.
    #[test]
    fn every_authored_knob_reaches_the_node_through_the_cook() {
        let baseline = cooked_with(&[]);
        for (name, value) in [
            ("length", 5.0),
            ("fade", 0.3),
            ("shrink", 0.5),
            ("spacing", 3.0),
            ("hue_shift", 90.0),
            ("saturation", 0.0),
            ("spin", 30.0),
        ] {
            let moved = cooked_with(&[(name, value)]);
            assert!(
                moved != baseline,
                "mover `{name}` para {value} não mudou nada na saída do cook —                  o param não chegou ao nó"
            );
        }
    }

    /// **A CAUDA MUDA DE COR, SATURA E GIRA** — o padrão-ouro pedido em 2026-08-08.
    ///
    /// Um oráculo por knob, cada um medindo a grandeza que o knob NOMEIA:
    /// - matiz: a cor do eco velho difere da do vivo **e a luma se conserva** (é o que
    ///   separa um giro de matiz de um filtro que escurece);
    /// - saturação: a distância de cada canal à luma ENCOLHE com a idade;
    /// - giro: o `rot` do eco velho está `n · spin` atrás do vivo.
    #[test]
    fn the_tail_shifts_hue_desaturates_and_spins() {
        let coloured = |x: f32| {
            Stream::new(1)
                .with("P", Column::Vec2(vec![[x, 0.0]]))
                .with("tint", Column::Vec4(vec![[0.9, 0.2, 0.1, 1.0]]))
                .with("size", Column::Vec2(vec![[1.0, 1.0]]))
        };
        let decay = Decay {
            fade: 1.0,
            shrink: 1.0,
            hue_shift: 40.0,
            saturation: 0.6,
            spin: 7.0,
        };
        let mut state = Stream::new(0);
        for t in 0..4 {
            state = step(&coloured(t as f32), &state, 4.0, decay, 1.0);
        }
        let tints = match state.get("tint").unwrap() {
            Column::Vec4(v) => v.clone(),
            _ => panic!("tint"),
        };
        let luma = |c: [f32; 4]| 0.213 * c[0] + 0.715 * c[1] + 0.072 * c[2];
        let spread = |c: [f32; 4]| {
            let l = luma(c);
            (c[0] - l).abs() + (c[1] - l).abs() + (c[2] - l).abs()
        };
        let (old_c, live) = (tints[0], tints[tints.len() - 1]);
        assert!(
            (old_c[0] - live[0]).abs() > 0.05 || (old_c[1] - live[1]).abs() > 0.05,
            "o eco velho tinha de estar noutra matiz: {old_c:?} vs {live:?}"
        );
        assert!(
            (luma(old_c) - luma(live)).abs() < 1e-3,
            "e na MESMA luma: {} vs {}",
            luma(old_c),
            luma(live)
        );
        assert!(
            spread(old_c) < spread(live) * 0.5,
            "a saturacao tinha de cair com a idade: {} vs {}",
            spread(old_c),
            spread(live)
        );
        match state.get("rot").unwrap() {
            Column::Scalar(v) => {
                assert!((v[v.len() - 1] - 0.0).abs() < 1e-6, "o vivo nao girou");
                assert!(
                    (v[0] - 3.0 * 7.0).abs() < 1e-4,
                    "o eco de 3 ticks: {}",
                    v[0]
                );
            }
            _ => panic!("rot"),
        }
    }

    /// **Os três knobs no NEUTRO não tocam um byte** — nem a coluna `rot`, que só nasce
    /// quando o artista pede giro: materializá-la sem pedido acrescentaria uma coluna a
    /// todo rastro do app.
    #[test]
    fn the_neutral_colour_knobs_change_nothing_and_add_no_column() {
        let coloured = Stream::new(1)
            .with("P", Column::Vec2(vec![[0.0, 0.0]]))
            .with("tint", Column::Vec4(vec![[0.9, 0.2, 0.1, 1.0]]));
        let mut a = Stream::new(0);
        let mut b = Stream::new(0);
        for _ in 0..4 {
            a = step(&coloured, &a, 4.0, Decay::new(0.8, 0.9), 1.0);
            b = step(
                &coloured,
                &b,
                4.0,
                Decay {
                    ..Decay::new(0.8, 0.9)
                },
                1.0,
            );
        }
        match (a.get("tint"), b.get("tint")) {
            (Some(Column::Vec4(x)), Some(Column::Vec4(y))) => assert_eq!(x, y),
            _ => panic!("tint"),
        }
        assert!(a.get("rot").is_none(), "sem `spin` nao nasce coluna `rot`");
    }

    /// Cozinha o grafo REAL (com o `pre` self-loop) por 8 ticks e devolve um retrato
    /// comparável da saída: contagem + toda coluna que o rastro pode tocar.
    fn cooked_with(params: &[(&str, f32)]) -> Vec<String> {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.trail.test.src2"),
            name: "motion.trail.test.src2",
            inputs: &[],
            outputs: &[PortSpec {
                name: "out",
                ty: INST_VEC2,
            }],
            effect: Effect::Pure,
            clock: Clock::Frame,
            params: &[],
            lowerings: &[LoweringKind::Cpu],
        };
        struct Src;
        impl NodeOp for Src {
            fn manifest(&self) -> &'static NodeManifest {
                &SRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                let x = ctx.playhead() as f32;
                ctx.emit(
                    Stream::new(1)
                        .with("P", Column::Vec2(vec![[x, 0.0]]))
                        .with("tint", Column::Vec4(vec![[0.9, 0.2, 0.1, 1.0]]))
                        .with("size", Column::Vec2(vec![[1.0, 1.0]])),
                );
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionTrail),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let src = g.add_node("motion.trail.test.src2");
        let tr = g.add_node("motion.trail");
        g.connect(Edge {
            from: (src, 0),
            to: (tr, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (tr, 0),
            to: (tr, 1),
            delayed: true,
        })
        .unwrap();
        for (k, v) in params {
            g.set_param(tr, *k, *v);
        }
        let mut cook = Cook::new();
        let mut last = Stream::new(0);
        for t in 0..8 {
            last = cook.cook(&g, &Ops, tr, f64::from(t)).unwrap()[0]
                .as_stream()
                .clone();
            cook.advance_tick(&g, &Ops, f64::from(t)).unwrap();
        }
        let mut out = vec![format!("n={}", last.count())];
        for (name, col) in last.columns() {
            out.push(format!("{name}={col:?}"));
        }
        out.sort();
        out
    }

    /// O teto do espaçamento é um recurso (a janela de idade é `length × spacing`), e um
    /// `f32` de documento é intocado — não-finito e negativo caem no contínuo.
    #[test]
    fn the_spacing_is_clamped_totally() {
        assert_eq!(spacing_of(f32::NAN), 1);
        assert_eq!(spacing_of(-3.0), 1);
        assert_eq!(spacing_of(0.4), 1);
        assert_eq!(spacing_of(2.6), 3);
        assert_eq!(spacing_of(1e9), MAX_SPACING);
    }

    /// A moving dot, run for several ticks with the previous output fed back
    /// (exactly what the `pre` self-loop does). The trail must hold the last
    /// `length` positions, oldest first, each dimmer than the next.
    #[test]
    fn the_echo_holds_the_last_n_positions_oldest_first() {
        let mut state = Stream::new(0);
        for t in 0..6 {
            state = step(&dot(t as f32, 1.0), &state, 3.0, Decay::new(0.5, 1.0), 1.0);
        }
        // Ticks 3, 4, 5 survive; the live head (x=5) is LAST so it draws on top.
        assert_eq!(xs(&state), vec![3.0, 4.0, 5.0]);
        // Geometric fade: the head is opaque, each older echo halved once more.
        assert_eq!(alphas(&state), vec![0.25, 0.5, 1.0]);
    }

    /// FALSIFICATION of the geometric decay: the fade must be applied ONCE per
    /// tick to the carried rows, never recomputed from the age. Re-deriving it
    /// (`alpha = fade^age`) reads the same here — but re-applying it to an
    /// already-faded row would give `0.5^2` on a one-tick-old echo. Pin the
    /// head at full alpha and the one-tick echo at exactly `fade`.
    #[test]
    fn a_one_tick_old_echo_has_faded_exactly_once() {
        let s = step(
            &dot(0.0, 1.0),
            &Stream::new(0),
            4.0,
            Decay::new(0.5, 1.0),
            1.0,
        );
        let s = step(&dot(1.0, 1.0), &s, 4.0, Decay::new(0.5, 1.0), 1.0);
        assert_eq!(alphas(&s), vec![0.5, 1.0]);
        assert_eq!(xs(&s), vec![0.0, 1.0]);
    }

    #[test]
    fn shrink_compounds_and_length_one_is_the_identity() {
        let mut state = Stream::new(0);
        for t in 0..3 {
            state = step(&dot(t as f32, 1.0), &state, 3.0, Decay::new(1.0, 0.5), 1.0);
        }
        match state.get("size").unwrap() {
            // Two ticks of 0.5, one tick, then the untouched head.
            Column::Vec2(v) => assert_eq!(
                v.iter().map(|s| s[0]).collect::<Vec<_>>(),
                vec![0.25, 0.5, 1.0]
            ),
            _ => panic!(),
        }

        // length = 1 → the live stream, verbatim, with no `trail_age` added.
        let live = dot(7.0, 1.0);
        let out = step(&live, &state, 1.0, Decay::new(0.5, 0.5), 1.0);
        assert_eq!(out.count(), 1);
        assert_eq!(xs(&out), vec![7.0]);
        assert!(out.get(AGE).is_none(), "the identity adds no column");
    }

    /// A run of ticks must not grow without bound: the ring drops the oldest
    /// generation every tick, so the count settles at `length × live`.
    #[test]
    fn the_element_count_settles_at_length_times_live() {
        let mut state = Stream::new(0);
        for t in 0..50 {
            state = step(&dot(t as f32, 1.0), &state, 8.0, Decay::new(0.9, 0.99), 1.0);
        }
        assert_eq!(state.count(), 8, "8 generations of a single dot");
    }

    /// `length` and the live count are both untrusted (a loaded document, an
    /// MCP edit). The instance budget clamps the GENERATIONS, so the trail gets
    /// shorter rather than allocating 4096 × 32 quads or emitting a half-drawn
    /// echo.
    #[test]
    fn the_instance_budget_clamps_generations_not_rows() {
        assert_eq!(generations(8.0, 100), 8);
        assert_eq!(generations(999.0, 1), MAX_LENGTH, "hard ceiling");
        assert_eq!(generations(f32::NAN, 10), 1, "junk → the identity");
        assert_eq!(generations(-5.0, 10), 1);
        // 32 generations × 4096 live = 131k > the 65_536 budget → 16 kept.
        assert_eq!(generations(32.0, 4096), MAX_INSTANCES / 4096);
        assert_eq!(generations(32.0, 999_999), 1, "never zero rows");
    }

    /// **UM STREAM POSICIONAL PURO DESBOTA E ENCOLHE** — o defeito de 2026-08-08.
    ///
    /// ⚠️ **Este gate PINAVA o bug.** Ele afirmava `state.get("tint").is_none()` e
    /// explicava, no próprio doc-comment, que *"o fade/shrink simplesmente não têm o que
    /// tocar"* — descrevendo como comportamento aquilo que o smoke reportou como
    /// *"Fade e Shrink não têm efeito algum"*. Medido na cena real: as colunas eram
    /// `["Count", "Index", "P", "trail_age"]`, e um `motion.grid` — a fonte mais comum
    /// que existe — não carrega nenhuma das duas.
    ///
    /// *Um gate que descreve o sintoma como contrato mantém o defeito vivo com a suíte
    /// verde.* Agora ele afirma o que o artista vê.
    #[test]
    fn a_bare_positional_stream_fades_and_shrinks() {
        let bare = |x: f32| Stream::new(1).with("P", Column::Vec2(vec![[x, 0.0]]));
        let mut state = Stream::new(0);
        for t in 0..4 {
            state = step(&bare(t as f32), &state, 3.0, Decay::new(0.5, 0.5), 1.0);
        }
        assert_eq!(xs(&state), vec![1.0, 2.0, 3.0]);
        // A cauda: dois ticks de meia-vida atrás do vivo.
        assert_eq!(alphas(&state), vec![0.25, 0.5, 1.0]);
        match state.get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[0.25, 0.25], [0.5, 0.5], [1.0, 1.0]]),
            _ => panic!("size"),
        }
    }

    /// **E a materialização é a IDENTIDADE da lowering** — o primeiro tick não muda um
    /// pixel, que é o que torna a cura segura para toda arte já autorada.
    #[test]
    fn the_materialised_columns_carry_the_lowerings_own_defaults() {
        let bare = Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]]));
        let out = step(&bare, &Stream::new(0), 4.0, Decay::new(0.5, 0.5), 1.0);
        assert_eq!(
            alphas(&out),
            vec![1.0, 1.0],
            "opaco, como a ausência significava"
        );
        match out.get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![SIZE_IDENTITY; 2]),
            _ => panic!("size"),
        }
    }
}
