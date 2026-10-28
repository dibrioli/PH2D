use ph2d_kanban_model::{CardRef, Due, Moment, mirror_card};

use super::*;
use crate::test_support::{DAY, NOW, add_card, add_list};

fn titles(set: &KanbanSet, k: KanbanId, l: ListId) -> Vec<String> {
    set.entries_in(k, l)
        .into_iter()
        .map(|e| set.card(e.shows(k)).unwrap().title.clone())
        .collect()
}

fn due(d: i64) -> Option<Due> {
    Some(Due {
        at: Moment(NOW.0 + d * DAY),
        done: false,
        remind_before_min: None,
    })
}

fn scene() -> (KanbanSet, KanbanId, ListId) {
    let mut set = KanbanSet::default();
    let k = set.create("K");
    let l = add_list(&mut set, k, "L");
    let rows = [
        ("delta", 3, Some(2)),
        ("Álvaro", 1, None),
        ("charlie", 4, Some(1)),
        ("bravo", 2, Some(5)),
    ];
    for (t, created, d) in rows {
        add_card(&mut set, k, l, t, Moment(created), |c| {
            c.due = d.and_then(due)
        });
    }
    (set, k, l)
}

fn sorted(by: SortBy) -> Vec<String> {
    let (mut set, k, l) = scene();
    let ops = sort_list(&set, k, l, by);
    set.apply(k, ops);
    titles(&set, k, l)
}

#[test]
fn sort_list_by_created_name_and_due() {
    assert_eq!(
        sorted(SortBy::Newest),
        ["charlie", "delta", "bravo", "Álvaro"]
    );
    assert_eq!(
        sorted(SortBy::Oldest),
        ["Álvaro", "bravo", "delta", "charlie"]
    );
    assert_eq!(
        sorted(SortBy::Name),
        ["Álvaro", "bravo", "charlie", "delta"]
    );
    assert_eq!(
        sorted(SortBy::Due),
        ["charlie", "delta", "bravo", "Álvaro"],
        "sem data no fim"
    );
}

#[test]
fn sort_rewrites_only_the_cards_that_move() {
    let (mut set, k, l) = scene();
    // delta, Álvaro, charlie, bravo → por nome: só «delta» e mais um mudam de sítio.
    let keys = sort_keys(&set, k, l, SortBy::Name);
    assert_eq!(
        keys.len(),
        2,
        "fica parada a maior subsequência já em ordem"
    );
    let ops = sort_list(&set, k, l, SortBy::Name);
    set.apply(k, ops);
    assert!(
        sort_list(&set, k, l, SortBy::Name).is_empty(),
        "ordenada: nada a escrever"
    );
}

#[test]
fn a_mirror_sorts_by_its_original() {
    let (mut set, k, l) = scene();
    let other = set.create("O");
    let ol = add_list(&mut set, other, "X");
    let zulu = add_card(&mut set, other, ol, "aardvark", Moment(9), |_| {});
    let at = set.place_at(k, l, 9, None).unwrap();
    let of = CardRef {
        kanban: other,
        card: zulu,
    };
    for (kk, ops) in mirror_card(&set, of, &at).unwrap().into_steps() {
        set.apply(kk, ops);
    }
    let ops = sort_list(&set, k, l, SortBy::Name);
    set.apply(k, ops);
    assert_eq!(
        titles(&set, k, l),
        ["aardvark", "Álvaro", "bravo", "charlie", "delta"]
    );
}
