//! ⭐ J4 — a referência do elemento Kanban viaja no id da sessão e mora no id local de cada par.

use super::super::Replica;
use super::KANBAN_WAITING;
use ph2d_board_model::{
    Board, BoardKanban, BoardOp, BoardSet, Element, ElementId, KanbanId, Rgba, Style,
};

fn put_kanban(b: &mut Board, k: KanbanId) -> ElementId {
    let bk = BoardKanban::new(k, Style::new(None, None, Rgba([0, 0, 0, 255])));
    let el = Element::new_kanban(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        bk,
        [0.0, 0.0, 960.0, 540.0],
    );
    let id = el.id;
    let _ = BoardOp::Put(el).apply(&mut b.doc);
    id
}

fn kanban(b: &Board, id: ElementId) -> Option<KanbanId> {
    b.doc.get(id).and_then(|e| e.kanban().map(|k| k.kanban))
}

#[test]
fn the_kanban_reference_travels_as_the_session_id_and_lives_as_the_local_one() {
    let mut set = BoardSet::default();
    let id = set.create("Quadro".into());
    let mut hb = set.get_mut(id).unwrap().clone();
    let a = put_kanban(&mut hb, KanbanId(1));
    let mut host = Replica::host(1, &mut hb, 10);
    host.map_kanbans(&mut hb, &[(KanbanId(1), KanbanId(1))]);

    // A Bia entra: antes do Kanban chegar, a referência espera no marcador — nunca no id 1 DELA.
    let (mut g, mut gb) = Replica::join(2, &host.snapshot(&hb), 11).unwrap();
    g.kanbans_on();
    g.kanbans_arrived(&mut gb);
    assert_eq!(kanban(&gb, a), Some(KANBAN_WAITING));
    g.map_kanbans(&mut gb, &[(KanbanId(1), KanbanId(5))]);
    assert_eq!(kanban(&gb, a), Some(KanbanId(5)), "o id local da Bia");
    assert!(g.local(&mut gb).is_empty(), "traduzir não é uma escrita");

    // A Bia põe o SEU Kanban 8 (partilhado com o id de sessão W): o anfitrião recebe W e mora no 3.
    let w = KanbanId((1 << 63) | 42);
    let b = put_kanban(&mut gb, KanbanId(8));
    g.map_kanbans(&mut gb, &[(w, KanbanId(8))]);
    let ws = g.local(&mut gb);
    host.receive(&mut hb, &ws);
    assert_eq!(
        kanban(&hb, b),
        Some(KANBAN_WAITING),
        "ainda não chegou ao anfitrião"
    );
    host.map_kanbans(&mut hb, &[(w, KanbanId(3))]);
    assert_eq!(kanban(&hb, b), Some(KanbanId(3)));

    // O retrato leva os ids da sessão (quem entra a seguir traduz para os seus).
    let (_, raw) = Replica::join(3, &host.snapshot(&hb), 12).unwrap();
    assert_eq!(kanban(&raw, a), Some(KanbanId(1)));
    assert_eq!(kanban(&raw, b), Some(w));
    // E o que o anfitrião reescreve sai no id da sessão.
    let mut el = hb.doc.get(b).unwrap().clone();
    el.kanban_mut().unwrap().kanban = KanbanId(1);
    let _ = BoardOp::Put(el).apply(&mut hb.doc);
    let out = host.local(&mut hb);
    g.receive(&mut gb, &out);
    assert_eq!(kanban(&gb, b), Some(KanbanId(5)), "1 da sessão = 5 da Bia");
}
