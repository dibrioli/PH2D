use std::collections::BTreeMap;

use super::*;
use crate::test_support::{checklist, comment, put, sample};
use crate::{
    Attachment, AttachmentId, CardId, CheckItem, CheckItemId, Cover, CoverFill, Due, FieldDef,
    FieldId, FieldKind, FieldOption, FieldValue, FracKey, Hue, KanbanOp, Label, LabelColor,
    LabelId, Link, Member, Meta, Moment, OptionId, Reaction, Record, Shade,
};

/// Um Kanban com TODO tipo de registo e todo campo fora do valor de nascença.
fn full() -> KanbanSet {
    let mut set = KanbanSet::default();
    let id = set.create("Full");
    let mut k = sample();
    k.id = id;
    let todo = k.lists_in_order()[0].id;
    let c = k.cards_in(todo)[0].id;
    let label = LabelId(k.mint());
    put(
        &mut k,
        KanbanOp::Put(Record::Label(Label {
            id: label,
            name: "Urgent".to_owned(),
            color: Some(LabelColor {
                hue: Hue::Red,
                shade: Shade::Bold,
            }),
            meta: Meta::default(),
        })),
    );
    let field = FieldId(k.mint());
    put(
        &mut k,
        KanbanOp::Put(Record::Field(FieldDef {
            id: field,
            name: "Priority".to_owned(),
            kind: FieldKind::Dropdown(vec![FieldOption {
                id: OptionId(1),
                text: "High".to_owned(),
                color: Some(Hue::Orange),
            }]),
            on_front: true,
            pos: FracKey::between(None, None),
            meta: Meta::default(),
        })),
    );
    let mut card = k.cards[&c].clone();
    card.description = "**bold** and a [link](https://x)".to_owned();
    card.labels = vec![label];
    card.members = vec![Member("Rui".to_owned())];
    card.start = Some(Moment(1_700_000_000_000));
    card.due = Some(Due {
        at: Moment(1_700_086_400_000),
        done: true,
        remind_before_min: Some(60),
    });
    card.cover = Some(Cover {
        fill: CoverFill::Image([7; 32]),
        full: true,
    });
    card.fields = BTreeMap::from([(field, FieldValue::Option(OptionId(1)))]);
    card.location = "Studio".to_owned();
    card.votes = vec![Member("Ana".to_owned())];
    put(&mut k, KanbanOp::Put(Record::Card(card)));
    let op = checklist(&k, c);
    put(&mut k, op);
    let cl = *k.checklists.keys().next().unwrap();
    let item = CheckItemId(k.mint());
    put(
        &mut k,
        KanbanOp::Put(Record::CheckItem(CheckItem {
            id: item,
            checklist: cl,
            pos: FracKey::between(None, None),
            text: "rough".to_owned(),
            done: true,
            member: Some(Member("Ana".to_owned())),
            due: Some(Moment(5)),
            meta: Meta::default(),
        })),
    );
    let op = comment(&k, c, "Rui", "@Ana look");
    put(&mut k, op);
    let cm = *k.comments.keys().next().unwrap();
    let mut com = k.comments[&cm].clone();
    com.reactions = vec![Reaction {
        emoji: "👍".to_owned(),
        by: vec![Member("Ana".to_owned())],
    }];
    put(&mut k, KanbanOp::Put(Record::Comment(com)));
    let att = AttachmentId(k.mint());
    put(
        &mut k,
        KanbanOp::Put(Record::Attachment(Attachment {
            id: att,
            card: c,
            pos: FracKey::between(None, None),
            name: "ref".to_owned(),
            link: Link::Card(id, CardId(2)),
            at: Moment(9),
            meta: Meta::default(),
        })),
    );
    let done = k.lists_in_order()[2].id;
    put(&mut k, KanbanOp::Delete(crate::RecordId::List(done)));
    set.insert(k);
    set.create("Second");
    set
}

#[test]
fn the_bytes_round_trip_every_record_kind() {
    let set = full();
    let bytes = set.to_bytes().unwrap();
    let back = KanbanSet::from_bytes(&bytes).unwrap();
    assert_eq!(back, set);
    assert_eq!(back.to_bytes().unwrap(), bytes, "os bytes são estáveis");
    let k = back.iter().next().unwrap();
    assert!(!k.labels.is_empty() && !k.fields.is_empty() && !k.checklists.is_empty());
    assert!(!k.items.is_empty() && !k.comments.is_empty() && !k.attachments.is_empty());
    assert!(k.lists.values().any(|l| l.meta.deleted), "a lápide viaja");
}

#[test]
fn empty_bytes_are_a_project_without_kanbans() {
    assert_eq!(KanbanSet::from_bytes(&[]).unwrap(), KanbanSet::default());
}

#[test]
fn a_newer_format_is_refused_by_name() {
    let mut bytes = postcard::to_allocvec(&(FORMAT_VERSION + 1, KanbanSet::default())).unwrap();
    let err = KanbanSet::from_bytes(&bytes).unwrap_err();
    assert!(err.contains("kanban format version"), "{err}");
    bytes.truncate(1);
    assert!(KanbanSet::from_bytes(&bytes).is_err());
}

#[test]
fn ids_survive_save_and_load_so_new_ones_never_collide() {
    let set = full();
    let mut back = KanbanSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    let k = back.iter().next().unwrap();
    let used = k
        .cards
        .keys()
        .map(|c| c.0)
        .chain(k.lists.keys().map(|l| l.0))
        .max();
    assert!(k.mint() > used.unwrap());
    let fresh = back.create("Third");
    assert!(set.iter().all(|k| k.id != fresh));
}

#[test]
fn remove_and_insert_put_a_kanban_back_whole() {
    let mut set = full();
    let id = set.iter().next().unwrap().id;
    let k = set.remove(id).unwrap();
    assert!(set.get(id).is_none());
    set.insert(k.clone());
    assert_eq!(set.get(id), Some(&k));
}
