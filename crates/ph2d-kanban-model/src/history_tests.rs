use super::*;
use crate::test_support::{card, card_of, content, sample};
use crate::{CardId, KanbanOp, Record};

fn rename(k: &Kanban, id: CardId, t: &str) -> KanbanOp {
    let mut c = card_of(k, id);
    c.title = t.to_owned();
    KanbanOp::Put(Record::Card(c))
}

#[test]
fn undo_and_redo_walk_the_gestures_back_and_forth() {
    let mut k = sample();
    let start = k.clone();
    let todo = k.lists_in_order()[0].id;
    let id = k.cards_in(todo)[0].id;
    let mut h = History::default();
    let ops = vec![rename(&k, id, "A")];
    h.apply(&mut k, ops, 1);
    let op = card(&k, todo, "New");
    h.apply(&mut k, vec![op], 2);
    let end_titles: Vec<_> = k.cards_in(todo).iter().map(|c| c.title.clone()).collect();
    assert_eq!(h.undo_at(), Some(2));

    assert!(h.undo(&mut k, 3));
    assert!(h.undo(&mut k, 4));
    assert!(!h.undo(&mut k, 5), "nada mais a desfazer");
    assert_eq!(content(&k), content(&start));

    assert!(h.redo(&mut k, 6));
    assert!(h.redo(&mut k, 7));
    let titles: Vec<_> = k.cards_in(todo).iter().map(|c| c.title.clone()).collect();
    assert_eq!(titles, end_titles);
    assert!(!h.can_redo());
}

#[test]
fn a_gesture_that_changed_nothing_is_not_a_step() {
    let mut k = sample();
    let todo = k.lists_in_order()[0].id;
    let id = k.cards_in(todo)[0].id;
    let mut h = History::default();
    let ops = vec![rename(&k, id, "Sketch")];
    h.apply(&mut k, ops, 1);
    assert!(!h.can_undo());
}

#[test]
fn a_new_gesture_clears_redo() {
    let mut k = sample();
    let todo = k.lists_in_order()[0].id;
    let id = k.cards_in(todo)[0].id;
    let mut h = History::default();
    let ops = vec![rename(&k, id, "A")];
    h.apply(&mut k, ops, 1);
    h.undo(&mut k, 2);
    assert!(h.can_redo());
    let ops = vec![rename(&k, id, "B")];
    h.apply(&mut k, ops, 3);
    assert!(!h.can_redo());
}

/// O desfazer não pisa o que outro escreveu DEPOIS do gesto (a lei do Quadro, ADR-0181 §3).
#[test]
fn undo_leaves_a_record_someone_else_changed_since() {
    let mut k = sample();
    let todo = k.lists_in_order()[0].id;
    let [a, b] = [k.cards_in(todo)[0].id, k.cards_in(todo)[1].id];
    let mut h = History::default();
    let ops = vec![rename(&k, a, "mine A"), rename(&k, b, "mine B")];
    h.apply(&mut k, ops, 1);
    // Outro (sem passar por este histórico) muda o cartão `b`.
    let theirs = vec![rename(&k, b, "theirs B")];
    apply_batch(&mut k, theirs);
    assert!(h.undo(&mut k, 2));
    assert_eq!(card_of(&k, a).title, "Sketch");
    assert_eq!(card_of(&k, b).title, "theirs B");
}

#[test]
fn the_stack_keeps_at_most_max_steps() {
    let mut k = sample();
    let todo = k.lists_in_order()[0].id;
    let id = k.cards_in(todo)[0].id;
    let mut h = History::default();
    for i in 0..MAX_STEPS + 5 {
        let ops = vec![rename(&k, id, &i.to_string())];
        h.apply(&mut k, ops, i as u64 + 1);
    }
    let mut n = 0;
    while h.undo(&mut k, 0) {
        n += 1;
    }
    assert_eq!(n, MAX_STEPS);
}
