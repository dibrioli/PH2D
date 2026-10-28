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
        KanbanOp::put(Record::Label(Label {
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
        KanbanOp::put(Record::Field(FieldDef {
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
        fill: CoverFill::Image(crate::FileId([7; 32])),
        full: true,
    });
    card.fields = BTreeMap::from([(field, FieldValue::Option(OptionId(1)))]);
    card.location = "Studio".to_owned();
    card.votes = vec![Member("Ana".to_owned())];
    put(&mut k, KanbanOp::put(Record::Card(card)));
    let op = checklist(&k, c);
    put(&mut k, op);
    let cl = *k.checklists.keys().next().unwrap();
    let item = CheckItemId(k.mint());
    put(
        &mut k,
        KanbanOp::put(Record::CheckItem(CheckItem {
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
    put(&mut k, KanbanOp::put(Record::Comment(com)));
    let att = AttachmentId(k.mint());
    put(
        &mut k,
        KanbanOp::put(Record::Attachment(Attachment {
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

/// A fixture gravada pelo código v1 (cabeçalho `#`, depois hex).
fn v1_fixture() -> Vec<u8> {
    let text = include_str!("../tests/fixtures/kanban_set_v1_full.hex");
    let hex: String = text.lines().filter(|l| !l.starts_with('#')).collect();
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn v1_bytes_read_as_v2_equal() {
    let bytes = v1_fixture();
    assert_eq!(postcard::take_from_bytes::<u32>(&bytes).unwrap().0, 1);
    let back = KanbanSet::from_bytes(&bytes).unwrap();
    assert_eq!(
        back,
        full(),
        "o degrau v1 → v2 dá o mesmo conjunto que o código de hoje monta"
    );
    let again = KanbanSet::from_bytes(&back.to_bytes().unwrap()).unwrap();
    assert_eq!(again, back, "e, regravado em v2, continua igual");
}

/// `full()` mais tudo o que o v2 trouxe: seguir (três níveis), origem, atividade, ficheiros.
fn full_v2() -> (KanbanSet, crate::FileId, crate::FileId) {
    let mut set = full();
    let cover = set.put_file(vec![1_u8, 2, 3], Some([4, 2]));
    let orphan = set.put_file(vec![9_u8; 16], None);
    let id = set.iter().next().unwrap().id;
    let who = Member("Ana".to_owned());
    let actor = crate::Actor {
        by: who.clone(),
        at: Moment(77),
    };
    let k = set.doc_mut(id).unwrap();
    let mut h = k.header.clone();
    h.watchers = vec![who.clone()];
    let todo = k.lists_in_order()[0].id;
    let mut l = k.lists[&todo].clone();
    l.watchers = vec![who.clone()];
    let c = k.cards_in(todo)[0].id;
    let mut card = k.cards[&c].clone();
    card.watchers = vec![who];
    card.origin = Some(CardId(2));
    card.cover = Some(Cover {
        fill: CoverFill::Image(cover),
        full: false,
    });
    card.title = "Sketch v2".to_owned();
    let ops = vec![
        KanbanOp::Header(h),
        KanbanOp::put(Record::List(l)),
        KanbanOp::put(Record::Card(card)),
    ];
    crate::apply_logged(k, ops, &actor);
    assert!(!k.activities.is_empty());
    (set, cover, orphan)
}

#[test]
fn the_v2_bytes_round_trip_watchers_activity_and_files() {
    let (set, cover, _) = full_v2();
    let bytes = set.to_bytes().unwrap();
    let back = KanbanSet::from_bytes(&bytes).unwrap();
    assert_eq!(back.to_bytes().unwrap(), bytes, "os bytes são estáveis");
    let k = back.iter().next().unwrap();
    assert_eq!(k, set.iter().next().unwrap());
    assert_eq!(k.header.watchers.len(), 1);
    assert!(k.lists.values().any(|l| !l.watchers.is_empty()));
    assert!(
        k.cards
            .values()
            .any(|c| !c.watchers.is_empty() && c.origin.is_some())
    );
    assert_eq!(back.file(cover), set.file(cover));
    assert_eq!(back.file(cover).unwrap().size, Some([4, 2]));
}

#[test]
fn orphan_files_are_not_saved_and_the_same_bytes_are_one_file() {
    let (mut set, cover, orphan) = full_v2();
    assert_eq!(
        set.put_file(vec![1_u8, 2, 3], None),
        cover,
        "a chave é o conteúdo"
    );
    assert!(
        set.file(orphan).is_some(),
        "na memória fica (o desfazer da sessão)"
    );
    let back = KanbanSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert!(back.file(cover).is_some(), "a capa usa-o");
    assert!(
        back.file(orphan).is_none(),
        "ninguém o usa: não vai para o disco"
    );
    let id = set.iter().next().unwrap().id;
    let k = set.doc_mut(id).unwrap();
    let c = *k
        .cards
        .iter()
        .find(|(_, c)| c.cover.is_some() && c.watchers.len() == 1)
        .unwrap()
        .0;
    put(k, KanbanOp::Delete(crate::RecordId::Card(c)));
    let back = KanbanSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert!(
        back.file(cover).is_none(),
        "o cartão apagado já não segura a capa"
    );
}

/// A fixture gravada pelo código v2 (cabeçalho `#`, depois hex).
fn v2_fixture() -> Vec<u8> {
    let text = include_str!("../tests/fixtures/kanban_set_v2_full.hex");
    let hex: String = text.lines().filter(|l| !l.starts_with('#')).collect();
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn v2_bytes_read_as_v3_equal() {
    let bytes = v2_fixture();
    assert_eq!(postcard::take_from_bytes::<u32>(&bytes).unwrap().0, 2);
    let back = KanbanSet::from_bytes(&bytes).unwrap();
    let (today, cover, _) = full_v2();
    assert_eq!(
        back.iter().collect::<Vec<_>>(),
        today.iter().collect::<Vec<_>>(),
        "o degrau v2 → v3 dá os mesmos Kanbans que o código de hoje monta"
    );
    assert_eq!(back.file(cover), today.file(cover));
    let again = KanbanSet::from_bytes(&back.to_bytes().unwrap()).unwrap();
    assert_eq!(again, back, "e, regravado em v3, continua igual");
}

#[test]
fn the_v3_bytes_round_trip_templates_aging_and_mirrors() {
    let (mut set, _, _) = full_v2();
    let id = set.iter().next().unwrap().id;
    let other = set.create("Outro");
    let k = set.doc_mut(id).unwrap();
    let mut h = k.header.clone();
    h.aging = Some(crate::Aging::Pirate);
    let todo = k.lists_in_order()[0].id;
    let c = k.cards_in(todo)[0].id;
    let mut card = k.cards[&c].clone();
    card.template = true;
    put(k, KanbanOp::Header(h));
    put(k, KanbanOp::put(Record::Card(card)));
    let l = crate::test_support::list(set.get(other).unwrap(), "Mirrors");
    set.apply(other, vec![l]);
    let ko = set.get(other).unwrap();
    let at = crate::Place {
        kanban: other,
        list: ko.lists_in_order()[0].id,
        pos: ko.pos_after_lists(),
    };
    let of = crate::CardRef {
        kanban: id,
        card: c,
    };
    let plan = crate::mirror_card(&set, of, &at).unwrap();
    for (k, ops) in plan.into_steps() {
        set.apply(k, ops);
    }
    let bytes = set.to_bytes().unwrap();
    assert_eq!(postcard::take_from_bytes::<u32>(&bytes).unwrap().0, 3);
    let back = KanbanSet::from_bytes(&bytes).unwrap();
    assert!(back.iter().eq(set.iter()), "os Kanbans voltam iguais");
    assert_eq!(back.mirrors_of(of).len(), 1);
    assert_eq!(
        back.get(id).unwrap().header.aging,
        Some(crate::Aging::Pirate)
    );
    assert!(back.card(of).unwrap().template);
}
