use super::*;
use crate::test_support::{card, card_of, content, put, sample};
use crate::{Background, FracKey, Hue, Kanban, RecordId};

#[test]
fn every_op_returns_the_op_that_undoes_it() {
    let mut k = sample();
    let before = k.clone();
    let todo = k.lists_in_order()[0].id;
    let first = k.cards_in(todo)[0].id;
    let mut renamed = card_of(&k, first);
    renamed.title = "Sketch v2".to_owned();
    let mut h = k.header.clone();
    h.background = Background::Color(Hue::Pink);
    let new_card = card(&k, todo, "Color");
    let ops = vec![
        KanbanOp::put(Record::card(renamed)),
        KanbanOp::Delete(RecordId::List(k.lists_in_order()[2].id)),
        KanbanOp::Header(h),
        new_card,
    ];
    let inverse = apply_batch(&mut k, ops);
    assert_eq!(inverse.len(), 4);
    assert_ne!(k.lists_in_order().len(), before.lists_in_order().len());
    apply_batch(&mut k, inverse);
    // O conteúdo volta; só as versões (e a lápide do cartão criado) andaram.
    assert_eq!(content(&k), content(&before));
    assert_eq!(k.header.background, before.header.background);
}

#[test]
fn the_same_content_is_not_a_change() {
    let mut k = sample();
    let todo = k.lists_in_order()[0].id;
    let c = k.cards_in(todo)[0].clone();
    assert_eq!(KanbanOp::put(Record::card(c)).apply(&mut k), None);
    assert_eq!(KanbanOp::Header(k.header.clone()).apply(&mut k), None);
    assert_eq!(
        KanbanOp::Delete(RecordId::Card(CardId(999))).apply(&mut k),
        None
    );
}

#[test]
fn a_delete_is_a_tombstone_and_its_inverse_revives_it() {
    let mut k = sample();
    let todo = k.lists_in_order()[0].id;
    let id = k.cards_in(todo)[0].id;
    let undo = KanbanOp::Delete(RecordId::Card(id))
        .apply(&mut k)
        .expect("apagou");
    assert!(k.cards[&id].meta.deleted, "fica no mapa como lápide");
    assert!(KanbanOp::Delete(RecordId::Card(id)).apply(&mut k).is_none());
    let v = k.cards[&id].meta.version;
    undo.apply(&mut k);
    assert!(!k.cards[&id].meta.deleted);
    assert!(k.cards[&id].meta.version > v, "a versão só sobe");
}

/// ⭐ Mover um cartão para entre dois vizinhos é UMA escrita — ninguém é renumerado.
#[test]
fn reordering_writes_one_record_and_renumbers_nobody() {
    let mut k = sample();
    let todo = k.lists_in_order()[0].id;
    for t in ["Color", "Light", "Export"] {
        let op = card(&k, todo, t);
        put(&mut k, op);
    }
    let order: Vec<_> = k.cards_in(todo).iter().map(|c| c.id).collect();
    let others: Vec<_> = order[..4].iter().map(|&id| card_of(&k, id)).collect();
    // O último vai para entre o 1.º e o 2.º.
    let mut moved = card_of(&k, order[4]);
    moved.pos = FracKey::between(Some(&others[0].pos), Some(&others[1].pos));
    let inverse = apply_batch(&mut k, vec![KanbanOp::put(Record::card(moved))]);
    assert_eq!(inverse.len(), 1);
    for c in &others {
        assert_eq!(&card_of(&k, c.id), c, "vizinho intocado");
    }
    let now: Vec<_> = k.cards_in(todo).iter().map(|c| c.id).collect();
    assert_eq!(now, [order[0], order[4], order[1], order[2], order[3]]);
}

#[test]
fn a_card_moves_between_lists_by_its_list_field() {
    let mut k = sample();
    let [todo, doing] = [k.lists_in_order()[0].id, k.lists_in_order()[1].id];
    let mut c = k.cards_in(todo)[0].clone();
    c.list = doing;
    c.pos = k.pos_after_cards(doing);
    let id = c.id;
    apply_batch(&mut k, vec![KanbanOp::put(Record::card(c))]);
    assert_eq!(k.cards_in(doing)[0].id, id);
    assert!(k.cards_in(todo).iter().all(|c| c.id != id));
}

#[test]
fn ids_are_one_space_per_kanban_and_a_put_advances_it() {
    let mut k = Kanban::new(crate::KanbanId(7), "x");
    let ids = k.mint_n(3);
    assert_eq!(ids, [1, 2, 3]);
    let op = crate::test_support::list(&k, "a");
    put(&mut k, op);
    assert_eq!(k.mint(), 2);
}
