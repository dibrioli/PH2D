use super::*;
use crate::records::Meta;
use crate::{History, KanbanOp, KanbanSet, Record};

fn hex_fixture(text: &str) -> Vec<u8> {
    let hex: String = text.lines().filter(|l| !l.starts_with('#')).collect();
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

fn me() -> Member {
    Member("Ana".into())
}

/// Uma agenda com um registo de cada tabela v5.
pub(crate) fn full_v5() -> KanbanSet {
    let (mut set, of, _) = crate::set::tests::full_v4();
    let a = set.doc_mut(KanbanId::AGENDA).unwrap();
    let ids = a.mint_n(4);
    let mut e = Event::new(
        EventId(ids[0]),
        "Review",
        Moment(1_000),
        Moment(2_000),
        false,
    );
    e.repeat = Some(Repeat::Weekly);
    e.color = Some(Hue::Purple);
    e.people = vec![me()];
    e.notes = "sala 2".into();
    e.remind_before_min = Some(15);
    let item = InboxItem {
        id: InboxId(ids[1]),
        owner: me(),
        text: "Comprar tinta".into(),
        created: Moment(500),
        pos: a.pos_after_inbox(),
        meta: Meta::default(),
    };
    let block = Block {
        id: BlockId(ids[2]),
        owner: me(),
        of: BlockOf::Card(of),
        start: Moment(3_600_000),
        minutes: 90,
        meta: Meta::default(),
    };
    let seen = Seen {
        id: SeenId(ids[3]),
        owner: me(),
        before: Moment(42),
        read: vec![7, 9],
        meta: Meta::default(),
    };
    let ops = vec![
        KanbanOp::put(Record::Event(e)),
        KanbanOp::put(Record::Inbox(item)),
        KanbanOp::put(Record::Block(block)),
        KanbanOp::put(Record::Seen(seen)),
    ];
    assert_eq!(set.apply(KanbanId::AGENDA, ops).len(), 4);
    set
}

#[test]
fn v4_bytes_read_as_v5_equal() {
    let bytes = hex_fixture(include_str!("../tests/fixtures/kanban_set_v4_full.hex"));
    assert_eq!(postcard::take_from_bytes::<u32>(&bytes).unwrap().0, 4);
    let back = KanbanSet::from_bytes(&bytes).unwrap();
    let (today, _, _) = crate::set::tests::full_v4();
    assert!(
        back.iter().eq(today.iter()),
        "o degrau v4 → v5 dá os mesmos Kanbans"
    );
    assert_eq!(back.agenda(), today.agenda(), "e a agenda vazia");
    assert!(
        back.iter()
            .all(|k| k.events.is_empty() && k.inbox.is_empty())
    );
    let again = KanbanSet::from_bytes(&back.to_bytes().unwrap()).unwrap();
    assert_eq!(again, back, "e, regravado em v5, continua igual");
}

#[test]
fn the_v5_bytes_round_trip_events_inbox_blocks_and_seen() {
    let set = full_v5();
    let bytes = set.to_bytes().unwrap();
    assert_eq!(postcard::take_from_bytes::<u32>(&bytes).unwrap().0, 5);
    let back = KanbanSet::from_bytes(&bytes).unwrap();
    assert!(back.iter().eq(set.iter()));
    assert_eq!(back.agenda(), set.agenda());
    let a = back.agenda();
    assert_eq!(
        (a.events.len(), a.inbox.len(), a.blocks.len(), a.seen.len()),
        (1, 1, 1, 1)
    );
    assert_eq!(a.inbox_of(&me()).len(), 1);
    assert_eq!(a.seen_of(&me()).unwrap().read, vec![7, 9]);
}

#[test]
fn the_agenda_is_no_kanban_of_the_list_and_answers_by_its_key() {
    let mut set = full_v5();
    assert!(set.iter().all(|k| k.id != KanbanId::AGENDA));
    assert!(set.get(KanbanId::AGENDA).is_some());
    let n = set.iter().count();
    let id = set.create("Novo");
    assert_ne!(id, KanbanId::AGENDA, "os Kanbans contam de 1");
    assert_eq!(set.iter().count(), n + 1);
    assert!(
        set.remove(KanbanId::AGENDA).is_none(),
        "a agenda não se apaga"
    );
}

#[test]
fn personal_records_never_leave_and_events_do() {
    let set = full_v5();
    let a = set.agenda();
    let shared = a.without_personal();
    assert_eq!(shared.events, a.events);
    assert!(shared.inbox.is_empty() && shared.blocks.is_empty() && shared.seen.is_empty());
    for (id, personal) in [
        (RecordId::Event(*a.events.keys().next().unwrap()), false),
        (RecordId::Inbox(*a.inbox.keys().next().unwrap()), true),
        (RecordId::Block(*a.blocks.keys().next().unwrap()), true),
        (RecordId::Seen(*a.seen.keys().next().unwrap()), true),
    ] {
        assert_eq!(id.is_personal(), personal, "{id:?}");
    }
    let mut back = shared.clone();
    back.keep_personal_of(a);
    assert_eq!(&back, a);
}

#[test]
fn an_agenda_gesture_undoes_like_a_kanbans() {
    let mut set = full_v5();
    let mut h = History::default();
    let a = set.doc_mut(KanbanId::AGENDA).unwrap();
    let id = *a.events.keys().next().unwrap();
    let mut e = a.events[&id].clone();
    e.start = Moment(9_000);
    h.apply(a, vec![KanbanOp::put(Record::Event(e))], 1);
    assert_eq!(a.events[&id].start, Moment(9_000));
    assert!(h.undo(a, 2));
    assert_eq!(a.events[&id].start, Moment(1_000));
    h.apply(a, vec![KanbanOp::Delete(RecordId::Event(id))], 3);
    assert!(a.events[&id].meta.deleted);
    assert!(h.undo(a, 4));
    assert!(!a.events[&id].meta.deleted);
}
