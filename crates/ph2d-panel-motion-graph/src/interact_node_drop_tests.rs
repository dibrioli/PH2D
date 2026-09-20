//! Os gates de [`super::largada`] — ordem do dono (2026-09-19). ⚠️ **As coordenadas do cursor são
//! DERIVADAS de [`geom::card_rect`]**, nunca escritas à mão: um teste com `(295, 50)` lá dentro
//! afirma o tamanho da carta, e reprova no dia em que ela mudar por um motivo que não é a lei.

use super::*;
use crate::snapshot::{GraphEdgeView, GraphNodeView, PortView};
use crate::state::MotionGraphPanelState;
use ph2d_editor_core::zones::Rect;
use ph2d_nodegraph::port::{Clock, Dim, Domain};

const RECT: Rect = Rect::new(0.0, 0.0, 800.0, 400.0);

fn porta() -> PortView {
    PortView {
        name: "p",
        domain: Domain::Instances,
        dim: Dim::Vec2,
        clock: Clock::Frame,
    }
}

fn no(id: u32, x: f32, y: f32, kind: NodeViewKind) -> GraphNodeView {
    GraphNodeView {
        kind,
        id,
        display_name: "n".into(),
        category: ph2d_node_registry::NodeUiCategory::Utility,
        silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        x,
        y,
        inputs: vec![porta()],
        outputs: vec![porta()],
        primary_input: 0,
        readout: None,
        count: None,
        hot: false,
        is_sink: false,
        preview: None,
        bypassed: false,
        inert: false,
        thumbnail: None,
        params: Vec::new(),
        sections: Vec::new(),
    }
}

fn cena(nodes: Vec<GraphNodeView>, edges: Vec<GraphEdgeView>) -> GraphViewSnapshot {
    GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes,
        edges,
        backdrops: Vec::new(),
        probe: None,
        now: 0.0,
    }
}

fn fio(from_node: u32, to_node: u32) -> GraphEdgeView {
    GraphEdgeView {
        from_node,
        from_port: 0,
        to_node,
        to_port: 0,
        delayed: false,
        out_domain: Domain::Instances,
    }
}

fn vista() -> View {
    View::new(RECT, MotionGraphPanelState::default().view)
}

/// O centro da carta de `id`, em píxeis de ecrã — derivado, ver o cabeçalho.
fn centro(snap: &GraphViewSnapshot, id: u32) -> (f32, f32) {
    let r = geom::card_rect(
        snap.nodes.iter().find(|n| n.id == id).expect("no'"),
        &vista(),
    );
    (r.x + 0.5 * r.w, r.y + 0.5 * r.h)
}

/// ⭐⭐⭐ **LARGAR SOBRE OUTRO NÓ PEDE A TROCA** — *«Se arrastar um nó no grafo em cima de outro
/// nó, eles mudam de posição na cadeia»*.
#[test]
fn largar_sobre_outro_no_pede_a_troca() {
    let snap = cena(
        vec![
            no(1, 0.0, 0.0, NodeViewKind::Node),
            no(2, 400.0, 0.0, NodeViewKind::Node),
        ],
        Vec::new(),
    );
    assert_eq!(
        super::largada(&snap, &vista(), 1, centro(&snap, 2)),
        Some(Largada::Troca(2))
    );
    // ⛔ E sobre canvas vazio não é gesto nenhum — a largada de sempre é MOVER.
    assert_eq!(super::largada(&snap, &vista(), 1, (700.0, 380.0)), None);
}

/// ⭐⭐⭐ **LARGAR SOBRE UM FIO PEDE O SPLICE** — *«Se arrastar num nó em cima de uma conexão
/// (linha) […] ele passa a ser conectado naquela linha»*. ⚠️ O sujeito aqui é a **CARTA** e não o
/// cursor: ele fica na carta arrastada (que o segue), longe da linha.
#[test]
fn largar_sobre_um_fio_pede_o_splice() {
    // 1 ── 3, com o 2 arrastado para o meio.
    let snap = cena(
        vec![
            no(1, 0.0, 0.0, NodeViewKind::Node),
            no(2, 200.0, 0.0, NodeViewKind::Node),
            no(3, 420.0, 0.0, NodeViewKind::Node),
        ],
        vec![fio(1, 3)],
    );
    assert_eq!(
        super::largada(&snap, &vista(), 2, centro(&snap, 2)),
        Some(Largada::Fio(3, 0)),
        "a carta arrastada atravessa o fio 1->3"
    );
}

/// ⛔⛔ **O NÓ GANHA DO FIO** — uma carta largada sobre outra tem quase sempre um fio a passar por
/// baixo, e a leitura mais específica é a que o artista apontou. *Sem a ordem, trocar dois nós
/// vizinhos seria um splice no fio que os liga.*
#[test]
fn o_no_ganha_do_fio() {
    let snap = cena(
        vec![
            no(1, 0.0, 0.0, NodeViewKind::Node),
            no(2, 200.0, 0.0, NodeViewKind::Node),
            no(3, 420.0, 0.0, NodeViewKind::Node),
        ],
        vec![fio(1, 3)],
    );
    // O cursor sobre a carta do 3 — e a carta do 2 continua a atravessar o fio.
    assert_eq!(
        super::largada(&snap, &vista(), 2, centro(&snap, 3)),
        Some(Largada::Troca(3))
    );
}

/// ⛔ **Os fios do PRÓPRIO nó não contam** — enfiá-lo num fio que já sai dele é um laço, e a
/// carta arrastada está sempre por cima deles.
///
/// ⛔⛔ **A 1.ª redacção deste gate era VÁCUA, e quem o mostrou foi a prova de mutação:** ela punha
/// os vizinhos à DIREITA, e um fio que ACABA no pino de entrada da carta não a *atravessa* — logo
/// a lista de candidatos vinha vazia por geometria e o `None` não dizia nada sobre o filtro.
/// Apagar o filtro inteiro deixava-a verde. ⇒ o fio vem agora de TRÁS PARA A FRENTE (a fonte à
/// direita), que é o único desenho em que um fio do próprio nó de facto cruza a carta dele.
///
/// ⚠️ **E a metade (B) é o CONTROLO POSITIVO do desenho:** o MESMO traçado, com o fio a pertencer
/// a outro, tem de dar `Some`. *Sem ela, um `None` por geometria e um `None` por lei leem-se
/// igual.*
#[test]
fn os_fios_do_proprio_no_nao_contam() {
    // (A) O fio ACABA no nó arrastado, vindo de trás: ele atravessa a carta dele.
    let snap = cena(
        vec![
            no(1, 420.0, 0.0, NodeViewKind::Node),
            no(2, 200.0, 0.0, NodeViewKind::Node),
        ],
        vec![fio(1, 2)],
    );
    assert_eq!(super::largada(&snap, &vista(), 2, centro(&snap, 2)), None);

    // (B) O CONTROLO: o mesmo traçado, com o fio a pertencer a OUTRO nó.
    let snap = cena(
        vec![
            no(1, 420.0, 0.0, NodeViewKind::Node),
            no(2, 200.0, 0.0, NodeViewKind::Node),
            no(3, 0.0, 0.0, NodeViewKind::Node),
        ],
        vec![fio(1, 3)],
    );
    assert_eq!(
        super::largada(&snap, &vista(), 2, centro(&snap, 2)),
        Some(Largada::Fio(3, 0)),
        "o desenho PRODUZ uma travessia — senao o `None` de (A) nao diz nada"
    );
}

/// ⛔ **Um cartão dobrado não é sujeito nem alvo** — as portas dele são derivadas dos membros, e
/// um fantasma não vive neste nível.
#[test]
fn um_cartao_dobrado_nao_e_sujeito_nem_alvo() {
    let snap = cena(
        vec![
            no(1, 0.0, 0.0, NodeViewKind::Subgraph),
            no(2, 400.0, 0.0, NodeViewKind::Node),
        ],
        Vec::new(),
    );
    assert_eq!(super::largada(&snap, &vista(), 1, centro(&snap, 2)), None);
    let snap = cena(
        vec![
            no(1, 0.0, 0.0, NodeViewKind::Node),
            no(2, 400.0, 0.0, NodeViewKind::Subgraph),
        ],
        Vec::new(),
    );
    assert_eq!(super::largada(&snap, &vista(), 1, centro(&snap, 2)), None);
}

/// ⭐⭐⭐ **E O GESTO REAL CHEGA AO PEDIDO** — a metade que a régua de [`super::largada`] não pode
/// afirmar.
///
/// ⛔⛔ **Um gate que chama a função em vez de percorrer a ROTA afirma que a peça certa existe,
/// nunca que o gesto a usa** — a lei que esta casa já pagou várias vezes. Aqui o arrasto entra
/// pelo `apply_gesture` (press → move → release sobre a carta do outro nó) e o que se mede são as
/// INTENÇÕES que saem, na ordem em que saem.
///
/// ⚠️ **A ordem é lei:** o `SwapInChain` tem de vir ANTES do `EndDrag`, senão o parênteses de undo
/// fecha primeiro e o artista precisa de **dois** Ctrl+Z para desfazer **um** gesto.
#[test]
fn o_arrasto_real_pede_a_troca_dentro_do_parenteses_do_undo() {
    use super::super::{GesturePhase, GraphHitKind, apply_gesture};
    use crate::interact::tests::gesture;
    use crate::snapshot::{GraphIntent, drain_intents};

    let snap = cena(
        vec![
            no(1, 0.0, 0.0, NodeViewKind::Node),
            no(2, 400.0, 0.0, NodeViewKind::Node),
        ],
        Vec::new(),
    );
    let _ = drain_intents();
    let mut st = MotionGraphPanelState::default();
    let alvo = centro(&snap, 2);
    let pega = GraphHitKind::Node { node: 1 };
    for (fase, x, y) in [
        (GesturePhase::Begin, 20.0, 20.0),
        (GesturePhase::Update, alvo.0, alvo.1),
        (GesturePhase::End, alvo.0, alvo.1),
    ] {
        apply_gesture(&mut st, gesture(pega, fase, x, y), RECT, RECT, &snap);
    }
    let saiu = drain_intents();
    let i_troca = saiu
        .iter()
        .position(|i| matches!(i, GraphIntent::SwapInChain { a: 1, b: 2, .. }))
        .expect("o arrasto tem de PEDIR a troca");
    let i_fim = saiu
        .iter()
        .position(|i| matches!(i, GraphIntent::EndDrag))
        .expect("e o parenteses tem de fechar");
    assert!(
        i_troca < i_fim,
        "a troca vem DENTRO do parenteses do undo: {saiu:?}"
    );
}

/// ⭐⭐⭐ **E O ARRASTO REAL PEDE O SPLICE** — a outra metade da rota, e ela **nasceu de uma
/// mutação que SOBREVIVEU**: apagar o braço `Fio` de [`super::pedir`] deixava os cinco gates de
/// [`super::largada`] verdes, porque nenhum deles percorria a rota. *Um gate sobre a LEI não
/// afirma nada sobre quem a chama.*
#[test]
fn o_arrasto_real_pede_o_splice_dentro_do_parenteses_do_undo() {
    use super::super::{GesturePhase, GraphHitKind, apply_gesture};
    use crate::interact::tests::gesture;
    use crate::snapshot::{GraphIntent, drain_intents};

    // 1 ── 3, com o 2 no meio: a carta dele atravessa o fio.
    let snap = cena(
        vec![
            no(1, 0.0, 0.0, NodeViewKind::Node),
            no(2, 200.0, 0.0, NodeViewKind::Node),
            no(3, 420.0, 0.0, NodeViewKind::Node),
        ],
        vec![fio(1, 3)],
    );
    let _ = drain_intents();
    let mut st = MotionGraphPanelState::default();
    let dentro = centro(&snap, 2);
    let pega = GraphHitKind::Node { node: 2 };
    for (fase, x, y) in [
        (GesturePhase::Begin, dentro.0 - 8.0, dentro.1),
        (GesturePhase::Update, dentro.0, dentro.1),
        (GesturePhase::End, dentro.0, dentro.1),
    ] {
        apply_gesture(&mut st, gesture(pega, fase, x, y), RECT, RECT, &snap);
    }
    let saiu = drain_intents();
    let i_splice = saiu
        .iter()
        .position(|i| {
            matches!(
                i,
                GraphIntent::SpliceExistingIntoWire {
                    node: 2,
                    to_node: 3,
                    to_port: 0
                }
            )
        })
        .expect("o arrasto sobre um fio tem de PEDIR o splice");
    let i_fim = saiu
        .iter()
        .position(|i| matches!(i, GraphIntent::EndDrag))
        .expect("e o parenteses tem de fechar");
    assert!(
        i_splice < i_fim,
        "o splice vem DENTRO do parenteses: {saiu:?}"
    );
}
