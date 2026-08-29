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

/// As cinco colunas: `(rótulo, axioma, regras, gerações, semente, tropismo, ângulo)`.
///
/// ⚠️ **A 2 e a 3 têm de partilhar TUDO menos a semente** — é isso que torna a diferença
/// entre elas uma afirmação sobre a estocástica em vez de sobre duas gramáticas.
pub(crate) const PLANTS: &[(&str, &str, &str, f32, f32, f32, f32)] = &[
    (
        "1. parametrica (o default)",
        ls::DEFAULT_AXIOM,
        ls::DEFAULT_RULES,
        6.0,
        1.0,
        0.0,
        25.0,
    ),
    (
        "2. estocastica, semente 1",
        "A(step)",
        STOCHASTIC,
        6.0,
        1.0,
        0.0,
        25.0,
    ),
    (
        "3. estocastica, semente 9",
        "A(step)",
        STOCHASTIC,
        6.0,
        9.0,
        0.0,
        25.0,
    ),
    (
        "4. a 1 com gravidade",
        ls::DEFAULT_AXIOM,
        ls::DEFAULT_RULES,
        6.0,
        1.0,
        GRAVITY,
        25.0,
    ),
    (
        "5. samambaia, a CRESCER",
        "A(step)",
        FERN,
        GROW_MAX,
        1.0,
        0.0,
        28.0,
    ),
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
        .map(|(k, p)| crate::motion_demo_legend::Caption::new([(k as f32 - 2.0) * COL_W, 2.4], p.0))
        .collect()
}

/// Os rótulos, para o anúncio no terminal.
pub(crate) fn labels() -> impl Iterator<Item = &'static str> {
    PLANTS.iter().map(|p| p.0)
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
    for (k, (_, axiom, rules, gens, seed, tropism, angle)) in PLANTS.iter().enumerate() {
        let lane = 120.0 + k as f32 * 300.0;
        let l = g.add_node("source.lsystem");
        g.set_pos(l, Pos { x: 260.0, y: lane });
        g.set_text_param(l, ls::AXIOM_PARAM, *axiom);
        g.set_text_param(l, ls::RULES_PARAM, *rules);
        g.set_param(l, ls::param::GENERATIONS, *gens);
        g.set_param(l, ls::param::SEED, *seed);
        g.set_param(l, ls::param::TROPISM, *tropism);
        g.set_param(l, ls::param::ANGLE, *angle);
        g.set_param(l, ls::param::WIDTH, TRUNK_W);
        g.set_param(l, ls::param::STEP, 0.28);

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
