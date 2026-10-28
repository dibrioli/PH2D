//! Construtores dos testes — um Kanban com listas e cartões feitos SÓ por ops (a porta real).

use std::collections::BTreeMap;

use crate::{
    Card, CardId, Checklist, ChecklistId, Comment, CommentId, FracKey, Kanban, KanbanOp, List,
    ListId, Member, Meta, Moment, Record,
};

pub(crate) fn list(k: &Kanban, title: &str) -> KanbanOp {
    KanbanOp::put(Record::List(List {
        id: ListId(k.mint()),
        title: title.to_owned(),
        pos: k.pos_after_lists(),
        color: None,
        card_limit: None,
        archived: false,
        meta: Meta::default(),
        watchers: Vec::new(),
    }))
}

pub(crate) fn card(k: &Kanban, list: ListId, title: &str) -> KanbanOp {
    KanbanOp::put(Record::card(Card {
        id: CardId(k.mint()),
        list,
        pos: k.pos_after_cards(list),
        title: title.to_owned(),
        description: String::new(),
        labels: Vec::new(),
        members: Vec::new(),
        start: None,
        due: None,
        cover: None,
        fields: BTreeMap::new(),
        location: String::new(),
        votes: Vec::new(),
        created: Moment(0),
        archived: false,
        meta: Meta::default(),
        watchers: Vec::new(),
        origin: None,
        template: false,
    }))
}

pub(crate) fn checklist(k: &Kanban, card: CardId) -> KanbanOp {
    KanbanOp::put(Record::Checklist(Checklist {
        id: ChecklistId(k.mint()),
        card,
        pos: FracKey::between(None, None),
        title: "Checklist".to_owned(),
        hide_checked: false,
        meta: Meta::default(),
    }))
}

pub(crate) fn comment(k: &Kanban, card: CardId, who: &str, text: &str) -> KanbanOp {
    KanbanOp::put(Record::Comment(Comment {
        id: CommentId(k.mint()),
        card,
        author: Member(who.to_owned()),
        at: Moment(1_000),
        text: text.to_owned(),
        edited: None,
        reactions: Vec::new(),
        meta: Meta::default(),
    }))
}

/// Aplica uma op a `k` (sem desfazer) — o passo de montar a cena.
pub(crate) fn put(k: &mut Kanban, op: KanbanOp) {
    assert!(op.apply(k).is_some(), "a op de montar não mudou nada");
}

/// Um Kanban «A Fazer · Fazendo · Feito» com dois cartões na primeira lista.
pub(crate) fn sample() -> Kanban {
    let mut k = Kanban::new(crate::KanbanId(1), "Projeto");
    for t in ["To Do", "Doing", "Done"] {
        let op = list(&k, t);
        put(&mut k, op);
    }
    let todo = k.lists_in_order()[0].id;
    for t in ["Sketch", "Ink"] {
        let op = card(&k, todo, t);
        put(&mut k, op);
    }
    k
}

/// O que se vê de `k`: cada lista viva e os seus cartões, sem `Meta` (a versão só sobe).
pub(crate) fn content(k: &Kanban) -> Vec<(List, Vec<Card>)> {
    let bare = |m: &mut Meta| *m = Meta::default();
    k.lists_in_order()
        .into_iter()
        .map(|l| {
            let mut l = l.clone();
            let cards = k
                .cards_in(l.id)
                .into_iter()
                .map(|c| {
                    let mut c = c.clone();
                    bare(&mut c.meta);
                    c
                })
                .collect();
            bare(&mut l.meta);
            (l, cards)
        })
        .collect()
}

pub(crate) fn card_of(k: &Kanban, id: CardId) -> Card {
    k.cards[&id].clone()
}
