//! **Um palco a tocar não guia o painel do editor, nem é guiado por ele** (ADR-0186 §Emenda W3, «A
//! fronteira»). As portas que o palco partilha com o editor escrevem os efeitos do painel — o pedido
//! de selecção, as edições encaminhadas aos params — e o quadro lê a mão do artista (a selecção, o
//! param em arrasto). Os testes mudam o painel do editor A CADA passo enquanto o palco toca.

use super::tocar::{Modo, TocadorDoMotion};
use crate::motion_state::MotionState;
use ph2d_nodegraph::graph::NodeId;
use ph2d_panel_motion_graph::{
    GraphIntent, pending_graph_selection, push_intent, request_graph_selection,
    set_graph_param_scrub, set_graph_selection,
};

/// Um pedido de selecção que o EDITOR deixou ao painel; o palco não o pode trocar.
const DO_EDITOR: u32 = 4242;

/// A porta do editor, como o `dispatch` a corre.
fn quadro(m: &mut MotionState) {
    let mut toasts = ph2d_editor_core::ToastQueue::default();
    let store = ph2d_editor_core::interaction::WidgetStore::default();
    let mut split = ph2d_editor_core::screens::layout::CenterSplit::None;
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    crate::motion_bridge::apply_graph_intents(m, &mut ph, &mut toasts, &mut split);
    crate::motion_bridge::params::apply_param_edits_for_tests(m, &store, &mut toasts);
    crate::motion_bridge::quadro::o_quadro(
        m,
        None,
        &ph,
        1.0 / 60.0,
        crate::motion_bridge::quadro::MaoDoEditor::do_painel(),
    );
}

struct Sessao {
    ini: Vec<u8>,
    cargas: Vec<Vec<u8>>,
    forma: NodeId,
    /// Quantos gestos da sessão pediram uma selecção ao painel (o controlo: as portas exercem-se).
    pedidos: usize,
}

/// Copiar e colar, duplicar, agrupar, entrar e sair do grupo, desagrupar — as portas que pedem uma
/// selecção ao painel —, cada gesto no seu quadro.
fn sessao() -> Sessao {
    let mut m = super::estado_de_teste();
    let forma = m.doc.graph.add_node("source.shape");
    let saida = m.doc.graph.add_node("motion.output");
    let ini = m.gravacao_arma().expect("arma");
    let mut cargas = Vec::new();
    let mut pedidos = 0;
    let mut gesto = |m: &mut MotionState, i: GraphIntent, cargas: &mut Vec<Vec<u8>>| {
        request_graph_selection(vec![DO_EDITOR]);
        push_intent(i);
        quadro(m);
        if pending_graph_selection() != Some(vec![DO_EDITOR]) {
            pedidos += 1;
        }
        cargas.extend(m.gravacao_passos());
    };
    gesto(
        &mut m,
        GraphIntent::CopySelection {
            nodes: vec![forma.0],
        },
        &mut cargas,
    );
    gesto(&mut m, GraphIntent::Paste, &mut cargas);
    gesto(
        &mut m,
        GraphIntent::DuplicateSelection {
            nodes: vec![forma.0],
        },
        &mut cargas,
    );
    gesto(
        &mut m,
        GraphIntent::GroupSelection {
            nodes: vec![forma.0, saida.0],
        },
        &mut cargas,
    );
    let sid = m.doc.subgraphs[0].id;
    gesto(&mut m, GraphIntent::EnterSubgraph { id: sid }, &mut cargas);
    gesto(&mut m, GraphIntent::GoToLevel { level: None }, &mut cargas);
    gesto(&mut m, GraphIntent::Ungroup { id: sid }, &mut cargas);
    cargas.extend(m.gravacao_para());
    Sessao {
        ini,
        cargas,
        forma,
        pedidos,
    }
}

#[test]
fn o_palco_a_tocar_nao_troca_o_pedido_de_seleccao_do_editor() {
    let s = sessao();
    assert!(
        s.pedidos >= 5,
        "CONTROLO: na sessão as portas pedem selecção ao painel ({} de 7)",
        s.pedidos
    );
    let mut t = TocadorDoMotion::novo(&s.ini, Modo::Fiel).expect("o inicial lê-se");
    for (i, c) in s.cargas.iter().enumerate() {
        let do_editor = vec![DO_EDITOR + i as u32];
        request_graph_selection(do_editor.clone());
        t.aplica(c).expect("o passo aplica-se");
        assert_eq!(
            pending_graph_selection(),
            Some(do_editor),
            "o passo {i} do palco trocou a selecção pedida pelo editor"
        );
    }
}

/// Os nós cozidos como tomada em cada quadro do palco.
fn tomadas_por_quadro(s: &Sessao, painel: impl Fn(usize)) -> Vec<Vec<NodeId>> {
    let mut t = TocadorDoMotion::novo(&s.ini, Modo::Fiel).expect("o inicial lê-se");
    let mut out = Vec::new();
    for (i, c) in s.cargas.iter().enumerate() {
        painel(i);
        t.aplica(c).expect("o passo aplica-se");
        out.push(
            t.palco()
                .pump
                .tap_streams()
                .iter()
                .map(|(n, _)| *n)
                .collect(),
        );
    }
    set_graph_selection(Vec::new());
    set_graph_param_scrub(None);
    out
}

#[test]
fn a_mao_do_editor_nao_entra_no_quadro_do_palco() {
    use ph2d_node_motion_shape::param::{PIVOT_X, PIVOT_Y};
    let s = sessao();
    // CONTROLO: no editor, arrastar o pivô da forma pede-a como tomada.
    let mut m = super::estado_de_teste();
    let f = m.doc.graph.add_node("source.shape");
    set_graph_param_scrub(Some((f.0, PIVOT_X)));
    quadro(&mut m);
    set_graph_param_scrub(None);
    assert!(
        m.pump.tap_streams().iter().any(|(n, _)| *n == f),
        "CONTROLO: o arrasto do pivô no editor coze a forma como tomada"
    );
    let parado = tomadas_por_quadro(&s, |_| {
        set_graph_selection(Vec::new());
        set_graph_param_scrub(None);
    });
    let forma = s.forma.0;
    let a_mexer = tomadas_por_quadro(&s, |i| {
        set_graph_selection(vec![forma]);
        set_graph_param_scrub(match i % 3 {
            0 => Some((forma, PIVOT_X)),
            1 => Some((forma, PIVOT_Y)),
            _ => None,
        });
    });
    assert_eq!(
        parado, a_mexer,
        "o palco cozeu tomadas pela mão do artista no editor"
    );
}

#[test]
fn o_lote_do_grafo_nao_escreve_nas_filas_do_editor() {
    let mut palco = super::estado_de_teste();
    let a = palco.doc.graph.add_node("source.shape");
    request_graph_selection(vec![DO_EDITOR]);
    let mut toasts = ph2d_editor_core::ToastQueue::default();
    let mut split = ph2d_editor_core::screens::layout::CenterSplit::None;
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    crate::motion_bridge::apply_graph_batch(
        &mut palco,
        &mut ph,
        &mut toasts,
        &mut split,
        vec![
            GraphIntent::DuplicateSelection { nodes: vec![a.0] },
            GraphIntent::SetParam {
                node: a.0,
                param: ph2d_node_motion_shape::param::PIVOT_X,
                value: 0.25,
            },
        ],
    );
    assert_eq!(
        palco.doc.graph.nodes().len(),
        2,
        "CONTROLO: o lote aplicou-se"
    );
    assert_eq!(
        pending_graph_selection(),
        Some(vec![DO_EDITOR]),
        "o lote trocou a selecção pedida pelo editor"
    );
    assert!(
        crate::drain_param_intents().is_empty(),
        "o lote deixou edições na fila de params do editor"
    );
}
