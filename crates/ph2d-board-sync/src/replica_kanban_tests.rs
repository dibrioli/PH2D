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

/// O Kanban do elemento `el` dentro da versão `n` (a 0 = a mais velha) do quadro.
fn in_version(b: &Board, n: usize, el: ElementId) -> Option<KanbanId> {
    let v = b.team.versions.get(n)?.read().ok()?;
    kanban(&v, el)
}

/// ⭐ J7 — uma VERSÃO do quadro guardada na sessão (o histórico da equipa) leva o documento inteiro:
/// as referências dela viajam no id da sessão e moram no id local de cada par, como as do quadro —
/// restaurar noutra máquina nunca abre o Kanban ERRADO (antes: o id local de quem a guardou).
#[test]
fn j7_a_version_saved_in_the_session_keeps_each_pairs_own_kanban() {
    let mut set = BoardSet::default();
    let id = set.create("Quadro".into());
    let mut hb = set.get_mut(id).unwrap().clone();
    let a = put_kanban(&mut hb, KanbanId(1));
    hb.save_version("antes".into(), 1);
    let mut host = Replica::host(1, &mut hb, 10);
    host.map_kanbans(&mut hb, &[(KanbanId(1), KanbanId(1))]);
    let (mut g, mut gb) = Replica::join(2, &host.snapshot(&hb), 11).unwrap();
    g.kanbans_on();
    g.kanbans_arrived(&mut gb);
    assert_eq!(
        in_version(&gb, 0, a),
        Some(KANBAN_WAITING),
        "a versão do retrato espera pelo Kanban — nunca o 1 da Bia"
    );
    g.map_kanbans(&mut gb, &[(KanbanId(1), KanbanId(5))]);
    assert_eq!(in_version(&gb, 0, a), Some(KanbanId(5)), "a do retrato");
    assert!(g.local(&mut gb).is_empty(), "traduzir não é uma escrita");

    // O anfitrião guarda uma versão A MEIO: chega à Bia no id dela.
    hb.save_version("durante".into(), 2);
    let ws = host.local(&mut hb);
    g.receive(&mut gb, &ws);
    assert_eq!(
        in_version(&gb, 1, a),
        Some(KanbanId(5)),
        "a guardada a meio"
    );
    assert_eq!(
        in_version(&hb, 1, a),
        Some(KanbanId(1)),
        "a dele fica no dele"
    );

    // A Bia guarda uma: o seu Kanban 5 é o 1 do anfitrião; um Kanban dela FORA da sala (9) nunca
    // chega como o 9 de outra pessoa.
    let b = put_kanban(&mut gb, KanbanId(9));
    gb.save_version("da Bia".into(), 3);
    let ws = g.local(&mut gb);
    host.receive(&mut hb, &ws);
    assert_eq!(
        in_version(&gb, 2, b),
        Some(KanbanId(9)),
        "a dela fica no dela"
    );
    assert_eq!(in_version(&hb, 2, a), Some(KanbanId(1)));
    assert_eq!(in_version(&hb, 2, b), Some(KANBAN_WAITING));
}

#[test]
fn r2_a_reference_to_a_deleted_kanban_leaves_as_waiting_never_as_the_local_id() {
    let mut ids = super::KanbanIds::default();
    ids.local.insert(KanbanId(1 << 63 | 5), KanbanId(7));
    ids.wire.insert(KanbanId(7), KanbanId(1 << 63 | 5));
    assert_eq!(
        ids.to_wire(ElementId(1), KanbanId(7)),
        KanbanId(1 << 63 | 5)
    );
    assert_eq!(ids.to_wire(ElementId(1), KanbanId(9)), KANBAN_WAITING);
}
