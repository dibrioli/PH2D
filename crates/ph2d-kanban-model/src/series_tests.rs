use super::*;
use crate::agenda::Repeat;
use crate::{History, KanbanId, KanbanSet};

const H: i64 = 3_600_000;
const DAY: i64 = 24 * H;
const A: KanbanId = KanbanId::AGENDA;

fn hex_fixture(text: &str) -> Vec<u8> {
    let hex: String = text.lines().filter(|l| !l.starts_with('#')).collect();
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

/// Uma agenda com «Standup» diário às 10:00 desde o dia 0.
fn daily() -> (KanbanSet, EventId) {
    let mut set = KanbanSet::default();
    let a = set.doc_mut(A).unwrap();
    let id = EventId(a.mint());
    let mut e = Event::new(id, "Standup", Moment(10 * H), Moment(11 * H), false);
    e.repeat = Some(Repeat::Daily);
    set.apply(A, vec![KanbanOp::put(Record::Event(e))]);
    (set, id)
}

fn ev(set: &KanbanSet, id: EventId) -> Event {
    set.agenda().events[&id].clone()
}

/// A ocorrência do dia `n`, levada para `to` (início) com a mesma hora de duração.
fn moved(set: &KanbanSet, id: EventId, n: i64, to: i64) -> (Moment, Event) {
    let at = Moment(n * DAY + 10 * H);
    let mut e = ev(set, id);
    (e.start, e.end) = (Moment(to), Moment(to + H));
    (at, e)
}

fn others(set: &KanbanSet, id: EventId) -> Vec<Event> {
    set.agenda()
        .events
        .values()
        .filter(|e| e.id != id && !e.meta.deleted)
        .cloned()
        .collect()
}

#[test]
fn this_event_moves_one_occurrence_out_of_the_series() {
    let (mut set, id) = daily();
    let (at, to) = moved(&set, id, 3, 3 * DAY + 14 * H);
    let ops = change_occurrence(set.agenda(), &ev(&set, id), at, &to, Scope::This);
    set.apply(A, ops);
    let s = ev(&set, id);
    assert!(s.skips(at), "a série já não tem o dia 3");
    assert!(!s.skips(Moment(4 * DAY + 10 * H)));
    let one = others(&set, id);
    assert_eq!(one.len(), 1);
    assert_eq!(
        (one[0].start, one[0].repeat),
        (Moment(3 * DAY + 14 * H), None)
    );
    assert_eq!(s.exception_at(at).unwrap().becomes, Some(one[0].id));
}

#[test]
fn this_event_unchanged_is_no_gesture() {
    let (set, id) = daily();
    let (at, to) = moved(&set, id, 3, 3 * DAY + 10 * H);
    assert!(change_occurrence(set.agenda(), &ev(&set, id), at, &to, Scope::This).is_empty());
}

#[test]
fn this_and_following_splits_the_series_at_the_occurrence() {
    let (mut set, id) = daily();
    // Uma excepção antes e outra depois: cada uma fica com a sua metade.
    let s = ev(&set, id);
    let mut before = s.clone();
    before.exceptions = vec![
        Exception {
            at: Moment(DAY + 10 * H),
            becomes: None,
        },
        Exception {
            at: Moment(6 * DAY + 10 * H),
            becomes: None,
        },
    ];
    set.apply(A, vec![KanbanOp::put(Record::Event(before))]);
    let (at, to) = moved(&set, id, 4, 4 * DAY + 9 * H);
    let ops = change_occurrence(set.agenda(), &ev(&set, id), at, &to, Scope::Following);
    set.apply(A, ops);
    let head = ev(&set, id);
    assert_eq!(head.until, Some(at));
    assert_eq!(head.exceptions.len(), 1);
    let tail = others(&set, id);
    assert_eq!(tail.len(), 1);
    assert_eq!(
        (tail[0].start, tail[0].repeat),
        (Moment(4 * DAY + 9 * H), Some(Repeat::Daily))
    );
    assert_eq!(
        tail[0].exceptions[0].at,
        Moment(6 * DAY + 9 * H),
        "a excepção anda com a série nova"
    );
}

#[test]
fn following_on_the_first_occurrence_is_all() {
    let (set, id) = daily();
    let (at, to) = moved(&set, id, 0, 12 * H);
    let ops = change_occurrence(set.agenda(), &ev(&set, id), at, &to, Scope::Following);
    assert_eq!(ops.len(), 1, "a série inteira, sem partir");
}

#[test]
fn all_events_shift_the_series_and_its_exceptions() {
    let (mut set, id) = daily();
    let mut s = ev(&set, id);
    s.exceptions = vec![Exception {
        at: Moment(2 * DAY + 10 * H),
        becomes: None,
    }];
    set.apply(A, vec![KanbanOp::put(Record::Event(s))]);
    let (at, mut to) = moved(&set, id, 5, 5 * DAY + 12 * H);
    to.title = "Daily sync".into();
    let ops = change_occurrence(set.agenda(), &ev(&set, id), at, &to, Scope::All);
    set.apply(A, ops);
    let s = ev(&set, id);
    assert_eq!((s.start, s.title.as_str()), (Moment(12 * H), "Daily sync"));
    assert!(
        s.skips(Moment(2 * DAY + 12 * H)),
        "a excepção andou duas horas"
    );
    assert!(others(&set, id).is_empty());
}

#[test]
fn delete_this_following_and_all() {
    let (mut set, id) = daily();
    let (at, to) = moved(&set, id, 2, 2 * DAY + 15 * H);
    let ops = change_occurrence(set.agenda(), &ev(&set, id), at, &to, Scope::This);
    set.apply(A, ops);
    let lone = others(&set, id)[0].id;
    // Só este (dia 1): uma excepção sem substituto.
    let d1 = Moment(DAY + 10 * H);
    set.apply(
        A,
        delete_occurrence(set.agenda(), &ev(&set, id), d1, Scope::This),
    );
    assert_eq!(ev(&set, id).exception_at(d1).unwrap().becomes, None);
    // Este e os seguintes (dia 2): a série acaba e o substituto do dia 2 vai com ela.
    set.apply(
        A,
        delete_occurrence(set.agenda(), &ev(&set, id), at, Scope::Following),
    );
    assert_eq!(ev(&set, id).until, Some(at));
    assert!(set.agenda().events[&lone].meta.deleted);
    // Todos.
    set.apply(
        A,
        delete_occurrence(set.agenda(), &ev(&set, id), Moment(10 * H), Scope::All),
    );
    assert!(ev(&set, id).meta.deleted);
}

#[test]
fn this_event_is_one_undo_step() {
    let (mut set, id) = daily();
    let before = set.agenda().clone();
    let (at, to) = moved(&set, id, 3, 3 * DAY + 14 * H);
    let ops = change_occurrence(set.agenda(), &ev(&set, id), at, &to, Scope::This);
    let mut h = History::default();
    let a = set.doc_mut(A).unwrap();
    h.apply(a, ops, 1);
    assert!(h.undo(a, 2));
    let mut back = a.events[&id].clone();
    back.meta = before.events[&id].meta;
    assert_eq!(
        back, before.events[&id],
        "a série volta (o desfazer é uma versão nova)"
    );
    assert!(
        a.events
            .values()
            .filter(|e| e.id != id)
            .all(|e| e.meta.deleted)
    );
}

#[test]
fn v6_bytes_read_as_v7_equal() {
    let bytes = hex_fixture(include_str!("../tests/fixtures/kanban_set_v6_full.hex"));
    assert_eq!(postcard::take_from_bytes::<u32>(&bytes).unwrap().0, 6);
    let back = KanbanSet::from_bytes(&bytes).unwrap();
    let (today, _) = crate::links::tests::full_v6();
    assert!(
        back.iter().eq(today.iter()),
        "o degrau v6 → v7 dá os mesmos Kanbans"
    );
    assert_eq!(back.agenda(), today.agenda(), "e a mesma agenda");
    assert!(
        back.agenda()
            .events
            .values()
            .all(|e| e.until.is_none() && e.exceptions.is_empty())
    );
    let again = KanbanSet::from_bytes(&back.to_bytes().unwrap()).unwrap();
    assert_eq!(again, back, "e, regravado em v7, continua igual");
}

#[test]
fn the_v7_bytes_round_trip_until_and_exceptions() {
    let (mut set, _) = crate::links::tests::full_v6();
    let id = *set.agenda().events.keys().next().unwrap();
    let mut s = ev(&set, id);
    s.until = Some(Moment(99 * DAY));
    s.exceptions = vec![
        Exception {
            at: Moment(7 * DAY),
            becomes: None,
        },
        Exception {
            at: Moment(14 * DAY),
            becomes: Some(EventId(77)),
        },
    ];
    set.apply(A, vec![KanbanOp::put(Record::Event(s.clone()))]);
    let bytes = set.to_bytes().unwrap();
    assert_eq!(
        postcard::take_from_bytes::<u32>(&bytes).unwrap().0,
        crate::FORMAT_VERSION
    );
    let back = KanbanSet::from_bytes(&bytes).unwrap();
    assert!(back.iter().eq(set.iter()));
    assert_eq!(back.agenda(), set.agenda());
    assert_eq!(back.agenda().events[&id].exceptions, s.exceptions);
}
