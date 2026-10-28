//! ⭐ O gate «duas vistas, um documento» (plano §7) no modelo: a aba e o elemento do Quadro são duas
//! [`KanbanView`] sobre o MESMO [`KanbanSet`].

use super::*;
use crate::test_support::{card, card_of, sample};
use crate::{History, KanbanOp, Record};

fn two_views() -> (KanbanSet, KanbanView, KanbanView) {
    let mut set = KanbanSet::default();
    let id = set.create("x");
    let mut k = sample();
    k.id = id;
    set.insert(k);
    (set, KanbanView::of(id), KanbanView::of(id))
}

/// Uma edição feita por QUALQUER uma das vistas aparece igual na outra, sem passo de sincronizar.
#[test]
fn both_views_show_the_same_state_after_every_op() {
    let (mut set, tab, canvas) = two_views();
    let id = tab.kanban;
    let mut h = History::default();
    let todo = set.get(id).unwrap().lists_in_order()[0].id;
    let doing = set.get(id).unwrap().lists_in_order()[1].id;
    let k = set.get(id).unwrap();
    let mut moved = card_of(k, k.cards_in(todo)[0].id);
    moved.list = doing;
    moved.pos = k.pos_after_cards(doing);
    let gestures: Vec<Vec<KanbanOp>> = vec![
        vec![card(k, todo, "from the tab")],
        vec![KanbanOp::Put(Record::Card(moved))],
    ];
    let mut tick = 0;
    for ops in gestures {
        tick += 1;
        h.apply(set.doc_mut(id).unwrap(), ops, tick);
        assert_eq!(tab.shown(&set), canvas.shown(&set));
    }
    for _ in 0..2 {
        tick += 1;
        assert!(h.undo(set.doc_mut(id).unwrap(), tick));
        assert_eq!(tab.shown(&set), canvas.shown(&set));
    }
    assert_eq!(tab.shown(&set).unwrap().lists[0].cards.len(), 2);
}

/// Recolher uma lista é da VISTA: a outra continua aberta e o documento não mexe um byte.
#[test]
fn collapsing_a_list_is_the_views_and_never_the_documents() {
    let (set, mut tab, canvas) = two_views();
    let bytes = set.to_bytes().unwrap();
    let todo = set.get(tab.kanban).unwrap().lists_in_order()[0].id;
    tab.toggle_collapsed(todo);
    assert!(tab.shown(&set).unwrap().lists[0].collapsed);
    assert!(!canvas.shown(&set).unwrap().lists[0].collapsed);
    assert_eq!(set.to_bytes().unwrap(), bytes);
    tab.toggle_collapsed(todo);
    assert_eq!(tab.shown(&set), canvas.shown(&set));
}

#[test]
fn a_view_of_a_removed_kanban_shows_nothing() {
    let (mut set, tab, _) = two_views();
    set.remove(tab.kanban);
    assert_eq!(tab.shown(&set), None);
}

#[test]
fn archived_and_deleted_records_are_not_shown() {
    let (mut set, tab, _) = two_views();
    let id = tab.kanban;
    let k = set.get(id).unwrap();
    let todo = k.lists_in_order()[0].id;
    let mut a = card_of(k, k.cards_in(todo)[0].id);
    a.archived = true;
    let b = k.cards_in(todo)[1].id;
    set.apply(
        id,
        vec![
            KanbanOp::Put(Record::Card(a)),
            KanbanOp::Delete(crate::RecordId::Card(b)),
        ],
    );
    assert!(tab.shown(&set).unwrap().lists[0].cards.is_empty());
}

#[test]
fn the_view_state_round_trips_for_whoever_saves_it() {
    let (set, mut tab, _) = two_views();
    let todo = set.get(tab.kanban).unwrap().lists_in_order()[0].id;
    tab.toggle_collapsed(todo);
    tab.scroll_x = 120.0;
    tab.scroll_y.insert(todo, 40.0);
    let back: KanbanView = postcard::from_bytes(&postcard::to_allocvec(&tab).unwrap()).unwrap();
    assert_eq!(back, tab);
}
