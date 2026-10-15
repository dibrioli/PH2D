//! Os gates do pedido «traz estes nós à vista», pela costura real (`MockPanelHost`).

use super::super::fit_tests::{grafo, layout, no, pinta};
use crate::geom::{View, card_rect};
use crate::snapshot::{
    GraphViewSnapshot, request_frame_nodes, set_current_motion_graph, take_frame_request,
};
use crate::state::MotionGraphPanelState;
use ph2d_editor_core::zones::Rect;

/// Uma fila à vista, um nó (`9`) muito abaixo dela e outro (`8`) muito à direita.
fn cena() -> GraphViewSnapshot {
    grafo(vec![
        no(1, "Grid", 0.0, 0.0, 2),
        no(2, "Output", 220.0, 0.0, 2),
        no(9, "Box", 0.0, 3000.0, 2),
        no(8, "Wind", 2400.0, 0.0, 2),
    ])
}

/// Abre o painel (o auto-enquadramento) e devolve o host e o estado, com a vista posta na fila.
fn aberto(snap: &GraphViewSnapshot) -> (ph2d_ui_testkit::MockPanelHost, MotionGraphPanelState) {
    let _ = take_frame_request();
    set_current_motion_graph(Some(snap.clone()));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::MotionGraphPanel>();
    let mut state = MotionGraphPanelState::default();
    let _ = pinta(&mut host, &mut state, layout(), snap);
    state.view.zoom = 1.0;
    state.view.pan_x = 40.0;
    state.view.pan_y = 40.0;
    (host, state)
}

fn rect_de(state: &MotionGraphPanelState, snap: &GraphViewSnapshot, id: u32) -> Rect {
    let n = snap.nodes.iter().find(|n| n.id == id).expect("nó");
    card_rect(n, &View::new(layout().motion_graph, state.view))
}

fn dentro(r: Rect, p: Rect) -> bool {
    r.x >= p.x && r.y >= p.y && r.x + r.w <= p.x + p.w && r.y + r.h <= p.y + p.h
}

#[test]
fn um_no_fora_do_painel_entra_numa_pintura_com_o_mesmo_zoom() {
    let snap = cena();
    let (mut host, mut state) = aberto(&snap);
    let painel = layout().motion_graph;
    assert!(
        !dentro(rect_de(&state, &snap, 9), painel),
        "a fixtura: o Box está fora"
    );
    request_frame_nodes(vec![9]);
    let _ = pinta(&mut host, &mut state, layout(), &snap);
    assert!(take_frame_request().is_none(), "o pedido foi consumido");
    assert!(dentro(rect_de(&state, &snap, 9), painel), "o Box entrou");
    assert!((state.view.zoom - 1.0).abs() < 1e-6, "cabia: o zoom fica");
    set_current_motion_graph(None);
}

#[test]
fn um_no_ja_dentro_nao_mexe_a_vista() {
    let snap = cena();
    let (mut host, mut state) = aberto(&snap);
    assert!(dentro(rect_de(&state, &snap, 1), layout().motion_graph));
    let antes = state.view;
    request_frame_nodes(vec![1, 2]);
    let _ = pinta(&mut host, &mut state, layout(), &snap);
    assert_eq!(state.view, antes);
    set_current_motion_graph(None);
}

/// Dois nós que não cabem ao zoom de agora: afasta só o bastante, e os dois ficam dentro.
#[test]
fn o_que_nao_cabe_afasta_ate_caber() {
    let snap = cena();
    let (mut host, mut state) = aberto(&snap);
    request_frame_nodes(vec![1, 8]);
    let _ = pinta(&mut host, &mut state, layout(), &snap);
    let painel = layout().motion_graph;
    assert!(state.view.zoom < 1.0);
    for id in [1, 8] {
        assert!(dentro(rect_de(&state, &snap, id), painel), "{id} dentro");
    }
    set_current_motion_graph(None);
}

/// O pedido feito ANTES de o grafo chegar ao painel (o 1.º quadro da cena) espera por ele, e
/// cumpre-se depois do auto-enquadramento.
#[test]
fn um_pedido_antes_do_grafo_espera_por_ele() {
    let snap = cena();
    let _ = take_frame_request();
    let vazio = grafo(Vec::new());
    set_current_motion_graph(Some(vazio.clone()));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::MotionGraphPanel>();
    let mut state = MotionGraphPanelState::default();
    request_frame_nodes(vec![9]);
    let _ = pinta(&mut host, &mut state, layout(), &vazio);
    set_current_motion_graph(Some(snap.clone()));
    let _ = pinta(&mut host, &mut state, layout(), &snap);
    assert!(
        dentro(rect_de(&state, &snap, 9), layout().motion_graph),
        "o Box entrou"
    );
    set_current_motion_graph(None);
}
