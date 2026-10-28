//! ⭐ W9 — a agenda do projecto na sessão: os eventos viajam e juntam-se; o pessoal nunca sai.

use ph2d_kanban_model::{
    Event, EventId, FracKey, InboxId, InboxItem, KanbanId, KanbanOp, KanbanSet, Member, Meta,
    Moment, Record,
};

use crate::KanbanReplica;

const A: KanbanId = KanbanId::AGENDA;

fn with_agenda(seed: u64, event: &str, who: &str) -> KanbanSet {
    let mut set = KanbanSet::default();
    let a = set.doc_mut(A).unwrap();
    a.set_session(Some(seed));
    let ids = a.mint_n(2);
    let e = Event::new(EventId(ids[0]), event, Moment(0), Moment(3_600_000), false);
    let i = InboxItem {
        id: InboxId(ids[1]),
        owner: Member(who.into()),
        text: format!("inbox {who}"),
        created: Moment(0),
        pos: FracKey::between(None, None),
        meta: Meta::default(),
    };
    set.apply(
        A,
        vec![
            KanbanOp::put(Record::Event(e)),
            KanbanOp::put(Record::Inbox(i)),
        ],
    );
    set
}

fn titles(set: &KanbanSet) -> Vec<String> {
    let mut v: Vec<String> = set
        .agenda()
        .events
        .values()
        .map(|e| e.title.clone())
        .collect();
    v.sort();
    v
}

#[test]
fn the_sessions_calendar_is_the_union_and_the_inbox_stays_home() {
    let mut host_set = with_agenda(1, "Host review", "host");
    let mut host = KanbanReplica::host(1, 11, &host_set, &[A]);
    let snap = host.snapshot(&host_set);
    let mut guest_set = with_agenda(2, "Guest deadline", "guest");
    let (mut guest, ids) = KanbanReplica::join(2, 22, &snap, &mut guest_set).unwrap();
    assert_eq!(
        ids,
        vec![A],
        "a agenda da sessão é a MINHA agenda, não um Kanban novo"
    );
    assert_eq!(guest_set.iter().count(), 0);
    assert_eq!(titles(&guest_set), ["Guest deadline", "Host review"]);
    for _ in 0..3 {
        let to_host = guest.local(&mut guest_set);
        host.receive(&mut host_set, &to_host);
        let to_guest = host.local(&mut host_set);
        guest.receive(&mut guest_set, &to_guest);
    }
    assert_eq!(
        titles(&host_set),
        titles(&guest_set),
        "os dois veem a união"
    );
    let inbox = |s: &KanbanSet| -> Vec<String> {
        s.agenda().inbox.values().map(|i| i.text.clone()).collect()
    };
    assert_eq!(
        inbox(&host_set),
        ["inbox host"],
        "a Inbox do colega nunca chega"
    );
    assert_eq!(inbox(&guest_set), ["inbox guest"], "e a minha fica minha");
}
