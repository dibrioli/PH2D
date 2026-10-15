//! Os gates do arrumar ao ABRIR (pedido do dono, 15/10): arrastar marca o dono, o undo desmarca,
//! abrir arruma tudo menos o dono arrumado à mão.

use super::*;
use crate::macro_track::estado_de_teste;
use ph2d_motion_doc::MotionDoc;
use ph2d_nodegraph::graph::{Edge, Pos};
use ph2d_panel_motion_graph::GraphIntent;

/// Dois objectos, cada um com uma cadeia `grid → output` largada em sítios absurdos.
fn baguncado() -> (MotionDoc, [NodeId; 2], [NodeId; 2]) {
    let mut doc = MotionDoc::new();
    let mut cadeia = |dono: u64, x: f32| {
        let a = doc.graph.add_node("motion.grid");
        let b = doc.graph.add_node("motion.output");
        doc.graph
            .connect(Edge {
                from: (a, 0),
                to: (b, 0),
                delayed: false,
            })
            .expect("fio");
        doc.graph.set_pos(
            a,
            Pos {
                x: x + 900.0,
                y: -700.0,
            },
        );
        doc.graph.set_pos(b, Pos { x, y: 1300.0 });
        doc.set_owner(a, dono);
        doc.set_owner(b, dono);
        [a, b]
    };
    let livre = cadeia(5, -333.0);
    let a_mao = cadeia(9, 4444.0);
    (doc, livre, a_mao)
}

fn lugares(m: &MotionState, nos: &[NodeId]) -> Vec<Option<Pos>> {
    nos.iter().map(|n| m.doc.graph.pos(*n)).collect()
}

fn arrasta(m: &mut MotionState, n: NodeId) {
    crate::motion_bridge::aplica_e_grava(
        m,
        &mut ph2d_core::Playhead::new(1.0 / 60.0),
        &mut ph2d_editor_core::ToastQueue::default(),
        &mut ph2d_editor_core::screens::layout::CenterSplit::None,
        vec![
            GraphIntent::BeginDrag,
            GraphIntent::MoveNodes {
                nodes: vec![n.0],
                dx: 30.0,
                dy: -12.0,
            },
            GraphIntent::EndDrag,
        ],
    );
}

#[test]
fn arrastar_um_no_marca_o_dono_e_o_desfazer_desmarca() {
    let mut m = estado_de_teste();
    let (doc, livre, a_mao) = baguncado();
    m.doc = doc;
    arrasta(&mut m, a_mao[1]);
    assert_eq!(
        m.doc.arrumados_a_mao,
        [9].into(),
        "só o dono do nó arrastado"
    );
    assert!(m.doc.to_text().contains("\noh 9\n"), "e grava-se");
    let antes = m.history.undo(&m.doc).expect("o arrasto é um passo");
    assert!(antes.arrumados_a_mao.is_empty(), "o undo leva a marca");
    assert_eq!(antes.graph.pos(livre[0]), m.doc.graph.pos(livre[0]));
}

#[test]
fn arrumar_ao_abrir_salta_o_dono_arrumado_a_mao() {
    let mut m = estado_de_teste();
    let (mut doc, livre, a_mao) = baguncado();
    doc.arrumados_a_mao.insert(9);
    m.doc = doc;
    let (livre0, a_mao0) = (lugares(&m, &livre), lugares(&m, &a_mao));
    arrumar_ao_abrir(&mut m);
    assert_ne!(lugares(&m, &livre), livre0, "o objecto livre arruma-se");
    assert_eq!(
        lugares(&m, &a_mao),
        a_mao0,
        "o arrumado à mão fica EXACTAMENTE"
    );
    assert!(!m.history.can_undo(), "abrir não é edição");
}

/// Gravar → abrir: o livre sai como o botão de arrumar o deixaria; o marcado, como foi gravado.
#[test]
fn abrir_um_projecto_arruma_menos_o_arrumado_a_mao() {
    let (mut doc, livre, a_mao) = baguncado();
    let mut botao = estado_de_teste();
    botao.doc = doc.clone();
    arrumar(&mut botao);

    let mut m = estado_de_teste();
    m.load_text(&doc.to_text()).expect("abre");
    assert_eq!(lugares(&m, &livre), lugares(&botao, &livre));
    assert_eq!(
        lugares(&m, &a_mao),
        lugares(&botao, &a_mao),
        "sem marca, tudo"
    );

    doc.arrumados_a_mao.insert(9);
    let gravado: Vec<_> = a_mao.iter().map(|n| doc.graph.pos(*n)).collect();
    let mut m = estado_de_teste();
    m.load_text(&doc.to_text()).expect("abre");
    assert_eq!(lugares(&m, &livre), lugares(&botao, &livre));
    assert_eq!(lugares(&m, &a_mao), gravado, "com marca, como foi gravado");
}
