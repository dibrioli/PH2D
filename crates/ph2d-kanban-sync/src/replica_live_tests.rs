//! ⭐ As pontas da W5 (lote S2): ids que não colidem quando todos criam ao mesmo tempo, e reacções
//! que não se apagam umas às outras.

use ph2d_kanban_model::{
    Card, CardId, Checklist, ChecklistId, Comment, CommentId, FracKey, KanbanOp, KanbanSet, Member,
    Meta, Moment, Reaction, Record,
};

use crate::test_support::{Net, Rng, doc, kanban};

fn put(set: &mut KanbanSet, r: Record) {
    let _ = KanbanOp::put(r).apply(doc(set));
}

/// ⭐ Três pares criam N cartões, checklists e comentários CADA UM, ao mesmo tempo (nada entregue
/// no meio), pelo `Kanban::mint` de sempre: nenhum id colide (todos os registos ficam) e todos
/// acabam com os mesmos bytes. Com o contador local, os três davam os mesmos ids e a fusão comia dois
/// terços.
#[test]
fn records_created_at_the_same_time_by_every_peer_never_collide() {
    const N: usize = 200;
    let mut net = Net::new(3);
    let before = kanban(&net.sets[0]).cards.len();
    for p in 0..3 {
        net.reps[p].arm(&mut net.sets[p]);
        let list = *kanban(&net.sets[p]).lists.keys().next().unwrap();
        for i in 0..N {
            let k = kanban(&net.sets[p]);
            assert!(k.in_session());
            let ids = k.mint_n(3);
            let pos = k.pos_after_cards(list);
            let card = CardId(ids[0]);
            let c = Card::new(card, list, pos, &format!("p{p}-{i}"), Moment(0));
            put(&mut net.sets[p], Record::card(c));
            let cl = Checklist {
                id: ChecklistId(ids[1]),
                card,
                pos: FracKey::between(None, None),
                title: format!("cl{p}"),
                hide_checked: false,
                meta: Meta::default(),
            };
            put(&mut net.sets[p], Record::Checklist(cl));
            let cm = Comment {
                id: CommentId(ids[2]),
                card,
                author: Member(format!("p{p}")),
                at: Moment(0),
                text: format!("c{p}-{i}"),
                edited: None,
                reactions: Vec::new(),
                meta: Meta::default(),
            };
            put(&mut net.sets[p], Record::Comment(cm));
        }
    }
    net.settle(&mut Rng(21));
    let bytes = net.bytes();
    assert!(bytes.iter().all(|b| *b == bytes[0]), "os pares divergiram");
    let k = kanban(&net.sets[0]);
    assert_eq!(
        k.cards.len(),
        before + 3 * N,
        "cartões perdidos numa colisão"
    );
    assert_eq!(k.checklists.len(), 3 * N);
    assert_eq!(k.comments.len(), 3 * N);
    assert!(k.cards.values().all(|c| !c.meta.deleted));
}

/// Fora de sessão, o contador de sempre (os ficheiros e os testes contam com ele); a cópia de um
/// Kanban em sessão sorteia noutra sequência.
#[test]
fn out_of_session_ids_are_the_counter_and_a_copy_draws_apart() {
    let mut net = Net::new(2);
    let k = doc(&mut net.sets[1]);
    assert!(k.in_session(), "quem entrou sorteia");
    k.set_session(None);
    let local = k.mint();
    assert_eq!(local >> 63, 0, "fora de sessão: o contador");
    k.set_session(Some(9));
    let copy = k.clone();
    let (a, b) = (k.mint_n(50), copy.mint_n(50));
    assert!(a.iter().chain(&b).all(|x| x >> 63 == 1));
    assert!(a.iter().all(|x| !b.contains(x)));
    k.set_session(None);
    assert_eq!(k.mint(), local);
}

fn comment(net: &mut Net) -> CommentId {
    let k = kanban(&net.sets[0]);
    let id = CommentId(k.mint());
    let card = *k.cards.keys().next().unwrap();
    let c = Comment {
        id,
        card,
        author: Member("p0".into()),
        at: Moment(0),
        text: "olá".into(),
        edited: None,
        reactions: Vec::new(),
        meta: Meta::default(),
    };
    put(&mut net.sets[0], Record::Comment(c));
    net.settle(&mut Rng(30));
    id
}

fn react(set: &mut KanbanSet, id: CommentId, f: impl FnOnce(&mut Vec<Reaction>)) {
    let mut c = kanban(set).comments[&id].clone();
    f(&mut c.reactions);
    put(set, Record::Comment(c));
}

fn add(rs: &mut Vec<Reaction>, emoji: &str, who: &str) {
    let m = Member(who.into());
    match rs.iter_mut().find(|r| r.emoji == emoji) {
        Some(r) => r.by.push(m),
        None => rs.push(Reaction {
            emoji: emoji.into(),
            by: vec![m],
        }),
    }
}

fn count(set: &KanbanSet, id: CommentId) -> Vec<(String, usize)> {
    let mut v: Vec<(String, usize)> = kanban(set).comments[&id]
        .reactions
        .iter()
        .map(|r| (r.emoji.clone(), r.by.len()))
        .collect();
    v.sort();
    v
}

/// ⭐ Duas pessoas a reagir ao MESMO comentário ao mesmo tempo: as duas reacções ficam (um elemento
/// por `(emoji, pessoa)`); tirar a minha não leva a do colega que chegou ao mesmo tempo.
#[test]
fn two_reactions_at_the_same_time_are_two_reactions() {
    let mut net = Net::new(2);
    let id = comment(&mut net);
    react(&mut net.sets[0], id, |rs| add(rs, "👍", "p0"));
    react(&mut net.sets[1], id, |rs| {
        add(rs, "👍", "p1");
        add(rs, "❤", "p1");
    });
    net.settle(&mut Rng(31));
    let want = vec![("❤".to_owned(), 1), ("👍".to_owned(), 2)];
    for s in &net.sets {
        assert_eq!(count(s, id), want);
    }
    react(&mut net.sets[0], id, |rs| {
        rs.iter_mut().for_each(|r| r.by.retain(|m| m.0 != "p0"));
        rs.retain(|r| !r.by.is_empty());
    });
    react(&mut net.sets[1], id, |rs| add(rs, "🎉", "p1"));
    net.settle(&mut Rng(32));
    let b = net.bytes();
    assert_eq!(b[0], b[1]);
    let want = vec![
        ("❤".to_owned(), 1),
        ("🎉".to_owned(), 1),
        ("👍".to_owned(), 1),
    ];
    for s in &net.sets {
        assert_eq!(count(s, id), want);
        let up = kanban(s).comments[&id]
            .reactions
            .iter()
            .find(|r| r.emoji == "👍")
            .map(|r| r.by.clone());
        assert_eq!(up, Some(vec![Member("p1".into())]));
    }
}
