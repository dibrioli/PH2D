//! Os gates da cerca da forma viva condicional — SEM placa, porque a varredura que a achou é
//! `#[ignore]` e o CI nunca a corre (`motion_bridge_gpu_varias_saidas_tests`).

use super::desenha_forma_condicional;
use crate::motion_state::MotionState;
use ph2d_node_source_lsystem as ls;
use ph2d_nodegraph::graph::Edge;
use ph2d_nodegraph::node::NodeTypeId;

/// O grafo que a `=108` tem e que um artista faz: um L-System seguido de um nó que a placa
/// despacha — é essa forma que punha a fita no caminho da placa.
fn lsystem_move_output(geometry: Option<f32>) -> MotionState {
    let mut m = MotionState::new();
    let g = &mut m.doc.graph;
    let l = g.add_node("source.lsystem");
    if let Some(v) = geometry {
        g.set_param(l, ls::param::GEOMETRY, v);
    }
    let mv = g.add_node("motion.move");
    let out = g.add_node("motion.output");
    for (a, b) in [(l, mv), (mv, out)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .unwrap();
    }
    m
}

/// ⭐⭐ **Em `Branches` (o DEFAULT do nó) o documento fica na CPU; em `Segments` vai à placa** —
/// as duas metades, porque cada uma sozinha mente: sem a primeira a placa desenha quadrados
/// onde a CPU desenha plantas, e sem a segunda um L-System de posições perdia a placa por nada.
#[test]
fn a_fita_do_lsystem_fica_na_cpu_e_os_segmentos_nao() {
    assert!(
        desenha_forma_condicional(&mut lsystem_move_output(None), 0.0),
        "o default do nó é `Branches` — a fita é uma forma viva e a placa não a desenha"
    );
    assert!(desenha_forma_condicional(
        &mut lsystem_move_output(Some(ls::GEOMETRY_BRANCHES as f32)),
        0.0
    ));
    assert!(
        !desenha_forma_condicional(
            &mut lsystem_move_output(Some(ls::GEOMETRY_SEGMENTS as f32)),
            0.0
        ),
        "em `Segments` o nó emite POSIÇÕES, e a placa desenha-as"
    );
}

/// ⚠️ **A bandeira de TIPO não mudou** — os outros dois leitores dela (a lei da aparência e a
/// das fontes de posições) continuam a ver o L-System como fonte de posições. A cerca é de
/// INSTÂNCIA e mora noutro canal do registo.
#[test]
fn a_bandeira_de_tipo_do_lsystem_nao_mudou() {
    let m = MotionState::new();
    let ty = NodeTypeId::of("source.lsystem");
    assert!(!m.registry.is_live_vector_source(ty));
    assert!(m.registry.has_live_vector_condition(ty));
    let em = |g: f32| move |n: &str| if n == ls::param::GEOMETRY { g } else { 0.0 };
    assert!(
        m.registry
            .emits_live_vector(ty, &em(ls::GEOMETRY_BRANCHES as f32))
    );
    assert!(
        !m.registry
            .emits_live_vector(ty, &em(ls::GEOMETRY_SEGMENTS as f32))
    );
    // O controlo: o `source.shape` é vectorial POR TIPO, com quaisquer params.
    assert!(
        m.registry
            .emits_live_vector(NodeTypeId::of("source.shape"), &|_| 0.0)
    );
}
