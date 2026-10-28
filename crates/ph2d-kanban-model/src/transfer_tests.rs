use std::collections::BTreeMap;

use super::*;
use crate::test_support::{card, checklist, comment, list};
use crate::{
    Actor, Attachment, AttachmentId, CheckItem, CheckItemId, FieldKind, FieldValue, History,
    LabelColor, Link, ListRef, Member, archive_all_cards, copy_list, mirror_card, move_all_cards,
    move_list,
};

pub(crate) fn me() -> Actor {
    Actor {
        by: Member("Enio".to_owned()),
        at: Moment(50),
    }
}

/// Dois Kanbans «A» e «B», cada um com «To Do» e «Doing»; «A/To Do» tem «Sketch» e «Ink».
pub(crate) fn two() -> (KanbanSet, KanbanId, KanbanId) {
    let mut set = KanbanSet::default();
    let (a, b) = (set.create("A"), set.create("B"));
    for id in [a, b] {
        for t in ["To Do", "Doing"] {
            let op = list(set.get(id).unwrap(), t);
            set.apply(id, vec![op]);
        }
    }
    let todo = lists(&set, a)[0];
    for t in ["Sketch", "Ink"] {
        let op = card(set.get(a).unwrap(), todo, t);
        set.apply(a, vec![op]);
    }
    (set, a, b)
}

pub(crate) fn lists(set: &KanbanSet, k: KanbanId) -> Vec<ListId> {
    let k = set.get(k).unwrap();
    k.lists_in_order().iter().map(|l| l.id).collect()
}

pub(crate) fn titles(set: &KanbanSet, k: KanbanId, l: ListId) -> Vec<String> {
    set.entries_in(k, l)
        .into_iter()
        .map(|e| set.card(e.shows(k)).unwrap().title.clone())
        .collect()
}

fn first(set: &KanbanSet, k: KanbanId) -> CardRef {
    let l = lists(set, k)[0];
    CardRef {
        kanban: k,
        card: set.get(k).unwrap().cards_in(l)[0].id,
    }
}

/// «Sketch» ganha etiqueta, campo, checklist com item, anexo e comentário.
fn dress(set: &mut KanbanSet, r: CardRef) {
    let k = set.get(r.kanban).unwrap();
    let ids = k.mint_n(2);
    let (label, field) = (LabelId(ids[0]), FieldId(ids[1]));
    let mut c = k.cards[&r.card].clone();
    c.labels = vec![label];
    c.members = vec![Member("Ana".to_owned())];
    c.fields = BTreeMap::from([(field, FieldValue::Text("alta".to_owned()))]);
    c.description = "**cor** final".to_owned();
    let ops = vec![
        KanbanOp::put(Record::Label(Label {
            id: label,
            name: "Bug".to_owned(),
            color: Some(LabelColor {
                hue: crate::Hue::Red,
                shade: crate::Shade::Normal,
            }),
            meta: Meta::default(),
        })),
        KanbanOp::put(Record::Field(FieldDef {
            id: field,
            name: "Prioridade".to_owned(),
            kind: FieldKind::Text,
            on_front: true,
            pos: FracKey::between(None, None),
            meta: Meta::default(),
        })),
        KanbanOp::put(Record::Card(c)),
    ];
    set.apply(r.kanban, ops);
    let op = checklist(set.get(r.kanban).unwrap(), r.card);
    set.apply(r.kanban, vec![op]);
    let k = set.get(r.kanban).unwrap();
    let cl = *k.checklists.keys().next().unwrap();
    let ids = k.mint_n(2);
    let ops = vec![
        KanbanOp::put(Record::CheckItem(CheckItem {
            id: CheckItemId(ids[0]),
            checklist: cl,
            pos: FracKey::between(None, None),
            text: "lineart".to_owned(),
            done: true,
            member: None,
            due: None,
            meta: Meta::default(),
        })),
        KanbanOp::put(Record::Attachment(Attachment {
            id: AttachmentId(ids[1]),
            card: r.card,
            pos: FracKey::between(None, None),
            name: "ref".to_owned(),
            link: Link::Url("https://x".to_owned()),
            at: Moment(3),
            meta: Meta::default(),
        })),
    ];
    set.apply(r.kanban, ops);
    let op = comment(set.get(r.kanban).unwrap(), r.card, "Ana", "ok");
    set.apply(r.kanban, vec![op]);
}

fn apply(set: &mut KanbanSet, h: &mut BTreeMap<KanbanId, History>, plan: Plan, at: u64) {
    set.apply_plan(plan, h, at, &me());
}

#[test]
fn copy_card_to_another_kanban_maps_labels_fields_and_children() {
    let (mut set, a, b) = two();
    let src = first(&set, a);
    dress(&mut set, src);
    let to = set.place_at(b, lists(&set, b)[1], 0, None).unwrap();
    let plan = copy_card(&set, src, &to, CopyKeep::default(), "Sketch (2)", Moment(9)).unwrap();
    assert_eq!(
        plan.kanbans().collect::<Vec<_>>(),
        vec![b],
        "a origem não muda"
    );
    let mut h = BTreeMap::new();
    apply(&mut set, &mut h, plan, 1);
    let kb = set.get(b).unwrap();
    let c = kb.cards_in(to.list)[0];
    assert_eq!((c.title.as_str(), c.created), ("Sketch (2)", Moment(9)));
    assert_eq!(c.description, "**cor** final");
    assert_eq!(
        kb.labels[&c.labels[0]].name, "Bug",
        "a etiqueta nasceu no destino"
    );
    let (f, v) = c.fields.iter().next().unwrap();
    assert_eq!(
        (kb.fields[f].name.as_str(), v),
        ("Prioridade", &FieldValue::Text("alta".to_owned()))
    );
    let cl = kb.checklists.values().find(|x| x.card == c.id).unwrap();
    let it = kb.items.values().find(|i| i.checklist == cl.id).unwrap();
    assert!(it.done && it.text == "lineart");
    assert_eq!(
        kb.attachments.values().filter(|x| x.card == c.id).count(),
        1
    );
    assert_eq!(kb.comments.values().filter(|x| x.card == c.id).count(), 1);
    // Sem comentários nem membros: o «Manter…» desmarcado.
    let keep = CopyKeep {
        comments: false,
        members: false,
        ..CopyKeep::default()
    };
    let plan = copy_card(&set, src, &to, keep, "bare", Moment(9)).unwrap();
    apply(&mut set, &mut h, plan, 2);
    let kb = set.get(b).unwrap();
    let c = kb.cards.values().find(|c| c.title == "bare").unwrap();
    assert!(c.members.is_empty());
    assert_eq!(kb.comments.values().filter(|x| x.card == c.id).count(), 0);
    assert_eq!(kb.labels.len(), 1, "a 2.ª cópia reusa a etiqueta que casa");
}

#[test]
fn move_card_across_kanbans_is_one_gesture_to_undo_and_redo() {
    let (mut set, a, b) = two();
    let src = first(&set, a);
    dress(&mut set, src);
    let start = set.clone();
    let to = set.place_at(b, lists(&set, b)[0], 0, None).unwrap();
    let plan = move_card(&set, src, &to).unwrap();
    let mut h = BTreeMap::new();
    apply(&mut set, &mut h, plan, 7);
    assert_eq!(titles(&set, a, lists(&set, a)[0]), ["Ink"]);
    assert_eq!(titles(&set, b, to.list), ["Sketch"]);
    let moved = set.get(b).unwrap().cards_in(to.list)[0].clone();
    assert_eq!(moved.members, vec![Member("Ana".to_owned())]);
    assert!(set.undo_gesture(&mut h, 8, &me()));
    assert_eq!(titles(&set, a, lists(&set, a)[0]), ["Sketch", "Ink"]);
    assert!(
        titles(&set, b, to.list).is_empty(),
        "um Ctrl+Z desfaz os dois lados"
    );
    let k0 = start.get(a).unwrap();
    let k1 = set.get(a).unwrap();
    for (id, c) in &k0.checklists {
        assert!(!k1.checklists[id].meta.deleted && c.title == k1.checklists[id].title);
    }
    assert!(!set.undo_gesture(&mut h, 9, &me()), "nada mais");
    assert!(set.redo_gesture(&mut h, 10, &me()));
    assert_eq!(titles(&set, b, to.list), ["Sketch"]);
    assert_eq!(titles(&set, a, lists(&set, a)[0]), ["Ink"]);
}

#[test]
fn move_card_in_the_same_kanban_is_one_put() {
    let (set, a, _) = two();
    let src = first(&set, a);
    let doing = lists(&set, a)[1];
    let to = set.place_at(a, doing, 0, None).unwrap();
    let plan = move_card(&set, src, &to).unwrap();
    assert_eq!(plan.ops(a).len(), 1);
}

#[test]
fn moving_the_original_keeps_its_mirrors_pointing_at_it() {
    let (mut set, a, b) = two();
    let src = first(&set, a);
    let mut h = BTreeMap::new();
    let doing_a = lists(&set, a)[1];
    let at = set.place_at(a, doing_a, 0, None).unwrap();
    let plan = mirror_card(&set, src, &at).unwrap();
    apply(&mut set, &mut h, plan, 1);
    let to = set.place_at(b, lists(&set, b)[0], 0, None).unwrap();
    let plan = move_card(&set, src, &to).unwrap();
    apply(&mut set, &mut h, plan, 2);
    assert_eq!(
        titles(&set, a, doing_a),
        ["Sketch"],
        "o espelho segue o cartão"
    );
    let e = set.entries_in(a, doing_a)[0];
    assert_eq!(e.shows(a).kanban, b);
}

#[test]
fn a_card_from_a_template_is_a_plain_card_and_only_templates_qualify() {
    let (mut set, a, _) = two();
    let src = first(&set, a);
    let to = set.place_at(a, lists(&set, a)[1], 0, None).unwrap();
    assert!(card_from_template(&set, src, &to, CopyKeep::template(), "x", Moment(1)).is_none());
    let mut c = set.card(src).unwrap().clone();
    c.template = true;
    set.apply(a, vec![KanbanOp::put(Record::Card(c))]);
    let op = comment(set.get(a).unwrap(), src.card, "Ana", "nota");
    set.apply(a, vec![op]);
    let plan =
        card_from_template(&set, src, &to, CopyKeep::template(), "Shot 14", Moment(1)).unwrap();
    let mut h = BTreeMap::new();
    apply(&mut set, &mut h, plan, 1);
    let k = set.get(a).unwrap();
    let n = k.cards_in(to.list)[0];
    assert_eq!(n.title, "Shot 14");
    assert!(!n.template);
    assert_eq!(k.comments.values().filter(|x| x.card == n.id).count(), 0);
}

#[test]
fn move_list_to_another_kanban_carries_cards_archived_and_mirrors() {
    let (mut set, a, b) = two();
    let todo = lists(&set, a)[0];
    let doing = lists(&set, a)[1];
    let ink = set.get(a).unwrap().cards_in(todo)[1].clone();
    set.apply(
        a,
        vec![KanbanOp::put(Record::Card(Card {
            archived: true,
            ..ink
        }))],
    );
    let op = card(set.get(a).unwrap(), doing, "Color");
    set.apply(a, vec![op]);
    let color = CardRef {
        kanban: a,
        card: set.get(a).unwrap().cards_in(doing)[0].id,
    };
    let mut h = BTreeMap::new();
    let at = set.place_at(a, todo, 1, None).unwrap();
    let plan = mirror_card(&set, color, &at).unwrap();
    apply(&mut set, &mut h, plan, 1);
    assert_eq!(titles(&set, a, todo), ["Sketch", "Color"]);
    let pos = set.get(b).unwrap().pos_after_lists();
    let src = ListRef {
        kanban: a,
        list: todo,
    };
    let plan = move_list(&set, src, b, pos).unwrap();
    apply(&mut set, &mut h, plan, 2);
    assert_eq!(lists(&set, a), [doing]);
    let nl = *lists(&set, b).last().unwrap();
    assert_eq!(titles(&set, b, nl), ["Sketch", "Color"]);
    let kb = set.get(b).unwrap();
    assert!(
        kb.cards
            .values()
            .any(|c| c.list == nl && c.archived && c.title == "Ink")
    );
    assert!(set.undo_gesture(&mut h, 3, &me()));
    assert_eq!(lists(&set, a), [todo, doing]);
    assert_eq!(titles(&set, a, todo), ["Sketch", "Color"]);
    assert_eq!(lists(&set, b).len(), 2);
}

#[test]
fn copy_list_and_move_all_cards_and_archive_all_cards() {
    let (mut set, a, b) = two();
    let [todo, doing] = [lists(&set, a)[0], lists(&set, a)[1]];
    let mut h = BTreeMap::new();
    let pos = set.get(b).unwrap().pos_after_lists();
    let plan = copy_list(
        &set,
        ListRef {
            kanban: a,
            list: todo,
        },
        b,
        pos,
        "To Do 2",
        Moment(4),
    )
    .unwrap();
    apply(&mut set, &mut h, plan, 1);
    let nl = *lists(&set, b).last().unwrap();
    assert_eq!(set.get(b).unwrap().lists[&nl].title, "To Do 2");
    assert_eq!(titles(&set, b, nl), ["Sketch", "Ink"]);
    assert_eq!(titles(&set, a, todo), ["Sketch", "Ink"], "copiar não tira");
    let op = card(set.get(a).unwrap(), doing, "Old");
    set.apply(a, vec![op]);
    let (s, t) = (
        ListRef {
            kanban: a,
            list: todo,
        },
        ListRef {
            kanban: a,
            list: doing,
        },
    );
    assert!(move_all_cards(&set, s, s).is_none());
    let plan = move_all_cards(&set, s, t).unwrap();
    apply(&mut set, &mut h, plan, 2);
    assert_eq!(
        titles(&set, a, doing),
        ["Old", "Sketch", "Ink"],
        "no fim, pela ordem"
    );
    let to_b = ListRef {
        kanban: b,
        list: lists(&set, b)[0],
    };
    let plan = move_all_cards(&set, t, to_b).unwrap();
    apply(&mut set, &mut h, plan, 3);
    assert_eq!(titles(&set, b, to_b.list), ["Old", "Sketch", "Ink"]);
    let ops = archive_all_cards(set.get(b).unwrap(), to_b.list);
    assert_eq!(ops.len(), 3);
    set.apply(b, ops);
    assert!(titles(&set, b, to_b.list).is_empty());
}

#[test]
fn a_kanban_from_a_template_keeps_lists_labels_and_card_templates() {
    let (mut set, a, _) = two();
    let src = first(&set, a);
    dress(&mut set, src);
    let ink = CardRef {
        kanban: a,
        card: set.get(a).unwrap().cards_in(lists(&set, a)[0])[1].id,
    };
    let mut c = set.card(ink).unwrap().clone();
    c.template = true;
    set.apply(a, vec![KanbanOp::put(Record::Card(c))]);
    let bare = set.create_from(a, "Novo", false).unwrap();
    let full = set.create_from(a, "Cheio", true).unwrap();
    assert_eq!(set.get(bare).unwrap().header.name, "Novo");
    let names = |k| -> Vec<String> {
        let k: &Kanban = set.get(k).unwrap();
        k.lists_in_order().iter().map(|l| l.title.clone()).collect()
    };
    assert_eq!(names(bare), ["To Do", "Doing"]);
    assert_eq!(
        titles(&set, bare, lists(&set, bare)[0]),
        ["Ink"],
        "só o modelo"
    );
    assert_eq!(titles(&set, full, lists(&set, full)[0]), ["Sketch", "Ink"]);
    let kf = set.get(full).unwrap();
    let sk = kf.cards.values().find(|c| c.title == "Sketch").unwrap();
    assert_eq!(kf.labels[&sk.labels[0]].name, "Bug");
    assert_eq!(kf.comments.len(), 0, "sem comentários");
    assert!(kf.activities.is_empty(), "nasce sem história");
}
