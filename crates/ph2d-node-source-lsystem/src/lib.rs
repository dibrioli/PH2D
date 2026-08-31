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

/// **O esqueleto vira RAMOS** — a metade da lei do varrimento que vive dentro do nó (a outra é
/// da shell, que tem o motor de traço). Pública porque a shell a chama.
pub mod branch;
mod manifest;
pub use manifest::{MANIFEST, param};
mod derive;
mod grammar;
/// **A LEI DO CRESCIMENTO** — a remapagem do `Growth` e a razão que a ancora (HR-18).
mod growth;
mod hash;
/// **OS NÚMEROS DO PAINEL** — o que o nó lê antes de fazer o que faz (HR-18).
mod params;
/// **OS MOLDES** — a tabela e o que cada um exige (HR-18).
mod presets;
/// **AS PORTAS DE SONDA** — por onde um gate ou uma bancada alcança o produto (HR-18).
mod probe;
/// **A PORTA DAS FITAS** — o vocabulário do modo `Branches` e o contrato com a shell (HR-18).
mod ribbons;
pub mod shape;
mod trig;
mod turtle;
/// **A FACE DO NÓ** — o que o painel mostra da lei (HR-18).
mod ui;

use growth::{growth_generations, measure_ratio};
use params::Params;
pub use presets::*;
pub use probe::*;
// ⚠️ Re-exportado sob o nome de sempre: `ph2d_node_source_lsystem::ribbon_key` continua a
// resolver, então o corte por LOC não mudou o endereço de ninguém.
pub use ribbons::*;
use ui::{PARAM_GATES, PARAM_GATES_ABOVE, PARAM_GROUPS, PARAM_HARD_MAX, PARAM_HINTS, PARAM_UNITS};

use ph2d_node_registry::{
    NodeRegistry, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget, RegistryError,
};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{NodeManifest, NodeOp, NodeTypeId};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// **O text param do AXIOMA** — a cadeia de partida (o canal do doc 32: um `ParamSpec` é
/// `f32` e o manifesto é congelado, então o texto vive no `Graph`).
pub const AXIOM_PARAM: &str = "axiom";

/// **O text param das REGRAS**, separadas por `;` — ver [`grammar`] para a forma, e para a
/// razão de serem de uma linha só (um `\n` corrompe o ficheiro do projeto).
pub const RULES_PARAM: &str = "rules";

/// **OS TRÊS OBJECTOS QUE AS LETRAS PLANTAM** — `J`, `K` e `M`, na ordem do alfabeto.
///
/// Report do Enio (2026-08-29): *"não há modos de escolher objetos para as fases de
/// crescimento. Exemplo: as folhas."*
///
/// ⭐ **É a resposta das quatro referências, e ela é IDÊNTICA:** o *L-System SOP* do Houdini tem
/// três entradas de geometria chamadas, literalmente, **`J`, `K` e `M`** — *"any geometry
/// connected to these inputs is created in the sequence by these letters"*. O nosso alfabeto já
/// tinha as três letras, com o mesmo significado e vindas da mesma fonte (o ABOP); faltava só
/// dizer **qual** objecto cada uma pousa.
///
/// ⚠️ **Um NOME, não uma porta.** O Houdini usa portas porque é um grafo de geometria; aqui o
/// nome de um objecto da cena é a porta de toda a casa (o `fx.glow`, o `motion.path`, o
/// `source.object`), e ela **é alcançável** — o painel pinta um [`ParamWidget::Source`], com
/// chips dos nomes que a app publicou.
///
/// ⚠️ **E «por fase de crescimento» resolve-se pela GRAMÁTICA, não por uma tabela de fases:** a
/// regra que dispara na geração `N` é que emite a letra, e a letra escolhe o objecto. É por isso
/// que nenhuma das quatro referências tem tabela de fases — e quem quiser filtrar tem a coluna
/// `gen`, que a tartaruga já emite por elemento.
pub const LEAF_PARAMS: [&str; 3] = ["leaf_j", "leaf_k", "leaf_m"];

/// As letras que [`LEAF_PARAMS`] serve, na mesma ordem — *o índice é o par*.
///
/// ⚠️ **Duas listas com a mesma ordem, e um gate a dizê-lo** (`the_letters_and_their_params_are_
/// one_list`): a shell percorre uma e o nó declara a outra, e trocar a ordem numa só faria a
/// flor nascer onde o artista pediu folha — sem erro nenhum.
pub const LEAF_SYMBOLS: &[u8; 3] = b"JKM";

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

/// **O modo GUIADO** — os sliders de forma mandam, e a gramática é derivada deles.
pub const MODE_GUIDED: i32 = 0;
/// **O modo GRAMÁTICA** — o texto manda, e os sliders de forma somem.
pub const MODE_GRAMMAR: i32 = 1;

/// Os dois modos de autoria. ⚠️ A ordem É o valor gravado: `Guided` tem de ficar em `0`.
pub const MODE_LABELS: &[&str] = &["Guided", "Grammar"];

/// **O que a coluna `rot` quer dizer** — ver [`crate::turtle::Setup::orient_world`] para o
/// mecanismo. `0` = mundo (o desenho alinha com o ramo) · `1` = local (o contrato do `rig.*`).
pub const ORIENT_LABELS: &[&str] = &["Growth", "Local"];

/// ⚠️ **`Keep Own Colour` é o DEFAULT**, e é o pedido do Enio de 2026-08-30: *"uma opção para
/// livrar as folhas, os frutos do tint que pinta tudo na árvore"*.
///
/// ⭐ **A cura não é um canal novo: é o que a casa inteira já fala.** O `motion.tint` faz
/// `lerp(existente, alvo, falloff)` — com `falloff = 0` a instância **mantém a cor que tem**.
/// Publicar `0` nas linhas de folha exclui-as de todo nó que honre essa máscara, e `1` deixa a
/// corrente sem coluna nenhuma ⇒ byte-idêntica ao que havia antes deste param.
pub const LEAF_EFFECTS_LABELS: &[&str] = &["Keep Own Colour", "Reached"];

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
pub const DEFAULT_RULES: &str = "A(s) -> F(s)[J]![+A(s*0.7)][-A(s*0.7)]";

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
        // ⭐ **O modo RAMOS lê a geometria que a shell construiu** (a lei do `source.shape`): um
        // nó não alcança a biblioteca vetorial, e é essa cerca que deixa o cook memoizar e
        // repetir ao bit. Sem externo publicado o stream é VAZIO — nada desenha, e nada estoura;
        // é o quadro entre o artista ligar o modo e a shell publicar.
        if p.ribbons() {
            // ⚠️ **A chave lê pelo `ctx.param`, NÃO pelo `Params::by_name`** — aquele resolve a
            // escada inteira e este é um `match` que devolve `0` para o que não lista (o
            // `preset`, por exemplo). Um `0` de «não listado» na chave do nó, contra o valor real
            // na chave da shell, daria duas chaves e uma planta invisível **sem erro nenhum**.
            let key = ribbon_key(|n| ctx.param(n), &axiom_src, &rules_src);
            let out = ctx.external(&key).clone();
            ctx.emit(out);
            return;
        }
        let out = build(&axiom_src, &rules_src, &p);
        ctx.emit(out);
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
    // ⭐ **O `Growth` remapeia as gerações para o TAMANHO crescer por igual** — e em `1.0`
    // (o default) devolve `p.generations` exactamente, sem medir nada.
    let generations = if p.growth < 1.0 {
        growth_generations(
            p.generations,
            p.growth,
            measure_ratio(axiom_src, rules_src, p),
        )
    } else {
        p.generations
    };
    let (gens, youngest) = generation_plan(generations);
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
        step: p.step * p.step_scale.max(1e-4).powf(generations.clamp(0.0, 64.0)),
        width: p.width,
        width_scale: p.width_scale,
        length_scale: p.length_scale,
        root_angle: p.root_angle,
        tropism: p.tropism,
        tropism_angle: p.tropism_angle,
        angle_frac: ang,
        youngest: (youngest.0, len * youngest.1),
        orient_world: p.orient.round() as i32 == 0,
        leaf_first_level: p.leaf_first_level,
        leaf_angle: p.leaf_angle,
        leaf_spread: p.leaf_spread,
        leaf_effects: p.leaf_effects.round() as i32 != 0,
        seed: p.seed,
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
    //
    // ⭐⭐⭐ **E O QUE SE MEDE MUDOU EM 2026-08-30** — a régua deixou de ser a caixa alinhada
    // aos eixos e passou a ser invariante à rotação, porque o dragão RODA enquanto cresce e a
    // lei estava a normalizar a orientação em vez do tamanho. O mecanismo, os números e as três
    // hipóteses refutadas vivem onde o assunto vive: [`growth`] (o §2026-08-30) e
    // [`turtle::mean_width`]. *A lei desta função não mudou uma linha; mudou a GRANDEZA que ela
    // iguala.*

    let frac = youngest.1;
    // ⚠️ **A MESMA PORTA que a remapagem do `Growth` usa** — ver [`derive::Derived::grows_by_refining`]:
    // até 2026-08-30 esta pergunta era respondida aqui pela estrutura e lá por um limiar sobre
    // a razão medida, e as duas respostas discordavam no modo GUIADO.
    let grows_by_refining = d.grows_by_refining();
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
        let antes = turtle::mean_width(&d.previous, &base(1.0, 1.0, (d.generations, 1.0)));
        let cheia = turtle::mean_width(&d.chain, &base(1.0, 1.0, (d.generations, 1.0)));
        let agora = turtle::mean_width(&d.chain, &base(1.0, frac, (d.generations, 1.0)));
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
            // Medido: `F -> F[+F]F[-F]F` e `F -> F+F-F-F+F` expandem `3,000` e `3,003` por
            // geracao, logo `step_scale = 1/3` deixa a figura ~do mesmo tamanho e so' lhe da'
            // detalhe. As que crescem pela ponta ficam entre `1,053` e `1,154` e nao precisam
            // dele -- por isso o default e' `1,0`, o neutro EXACTO.
            //
            // ⛔ **Os `3,00`/`1,06` que aqui estavam eram da regua ANTIGA.** A auditoria de
            // 2026-08-30 apanhou-os: a Koch e' `3,0028` com a regua invariante (a caixa de eixo
            // dava `3,000000` exacto), e o `1,06` descrevia UM dos quatro que crescem pela
            // ponta -- o mais forte esta' 9 % acima dele. ⚠️ Ou seja a regua nova troca `0,09 %`
            // de exactidao na razao por invariancia a rotacao: e' um bom negocio, e nao estava
            // escrito em lado nenhum.
            //
            // O expoente e' o `generations` FRACCIONARIO e nao o `ceil`: com o inteiro o
            // passo daria um degrau em cada travessia, que e' o defeito que isto cura.
            // Um `powf` e' transcendental (HR-5) e corre DUAS vezes por cozedura na rota
            // fraccionaria (o `base()` fundido pelo compilador + este `Setup` final -- contado
            // na asm pela auditoria de 2026-08-30, que refutou o «UMA vez» que aqui estava),
            // nunca por
            // elemento -- a mesma cerca (e o mesmo lado dela) da avaliacao de expressoes.
            step: p.step * p.step_scale.max(1e-4).powf(generations.clamp(0.0, 64.0)),
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
            leaf_first_level: p.leaf_first_level,
            leaf_angle: p.leaf_angle,
            leaf_spread: p.leaf_spread,
            leaf_effects: p.leaf_effects.round() as i32 != 0,
            seed: p.seed,
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
    reg.register_param_gates_above(MANIFEST.id, PARAM_GATES_ABOVE);
    reg.register_param_groups(MANIFEST.id, PARAM_GROUPS);
    Ok(())
}

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

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "lib_marks_tests.rs"]
mod marks_tests;
