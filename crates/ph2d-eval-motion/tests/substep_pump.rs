//! **O PUMP SUBSTEPA** (doc 89, folha 13 — o último P1), pela porta do produto.
//!
//! O motor é `Cook::substep` e tem gates próprios (`ph2d-node-registry-init/tests/substeps.rs`).
//! Estes provam a outra metade: que a marcha do pump o CHAMA, que a declaração vem do MANIFESTO
//! (um param chamado `substeps`, não uma tabela paralela) e que o default de 1 deixa o quadro
//! **byte-idêntico** ao mundo que não conhecia substeps.

use ph2d_eval_motion::MotionCookPump;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::TimeScopes;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

const DT: f64 = 1.0 / 60.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).expect("grid");
    ph2d_node_sim_zone::register(&mut reg).expect("zone");
    ph2d_node_sim_step::register(&mut reg).expect("step");
    ph2d_node_force_wind::register(&mut reg).expect("wind");
    reg
}

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .expect("wire");
}

/// grid → zone(init) ; zone =pre=> wind → step → zone(state)
fn falling_zone(g: &mut Graph) -> NodeId {
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 1.0);
    g.set_param(seed, "cols", 1.0);
    let zone = g.add_node("sim.zone");
    let wind = g.add_node("force.wind");
    g.set_param(wind, "angle", 0.0);
    g.set_param(wind, "strength", 40.0);
    g.set_param(wind, "gust", 0.0);
    let step = g.add_node("sim.step");
    g.set_param(step, "damping", 1.0);
    wire(g, seed, 0, zone, 0, false);
    wire(g, zone, 0, wind, 0, true);
    wire(g, wind, 0, step, 0, false);
    wire(g, step, 0, zone, 1, false);
    zone
}

fn px(s: &Stream) -> f32 {
    match s.get("P") {
        Some(Column::Vec2(v)) if !v.is_empty() => v[0][0],
        _ => f32::NAN,
    }
}

/// Marcha `frames` tiques pela porta do pump e devolve o P.x da zona.
fn march(g: &Graph, reg: &NodeRegistry, zone: NodeId, frames: u64) -> f32 {
    let mut pump = MotionCookPump::new();
    let scopes = TimeScopes::new();
    for tick in 0..frames {
        pump.mark_dirty();
        pump.advance_or_scrub_to_nodes_scoped(g, reg, &[zone], tick, |t| t as f64 * DT, &scopes);
    }
    pump.boundary_streams()
        .iter()
        .find(|(n, _)| *n == zone)
        .map(|(_, s)| px(s))
        .expect("a zona e uma fronteira")
}

/// **A entrega:** o param na zona chega ao motor pela marcha do pump — quatro sub-passadas
/// integram MAIS perto da resposta exata que uma.
///
/// FALSIFICADO se o pump não chamasse o achador: as duas colunas seriam o mesmo número.
#[test]
fn the_pump_substeps_a_zone_that_declares_it() {
    let reg = registry();
    // ⚠️ O `tick` do pump e 0-based: marchar `0..=60` e o que termina no playhead 1,0 s.
    // Marchar `0..60` para em 59/60 e a comparacao contra a exata em t=1 mede a defasagem
    // da FIXTURE em vez do integrador -- foi o que a 1ª versao deste gate mediu.
    const TICKS: u64 = 61;
    let exact = 40.0f32 / 2.0;

    let mut g1 = Graph::new();
    let z1 = falling_zone(&mut g1);
    let coarse = march(&g1, &reg, z1, TICKS);

    let mut g4 = Graph::new();
    let z4 = falling_zone(&mut g4);
    g4.set_param(z4, "substeps", 4.0);
    let fine = march(&g4, &reg, z4, TICKS);

    let (e1, e4) = ((coarse - exact).abs(), (fine - exact).abs());
    let ratio = e1 / e4;
    assert!(
        (3.6..4.4).contains(&ratio),
        "quatro sub-passadas cortam o erro em QUATRO: {ratio:.3}x ({e1:.4} -> {e4:.4})"
    );
}

/// **O default de 1 é o mundo de antes, ao BIT.** Um param novo que mudasse o quadro em repouso
/// seria uma regressão em toda cena já autorada.
#[test]
fn the_default_of_one_leaves_the_frame_byte_identical() {
    let reg = registry();
    let mut g = Graph::new();
    let zone = falling_zone(&mut g);
    let implicit = march(&g, &reg, zone, 40);

    // O MESMO grafo, com o 1 escrito à mão: o caminho do override e o do default do manifesto
    // têm de pousar no mesmo bit.
    let mut g2 = Graph::new();
    let z2 = falling_zone(&mut g2);
    g2.set_param(z2, "substeps", 1.0);
    let explicit = march(&g2, &reg, z2, 40);

    assert_eq!(implicit.to_bits(), explicit.to_bits());
}

/// **A declaração vem do MANIFESTO**, e é por isso que não há tabela paralela a manter: quem
/// oferece o param é quem sub-tica. Um nó que não o declara é invisível ao achador.
///
/// ⚠️ **ISTO É UM SPOT-CHECK, NÃO O GUARDA — e a distinção custou um defeito.** Nomear três nós
/// prova que *aqueles três* estão bem e não diz nada sobre os outros ~118: foi por aí que a
/// `motion.verlet_rope` entrou, em 2026-08-16, com um param `substeps` que era um laço dentro do
/// `eval` dela, e o app passou a compor as duas leis (a cauda caía 4,8× menos do que os gates
/// daquele crate medem). O guarda é o **CENSO** sobre o registry inteiro,
/// `substeps::only_the_declared_clock_owners_offer_the_substeps_param` no `ph2d-node-registry-init`
/// — este teste fica por ser barato e por correr onde o registry completo não está.
#[test]
fn the_declaration_is_the_manifest_param_not_a_side_table() {
    assert!(
        ph2d_node_sim_zone::MANIFEST
            .param_default("substeps")
            .is_some(),
        "a zona declara o substep"
    );
    for m in [
        ph2d_node_sim_step::MANIFEST,
        ph2d_node_force_wind::MANIFEST,
        ph2d_node_motion_grid::MANIFEST,
    ] {
        assert!(
            m.param_default("substeps").is_none(),
            "{} nao sub-tica, e o achador nao pode inventa-lo",
            m.name
        );
    }
}
