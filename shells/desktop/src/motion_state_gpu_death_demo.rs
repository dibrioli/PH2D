//! **A CENTELHA QUE ESTOURA** (`PH2D_GPU_COOK_DEMO=27`) — o P0 da folha 13 do doc 89 montado
//! como documento pronto para smoke: *nada era disparado por uma MORTE*.
//!
//! A cena `=24` mostra um evento decidindo **o que passa a existir**; esta mostra o evento
//! sendo a **própria morte** — o `Trigger Event On Die` do VFX Graph, o `POP Replicate` do
//! Houdini. E o achado é que ele **não é um nó**: é uma fiação.
//!
//! **A cadeia:**
//! `motion.grid(1×3) → sim.zone → force.wind → force.curl → sim.step → sim.lifetime`, e daí
//! as duas saídas novas do lifetime fecham o laço por fora:
//!
//! ```text
//!   sim.lifetime.died  ──→ sim.spawn.template   (a carga: ONDE, com que velocidade)
//!   sim.lifetime.pulse ──→ sim.spawn.pulse      (o gatilho: estas linhas morreram)
//!   sim.lifetime.out ─┬─→ motion.combine.0      (os vivos)
//!   sim.spawn.out ────┴─→ motion.combine.1      (os recém-nascidos)
//! ```
//!
//! ⚠️ **`rate = 0`, como na `=24`, e pela mesma razão:** o nascimento por taxa e o nascimento
//! por evento são aditivos, então zerar a taxa deixa a **morte como única autora** de tudo o
//! que aparece depois das três sementes. Se a fiação não funcionasse, a tela ficaria **VAZIA
//! para sempre** ao fim da primeira vida — e não meio cheia.
//!
//! ⚠️ **A CASCATA é o fenômeno, não um efeito colateral.** Os filhos também envelhecem e
//! também morrem, então cada geração multiplica a anterior por `BURST`: é o que *"uma morte
//! dá à luz"* significa quando quem nasce pode morrer. Com `BURST = 2` ela é **contável a
//! olho** nas primeiras gerações, que é o que faz dela um oráculo em vez de uma nuvem.
//!
//! ⚠️ **E o `force.curl` não é enfeite:** um filho herda **toda** coluna do cadáver, inclusive
//! a velocidade, então sem um campo que os separe os `BURST` filhos viajariam empilhados como
//! um ponto só — a lei estaria certa e **invisível**. O curl é divergence-free (Bridson), logo
//! espalha sem inflar o conjunto.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// Quantas sementes. Três porque a cascata precisa de mais de uma origem para o olho separar
/// *"multiplicou"* de *"andou"*, e mais que três já entra na segunda geração antes de a
/// primeira ser contada.
pub(super) const SEEDS: f32 = 3.0;
/// Quantos filhos cada morte gera. **DOIS**, e o número é o oráculo: a população de cada
/// geração é a anterior VEZES este número, e um artista conta 3 → 6 → 12 sem sonda nenhuma.
/// Um burst grande faria a mesma lei virar uma nuvem em dois segundos.
pub(super) const BURST: f32 = 2.0;
/// A vida nominal de um elemento. **MEDIDA contra o relógio do smoke:** a 1,5 s cabem umas
/// quatro gerações nos primeiros seis segundos, que é o tempo que alguém olha uma cena antes
/// de decidir se ela está certa.
pub(super) const LIFE: f32 = 1.5;

/// **A CENTELHA QUE ESTOURA** (`PH2D_GPU_COOK_DEMO=27`).
///
/// O que o artista vê: três centelhas subindo, cada uma **estourando em duas** no instante e
/// no lugar em que morre, e as filhas fazendo o mesmo — uma cascata que enche a tela a partir
/// de três pontos, sem que nada além da morte tenha dado à luz.
pub(super) fn build_gpu_death_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::{Edge, Pos};
    let g = &mut doc.graph;

    // AS SEMENTES: três pontos na base, bem separados.
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", SEEDS);
    g.set_param(grid, "gap_x", 2.6);
    let scale = g.add_node("motion.scale");
    g.set_param(scale, "amount", 0.07);

    // A ZONA e o interior.
    let zone = g.add_node("sim.zone");
    let wind = g.add_node("force.wind");
    g.set_param(wind, "angle", 90.0); // para cima: as centelhas SOBEM
    g.set_param(wind, "strength", 1.4);
    g.set_param(wind, "gust", 0.0);
    // O espalhador — sem ele os filhos de uma morte viajam empilhados (ver o doc do módulo).
    let curl = g.add_node("force.curl");
    g.set_param(curl, "strength", 2.6);
    g.set_param(curl, "scale", 0.7);
    let step = g.add_node("sim.step");
    let life = g.add_node("sim.lifetime");
    g.set_param(life, "life", LIFE);
    // ⚠️ **A variância é INERTE nas três sementes, e a medição é que disse isso.** O
    // `life_of` espalha a vida por um `hash(seed, id, lane)`, e **`motion.grid` não escreve
    // `id`** (só `motion.emitter` e `sim.spawn` o fazem — conferido por grep): as três leem
    // `id = 0` pelo zero-fill do `scalar()`, tiram o MESMO span e morrem no mesmo tique.
    // Acontece que para esta cena isso é o melhor dos dois mundos — o 1º estouro sai
    // **limpo e contável** (3 → 6 de uma vez, medido em t = 1,10 s) e as gerações seguintes,
    // essas sim com id de nascimento, escalonam sozinhas. A nota fica porque a frase que eu
    // ia escrever aqui (*"sem isto a geração inteira estoura junto"*) era falsa.
    g.set_param(life, "variance", 0.3);
    // O NASCIMENTO POR MORTE.
    let spawn = g.add_node("sim.spawn");
    g.set_param(spawn, "rate", 0.0); // SÓ a morte dá à luz
    g.set_param(spawn, "burst", BURST);
    let combine = g.add_node("motion.combine");
    let out = g.add_node("motion.output");

    for (i, n) in [grid, scale, zone, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 190.0,
                y: 200.0,
            },
        );
    }
    for (i, n) in [wind, curl, step, life, combine].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 80.0 + i as f32 * 170.0,
                y: 360.0,
            },
        );
    }
    // O nascimento fica ABAIXO do lifetime: é dele que ele se alimenta.
    g.set_pos(spawn, Pos { x: 590.0, y: 520.0 });

    for (a, ap, b, bp) in [
        (grid, 0u16, scale, 0u16),
        (scale, 0, zone, 0),
        // A entrada de estado que o motor gerencia: o `pre` sai da ZONA para o primeiro nó
        // do corpo, e a volta ao `state` é aresta normal.
        (zone, 0, wind, 0),
        (wind, 0, curl, 0),
        (curl, 0, step, 0),
        (step, 0, life, 0),
        // As duas saídas novas: a carga e o gatilho, no mesmo nascimento.
        (life, 1, spawn, 0),
        (life, 2, spawn, 1),
        // Os vivos e os recém-nascidos entram juntos no estado do próximo tique.
        (life, 0, combine, 0),
        (spawn, 0, combine, 1),
        (combine, 0, zone, 1),
        (zone, 0, out, 0),
    ] {
        let delayed = (a, b) == (zone, wind);
        g.connect(Edge {
            from: (a, ap),
            to: (b, bp),
            delayed,
        })
        .ok()?;
    }
    g.validate(reg).ok()?;
    Some(vec![out])
}

#[cfg(test)]
#[path = "motion_state_gpu_death_demo_tests.rs"]
mod tests;
