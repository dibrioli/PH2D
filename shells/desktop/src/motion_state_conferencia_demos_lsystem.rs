//! **A CENA DO L-SYSTEM** (`PH2D_GPU_COOK_DEMO=108`) — cinco plantas, e cada uma isola UMA
//! coisa que o nó faz e que mais nenhum nó desta casa faz.
//!
//! ## Por que cinco e não uma bonita
//!
//! Uma planta só seria bonita e não provaria nada: um `motion.scatter` com um `field.remap`
//! também faz um borrão vegetal. O que só um L-System faz é **estrutura que se reescreve**,
//! e as cinco colunas separam as dimensões dela para que uma falha tenha ENDEREÇO:
//!
//! | coluna | o que ela isola | como se lê que falhou |
//! |---|---|---|
//! | 1 | a **árvore paramétrica** (o default de fábrica) | os ramos não afinam ao subir |
//! | 2 e 3 | **estocástica** — a MESMA gramática, sementes diferentes | as duas saem gémeas |
//! | 4 | **tropismo** — a mesma planta 1 com gravidade | ela não verga |
//! | 5 | o arbusto clássico do ABOP, com **`Generations` DIRIGIDO por um relógio** | ela não cresce, ou cresce aos saltos |
//!
//! ⭐ **A quinta é a que prova a feature mais difícil**: as gerações fraccionárias. Um
//! gerador que só aceitasse inteiros faria a planta SALTAR de tamanho quatro vezes por
//! ciclo; esta cresce continuamente, e o rebento novo estica a partir do ramo que já lá
//! estava.
//!
//! ⚠️ **`Tropism` POSITIVO puxa PARA a `Tropism Direction`** (negativo empurra para longe —
//! um fototropismo às avessas). A 1.ª redacção desta cena pôs `−14` a querer dizer «para
//! baixo» e a planta saiu **mais direita** do que a que não tem gravidade nenhuma; o gate
//! `the_gravity_plant_hangs_lower_than_the_one_it_copies` apanhou-o. *A direcção já é um
//! param — o sinal é a FORÇA, não um segundo eixo.*
//!
//! ⚠️ **A espessura vai na coluna `size`**, então a planta desenha-se sozinha sem um nó a
//! jusante: cada elemento é um ponto do tamanho do ramo naquele sítio. É por isso que o
//! tronco é grosso e a ponta é fina — o `!` da gramática, visto.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_node_source_lsystem as ls;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

/// A largura de uma coluna, em unidades de mundo.
const COL_W: f32 = 3.2;
/// A espessura do tronco. Pequena de propósito: cada elemento é um ponto, e um `size` de
/// `1` faria as pontas colidirem num borrão.
const TRUNK_W: f32 = 0.09;
/// Quantos segundos o relógio da quinta coluna leva a crescer e a voltar.
const GROW_PERIOD: f32 = 6.0;
/// Até onde o relógio faz a samambaia crescer. `6` dá 64 elementos no pico — densidade
/// comparável às outras quatro colunas, que ficam entre 26 e 64.
const GROW_MAX: f32 = 6.0;
/// **E de onde ele parte.**
///
/// ⚠️ **Report do Enio, 2026-08-28: *"o tronco pisca uma vez"*.** Medido: no fundo do ciclo o
/// relógio levava o `Generations` a **zero**, e zero gerações é o axioma por derivar — um
/// módulo MUDO. A planta ficava com **um** elemento (a raiz, que não desenha nada) e sumia,
/// uma vez por volta. *O pisca não estava no crescimento: estava no fundo do relógio.*
///
/// `1` é o menor valor que ainda desenha — um rebento de um segmento. Ele é o PISO do LFO, e
/// é isso que faz a coluna nascer de um broto em vez de nascer do nada.
const GROW_MIN: f32 = 1.0;
/// Quanto a quarta planta verga.
///
/// ⚠️ **É um número de DEMO, e a barra é a vista.** A lei do ABOP anula-se quando o ramo já
/// aponta ao longo da gravidade (`α = e·(H × T)`), então uma árvore de ramos curtos e
/// simétricos sente pouco: a `14` a massa desce `3 %` da altura, que da cadeira não se vê. O
/// gate `the_gravity_plant_hangs_lower_than_the_one_it_copies` mede a descida contra a
/// ALTURA da planta, então o número aqui responde a *«dá para ver?»* e não a *«funciona?»*.
const GRAVITY: f32 = 35.0;

/// Quantos rebentos a coluna guiada abre — **3, e não o default 2**, para que a coluna
/// DEMONSTRE o slider em vez de o deixar no valor que já existia.
const GUIDED_BRANCHES: f32 = 3.0;
/// E quanto de tronco limpo antes da primeira bifurcação — o outro slider que só este modo
/// tem, e o que mais muda a silhueta.
const GUIDED_SEGMENTS: f32 = 2.0;

/// ⭐ **O número que torna a planta GUIADA idêntica à gramática de fábrica.**
///
/// O modo guiado emite `A(s*length_scale)`; a gramática de fábrica traz o literal `0.7` lá
/// dentro. Com o slider aqui, as duas expressões são **a mesma**, e por isso a coluna 1 pode
/// ser autorada por sliders sem que a coluna 4 (que a copia por gramática) deixe de a
/// espelhar. O gate mede-o **ao bit**.
const GUIDED_LENGTH_SCALE: f32 = 0.7;

/// Uma coluna da cena.
///
/// ⚠️ **Campos com NOME, e não uma tupla de oito** — a tupla passou a ser «muito complexa»
/// para o clippy no dia em que o `guided` entrou, e o aviso estava certo por outra razão:
/// `PLANTS[3].5` não diz a ninguém que aquilo é o tropismo, e os gates desta cena leem a
/// tabela por índice.
pub(crate) struct Plant {
    /// A ficha por cima da coluna, no canvas e no terminal.
    pub label: &'static str,
    /// O axioma AUTORADO. ⚠️ Na coluna guiada ele não é escrito no grafo — fica aqui a ser o
    /// ORÁCULO contra o qual o gate da identidade ao bit compara.
    pub axiom: &'static str,
    /// As regras autoradas, com a mesma nota do [`Plant::axiom`].
    pub rules: &'static str,
    pub generations: f32,
    pub seed: f32,
    pub tropism: f32,
    pub angle: f32,
    /// `true` ⇒ a coluna é autorada por SLIDERS e o texto acima não vai ao grafo.
    pub guided: bool,
}

/// As cinco colunas.
///
/// ⚠️ **A 2 e a 3 têm de partilhar TUDO menos a semente** — é isso que torna a diferença
/// entre elas uma afirmação sobre a estocástica em vez de sobre duas gramáticas.
pub(crate) const PLANTS: &[Plant] = &[
    Plant {
        label: "1. GUIADA: 3 ramos, tronco de 2",
        axiom: ls::DEFAULT_AXIOM,
        rules: ls::DEFAULT_RULES,
        generations: 6.0,
        seed: 1.0,
        tropism: 0.0,
        angle: 25.0,
        guided: true,
    },
    Plant {
        label: "2. estocastica, semente 1",
        axiom: "A(step)",
        rules: STOCHASTIC,
        generations: 6.0,
        seed: 1.0,
        tropism: 0.0,
        angle: 25.0,
        guided: false,
    },
    Plant {
        label: "3. estocastica, semente 9",
        axiom: "A(step)",
        rules: STOCHASTIC,
        generations: 6.0,
        seed: 9.0,
        tropism: 0.0,
        angle: 25.0,
        guided: false,
    },
    Plant {
        label: "4. a 1 com gravidade",
        axiom: ls::DEFAULT_AXIOM,
        rules: ls::DEFAULT_RULES,
        generations: 6.0,
        seed: 1.0,
        tropism: GRAVITY,
        angle: 25.0,
        guided: false,
    },
    Plant {
        label: "5. samambaia, a CRESCER",
        axiom: "A(step)",
        rules: FERN,
        generations: GROW_MAX,
        seed: 1.0,
        tropism: 0.0,
        angle: 28.0,
        guided: false,
    },
];

/// **A gramática que de facto CRESCE** — o eixo principal estende-se e cada nó deixa um ramo
/// lateral, e o `F` é TERMINAL (nenhuma regra o reescreve).
///
/// ⚠️⚠️ **A 1.ª versão desta coluna usava o arbusto clássico do ABOP (`F -> F[+F]F[-F]F`) e
/// PISCAVA** — report do Enio, 2026-08-28: *"a cada ramo que vai nascer tudo se apaga e aparece
/// de vez"*. Aquela regra reescreve o próprio símbolo que desenha, então ao fim de cada
/// passagem a planta INTEIRA é nova: não há nada velho contra o qual um rebento se destaque, e
/// a fracção encolhia tudo (altura medida: `13,5 → 10,1 → 40,5 → 30,4`).
///
/// O nó passou a recusar a fracção nesse caso (ver `turtle::walk`), o que tira o pisca-pisca —
/// mas uma gramática de REFINAMENTO continua a não ter crescimento para mostrar, e esta coluna
/// existe para mostrar crescimento. ⇒ a gramática é outra, e a altura dela sobe **sem uma
/// queda**: `1,31 → 2,18` ao longo de três gerações.
const FERN: &str = "A(s) -> F(s)[+B(s*0.55)]!A(s*0.87) ; B(s) -> F(s)[-B(s*0.72)]B(s*0.8)";

/// Três produções para o mesmo predecessor: uma que se abre, uma que se dobra à esquerda, e
/// uma que só continua. Os pesos somam livremente — o nó normaliza.
const STOCHASTIC: &str = "A(s) -> (0.4) F(s)![+A(s*0.72)][-A(s*0.72)] ; \
                          A(s) -> (0.35) F(s)![+A(s*0.66)]-A(s*0.78) ; \
                          A(s) -> (0.25) F(s)!F(s*0.8)[+A(s*0.6)]";

/// As fichas no canvas — uma por cima de cada planta, na coordenada de mundo em que ela
/// está. É o que torna «a 4 verga» uma leitura e não uma contagem da esquerda para a direita.
pub(crate) fn captions() -> Vec<crate::motion_demo_legend::Caption> {
    PLANTS
        .iter()
        .enumerate()
        .map(|(k, p)| {
            crate::motion_demo_legend::Caption::new([(k as f32 - 2.0) * COL_W, 2.4], p.label)
        })
        .collect()
}

/// Os rótulos, para o anúncio no terminal.
pub(crate) fn labels() -> impl Iterator<Item = &'static str> {
    PLANTS.iter().map(|p| p.label)
}

fn wire(g: &mut ph2d_nodegraph::graph::Graph, a: NodeId, b: NodeId) -> Option<()> {
    g.connect(Edge {
        from: (a, 0),
        to: (b, 0),
        delayed: false,
    })
    .ok()
}

/// O documento da cena — uma sink por planta.
pub(crate) fn build_lsystem_demo_document(
    doc: &mut MotionDoc,
    _reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::new();
    for (k, p) in PLANTS.iter().enumerate() {
        let lane = 120.0 + k as f32 * 300.0;
        let l = g.add_node("source.lsystem");
        g.set_pos(l, Pos { x: 260.0, y: lane });
        // ⚠️⚠️ **CADA PLANTA DECLARA O MODO EM QUE FOI AUTORADA.** Desde 2026-08-29 o default
        // do nó é `Guided` (os sliders de forma), e uma cena que escrevesse os dois textos sem
        // dizer o modo mostraria **cinco vezes a mesma árvore derivada** — os textos
        // ignorados, a cena a compilar e a não provar nada.
        if p.guided {
            g.set_param(l, ls::param::MODE, ls::MODE_GUIDED as f32);
            // ⭐⭐ **E ELA MEXE NOS SLIDERS QUE SÃO O MODO** — achado do crítico de completude
            // da auditoria de 2026-08-29: a 1.ª versão desta coluna deixava `Branches`,
            // `Trunk Segments`, `Variation` e `Bend` nos DEFAULTS, então ela desenhava
            // exactamente a planta que o nó já fazia antes da feature existir. *Um smoke em
            // que a feature é indistinguível da ausência dela ensina que ela não foi
            // construída* — e o gate que havia (`every_plant_declares_the_grammar_mode...`)
            // media a DECLARAÇÃO, não a demonstração.
            g.set_param(l, ls::param::BRANCHES, GUIDED_BRANCHES);
            g.set_param(l, ls::param::SEGMENTS, GUIDED_SEGMENTS);
            // ⭐ **E este é o número que torna a derivada IDÊNTICA à gramática de fábrica.**
            // O guiado emite `A(s*length_scale)`; a gramática de fábrica tem o literal `0.7`
            // lá dentro. Com o slider em `0,7` as duas expressões são a mesma, e o gate
            // `the_guided_plant_draws_exactly_what_the_factory_grammar_draws` mede-o AO BIT
            // — é a prova de que os sliders não são uma segunda planta parecida.
            g.set_param(l, ls::param::LENGTH_SCALE, GUIDED_LENGTH_SCALE);
        } else {
            g.set_param(l, ls::param::MODE, ls::MODE_GRAMMAR as f32);
            g.set_text_param(l, ls::AXIOM_PARAM, p.axiom);
            g.set_text_param(l, ls::RULES_PARAM, p.rules);
        }
        g.set_param(l, ls::param::GENERATIONS, p.generations);
        g.set_param(l, ls::param::SEED, p.seed);
        g.set_param(l, ls::param::TROPISM, p.tropism);
        g.set_param(l, ls::param::ANGLE, p.angle);
        g.set_param(l, ls::param::WIDTH, TRUNK_W);
        g.set_param(l, ls::param::STEP, 0.28);

        // ⭐ **A SEGUNDA PLANTA AFINA AS PONTAS** — o knob que nasceu do report de 2026-08-30
        // (*"as pontas não têm opção de afinar"*).
        //
        // ⚠️ **Só ela, e é o ponto:** a `1` e a `3` correm a MESMA gramática estocástica com
        // sementes diferentes, então pôr o afinamento numa e não na outra dá o par lado a lado
        // — *uma cena que só mostra o estado novo não ensina o que ele mudou*. O param nasce em
        // `0` no nó (o carácter da ponta é decisão de quem vê), e é aqui que ele ganha um
        // exemplo em vez de um slider por descobrir.
        if k == 1 {
            g.set_param(l, ls::param::TIP_TAPER, 1.0);
        }

        // ⭐ **A QUINTA CRESCE.** O `Generations` é dirigido por um relógio, e é a única
        // maneira de ver a fracção: com o número a subir continuamente, o rebento novo
        // ESTICA a partir do ramo que já lá estava, em vez de a planta saltar de tamanho.
        if k == 4 {
            let clock = g.add_node("value.lfo");
            g.set_pos(clock, Pos { x: 60.0, y: lane });
            g.set_param(clock, "period", GROW_PERIOD);
            // A faixa é `[GROW_MIN, GROW_MAX]`, e o piso não é decoração — ver [`GROW_MIN`].
            g.set_param(clock, "amplitude", (GROW_MAX - GROW_MIN) / 2.0);
            g.set_param(clock, "offset", (GROW_MAX + GROW_MIN) / 2.0);
            g.drive_param(l, ls::param::GENERATIONS, (clock, 0)).ok()?;
        }

        // As colunas ficam lado a lado — o `motion.move` é a única coisa entre a planta e a
        // saída, para que o que se vê seja o nó e não uma cadeia.
        let mv = g.add_node("motion.move");
        g.set_pos(mv, Pos { x: 560.0, y: lane });
        g.set_param(mv, "dx", (k as f32 - 2.0) * COL_W);
        g.set_param(mv, "dy", -1.6);
        wire(g, l, mv)?;

        let out = g.add_node("motion.output");
        g.set_pos(out, Pos { x: 860.0, y: lane });
        wire(g, mv, out)?;
        sinks.push(out);
    }
    Some(sinks)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_lsystem_tests.rs"]
mod tests;
