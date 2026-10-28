//! A cena e a rede de mentira dos testes: um Kanban feito por ops, pares que mexem ao acaso, e uma
//! entrega que BARALHA, ATRASA e DUPLICA cada escrita.

use ph2d_board_sync::PeerId;
use ph2d_kanban_model::{
    Card, CardId, CheckItem, CheckItemId, Checklist, ChecklistId, FieldId, FieldValue, FracKey,
    Kanban, KanbanId, KanbanOp, KanbanSet, Label, LabelId, List, ListId, Member, Meta, Moment,
    Record, RecordId,
};

use crate::{KWrite, KanbanReplica};

pub(crate) struct Rng(pub u64);

impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }

    pub fn chance(&mut self, percent: u64) -> bool {
        self.next() % 100 < percent
    }

    pub fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = self.below(i + 1);
            v.swap(i, j);
        }
    }
}

pub(crate) const K: KanbanId = KanbanId(1);

fn put(k: &mut Kanban, r: Record) {
    let _ = KanbanOp::put(r).apply(k);
}

/// Um Kanban com três listas, seis cartões, três etiquetas — tudo por ops, com ids do contador.
pub(crate) fn base() -> KanbanSet {
    let mut set = KanbanSet::default();
    let id = set.create("Projeto");
    assert_eq!(id, K);
    let k = set.doc_mut(id).unwrap();
    for t in ["To Do", "Doing", "Done"] {
        let l = List {
            id: ListId(k.mint()),
            title: t.into(),
            pos: k.pos_after_lists(),
            color: None,
            card_limit: None,
            archived: false,
            meta: Meta::default(),
            watchers: Vec::new(),
        };
        put(k, Record::List(l));
    }
    for n in ["Bug", "Feature", "Art"] {
        let l = Label {
            id: LabelId(k.mint()),
            name: n.into(),
            color: None,
            meta: Meta::default(),
        };
        put(k, Record::Label(l));
    }
    let lists: Vec<ListId> = k.lists.keys().copied().collect();
    for i in 0..6 {
        let list = lists[i % 3];
        let c = Card::new(
            CardId(k.mint()),
            list,
            k.pos_after_cards(list),
            &format!("card {i}"),
            Moment(0),
        );
        put(k, Record::card(c));
    }
    set
}

pub(crate) fn kanban(set: &KanbanSet) -> &Kanban {
    set.iter().next().unwrap()
}

pub(crate) fn doc(set: &mut KanbanSet) -> &mut Kanban {
    let id = set.iter().next().unwrap().id;
    set.doc_mut(id).unwrap()
}

pub(crate) fn edit_card(set: &mut KanbanSet, id: CardId, f: impl FnOnce(&mut Card)) {
    let k = doc(set);
    let mut c = k.cards[&id].clone();
    f(&mut c);
    put(k, Record::card(c));
}

/// Um gesto ao acaso de `peer` no Kanban dele (ids novos pelo `Kanban::mint` de sempre: em sessão a
/// réplica ligou-lhe o sorteio).
pub(crate) fn random_op(set: &mut KanbanSet, r: &mut KanbanReplica, peer: PeerId, rng: &mut Rng) {
    r.arm(set);
    let k = kanban(set);
    let cards: Vec<CardId> = k.cards.keys().copied().collect();
    let lists: Vec<ListId> = k.lists.keys().copied().collect();
    let labels: Vec<LabelId> = k.labels.keys().copied().collect();
    let checklists: Vec<ChecklistId> = k.checklists.keys().copied().collect();
    let card = cards[rng.below(cards.len())];
    let me = Member(format!("p{peer}"));
    match rng.below(14) {
        0 | 1 => {
            let list = lists[rng.below(lists.len())];
            let pos = kanban(set).pos_after_cards(list);
            edit_card(set, card, |c| (c.list, c.pos) = (list, pos));
        }
        2 => {
            let t = format!("t{peer}-{}", rng.below(1000));
            edit_card(set, card, |c| c.title = t);
        }
        3 => edit_card(set, card, |c| c.description.push('·')),
        4 => edit_card(set, card, |c| c.archived = !c.archived),
        5 => {
            let _ = KanbanOp::Delete(RecordId::Card(card)).apply(doc(set));
        }
        6 => {
            let k = kanban(set);
            let l = k.lists[&lists[rng.below(lists.len())]].clone();
            let first = k.lists.values().map(|l| l.pos.clone()).min();
            let pos = FracKey::between(None, first.as_ref());
            put(doc(set), Record::List(List { pos, ..l }));
        }
        7 => {
            let c = Checklist {
                id: ChecklistId(kanban(set).mint()),
                card,
                pos: FracKey::between(None, None),
                title: format!("cl{peer}"),
                hide_checked: false,
                meta: Meta::default(),
            };
            put(doc(set), Record::Checklist(c));
        }
        8 => {
            let l = labels[rng.below(labels.len())];
            edit_card(set, card, |c| match c.labels.iter().position(|x| *x == l) {
                Some(i) => {
                    c.labels.remove(i);
                }
                None => c.labels.push(l),
            });
        }
        9 => edit_card(set, card, |c| match c.votes.iter().position(|x| *x == me) {
            Some(i) => {
                c.votes.remove(i);
            }
            None => c.votes.push(me),
        }),
        10 => {
            let f = FieldId(1 + rng.below(2) as u64);
            let v = FieldValue::Text(format!("v{peer}-{}", rng.below(9)));
            edit_card(set, card, |c| {
                c.fields.insert(f, v);
            });
        }
        11 => {
            let mut h = kanban(set).header.clone();
            if rng.chance(50) {
                h.starred = !h.starred;
            } else {
                h.name = format!("K{peer}-{}", rng.below(99));
            }
            let _ = KanbanOp::Header(h).apply(doc(set));
        }
        12 => {
            let list = lists[rng.below(lists.len())];
            let pos = kanban(set).pos_after_cards(list);
            let c = Card::new(
                CardId(kanban(set).mint()),
                list,
                pos,
                &format!("new{peer}"),
                Moment(1),
            );
            put(doc(set), Record::card(c));
        }
        _ => {
            let Some(cl) = checklists.get(rng.below(checklists.len().max(1))).copied() else {
                return;
            };
            let it = CheckItem {
                id: CheckItemId(kanban(set).mint()),
                checklist: cl,
                pos: FracKey::between(None, None),
                text: format!("i{peer}"),
                done: rng.chance(50),
                member: Some(me),
                due: None,
                meta: Meta::default(),
            };
            put(doc(set), Record::CheckItem(it));
        }
    }
}

/// Os pares de uma sessão: o 0 é o anfitrião, os outros entraram pelo retrato dele.
pub(crate) struct Net {
    pub sets: Vec<KanbanSet>,
    pub reps: Vec<KanbanReplica>,
    /// As escritas por entregar a cada par (soltas: cada uma pode chegar antes da que a fez nascer).
    pub inbox: Vec<Vec<KWrite>>,
}

impl Net {
    pub fn new(n: usize) -> Self {
        let host_set = base();
        let host = KanbanReplica::host(1, 11, &host_set, &[K]);
        let snap = host.snapshot(&host_set);
        let mut sets = vec![host_set];
        let mut reps = vec![host];
        for p in 1..n {
            let mut s = KanbanSet::default();
            let (r, ids) =
                KanbanReplica::join(p as u64 + 1, 100 + p as u64, &snap, &mut s).unwrap();
            assert_eq!(ids, vec![K]);
            sets.push(s);
            reps.push(r);
        }
        Self {
            sets,
            reps,
            inbox: vec![Vec::new(); n],
        }
    }

    /// O que `p` mudou, para a caixa de todos os outros (às vezes em duplicado).
    pub fn send(&mut self, p: usize, rng: &mut Rng) -> usize {
        let ws = self.reps[p].local(&mut self.sets[p]);
        for q in (0..self.sets.len()).filter(|q| *q != p) {
            for w in &ws {
                self.inbox[q].push(w.clone());
                if rng.chance(20) {
                    self.inbox[q].push(w.clone());
                }
            }
        }
        ws.len()
    }

    /// Entrega a `q` uma parte baralhada da caixa (`all` = tudo).
    pub fn deliver(&mut self, q: usize, rng: &mut Rng, all: bool) {
        let mut box_ = std::mem::take(&mut self.inbox[q]);
        rng.shuffle(&mut box_);
        let keep = if all { 0 } else { rng.below(box_.len() + 1) };
        self.inbox[q] = box_.split_off(box_.len() - keep);
        for chunk in box_.chunks(1 + rng.below(4)) {
            self.reps[q].receive(&mut self.sets[q], chunk);
        }
    }

    /// Até ninguém ter mais nada a dizer.
    pub fn settle(&mut self, rng: &mut Rng) {
        for _ in 0..50 {
            let mut said = 0;
            for p in 0..self.sets.len() {
                said += self.send(p, rng);
            }
            for q in 0..self.sets.len() {
                self.deliver(q, rng, true);
            }
            if said == 0 && self.inbox.iter().all(Vec::is_empty) {
                return;
            }
        }
        panic!("a rede não sossegou em 50 voltas");
    }

    pub fn bytes(&self) -> Vec<Vec<u8>> {
        self.sets.iter().map(|s| s.to_bytes().unwrap()).collect()
    }
}
