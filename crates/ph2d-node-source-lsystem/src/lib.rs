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
pub mod shape;
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
        // ⚠️ **APENDADO**, nunca inserido — um documento salvo guarda o NOME do param, mas a
        // ordem é o que uma leitura por índice veria. E o default é `0` = `Growth`: é o que o
        // desenho quer, e foi o report do Enio (2026-08-28).
        ParamSpec {
            name: param::ORIENT,
            default: 0.0,
        },
        // ⚠️⚠️ **O default é o `Custom`, e não o molde `0`** — auditoria de 2026-08-29.
        // Desde que o `Mode` nasce `Guided`, «o que um nó novo já é» deixou de ser a gramática
        // do Tree e passou a ser a derivada dos sliders (`grammar_for(2,1,0,0)`), que é OUTRA
        // planta — 76 % mais alta, medido. Um selector a dizer «Tree» sobre ela é o painel a
        // mentir sobre o próprio estado, que é exactamente o que o gate
        // `the_first_preset_is_what_a_fresh_node_already_is` dizia proibir enquanto a premissa
        // dele era verdadeira. *O `Custom` é a resposta honesta a «que molde é este?» quando
        // não é nenhum.*
        ParamSpec {
            name: param::PRESET,
            default: PRESET_CUSTOM as f32,
        },
        // ⚠️ **`0` = `Guided`, e o default é a resposta ao report de 2026-08-29.** Um nó
        // recém-dropado abre com sliders de forma; a gramática está a UM clique, e é ela
        // que o `Mode` assa quando o artista lá vai. Ver [`shape`] para o porquê.
        ParamSpec {
            name: param::MODE,
            default: MODE_GUIDED as f32,
        },
        ParamSpec {
            name: param::BRANCHES,
            default: 2.0,
        },
        ParamSpec {
            name: param::SEGMENTS,
            default: 1.0,
        },
        ParamSpec {
            name: param::VARIATION,
            default: 0.0,
        },
        ParamSpec {
            name: param::BEND,
            default: 0.0,
        },
        // AS TRES QUE FAZEM O CRESCIMENTO SUAVE (2026-08-29, a pedido do Enio, com o L-System
        // SOP do Houdini como referencia -- ver `turtle::walk` para o mecanismo e a tabela de
        // razoes de expansao que separou as duas familias).
        //
        // Os dois interruptores nascem LIGADOS: e' o que o artista quer, e a razao de o no'
        // existir e' animar o `Generations`. O `step_scale` nasce em `1,0` -- neutro exacto,
        // entao nenhum documento se mexe por ele.
        ParamSpec {
            name: param::CONTINUOUS_LENGTH,
            default: 1.0,
        },
        // ⭐⭐⭐ **LIGADO**, e o caminho ate' aqui esta' registado porque ele e' a licao:
        //   1. o Enio previu *"os que vc tentou corrigir nao ficarao bons"* -- e eu shipei
        //      desligado, com a medicao que concordava com ele (9-31% de pior passo);
        //   2. ele SMOKOU e retirou a previsao: *"Melhorou muito. Mas o crescimento dos que
        //      nao cresciam suavemente nao e' linear"*;
        //   3. medi a DERIVADA (nao o pior passo) e ele tinha razao pela segunda vez: Bush e
        //      Weed ja' eram lineares (ondulacao `0,0x`), e as CURVAS passavam do alvo e
        //      VOLTAVAM (Koch `2,3x`, Dragon `4,2x`);
        //   4. normalizar pelo tamanho MEDIDO poe as quatro em `0,0x`.
        //
        // ⭐ *A previsao dele era sobre a versao que ele viu, e a queixa dele era um DEFEITO
        // com endereco.* `PH2D_*` nenhum: desligar o `Grow Angle` devolve o degrau inteiro de
        // sempre, byte a byte.
        ParamSpec {
            name: param::CONTINUOUS_ANGLE,
            default: 1.0,
        },
        ParamSpec {
            name: param::STEP_SCALE,
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
    pub const ORIENT: &str = "orient";
    pub const PRESET: &str = "preset";
    pub const MODE: &str = "mode";
    pub const BRANCHES: &str = "branches";
    pub const SEGMENTS: &str = "segments";
    pub const VARIATION: &str = "variation";
    pub const BEND: &str = "bend";
    pub const CONTINUOUS_LENGTH: &str = "continuous_length";
    pub const CONTINUOUS_ANGLE: &str = "continuous_angle";
    pub const STEP_SCALE: &str = "step_scale";
}

/// **O modo GUIADO** — os sliders de forma mandam, e a gramática é derivada deles.
pub const MODE_GUIDED: i32 = 0;
/// **O modo GRAMÁTICA** — o texto manda, e os sliders de forma somem.
pub const MODE_GRAMMAR: i32 = 1;

/// Os dois modos de autoria. ⚠️ A ordem É o valor gravado: `Guided` tem de ficar em `0`.
pub const MODE_LABELS: &[&str] = &["Guided", "Grammar"];

/// **O que a coluna `rot` quer dizer** — ver [`crate::turtle::Setup::orient_world`] para o
/// mecanismo. `0` = mundo (o desenho alinha com o ramo) · `1` = local (o contrato do `rig.*`).
pub const ORIENT_LABELS: &[&str] = &["Growth", "Local"];

/// **OS MOLDES** — o que responde à pergunta *"Axiom e Rules não são nada intuitivos"*
/// (Enio, 2026-08-28).
///
/// ⭐ **A resposta NÃO é inventar uma sintaxe amigável.** `F[+F]F` é a notação de Lindenmayer:
/// é o que está no ABOP, nos tutoriais, nos fóruns e em todo exemplo que o artista vai
/// encontrar. Trocá-la tornaria este nó **incompatível com o conhecimento do mundo** — ele
/// deixaria de aceitar o que se copia de qualquer lado.
///
/// ⇒ O que se dá é um SÍTIO POR ONDE COMEÇAR. O artista escolhe um molde, vê a planta, e
/// edita — que é como toda a gente aprende esta linguagem. É o que o L-System SOP do Houdini
/// e o L-studio fazem.
///
/// ⚠️⚠️ **E os moldes NÃO chegaram** — report do Enio, 2026-08-29. A 1.ª redacção desta nota
/// citava «o `sca-tools` do Blender» como se ele fizesse isto, e estava **errada duas vezes**:
/// o Blender não tem L-System nenhum, e o `sca-tools` é colonização de espaço (sliders, sem
/// gramática). *A referência que eu invoquei para justificar a interface era a referência que
/// prova o contrário.* A cura é o [`crate::shape`]: os moldes ficam, mas atrás deles passa a
/// haver um modo GUIADO, que é o default.
///
/// ⚠️⚠️ **E UM MOLDE NÃO É SÓ UM TEXTO** — auditoria de 2026-08-29, sobre o report do Enio
/// (*"o modo tree funciona aparentemente bem. os demais tem resultado questionável"*).
///
/// A 1.ª tabela escrevia **só** o axioma e as regras, e deixava por escrever tudo o resto que
/// aquela figura EXIGE. Medido, com os defaults do painel, pela bancada
/// [`examples/preset_report.rs`](../../examples/preset_report.rs):
///
/// | molde  | `maior/step` | tamanho de mundo | o que ficava a mentir |
/// |---|---|---|---|
/// | Tree   |   2,7 | 1,34 | — |
/// | Fern   |   3,9 | 1,93 | — |
/// | Wild   |   3,7 | 1,84 | — |
/// | Sprig  |   3,4 | 1,68 | o `Angle` é **byte-inerte** (família C da auditoria) |
/// | Dragon |  25,2 | 12,59 | pede **90°** e chegava a 25 |
/// | Weed   |  60,1 | 30,07 | — |
/// | Bush   | 243,0 | 121,50 | — |
/// | Koch   | 2581,8 | **1290,90** | pede **90°**; 322× a coluna da cena |
///
/// ⭐ **963× entre dois itens do mesmo selector**, e uma coluna da cena `=108` tem ~4 unidades.
///
/// ⇒ O molde passa a carregar o **enquadramento**: o ângulo que a figura exige, as gerações em
/// que ela se lê, e o par `step`/`width` que a põe do mesmo tamanho dos irmãos.
///
/// ⚠️ **O `step` e o `width` CONTAM-SE, não se escolhem.** A razão `maior_dimensão / step` é
/// invariante à escala, então o passo sai de `step = step_base · alvo ÷ razão_medida`, com o
/// **alvo = mediana dos quatro que o dono já aceitou** (`3,522`). Os oito enquadram hoje em
/// **1,76 unidades de mundo**, medido. E o `width` sai de `0,321 · step` — a razão da única
/// configuração que ele aprovou (a coluna da cena `=108`, `width 0,09` sobre `step 0,28`).
/// Sem ela a cura seria meia: a Koch a 4 gerações tem **626** elementos, e o renderer desenha
/// cada um como um ponto de raio `size` — com o `width` de fábrica saía um borrão sólido.
///
/// ⛔ **O TEXTO de cada molde fica INTOCADO**, e é uma recusa deliberada: `F -> F+F-F-F+F` é a
/// notação de Lindenmayer, e reescrevê-la em forma paramétrica (para ganhar o `!` e o `"`)
/// tornaria o molde incompatível com o que se copia de um tutorial. O preço declarado é que
/// nos quatro clássicos o `Width Scale` e o `Length Scale` **não têm consumidor** — é o que o
/// campo [`Preset::reads`] declara, e é o painel que os esconde.
///
/// ⚠️ **O molde `0` é o de fábrica**, para um nó recém-dropado e o selector concordarem.
pub struct Preset {
    pub label: &'static str,
    pub axiom: &'static str,
    pub rules: &'static str,
    /// O ângulo que a figura EXIGE. Koch e Dragon são `90` **por definição**, não por gosto.
    pub angle: f32,
    /// As gerações em que a figura se lê. Um dragão só é um dragão a partir de ~10.
    pub generations: f32,
    /// O passo que põe esta figura do tamanho dos irmãos — DERIVADO, ver a nota acima.
    pub step: f32,
    /// E a espessura que a mantém uma linha em vez de um borrão — `0,321 · step`.
    pub width: f32,
    /// **Que knobs de interpretação este texto de facto LÊ.** Uma gramática sem `!` ignora o
    /// `Width Scale`; uma sem `"` ignora o `Length Scale`. O painel esconde o que o molde não
    /// lê, em vez de o pintar inerte.
    pub reads: Reads,
}

/// Os símbolos de interpretação que uma gramática contém — e portanto os knobs que ela honra.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Reads {
    /// Contém `!` ⇒ o `Width Scale` age.
    pub width_scale: bool,
    /// Contém `"` ⇒ o `Length Scale` age.
    pub length_scale: bool,
}

impl Reads {
    /// ⚠️ **DERIVADO do texto, nunca declarado à mão** — um campo escrito à mão seria uma
    /// segunda resposta à mesma pergunta, e envelheceria na primeira vez que alguém editasse
    /// uma regra. Há gate a comparar os dois (`what_each_preset_reads_is_derived_from_its_text`).
    #[must_use]
    pub const fn of(rules: &str) -> Self {
        let b = rules.as_bytes();
        let (mut i, mut bang, mut quote) = (0usize, false, false);
        while i < b.len() {
            if b[i] == b'!' {
                bang = true;
            }
            if b[i] == b'"' {
                quote = true;
            }
            i += 1;
        }
        Self {
            width_scale: bang,
            length_scale: quote,
        }
    }
}

/// ⚠️ **O ÍNDICE `CUSTOM` é o último, e ele não é um molde** — é *"nenhum destes"*.
///
/// Sem ele o selector MENTE: `preset` é um `ParamSpec` persistido que o `build` **nunca lê**,
/// e três escritores mudam o texto sem lhe tocar (o `bake` do modo guiado, a edição à mão da
/// caixa, e uma cena). O estado de chegada normal — abrir em `Guided` e converter — deixava o
/// selector a dizer «Tree» sobre uma planta **76% mais alta**, com o clique em «Tree» mudo
/// (a guarda de igualdade do despacho). *Um número que é o eco de um gesto passado não é um
/// facto sobre a planta.*
pub const PRESET_CUSTOM: usize = PRESETS.len();

pub const PRESETS: &[Preset] = &[
    Preset {
        label: "Tree",
        axiom: DEFAULT_AXIOM,
        rules: DEFAULT_RULES,
        angle: 25.0,
        generations: 5.0,
        step: 0.658,
        width: 0.212,
        reads: Reads::of(DEFAULT_RULES),
    },
    Preset {
        label: "Fern",
        axiom: "A(step)",
        rules: "A(s) -> F(s)[+B(s*0.55)]!A(s*0.87) ; B(s) -> F(s)[-B(s*0.72)]B(s*0.8)",
        angle: 25.0,
        generations: 5.0,
        step: 0.456,
        width: 0.147,
        reads: Reads::of("A(s) -> F(s)[+B(s*0.55)]!A(s*0.87) ; B(s) -> F(s)[-B(s*0.72)]B(s*0.8)"),
    },
    // ABOP fig. 1.24: o arbusto clássico lê-se a **4** gerações (a 5 são 3 126 módulos), e o
    // ângulo do livro é 25,7°.
    Preset {
        label: "Bush",
        axiom: "F",
        rules: "F -> F[+F]F[-F]F",
        angle: 25.7,
        generations: 4.0,
        step: 0.022,
        width: 0.007,
        reads: Reads::of("F -> F[+F]F[-F]F"),
    },
    // ABOP fig. 1.24d — 20°.
    Preset {
        label: "Weed",
        axiom: "X",
        rules: "X -> F[+X]F[-X]+X ; F -> FF",
        angle: 20.0,
        generations: 5.0,
        step: 0.029,
        width: 0.009,
        reads: Reads::of("X -> F[+X]F[-X]+X ; F -> FF"),
    },
    Preset {
        label: "Wild",
        axiom: "A(step)",
        rules: "A(s) -> (0.4) F(s)![+A(s*0.72)][-A(s*0.72)] ; \
                A(s) -> (0.35) F(s)![+A(s*0.66)]-A(s*0.78) ; \
                A(s) -> (0.25) F(s)!F(s*0.8)[+A(s*0.6)]",
        angle: 25.0,
        generations: 5.0,
        step: 0.478,
        width: 0.154,
        reads: Reads::of("A(s) -> (0.4) F(s)![+A(s*0.72)][-A(s*0.72)]"),
    },
    // ⚠️ A ilha de Koch quadrática é **90° por definição** — a 25 ela não é a figura, é um
    // risco. Foi o que o dono do produto viu.
    Preset {
        label: "Koch",
        axiom: "F",
        rules: "F -> F+F-F-F+F",
        angle: 90.0,
        generations: 4.0,
        step: 0.022,
        width: 0.007,
        reads: Reads::of("F -> F+F-F-F+F"),
    },
    // ⚠️ A curva do dragão: 90°, e só se lê como dragão a partir de ~10 iterações.
    Preset {
        label: "Dragon",
        axiom: "F",
        rules: "F -> F+G ; G -> F-G",
        angle: 90.0,
        generations: 12.0,
        step: 0.019,
        width: 0.006,
        reads: Reads::of("F -> F+G ; G -> F-G"),
    },
    // ⚠️ O `[+F(s*0.35)J]` e não o `[+J]` da 1.ª redacção: uma MARCA lê o osso do PAI e não o
    // rumo da tartaruga (`turtle.rs`, com gate), então `[+J][-J]` punha as duas folhas
    // exactamente no mesmo ponto — e o molde saía uma linha recta de largura `0,00`, com o
    // `Angle` byte-inerte. A folha precisa de um ramo a levá-la.
    Preset {
        label: "Sprig",
        axiom: "A(step)",
        rules: "A(s) -> F(s)[+F(s*0.35)J][-F(s*0.35)J]!A(s*0.8) ; J -> J",
        angle: 25.0,
        generations: 5.0,
        step: 0.524,
        width: 0.168,
        reads: Reads::of("A(s) -> F(s)[+F(s*0.35)J][-F(s*0.35)J]!A(s*0.8) ; J -> J"),
    },
];

/// Os rótulos do selector — **derivados** de [`PRESETS`], mais o `Custom` do fim.
///
/// ⚠️ Uma `const` não pode iterar, então isto é escrito e há gate a exigir que cada entrada
/// bata com `PRESETS[k].label` e que o último seja o `Custom`.
pub const PRESET_LABELS: &[&str] = &[
    "Tree", "Fern", "Bush", "Weed", "Wild", "Koch", "Dragon", "Sprig", "Custom",
];

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
    orient: f32,
    mode: f32,
    branches: f32,
    segments: f32,
    variation: f32,
    bend: f32,
    continuous_length: f32,
    continuous_angle: f32,
    step_scale: f32,
}

impl Params {
    /// **Os sliders mandam?** — a pergunta que decide de onde vem a gramática.
    fn guided(&self) -> bool {
        self.mode.round() as i32 != MODE_GRAMMAR
    }

    /// Os números de forma, na cara que o [`shape`] pede.
    fn shape(&self) -> shape::Shape {
        shape::Shape {
            branches: self.branches,
            segments: self.segments,
            variation: self.variation,
            bend: self.bend,
        }
    }

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
            orient: ctx.param(param::ORIENT),
            mode: ctx.param(param::MODE),
            branches: ctx.param(param::BRANCHES),
            segments: ctx.param(param::SEGMENTS),
            variation: ctx.param(param::VARIATION),
            bend: ctx.param(param::BEND),
            continuous_length: ctx.param(param::CONTINUOUS_LENGTH),
            continuous_angle: ctx.param(param::CONTINUOUS_ANGLE),
            step_scale: ctx.param(param::STEP_SCALE),
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
            param::ORIENT => self.orient,
            param::MODE => self.mode,
            param::BRANCHES => self.branches,
            param::SEGMENTS => self.segments,
            param::VARIATION => self.variation,
            param::BEND => self.bend,
            param::CONTINUOUS_LENGTH => self.continuous_length,
            param::CONTINUOUS_ANGLE => self.continuous_angle,
            param::STEP_SCALE => self.step_scale,
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
    // ⚠️ **A escolha do MODO vive aqui, pela mesma razão que a queda para o default**: esta
    // é a porta que os gates e a bancada de medição chamam, e uma decisão tomada no `eval`
    // seria inalcançável por qualquer um dos dois. No guiado o texto do artista **não é
    // lido** — ele fica intacto no documento, à espera de que alguém volte a `Grammar`.
    let generated;
    let (axiom_src, rules_src) = if p.guided() {
        generated = shape::rules(&p.shape());
        (shape::AXIOM, generated.as_str())
    } else {
        // ⚠️ **A queda para o default vive AQUI, e não em quem lê o param.** Um text param
        // apagado não pode apagar a planta.
        (
            or_default(axiom_src, DEFAULT_AXIOM),
            or_default(rules_src, DEFAULT_RULES),
        )
    };
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
    // O `Setup` de uma travessia de MEDIÇÃO — as duas fracções à escolha de quem mede.
    let base = |len: f32, ang: f32, youngest: (u16, f32)| turtle::Setup {
        angle: p.angle,
        step: p.step * p.step_scale.max(1e-4).powf(p.generations.clamp(0.0, 64.0)),
        width: p.width,
        width_scale: p.width_scale,
        length_scale: p.length_scale,
        root_angle: p.root_angle,
        tropism: p.tropism,
        tropism_angle: p.tropism_angle,
        angle_frac: ang,
        youngest: (youngest.0, len * youngest.1),
        orient_world: p.orient.round() as i32 == 0,
    };

    // ⭐⭐⭐ **A LEI DO CRESCIMENTO, e ela agora MEDE em vez de supor** (2026-08-29).
    //
    // Report do Enio depois de ver a 1.ª versão: *"Melhorou muito. Mas o crescimento dos que
    // não cresciam suavemente não é linear"*. A medição deu-lhe razão e nomeou a causa — o
    // Bush e o Weed já eram **perfeitamente lineares** (ondulação `0,0×`), e quem não era eram
    // as CURVAS: a Koch sobe a `3,05` e **volta** a `3,00` (ondulação `2,3×`), o Dragon sobe a
    // `1,62` e volta a `1,51` (`4,2×`).
    //
    // ⚠️ **O mecanismo: as duas rampas brigam.** O comprimento cresce linearmente enquanto as
    // dobras, ao abrir, **encurtam** a projecção — a `90°` uma zig-zag ocupa menos do que a
    // mesma linha meio aberta. O produto das duas tem um pico no meio, e o último quinto do
    // slider anda para TRÁS. *Andar para trás é o que se vê da cadeira.*
    //
    // ⇒ A cura não é uma constante: é **normalizar pelo que se mede**. A cada instante mede-se
    // o tamanho da figura com as dobras onde elas estão (`agora`) e escolhe-se o comprimento
    // que a põe exactamente na rampa linear entre a geração anterior (`antes`) e a nova
    // inteira (`cheia`). A âncora de que a versão anterior falava é só o valor disto em
    // `frac = 0` — ela generalizou-se.
    //
    // ⚠️ **Três travessias por cozedura, e SÓ numa geração fraccionária** — numa inteira nada
    // disto corre e o custo é exactamente o de sempre. É o preço que o Enio aprovou
    // (*"desenhar a planta duas vezes por quadro"*).
    let frac = youngest.1;
    let has_old_drawing = d
        .chain
        .iter()
        .any(|m| m.born != youngest.0 && turtle::draws_or_marks(m.sym));
    let grows_by_refining = !has_old_drawing && !d.previous.is_empty();
    let want_len = p.continuous_length.round() as i32 != 0;
    let want_ang = p.continuous_angle.round() as i32 != 0;

    let (len_frac, ang_frac) = if frac >= 1.0 {
        // Geração inteira: não há nada a interpolar, e nada se mede.
        (1.0, 1.0)
    } else if !grows_by_refining {
        // Cresce pela PONTA: o rebento estica de zero (a lei de sempre, e a cura do
        // pisca-pisca de 28/08). A viragem é inerte aqui por construção — ver `turtle`.
        (if want_len { frac } else { 1.0 }, frac)
    } else if want_ang {
        // REFINA: normaliza pelo que se mede, para a rampa sair recta.
        let antes = turtle::span(&d.previous, &base(1.0, 1.0, (d.generations, 1.0)));
        let cheia = turtle::span(&d.chain, &base(1.0, 1.0, (d.generations, 1.0)));
        let agora = turtle::span(&d.chain, &base(1.0, frac, (d.generations, 1.0)));
        let alvo = antes + (cheia - antes) * frac;
        if agora > 1e-6 && alvo > 0.0 {
            ((alvo / agora).clamp(0.02, 4.0), frac)
        } else {
            (1.0, frac)
        }
    } else {
        // ⛔ O `Grow Angle` desligado é o degrau inteiro de sempre, byte a byte.
        (1.0, 1.0)
    };

    turtle::walk(
        &d.chain,
        &turtle::Setup {
            angle: p.angle,
            // O PASSO ENCOLHE POR GERACAO (*Step Size Scale* do Houdini) -- e' o que
            // torna uma gramatica de REFINAMENTO um refinamento em vez de um crescimento.
            //
            // Medido: `F -> F[+F]F[-F]F` e `F -> F+F-F-F+F` expandem EXACTAMENTE `3,00` em
            // toda geracao; com `step_scale = 1/3` a figura fica do mesmo tamanho e so'
            // ganha detalhe. As que crescem pela ponta convergem para `~1,06` e nao
            // precisam dele -- por isso o default e' `1,0`, o neutro EXACTO.
            //
            // O expoente e' o `generations` FRACCIONARIO e nao o `ceil`: com o inteiro o
            // passo daria um degrau em cada travessia, que e' o defeito que isto cura.
            // Um `powf` e' transcendental (HR-5) e corre UMA vez por cozedura, nunca por
            // elemento -- a mesma cerca (e o mesmo lado dela) da avaliacao de expressoes.
            step: p.step * p.step_scale.max(1e-4).powf(p.generations.clamp(0.0, 64.0)),
            width: p.width,
            width_scale: p.width_scale,
            length_scale: p.length_scale,
            root_angle: p.root_angle,
            tropism: p.tropism,
            tropism_angle: p.tropism_angle,
            youngest: (youngest.0, len_frac),
            angle_frac: ang_frac,
            // `0` = mundo (o desenho alinha com o ramo). Qualquer outro valor é o local.
            orient_world: p.orient.round() as i32 == 0,
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
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_groups(MANIFEST.id, PARAM_GROUPS);
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
    // ⚠️ **O MODO vem antes de tudo** — ele decide qual metade do painel existe.
    ParamUiHint {
        param: param::MODE,
        label: "Mode",
        min: 0.0,
        max: (MODE_LABELS.len() - 1) as f32,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: MODE_LABELS,
        },
    },
    // Os quatro números de FORMA — o modo guiado inteiro. Ver [`shape`].
    ParamUiHint {
        param: param::BRANCHES,
        label: "Branches",
        min: 1.0,
        max: shape::MAX_BRANCHES,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: param::SEGMENTS,
        label: "Trunk Segments",
        min: 1.0,
        max: shape::MAX_SEGMENTS,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: param::VARIATION,
        label: "Variation",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::BEND,
        label: "Bend",
        min: -30.0,
        max: 30.0,
        step: 0.5,
        widget: ParamWidget::Angle,
    },
    // ⚠️ **Depois os dois textos, e é o que o nó É por dentro**: um L-System é a gramática.
    // Os números são a interpretação dela.
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
    // ⚠️ **O molde vem PRIMEIRO de todos** — antes até do axioma. É a resposta ao *"não são
    // nada intuitivos"*: o artista escolhe um sítio por onde começar, vê a planta, e só depois
    // edita o texto. Um selector abaixo das caixas seria a ajuda escondida atrás do problema.
    ParamUiHint {
        param: param::PRESET,
        label: "Preset",
        min: 0.0,
        // ⚠️ **`PRESET_LABELS`, e não `PRESETS`** — a lista tem uma entrada a mais, o
        // [`PRESET_CUSTOM`], que não é um molde e sim *"nenhum destes"*.
        max: (PRESET_LABELS.len() - 1) as f32,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: PRESET_LABELS,
        },
    },
    ParamUiHint {
        param: param::ORIENT,
        label: "Shape Faces",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: ORIENT_LABELS,
        },
    },
    // As tres do CRESCIMENTO SUAVE (2026-08-29). Ver `turtle::walk` para a medicao.
    ParamUiHint {
        param: param::CONTINUOUS_LENGTH,
        label: "Grow Length",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    ParamUiHint {
        param: param::CONTINUOUS_ANGLE,
        label: "Grow Angle",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    ParamUiHint {
        param: param::STEP_SCALE,
        label: "Step Scale",
        min: 0.1,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
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

/// **AS DUAS METADES NÃO SE VÊEM UMA À OUTRA** — o gate de visibilidade que faz o `Mode` ser
/// um modo em vez de um rótulo.
///
/// ⚠️ *Um controle que não faz nada não é pintado.* No guiado a gramática é derivada, então
/// as caixas de texto mostrariam o que o nó **não lê** — a pior forma de mentir num painel,
/// porque o artista edita e nada acontece. No modo gramática os quatro números de forma
/// deixam de alimentar seja o que for, pela mesma razão do outro lado.
///
/// ⚠️ **O `Preset` fica com a GRAMÁTICA**, e não com os sliders: um molde É uma gramática, e
/// escolher um no guiado escreveria num texto que ninguém está a ler.
static PARAM_GATES: &[ph2d_node_registry::ParamGate] = &[
    ph2d_node_registry::ParamGate {
        param: AXIOM_PARAM,
        when: param::MODE,
        values: &[MODE_GRAMMAR],
    },
    ph2d_node_registry::ParamGate {
        param: RULES_PARAM,
        when: param::MODE,
        values: &[MODE_GRAMMAR],
    },
    ph2d_node_registry::ParamGate {
        param: param::PRESET,
        when: param::MODE,
        values: &[MODE_GRAMMAR],
    },
    ph2d_node_registry::ParamGate {
        param: param::BRANCHES,
        when: param::MODE,
        values: &[MODE_GUIDED],
    },
    ph2d_node_registry::ParamGate {
        param: param::SEGMENTS,
        when: param::MODE,
        values: &[MODE_GUIDED],
    },
    ph2d_node_registry::ParamGate {
        param: param::VARIATION,
        when: param::MODE,
        values: &[MODE_GUIDED],
    },
    ph2d_node_registry::ParamGate {
        param: param::BEND,
        when: param::MODE,
        values: &[MODE_GUIDED],
    },
    // ⭐⭐ **O knob que a GRAMÁTICA ESCOLHIDA não lê não é pintado** — a outra metade da cura
    // dos moldes (auditoria 2026-08-29). Uma gramática sem `!` ignora o *Width Scale*; uma sem
    // `"` ignora o *Length Scale*. Medido: o `Length Scale` está **inerte nos 8/8 moldes**
    // (bbox bit-idêntica a `0,10` e a `1,50`) e **vivo** no `Custom` — que é onde o modo
    // guiado e a gramática assada aterram, e onde ele mexe a peça de `0,05` para `10,60`.
    // ⇒ *o knob não está morto: ele MORRE quando um molde é escolhido*, e é o molde que é o
    // sujeito do gate, nunca o modo.
    ph2d_node_registry::ParamGate {
        param: param::WIDTH_SCALE,
        when: param::PRESET,
        values: PRESETS_READING_WIDTH_SCALE,
    },
    ph2d_node_registry::ParamGate {
        param: param::LENGTH_SCALE,
        when: param::PRESET,
        values: PRESETS_READING_LENGTH_SCALE,
    },
];

/// Os índices de molde cuja gramática contém `!` — mais o [`PRESET_CUSTOM`].
///
/// ⚠️ **Escrito à mão e GATEADO contra a derivação** (`Reads::of`), como os `PRESET_LABELS`:
/// uma `const` não pode iterar uma tabela, então a defesa contra as duas respostas divergirem
/// é o gate `the_read_gates_agree_with_what_each_grammar_contains`, não a boa vontade.
static PRESETS_READING_WIDTH_SCALE: &[i32] = &[0, 1, 4, 7, PRESET_CUSTOM as i32];

/// Os índices cuja gramática contém `"`. **Nenhum molde o tem** — só o `Custom`, que é onde o
/// modo guiado e o texto assado vivem.
static PRESETS_READING_LENGTH_SCALE: &[i32] = &[PRESET_CUSTOM as i32];

/// **AS SEÇÕES** — quatro perguntas, e cada uma responde-se sem ler as outras.
///
/// ⚠️ **O `Mode` fica FORA de todas**, de propósito: as soltas são pintadas primeiro
/// (`split_into_sections`), e o controle que decide o que as seções contêm não pode viver
/// dentro de uma delas — muito menos dentro de uma que nasça fechada.
static PARAM_GROUPS: &[ph2d_node_registry::ParamGroup] = &[
    ph2d_node_registry::ParamGroup::new(param::BRANCHES, "Shape"),
    ph2d_node_registry::ParamGroup::new(param::SEGMENTS, "Shape"),
    ph2d_node_registry::ParamGroup::new(param::ANGLE, "Shape"),
    ph2d_node_registry::ParamGroup::new(param::BEND, "Shape"),
    ph2d_node_registry::ParamGroup::new(param::VARIATION, "Shape"),
    ph2d_node_registry::ParamGroup::new(param::PRESET, "Grammar"),
    ph2d_node_registry::ParamGroup::new(AXIOM_PARAM, "Grammar"),
    ph2d_node_registry::ParamGroup::new(RULES_PARAM, "Grammar"),
    ph2d_node_registry::ParamGroup::new(param::GENERATIONS, "Growth"),
    ph2d_node_registry::ParamGroup::new(param::STEP, "Growth"),
    ph2d_node_registry::ParamGroup::new(param::LENGTH_SCALE, "Growth"),
    ph2d_node_registry::ParamGroup::new(param::WIDTH, "Growth"),
    ph2d_node_registry::ParamGroup::new(param::WIDTH_SCALE, "Growth"),
    ph2d_node_registry::ParamGroup::new(param::STEP_SCALE, "Growth"),
    ph2d_node_registry::ParamGroup::new(param::CONTINUOUS_LENGTH, "Growth"),
    ph2d_node_registry::ParamGroup::new(param::CONTINUOUS_ANGLE, "Growth"),
    // ⚠️ Esta nasce FECHADA: é a única cujos cinco defaults já dão uma planta de pé, e o
    // artista que nunca a abrir não perde nada.
    ph2d_node_registry::ParamGroup::new(param::ROOT_ANGLE, "Lean & Look").folded(),
    ph2d_node_registry::ParamGroup::new(param::TROPISM, "Lean & Look").folded(),
    ph2d_node_registry::ParamGroup::new(param::TROPISM_ANGLE, "Lean & Look").folded(),
    ph2d_node_registry::ParamGroup::new(param::ORIENT, "Lean & Look").folded(),
    ph2d_node_registry::ParamGroup::new(param::SEED, "Lean & Look").folded(),
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
/// **A GRAMÁTICA QUE OS SLIDERS ESTÃO A FAZER AGORA** — a porta que a shell chama para
/// ASSAR o guiado no texto quando o artista muda para `Grammar`.
///
/// ⚠️ **Uma porta e não uma segunda conta na shell.** O `build` deriva a mesma coisa a cada
/// cozedura; se a shell montasse a string à mão, os dois lados divergiriam no dia em que um
/// deles ganhasse um símbolo — e o artista veria a planta MUDAR ao converter, que é
/// exactamente o que uma conversão não pode fazer.
#[must_use]
pub fn grammar_for(
    branches: f32,
    segments: f32,
    variation: f32,
    bend: f32,
) -> (&'static str, String) {
    (
        shape::AXIOM,
        shape::rules(&shape::Shape {
            branches,
            segments,
            variation,
            bend,
        }),
    )
}

/// **OS PESOS QUE O PARSER DE FACTO DEVOLVE** — a porta de sonda que impede um gate de
/// escrever o próprio oráculo.
///
/// ⚠️ O gate `variation_gives_three_weighted_rules_whose_weights_close_at_one` lia os pesos do
/// texto com um `str::parse::<f32>()` PRÓPRIO, e por isso ficava verde em `v = 1,0`: o texto
/// somava `1,0` (`0.000 + 0.500 + 0.500`) enquanto o motor somava `2,0` (o `(0.000)` virava o
/// neutro). **Dois leitores do mesmo texto, e o gate escolheu o que não está no produto.**
/// O default que o MANIFESTO declara para um param — a única fonte.
fn manifest_default(name: &str) -> f32 {
    MANIFEST
        .params
        .iter()
        .find(|p| p.name == name)
        .map_or(0.0, |p| p.default)
}

/// **A porta de SONDA da ÂNCORA** — o factor de comprimento em `frac → 0`, que é onde a
/// interpolação começa.
///
/// ⚠️ A âncora deixou de ser uma constante em 2026-08-29: ela é o valor, no início da
/// travessia, da normalização que o [`build`] faz a cada instante (*«que comprimento põe a
/// figura na rampa recta entre a geração anterior e a nova inteira?»*). Aqui mede-se esse
/// valor sozinho, para um gate poder afirmar o NÚMERO e não só o efeito.
#[must_use]
pub fn probe_anchor(axiom: &str, rules: &str, generations: f32) -> f32 {
    let p = probe_params(generations, &[(param::CONTINUOUS_ANGLE, 1.0)]);
    let params = |n: &str| p.by_name(n);
    let (gens, _) = generation_plan(p.generations);
    let d = derive::derive(
        &derive::axiom_modules(axiom, &params),
        &grammar::parse_rules(rules),
        gens,
        1,
        MAX_MODULES,
        &params,
    );
    if d.previous.is_empty() {
        return 1.0;
    }
    let setup = |ang: f32| turtle::Setup {
        angle: p.angle,
        step: p.step,
        width: p.width,
        width_scale: p.width_scale,
        length_scale: p.length_scale,
        root_angle: p.root_angle,
        tropism: p.tropism,
        tropism_angle: p.tropism_angle,
        angle_frac: ang,
        youngest: (d.generations, 1.0),
        orient_world: true,
    };
    let antes = turtle::span(&d.previous, &setup(1.0));
    // ⚠️ **Com as dobras FECHADAS** — é a pose de onde a interpolação parte. Medi-la aberta dá
    // `1/3` onde a resposta é `1/5`, e uma mutação que trocasse as duas já SOBREVIVEU uma vez.
    let achatada = turtle::span(&d.chain, &setup(0.0));
    if antes > 1e-6 && achatada > 1e-6 {
        (antes / achatada).clamp(0.02, 1.0)
    } else {
        1.0
    }
}

#[must_use]
pub fn probe_rule_weights(rules: &str) -> Vec<f32> {
    grammar::parse_rules(rules)
        .iter()
        .map(|r| r.weight)
        .collect()
}

#[must_use]
fn probe_params(generations: f32, overrides: &[(&str, f32)]) -> Params {
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
        orient: 0.0,
        // ⚠️⚠️ **`Grammar`, e NÃO o default do manifesto.** Esta porta recebe um axioma e
        // umas regras nos ARGUMENTOS; abri-la em `Guided` faria o nó ignorá-los, e as
        // dezenas de gates que a chamam passariam a medir a gramática derivada em vez da
        // que escreveram — todos verdes, todos sobre outra coisa. *Uma porta de sonda tem
        // de honrar o que lhe é passado, e o modo é o que decide se ela o honra.*
        mode: MODE_GRAMMAR as f32,
        branches: 2.0,
        segments: 1.0,
        variation: 0.0,
        bend: 0.0,
        // ⚠️⚠️ **LIDOS DO MANIFESTO, nunca cravados** — e a diferença mordeu no mesmo dia: com
        // o `continuous_angle` a `1.0` aqui, a bancada continuou a imprimir os números CURADOS
        // depois de o default do produto ter ido a `0.0`. *Uma sonda com o default cravado
        // mede o que ela acha que o produto é.*
        continuous_length: manifest_default(param::CONTINUOUS_LENGTH),
        continuous_angle: manifest_default(param::CONTINUOUS_ANGLE),
        step_scale: manifest_default(param::STEP_SCALE),
    };
    for (n, v) in overrides {
        match *n {
            param::ANGLE => p.angle = *v,
            param::MODE => p.mode = *v,
            param::BRANCHES => p.branches = *v,
            param::SEGMENTS => p.segments = *v,
            param::VARIATION => p.variation = *v,
            param::BEND => p.bend = *v,
            param::CONTINUOUS_LENGTH => p.continuous_length = *v,
            param::CONTINUOUS_ANGLE => p.continuous_angle = *v,
            param::STEP_SCALE => p.step_scale = *v,
            param::STEP => p.step = *v,
            param::WIDTH => p.width = *v,
            param::WIDTH_SCALE => p.width_scale = *v,
            param::LENGTH_SCALE => p.length_scale = *v,
            param::ROOT_ANGLE => p.root_angle = *v,
            param::TROPISM => p.tropism = *v,
            param::TROPISM_ANGLE => p.tropism_angle = *v,
            param::SEED => p.seed = *v,
            param::ORIENT => p.orient = *v,
            other => panic!("probe_params: param desconhecido {other}"),
        }
    }
    p
}

/// **A porta de SONDA** — derivar + interpretar com os defaults do manifesto, mudando só o
/// que quem mede quer mudar.
///
/// ⚠️ `pub` sem `#[cfg(test)]` de propósito: a bancada que MEDE o tecto
/// (`tests/measure_lsystem_ceiling.rs`) é um alvo de integração e não vê itens de teste.
#[must_use]
pub fn probe_build(
    axiom: &str,
    rules: &str,
    generations: f32,
    overrides: &[(&str, f32)],
) -> ph2d_nodegraph::attr::Stream {
    build(axiom, rules, &probe_params(generations, overrides))
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
