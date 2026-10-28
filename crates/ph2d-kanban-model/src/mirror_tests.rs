use std::collections::BTreeMap;

use super::*;
use crate::transfer::tests::{lists, me, titles, two};

fn sketch(set: &KanbanSet, a: KanbanId) -> CardRef {
    let l = lists(set, a)[0];
    CardRef {
        kanban: a,
        card: set.get(a).unwrap().cards_in(l)[0].id,
    }
}

#[test]
fn a_mirror_shows_the_original_and_an_edit_on_either_side_is_the_same() {
    let (mut set, a, b) = two();
    let of = sketch(&set, a);
    let doing = lists(&set, b)[1];
    let at = set.place_at(b, doing, 0, None).unwrap();
    let plan = mirror_card(&set, of, &at).unwrap();
    let mut h = BTreeMap::new();
    set.apply_plan(plan, &mut h, 1, &me());
    let e = set.entries_in(b, doing);
    assert!(matches!(e[0], Entry::Mirror { of: o, .. } if o == of));
    // Editar «pelo espelho» é escrever o original.
    let mut c = set.card(e[0].shows(b)).unwrap().clone();
    c.title = "Sketch v2".to_owned();
    set.apply(a, vec![KanbanOp::put(Record::Card(c))]);
    assert_eq!(titles(&set, b, doing), ["Sketch v2"]);
    assert_eq!(set.mirrors_of(of).len(), 1);
}

#[test]
fn a_mirror_is_refused_on_its_own_list_and_twice_in_one_list() {
    let (mut set, a, b) = two();
    let of = sketch(&set, a);
    let here = set.place_at(a, lists(&set, a)[0], 0, None).unwrap();
    assert!(mirror_card(&set, of, &here).is_none());
    let at = set.place_at(b, lists(&set, b)[0], 0, None).unwrap();
    let plan = mirror_card(&set, of, &at).unwrap();
    set.apply_plan(plan, &mut BTreeMap::new(), 1, &me());
    assert!(mirror_card(&set, of, &at).is_none());
}

#[test]
fn removing_or_archiving_the_original_takes_its_mirrors_away() {
    for archive in [true, false] {
        let (mut set, a, b) = two();
        let of = sketch(&set, a);
        let mut h = BTreeMap::new();
        for (k, l) in [(b, lists(&set, b)[0]), (a, lists(&set, a)[1])] {
            let at = set.place_at(k, l, 0, None).unwrap();
            let plan = mirror_card(&set, of, &at).unwrap();
            set.apply_plan(plan, &mut h, 1, &me());
        }
        assert_eq!(set.mirrors_of(of).len(), 2);
        // Arquivado à mão (sem `remove_card`): o espelho já não aparece.
        let mut c = set.card(of).unwrap().clone();
        c.archived = true;
        let mut s2 = set.clone();
        s2.apply(a, vec![KanbanOp::put(Record::Card(c))]);
        assert!(s2.entries_in(b, lists(&s2, b)[0]).is_empty());
        let plan = remove_card(&set, of, archive).unwrap();
        set.apply_plan(plan, &mut h, 2, &me());
        assert!(set.mirrors_of(of).is_empty(), "archive={archive}");
        assert!(set.entries_in(b, lists(&set, b)[0]).is_empty());
        assert!(set.undo_gesture(&mut h, 3, &me()));
        assert_eq!(
            set.mirrors_of(of).len(),
            2,
            "o desfazer devolve os espelhos"
        );
    }
}

#[test]
fn place_at_puts_a_row_at_the_index_and_past_the_end_at_the_end() {
    let (set, a, _) = two();
    let todo = lists(&set, a)[0];
    let k = set.get(a).unwrap();
    let rows: Vec<FracKey> = k.cards_in(todo).iter().map(|c| c.pos.clone()).collect();
    let p0 = set.place_at(a, todo, 0, None).unwrap().pos;
    let p1 = set.place_at(a, todo, 1, None).unwrap().pos;
    let p9 = set.place_at(a, todo, 9, None).unwrap().pos;
    assert!(p0 < rows[0]);
    assert!(rows[0] < p1 && p1 < rows[1]);
    assert!(p9 > rows[1]);
    let skip = Some(Entry::Card(k.cards_in(todo)[0].id));
    let s0 = set.place_at(a, todo, 1, skip).unwrap().pos;
    assert!(s0 > rows[1], "sem a linha que se move, o índice 1 é o fim");
}
