//! ⭐ W11 — as ligações dos cartões ao projecto viajam na sessão (e a lápide delas também).

use ph2d_kanban_model::{
    Card, CardId, FracKey, KanbanOp, KanbanSet, List, ListId, Meta, Moment, ProjectLink,
    ProjectLinkId, Record, RecordId, Target,
};

use crate::KanbanReplica;

fn exchange(a: &mut KanbanReplica, sa: &mut KanbanSet, b: &mut KanbanReplica, sb: &mut KanbanSet) {
    for _ in 0..3 {
        let w = a.local(sa);
        b.receive(sb, &w);
        let w = b.local(sb);
        a.receive(sa, &w);
    }
}

fn targets(set: &KanbanSet) -> Vec<Target> {
    let k = set.iter().next().unwrap();
    let card = *k.cards.keys().next().unwrap();
    k.links_of(card).iter().map(|l| l.target.clone()).collect()
}

#[test]
fn project_links_travel_in_the_session_and_so_do_their_tombstones() {
    let mut host_set = KanbanSet::default();
    let kid = host_set.create("Shared");
    let k = host_set.doc_mut(kid).unwrap();
    let ids = k.mint_n(2);
    let list = List {
        id: ListId(ids[0]),
        title: "To Do".into(),
        pos: FracKey::between(None, None),
        color: None,
        card_limit: None,
        archived: false,
        meta: Meta::default(),
        watchers: Vec::new(),
    };
    let card = Card::new(
        CardId(ids[1]),
        list.id,
        FracKey::between(None, None),
        "Jump",
        Moment(0),
    );
    let c = card.id;
    host_set.apply(
        kid,
        vec![
            KanbanOp::put(Record::List(list)),
            KanbanOp::put(Record::card(card)),
        ],
    );
    let mut host = KanbanReplica::host(1, 11, &host_set, &[kid]);
    let snap = host.snapshot(&host_set);
    let mut guest_set = KanbanSet::default();
    let (mut guest, _) = KanbanReplica::join(2, 22, &snap, &mut guest_set).unwrap();

    let k = host_set.get(kid).unwrap();
    let id = ProjectLinkId(k.mint());
    let t = Target::Object(41);
    let l = ProjectLink::new(id, c, k.pos_after_links(c), t.clone(), "Player", Moment(1));
    host_set.apply(kid, vec![KanbanOp::put(Record::ProjectLink(l))]);
    exchange(&mut host, &mut host_set, &mut guest, &mut guest_set);
    assert_eq!(targets(&guest_set), vec![t], "o colega vê a ligação");

    let gk = guest_set.iter().next().unwrap().id;
    let gid = *guest_set
        .get(gk)
        .unwrap()
        .project_links
        .keys()
        .next()
        .unwrap();
    guest_set.apply(gk, vec![KanbanOp::Delete(RecordId::ProjectLink(gid))]);
    exchange(&mut guest, &mut guest_set, &mut host, &mut host_set);
    assert!(
        targets(&host_set).is_empty(),
        "e tirá-la chega ao anfitrião"
    );
}
