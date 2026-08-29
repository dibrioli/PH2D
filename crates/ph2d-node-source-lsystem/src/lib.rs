#![forbid(unsafe_code)]
//! `source.lsystem` — **a estrutura recursiva**, o buraco que o catálogo não tinha
//! ([doc 92](../../../docs/Motion%20Nodes/92_o_que_o_mini_cavalry_tem_e_nos_nao.md) §2
//! item 1; plano [93](../../../docs/Motion%20Nodes/93_plano_lsystem_datasource_celanim.md) §2).
//!
//! Árvores, samambaias, corais, raios, ramificação: nada nos ~130 nós desta casa gerava
//! **estrutura que se reescreve**. Um L-System (Lindenmayer 1968) gera-a a partir de três
//! coisas — um axioma, um punhado de regras, e um número de gerações.
//!
//! # O que aqui está, e por que é o estado da arte e não um brinquedo
//!
//! As três dimensões do ABOP estão TODAS presentes, e cada uma compra uma classe de forma
//! que as outras não alcançam:
//!
//! | | exemplo | o que compra |
//! |---|---|---|
//! | **paramétrica** ([`grammar`], §1.10) | `A(s) -> F(s)![+A(s*0.7)][-A(s*0.7)]` | proporção: o galho encolhe, a planta converge |
//! | **estocástica** ([`derive::derive`], §1.7) | `F -> (0.4) F[+F]F ; F -> (0.6) FF` | duas plantas do mesmo grafo não são gémeas |
//! | **sensível a contexto** ([`derive`], §1.8) | `A < B > C -> D` | sinais que PERCORREM a planta (a flor abre de baixo para cima) |
//!
//! Mais, do lado do desenho: **tropismo** (a gravidade a curvar o ramo, §2.3.2), **espessura**
//! e **passo** que decaem por profundidade, o **corte** `%`, e as marcas de folha/flor
//! `J`/`K`/`M`. ⭐ E **gerações fraccionárias**, que é o que faz um `Generations` animado
//! CRESCER uma planta em vez de a fazer saltar entre inteiros — a razão de existir deste
//! módulo é a animação, e um gerador que só aceita inteiros não anima.
//!
//! ⚠️ **A expressão é a mesma do resto do app** (`ph2d-expr-parse`, ADR-0144). Não é
//! reutilização por economia: um segundo parser obrigaria o artista a aprender **duas**
//! linguagens no mesmo programa, e as duas divergiriam em silêncio.
//!
//! # O que sai: uma ÁRVORE (ver [`turtle`])
//!
//! `parent · len · rot · wrot · P` — o contrato de colunas do `rig.*`, tal e qual. Um
//! L-System entra na maquinaria de esqueleto da casa sem uma linha nova, e `source.lsystem →
//! rig.fk` é a **identidade ao bit** (gate em `ph2d-node-registry-init`).
//!
//! # Fronteiras NOMEADAS (nenhuma é preguiça)
//!
//! - ⛔ **`Effect::Pure`, nunca `Temporal`.** `Temporal` põe o playhead na impressão digital
//!   do memo e mata-o — recozeria a reescrita exponencial a 60 fps sem que nada tivesse
//!   mudado. O crescimento vem de FORA, animando `Generations`.
//! - ⛔ **CPU-only, e o bloqueador tem nome.** A rota de device exige uma `count_law` que
//!   devolva a contagem de saída **antes** de o kernel correr, só a partir dos params
//!   (`ph2d-node-motion-grid`). Um L-System **não tem forma fechada** para essa contagem: ela
//!   é o resultado da reescrita. Não é «ainda não foi feito» — é o que impede.
//! - ⛔ **Sem `{ } .` (polígonos).** O substrato não tem coluna de região preenchida, então um
//!   `{}` emitiria hoje uma coluna que consumidor nenhum lê. O gatilho que o acorda: um nó
//!   que faça um caminho fechado a partir de uma sequência de elementos.
//! - ⛔ **Sem ramificação 3D** (`&` `^` `/` `\` `$`): em 2D o `+`/`-` esgota o grupo de
//!   rotação, e os símbolos de rolamento não teriam nada para rodar.
//! - ⛔ **Sem índice de cor na gramática** (`;`/`,` do cpfg), e a composição é ESTRITAMENTE
//!   melhor: as colunas `depth`, `gen` e `sym` saem daqui e o `motion.color_ramp` /
//!   `field.remap` fazem cor sobre qualquer uma delas — um gradiente, não um índice.
//!
//! Transcendental-free no desenho (HR-5, o seno parabólico copiado). ⚠️ A avaliação de uma
//! EXPRESSÃO passa por `ph2d_expr::eval`, que usa transcendentais de `f32` — a mesma cerca
//! (e o mesmo lado dela) do `motion.expression`: este é o lado de apresentação, não o de
//! jogabilidade.

mod derive;
mod grammar;
mod hash;
mod trig;
mod turtle;

use ph2d_node_registry::{
    NodeRegistry, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget, RegistryError,
};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// **O text param do AXIOMA** — a cadeia de partida (o canal do doc 32: um `ParamSpec` é
/// `f32` e o manifesto é congelado, então o texto vive no `Graph`).
pub const AXIOM_PARAM: &str = "axiom";

/// **O text param das REGRAS**, separadas por `;` — ver [`grammar`] para a forma, e para a
/// razão de serem de uma linha só (um `\n` corrompe o ficheiro do projeto).
pub const RULES_PARAM: &str = "rules";

/// **O tecto da cadeia derivada — MEDIDO**, e ele é o MESMO número que os outros três tetos
/// de instância desta casa (gate `the_three_instance_ceilings_agree`).
///
/// ## De que recurso ele é
///
/// Não é memória: 262 144 módulos × 24 bytes são 6,3 MB, e nenhuma máquina desta casa nota
/// isso. São **dois** recursos, e os dois deram o mesmo número:
///
/// 1. **O orçamento do quadro para a derivação.** Este nó existe para ser animado —
///    `Generations` a subir é como a planta cresce — e como ele é `Effect::Pure`, cada valor
///    novo do slider **re-deriva a cadeia inteira dentro do quadro**. Varredura de
///    2026-08-28 (`tests/measure_lsystem_ceiling.rs`, `--release`, `load 0,58`, gramática
///    `F -> FF`, tempo de **derivar + interpretar**):
///
///    | elementos | ms | fracção de um quadro de 16,7 ms |
///    |---|---|---|
///    | 32 769 | 0,67 | 4,0 % |
///    | 65 537 | 1,35 | 8,1 % |
///    | 131 073 | 3,21 | 19,2 % |
///    | **262 145** | **6,47** | **38,8 %** |
///    | 524 289 | 13,37 | 80,2 % |
///    | 1 048 577 | 25,52 | 153 % |
///
///    A curva é **linear** (~24 ns por elemento, e há gate:
///    `the_cost_grows_with_the_chain_and_not_faster`), então o degrau escolhe-se lendo a
///    tabela: `524 289` deixa 20 % de quadro para o resto do grafo e do desenho, e `262 145`
///    deixa 61 %.
///
/// 2. **O que o resto do pipeline foi medido a aguentar.** A casa já mediu este número por
///    outro caminho — `motion.trail` / `fx.drop_shadow` / `fx.rgb_split` carregam
///    `MAX_INSTANCES = 262 144`, *"o ponto em que **um** nó passa a ocupar cerca de um terço
///    de um quadro"*. ⭐ **As duas medições concordam sem se conhecerem**: a minha diz 38,8 %,
///    a deles diz «cerca de um terço». Um L-System que emitisse mais linhas do que isso
///    entregaria a jusante uma corrente que o resto do caminho de CPU não foi medido a levar.
///
/// ⚠️ **Um elemento a mais do que módulos**: a tartaruga planta a raiz ANTES do primeiro
/// símbolo, então a contagem emitida é `≤ MAX_MODULES + 1`.
///
/// ⚠️ **E o tecto NÃO pode ser em ITERAÇÕES** — a taxa de expansão é propriedade da REGRA
/// (`F -> FF` dobra, `F -> F[+F]F[-F]F` quintuplica). *A saturação é ao fim de uma geração
/// INTEIRA* — ver [`derive`].
pub const MAX_MODULES: usize = 262_144;

/// O static contract deste tipo de nó (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("source.lsystem"),
    name: "source.lsystem",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: param::GENERATIONS,
            default: 5.0,
        },
        ParamSpec {
            name: param::ANGLE,
            default: 25.0,
        },
        ParamSpec {
            name: param::STEP,
            default: 0.5,
        },
        ParamSpec {
            name: param::WIDTH,
            default: 1.0,
        },
        ParamSpec {
            name: param::WIDTH_SCALE,
            default: 0.7,
        },
        ParamSpec {
            name: param::LENGTH_SCALE,
            default: 0.9,
        },
        ParamSpec {
            name: param::ROOT_ANGLE,
            default: 90.0,
        },
        ParamSpec {
            name: param::TROPISM,
            default: 0.0,
        },
        ParamSpec {
            name: param::TROPISM_ANGLE,
            default: -90.0,
        },
        ParamSpec {
            name: param::SEED,
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Os nomes dos params. ⚠️ Eles são também os nomes que uma EXPRESSÃO da gramática vê
/// (`F(step*0.5)`), então renomear um aqui muda a linguagem que o artista escreveu — é um
/// nome de contrato, não uma etiqueta.
pub mod param {
    pub const GENERATIONS: &str = "generations";
    pub const ANGLE: &str = "angle";
    pub const STEP: &str = "step";
    pub const WIDTH: &str = "width";
    pub const WIDTH_SCALE: &str = "width_scale";
    pub const LENGTH_SCALE: &str = "length_scale";
    pub const ROOT_ANGLE: &str = "root_angle";
    pub const TROPISM: &str = "tropism";
    pub const TROPISM_ANGLE: &str = "tropism_angle";
    pub const SEED: &str = "seed";
}

/// O axioma de fábrica: um módulo `A` que carrega o `step` do painel.
///
/// ⚠️ **`A(step)` e não `A(0.5)`**: uma expressão vê os params do nó pelo nome, então o
/// slider *Step* fica vivo mesmo com o comprimento a vir da gramática. Um literal aqui
/// deixaria o slider inerte no default — o knob morto que o doc 90 caça.
pub const DEFAULT_AXIOM: &str = "A(step)";

/// As regras de fábrica: uma árvore binária paramétrica que **converge**.
///
/// `A(s) -> F(s) ! [ +A(s*0.7) ] [ -A(s*0.7) ]` — o `F(s)` desenha, o `!` afina a espessura
/// para os filhos (depois do tronco, para o tronco sair cheio), e o `0.7` faz a altura
/// somar `s/(1−0.7) ≈ 3,3·step` em vez de crescer sem limite. É o exemplo mínimo que exibe as
/// três coisas que este nó tem e um gerador de pontos não tem: parâmetro, espessura, e árvore.
pub const DEFAULT_RULES: &str = "A(s) -> F(s)![+A(s*0.7)][-A(s*0.7)]";

/// Quantas gerações derivar, e quanto da mais nova já cresceu.
///
/// ⚠️ Um `generations` **fraccionário** deriva `ceil` e faz a mais nova crescer por `frac` —
/// é o que torna o slider uma ANIMAÇÃO de crescimento. `frac == 0` ⇒ nada a crescer e
/// `ceil == floor`, então o caso inteiro é o mesmo código sem nenhum caso especial.
fn generation_plan(generations: f32) -> (u16, (u16, f32)) {
    if !generations.is_finite() || generations <= 0.0 {
        return (0, (0, 1.0));
    }
    let g = generations.min(u16::MAX as f32);
    let whole = g.floor();
    let frac = g - whole;
    if frac <= 0.0 {
        let n = whole as u16;
        (n, (n, 1.0))
    } else {
        let n = whole as u16 + 1;
        (n, (n, frac))
    }
}

struct SourceLSystem;

impl NodeOp for SourceLSystem {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // Os params saem PRIMEIRO: as expressões da gramática vêem-nos por nome, e o
        // fecho que as serve não pode continuar a emprestar o `ctx`.
        let p = Params::read(ctx);
        let axiom_src = ctx.text_param(AXIOM_PARAM).unwrap_or_default().to_string();
        let rules_src = ctx.text_param(RULES_PARAM).unwrap_or_default().to_string();
        let out = build(&axiom_src, &rules_src, &p);
        ctx.emit(out);
    }
}

/// Os dez números do painel, lidos uma vez.
#[derive(Clone, Copy)]
struct Params {
    generations: f32,
    angle: f32,
    step: f32,
    width: f32,
    width_scale: f32,
    length_scale: f32,
    root_angle: f32,
    tropism: f32,
    tropism_angle: f32,
    seed: f32,
}

impl Params {
    fn read(ctx: &EvalCtx<'_>) -> Self {
        Self {
            generations: ctx.param(param::GENERATIONS),
            angle: ctx.param(param::ANGLE),
            step: ctx.param(param::STEP),
            width: ctx.param(param::WIDTH),
            width_scale: ctx.param(param::WIDTH_SCALE),
            length_scale: ctx.param(param::LENGTH_SCALE),
            root_angle: ctx.param(param::ROOT_ANGLE),
            tropism: ctx.param(param::TROPISM),
            tropism_angle: ctx.param(param::TROPISM_ANGLE),
            seed: ctx.param(param::SEED),
        }
    }

    /// O valor de um param pelo NOME — a ponte que deixa uma expressão da gramática ler o
    /// painel (`F(step*0.5)`). Um nome desconhecido é `0`, como em toda expressão da casa.
    fn by_name(&self, n: &str) -> f32 {
        match n {
            param::GENERATIONS => self.generations,
            param::ANGLE => self.angle,
            param::STEP => self.step,
            param::WIDTH => self.width,
            param::WIDTH_SCALE => self.width_scale,
            param::LENGTH_SCALE => self.length_scale,
            param::ROOT_ANGLE => self.root_angle,
            param::TROPISM => self.tropism,
            param::TROPISM_ANGLE => self.tropism_angle,
            param::SEED => self.seed,
            _ => 0.0,
        }
    }
}

/// O texto do artista, ou o de fábrica se ele estiver vazio.
fn or_default<'a>(src: &'a str, fallback: &'a str) -> &'a str {
    if src.trim().is_empty() { fallback } else { src }
}

/// Deriva e interpreta — a função inteira do nó, sem o `EvalCtx` à volta (é ela que os
/// gates e a bancada de medição chamam).
fn build(axiom_src: &str, rules_src: &str, p: &Params) -> ph2d_nodegraph::attr::Stream {
    // ⚠️ **A queda para o default vive AQUI, e não em quem lê o param.** Um text param
    // apagado não pode apagar a planta — e enquanto esta regra morou no `eval`, nenhum gate a
    // alcançava: a porta que os testes e a bancada de medição chamam é esta.
    let axiom_src = or_default(axiom_src, DEFAULT_AXIOM);
    let rules_src = or_default(rules_src, DEFAULT_RULES);
    let params = |n: &str| p.by_name(n);
    let axiom = derive::axiom_modules(axiom_src, &params);
    let rules = grammar::parse_rules(rules_src);
    let (gens, youngest) = generation_plan(p.generations);
    let seed = if p.seed.is_finite() {
        p.seed.abs() as u32
    } else {
        0
    };
    let d = derive::derive(&axiom, &rules, gens, seed, MAX_MODULES, &params);
    // ⚠️ Se o orçamento saturou, a geração mais nova que EXISTE já é inteira — fazê-la
    // crescer por `frac` encolheria uma geração que ninguém pediu para encolher.
    let youngest = if d.generations < gens {
        (d.generations, 1.0)
    } else {
        youngest
    };
    turtle::walk(
        &d.chain,
        &turtle::Setup {
            angle: p.angle,
            step: p.step,
            width: p.width,
            width_scale: p.width_scale,
            length_scale: p.length_scale,
            root_angle: p.root_angle,
            tropism: p.tropism,
            tropism_angle: p.tropism_angle,
            youngest,
        },
    )
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(SourceLSystem))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "L-System",
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::TrapezoidDown,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    Ok(())
}

/// O tecto digitável de `Generations`.
///
/// ⚠️ **É o tecto da CAIXA, não o da cadeia** — o que de facto pára a derivação é o
/// [`MAX_MODULES`], porque a taxa de expansão é propriedade da REGRA: `F -> FF` duplica e
/// `F -> F[+F]F[-F]F` quintuplica, então 20 gerações de uma são triviais e da outra são
/// impossíveis. Este número existe só para a caixa não aceitar um `1e9` que faria o laço
/// externo girar mil milhões de vezes a não fazer nada depois de saturar.
const MAX_GENERATIONS: f32 = 32.0;

/// O tecto DIGITÁVEL, acima do que o slider arrasta — a mesma escada que o `sim.spawn` e o
/// `motion.emitter` usam: o arrasto fica na faixa útil, e quem sabe o que quer digita.
static PARAM_HARD_MAX: &[ph2d_node_registry::ParamHardMax] = &[ph2d_node_registry::ParamHardMax {
    param: param::GENERATIONS,
    max: MAX_GENERATIONS,
}];

static PARAM_HINTS: &[ParamUiHint] = &[
    // ⚠️ **Primeiro os dois textos, e é o que o nó É**: um L-System é a gramática. Os dez
    // números são a interpretação dela.
    ParamUiHint {
        param: AXIOM_PARAM,
        label: "Axiom",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Text,
    },
    ParamUiHint {
        param: RULES_PARAM,
        label: "Rules",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Text,
    },
    // ⚠️ **`Slider`, não `IntSlider`** — a fracção é a feature: com o número a subir
    // continuamente a planta CRESCE, e com ele em degraus ela salta.
    ParamUiHint {
        param: param::GENERATIONS,
        label: "Generations",
        min: 0.0,
        max: 12.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::ANGLE,
        label: "Angle",
        min: 0.0,
        max: 180.0,
        step: 0.5,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: param::STEP,
        label: "Step",
        min: 0.01,
        max: 4.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::WIDTH,
        label: "Width",
        min: 0.01,
        max: 8.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::WIDTH_SCALE,
        label: "Width Scale",
        min: 0.1,
        max: 1.5,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::LENGTH_SCALE,
        label: "Length Scale",
        min: 0.1,
        max: 1.5,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::ROOT_ANGLE,
        label: "Root Angle",
        min: -180.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    // ⚠️ **POSITIVO puxa PARA a direcção; negativo empurra para longe dela.** A direcção já
    // tem um param próprio, então o SINAL aqui é a força e não um segundo eixo — e uma cena
    // desta linha nasceu com ele trocado, a fazer a planta com «gravidade» sair mais direita
    // do que a sem.
    ParamUiHint {
        param: param::TROPISM,
        label: "Tropism",
        min: -45.0,
        max: 45.0,
        step: 0.5,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: param::TROPISM_ANGLE,
        label: "Tropism Direction",
        min: -180.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: param::SEED,
        label: "Seed",
        min: 0.0,
        max: 9999.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
];

/// **O que cada número É** (doc 88) — só as grandezas que são uma DISTÂNCIA de mundo.
///
/// O `step` é a única: um ângulo já tem a face dele pelo widget, e `width` é uma ESCALA
/// (vai para a coluna `size`, que é adimensional), não uma distância — declará-la como
/// `Length` faria a caixa mostrar pixels para um multiplicador.
static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: param::STEP,
    unit: ParamUnit::Length,
}];

/// **A porta de SONDA** — derivar + interpretar com os defaults do manifesto, mudando só o
/// que quem mede quer mudar.
///
/// ⚠️ `pub` sem `#[cfg(test)]` de propósito: a bancada que MEDE o tecto
/// (`tests/measure_lsystem_ceiling.rs`) é um alvo de integração e não vê itens de teste. Uma
/// porta que só o teste unitário alcança obrigaria a bancada a reimplementar o caminho — e
/// aí ela mediria outro código.
#[must_use]
pub fn probe_build(
    axiom: &str,
    rules: &str,
    generations: f32,
    overrides: &[(&str, f32)],
) -> ph2d_nodegraph::attr::Stream {
    let mut p = Params {
        generations,
        angle: 25.0,
        step: 0.5,
        width: 1.0,
        width_scale: 0.7,
        length_scale: 0.9,
        root_angle: 90.0,
        tropism: 0.0,
        tropism_angle: -90.0,
        seed: 1.0,
    };
    for (n, v) in overrides {
        match *n {
            param::ANGLE => p.angle = *v,
            param::STEP => p.step = *v,
            param::WIDTH => p.width = *v,
            param::WIDTH_SCALE => p.width_scale = *v,
            param::LENGTH_SCALE => p.length_scale = *v,
            param::ROOT_ANGLE => p.root_angle = *v,
            param::TROPISM => p.tropism = *v,
            param::TROPISM_ANGLE => p.tropism_angle = *v,
            param::SEED => p.seed = *v,
            other => panic!("probe_build: param desconhecido {other}"),
        }
    }
    build(axiom, rules, &p)
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
