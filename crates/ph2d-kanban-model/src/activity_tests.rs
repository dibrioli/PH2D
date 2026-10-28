use super::*;
use crate::test_support::{card, card_of, checklist, sample};
use crate::{
    Attachment, AttachmentId, CheckItem, Due, FracKey, History, Link, convert_item_to_card,
};

fn ana() -> Actor {
    Actor {
        by: Member("Ana".to_owned()),
        at: Moment(500),
    }
}

fn whats(k: &Kanban, s: Subject) -> Vec<Happened> {
    k.activity_of(s)
        .into_iter()
        .map(|a| a.what.clone())
        .collect()
}

#[test]
fn every_card_change_the_trello_logs_becomes_one_entry() {
    let mut k = sample();
    let [todo, doing] = [k.lists_in_order()[0].id, k.lists_in_order()[1].id];
    let id = k.cards_in(todo)[0].id;
    let mut c = card_of(&k, id);
    c.list = doing;
    c.title = "Sketch v2".to_owned();
    c.description = "x".to_owned();
    c.start = Some(Moment(1));
    c.due = Some(Due {
        at: Moment(2),
        done: false,
        remind_before_min: None,
    });
    c.members = vec![Member("Rui".to_owned())];
    c.labels = vec![crate::LabelId(99)];
    apply_logged(&mut k, vec![KanbanOp::put(Record::Card(c.clone()))], &ana());
    let rui = Member("Rui".to_owned());
    assert_eq!(
        whats(&k, Subject::Card(id)),
        vec![
            Happened::Moved {
                from: todo,
                to: doing
            },
            Happened::Renamed {
                from: "Sketch".to_owned()
            },
            Happened::DescriptionChanged,
            Happened::StartSet(Some(Moment(1))),
            Happened::DueSet(Some(Moment(2))),
            Happened::MemberAdded(rui.clone()),
        ],
        "a etiqueta não é registada (o Trello não a regista)"
    );
    let entry = k.activity_of(Subject::Card(id))[0];
    assert_eq!((entry.by.clone(), entry.at), (ana().by, ana().at));

    let mut d = card_of(&k, id);
    d.due = d.due.map(|x| Due { done: true, ..x });
    d.members.clear();
    d.archived = true;
    apply_logged(&mut k, vec![KanbanOp::put(Record::Card(d))], &ana());
    apply_logged(&mut k, vec![KanbanOp::Delete(RecordId::Card(id))], &ana());
    let tail = whats(&k, Subject::Card(id)).split_off(6);
    assert_eq!(
        tail,
        vec![
            Happened::DueDone(true),
            Happened::MemberRemoved(rui),
            Happened::Archived,
            Happened::Deleted,
        ]
    );
}

#[test]
fn lists_checklists_items_attachments_and_the_name_are_logged() {
    let mut k = sample();
    let todo = k.lists_in_order()[0].id;
    let c = k.cards_in(todo)[0].id;
    let op = crate::test_support::list(&k, "Review");
    let l = match &op {
        KanbanOp::Put(r) => match &**r {
            Record::List(l) => l.id,
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };
    apply_logged(&mut k, vec![op], &ana());
    let op = checklist(&k, c);
    apply_logged(&mut k, vec![op], &ana());
    let cl = *k.checklists.keys().next().unwrap();
    let item = CheckItem {
        id: crate::CheckItemId(k.mint()),
        checklist: cl,
        pos: FracKey::between(None, None),
        text: "rough".to_owned(),
        done: false,
        member: None,
        due: None,
        meta: Meta::default(),
    };
    apply_logged(
        &mut k,
        vec![KanbanOp::put(Record::CheckItem(item.clone()))],
        &ana(),
    );
    let done = CheckItem {
        done: true,
        ..item.clone()
    };
    let att = Attachment {
        id: AttachmentId(k.mint() + 1),
        card: c,
        pos: FracKey::between(None, None),
        name: "ref".to_owned(),
        link: Link::Url("https://x".to_owned()),
        at: Moment(3),
        meta: Meta::default(),
    };
    let mut h = k.header.clone();
    h.name = "Renamed".to_owned();
    let ops = vec![
        KanbanOp::put(Record::CheckItem(done)),
        KanbanOp::put(Record::Attachment(att.clone())),
        KanbanOp::Header(h),
    ];
    apply_logged(&mut k, ops, &ana());
    assert_eq!(
        whats(&k, Subject::List(l)),
        vec![Happened::Created { list: None }]
    );
    assert_eq!(
        whats(&k, Subject::Card(c)),
        vec![
            Happened::ChecklistAdded {
                checklist: cl,
                title: "Checklist".to_owned()
            },
            Happened::ItemChecked {
                item: item.id,
                text: "rough".to_owned(),
                done: true
            },
            Happened::Attached {
                attachment: att.id,
                name: "ref".to_owned()
            },
        ],
        "um item novo não é registado; marcá-lo é"
    );
    assert_eq!(
        whats(&k, Subject::Kanban),
        vec![Happened::Renamed {
            from: "Projeto".to_owned()
        }]
    );
}

#[test]
fn activity_for_is_pure_the_same_op_on_the_same_state_says_the_same() {
    let k = sample();
    let todo = k.lists_in_order()[0].id;
    let op = card(&k, todo, "New");
    let say = |k: &Kanban| {
        let mut k = k.clone();
        let t = op.target();
        let before = Snapshot::at(t, &k);
        op.clone().apply(&mut k).unwrap();
        activity_for(&before, &Snapshot::at(t, &k), &k)
    };
    assert_eq!(say(&k), say(&k));
    assert!(matches!(say(&k)[0].1, Happened::Created { list: Some(l) } if l == todo));
}

#[test]
fn undo_does_not_erase_history_it_logs_the_reverse() {
    let mut k = sample();
    let [todo, doing] = [k.lists_in_order()[0].id, k.lists_in_order()[1].id];
    let id = k.cards_in(todo)[0].id;
    let mut c = card_of(&k, id);
    c.list = doing;
    let mut h = History::default();
    h.apply_logged(&mut k, vec![KanbanOp::put(Record::Card(c))], 1, &ana());
    assert!(h.undo_logged(&mut k, 2, &ana()));
    assert_eq!(k.cards[&id].list, todo, "o desfazer devolveu o cartão");
    assert_eq!(
        whats(&k, Subject::Card(id)),
        vec![
            Happened::Moved {
                from: todo,
                to: doing
            },
            Happened::Moved {
                from: doing,
                to: todo
            },
        ],
        "a história fica e o desfazer é mais uma entrada"
    );
    assert!(h.redo_logged(&mut k, 3, &ana()));
    assert_eq!(k.cards[&id].list, doing);
    assert_eq!(whats(&k, Subject::Card(id)).len(), 3);
    assert!(
        h.undo(&mut k, 4),
        "o desfazer simples continua a não registar"
    );
    assert_eq!(whats(&k, Subject::Card(id)).len(), 3);
}

#[test]
fn an_item_becomes_a_card_right_below_its_origin_and_leaves_the_checklist() {
    let mut k = sample();
    let todo = k.lists_in_order()[0].id;
    let [first, second] = [k.cards_in(todo)[0].id, k.cards_in(todo)[1].id];
    let op = checklist(&k, first);
    apply_logged(&mut k, vec![op], &ana());
    let cl = *k.checklists.keys().next().unwrap();
    let item = CheckItem {
        id: crate::CheckItemId(k.mint()),
        checklist: cl,
        pos: FracKey::between(None, None),
        text: "Clean lines".to_owned(),
        done: false,
        member: Some(Member("Rui".to_owned())),
        due: Some(Moment(42)),
        meta: Meta::default(),
    };
    apply_logged(
        &mut k,
        vec![KanbanOp::put(Record::CheckItem(item.clone()))],
        &ana(),
    );
    let ops = convert_item_to_card(&k, item.id, Moment(9)).unwrap();
    let mut h = History::default();
    h.apply_logged(&mut k, ops, 1, &ana());
    let order: Vec<_> = k.cards_in(todo).iter().map(|c| c.id).collect();
    assert_eq!(order.len(), 3);
    assert_eq!(
        (order[0], order[2]),
        (first, second),
        "logo abaixo da origem"
    );
    let new = &k.cards[&order[1]];
    assert_eq!(new.title, "Clean lines");
    assert_eq!(new.members, vec![Member("Rui".to_owned())]);
    assert_eq!(new.due.map(|d| d.at), Some(Moment(42)));
    assert_eq!(new.origin, Some(first));
    assert!(k.items[&item.id].meta.deleted, "o item sai da checklist");
    assert_eq!(
        whats(&k, Subject::Card(new.id)),
        vec![Happened::Converted { from: first }]
    );
    let new_id = new.id;
    assert!(h.undo(&mut k, 2));
    assert!(
        !k.items[&item.id].meta.deleted && k.cards[&new_id].meta.deleted,
        "um passo só"
    );
    assert!(convert_item_to_card(&k, crate::CheckItemId(999), Moment(0)).is_none());
}
