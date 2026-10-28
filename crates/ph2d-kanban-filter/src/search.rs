//! **A pesquisa global** (`/`) — sobre TODOS os Kanbans do projecto, com os operadores do Trello
//! («Searching for cards (all boards)»): `@membro` (`@me` = eu), `#etiqueta` (nome ou cor),
//! `due:day|week|month|overdue`, `is:archived|open|complete|incomplete|template`, `list:"nome"`,
//! `board:`/`kanban:"nome"`, `has:attachments|description|cover|members|checklists`, `-` nega,
//! aspas fazem frase. Sem `is:archived`, só cartões à vista (o Trello).
//!
//! O texto procura no título, nas etiquetas, nos itens de checklist, na descrição (texto plano),
//! nos comentários e nos nomes dos anexos; cada termo positivo tem de aparecer em algum. A
//! relevância soma o peso do campo onde cada termo cai (título ≫ etiqueta > checklist = descrição >
//! comentário > anexo), com bónus para o título que começa pelo termo; o empate segue a ordem do
//! projecto (Kanban, lista, posição, id) — o mesmo resultado sempre.

use ph2d_kanban_dates::Zone;
use ph2d_kanban_model::{Card, CardRef, Kanban, KanbanSet, Member, Moment};

use crate::filter::{DueWindow, due_in};
use crate::text::{normalize, tokens, unquote};

/// O que um `is:` pergunta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Is {
    Archived,
    Open,
    Complete,
    Incomplete,
    Template,
}

/// O que um `has:` pergunta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Has {
    Attachments,
    Description,
    Cover,
    Members,
    Checklists,
}

/// Um termo da consulta (os textos já normalizados).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Term {
    Text(String),
    Member(String),
    /// `@me`.
    Me,
    Label(String),
    Due(DueWindow),
    Is(Is),
    List(String),
    Kanban(String),
    Has(Has),
}

/// Um termo e se está negado (`-`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clause {
    pub negate: bool,
    pub term: Term,
}

/// Um cartão encontrado e a relevância dele.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    pub card: CardRef,
    pub score: u32,
}

fn op(name: &str, arg: &str) -> Option<Term> {
    let a = normalize(unquote(arg));
    Some(match name {
        "due" => Term::Due(match a.as_str() {
            "day" => DueWindow::NextDay,
            "week" => DueWindow::NextWeek,
            "month" => DueWindow::NextMonth,
            "overdue" => DueWindow::Overdue,
            _ => return None,
        }),
        "is" => Term::Is(match a.as_str() {
            "archived" => Is::Archived,
            "open" => Is::Open,
            "complete" => Is::Complete,
            "incomplete" => Is::Incomplete,
            "template" => Is::Template,
            _ => return None,
        }),
        "has" => Term::Has(match a.as_str() {
            "attachments" => Has::Attachments,
            "description" => Has::Description,
            "cover" => Has::Cover,
            "members" => Has::Members,
            "checklists" => Has::Checklists,
            _ => return None,
        }),
        "list" if !a.is_empty() => Term::List(a),
        "board" | "kanban" if !a.is_empty() => Term::Kanban(a),
        _ => return None,
    })
}

/// Lê a consulta. Um operador desconhecido é texto (como no Trello).
#[must_use]
pub fn parse_query(q: &str) -> Vec<Clause> {
    let mut out = Vec::new();
    for t in tokens(q) {
        let (negate, body) = match t.strip_prefix('-') {
            Some(b) if !b.is_empty() => (true, b),
            _ => (false, t.as_str()),
        };
        let term = if let Some(m) = body.strip_prefix('@') {
            let m = normalize(unquote(m));
            if m == "me" { Term::Me } else { Term::Member(m) }
        } else if let Some(l) = body.strip_prefix('#') {
            Term::Label(normalize(unquote(l)))
        } else if let Some(term) = body
            .split_once(':')
            .and_then(|(n, a)| op(&n.to_lowercase(), a))
        {
            term
        } else {
            Term::Text(normalize(unquote(body)))
        };
        if !matches!(&term, Term::Text(s) | Term::Member(s) | Term::Label(s) if s.is_empty()) {
            out.push(Clause { negate, term });
        }
    }
    out
}

/// Os campos de texto do cartão com o peso de cada um.
fn fields(card: &Card, k: &Kanban) -> Vec<(u32, String)> {
    let mut v = vec![(10, normalize(&card.title))];
    for l in &card.labels {
        if let Some(l) = k.labels.get(l).filter(|l| !l.meta.deleted) {
            v.push((5, normalize(&l.name)));
        }
    }
    for cl in k
        .checklists
        .values()
        .filter(|c| c.card == card.id && !c.meta.deleted)
    {
        v.push((3, normalize(&cl.title)));
        for it in k
            .items
            .values()
            .filter(|i| i.checklist == cl.id && !i.meta.deleted)
        {
            v.push((3, normalize(&it.text)));
        }
    }
    if !card.description.is_empty() {
        let plain = ph2d_kanban_md::parse(&card.description).plain_text();
        v.push((3, normalize(&plain)));
    }
    for c in k
        .comments
        .values()
        .filter(|c| c.card == card.id && !c.meta.deleted)
    {
        v.push((2, normalize(&c.text)));
    }
    for a in k
        .attachments
        .values()
        .filter(|a| a.card == card.id && !a.meta.deleted)
    {
        v.push((1, normalize(&a.name)));
    }
    v
}

fn label_matches(card: &Card, k: &Kanban, want: &str) -> bool {
    card.labels.iter().any(|l| {
        k.labels.get(l).is_some_and(|l| {
            !l.meta.deleted
                && (normalize(&l.name) == want
                    || l.color
                        .is_some_and(|c| normalize(&format!("{:?}", c.hue)) == want))
        })
    })
}

struct Ctx<'a> {
    k: &'a Kanban,
    now: Moment,
    zone: &'a Zone,
    me: &'a Member,
}

fn holds(t: &Term, card: &Card, x: &Ctx) -> bool {
    let k = x.k;
    match t {
        Term::Text(_) => unreachable!("o texto pontua-se à parte"),
        Term::Member(m) => card
            .members
            .iter()
            .any(|c| normalize(&c.0).contains(m.as_str())),
        Term::Me => card.members.contains(x.me),
        Term::Label(l) => label_matches(card, k, l),
        Term::Due(w) => due_in(*w, card, x.now, x.zone),
        Term::Is(Is::Archived) => card.archived,
        Term::Is(Is::Open) => !card.archived,
        Term::Is(Is::Complete) => card.due.is_some_and(|d| d.done),
        Term::Is(Is::Incomplete) => !card.due.is_some_and(|d| d.done),
        Term::Is(Is::Template) => card.template,
        Term::List(l) => k
            .lists
            .get(&card.list)
            .is_some_and(|x| normalize(&x.title).contains(l.as_str())),
        Term::Kanban(n) => normalize(&k.header.name).contains(n.as_str()),
        Term::Has(Has::Attachments) => k
            .attachments
            .values()
            .any(|a| a.card == card.id && !a.meta.deleted),
        Term::Has(Has::Description) => !card.description.trim().is_empty(),
        Term::Has(Has::Cover) => card.cover.is_some(),
        Term::Has(Has::Members) => !card.members.is_empty(),
        Term::Has(Has::Checklists) => k
            .checklists
            .values()
            .any(|c| c.card == card.id && !c.meta.deleted),
    }
}

/// A relevância do cartão para a consulta, ou `None` se não casa.
fn score(q: &[Clause], card: &Card, x: &Ctx) -> Option<u32> {
    let mut hay: Option<Vec<(u32, String)>> = None;
    let mut total = 0;
    for c in q {
        let hit = match &c.term {
            Term::Text(t) => {
                let hay = hay.get_or_insert_with(|| fields(card, x.k));
                let mut s: u32 = hay
                    .iter()
                    .filter(|(_, h)| h.contains(t.as_str()))
                    .map(|(w, _)| w)
                    .sum();
                if s > 0 && hay[0].1.starts_with(t.as_str()) {
                    s += 5;
                }
                if !c.negate {
                    total += s;
                }
                s > 0
            }
            t => holds(t, card, x),
        };
        if hit == c.negate {
            return None;
        }
    }
    Some(total)
}

/// A pesquisa `query` sobre todos os Kanbans de `set`, da mais relevante para a menos.
#[must_use]
pub fn search(set: &KanbanSet, query: &str, now: Moment, zone: &Zone, me: &Member) -> Vec<Hit> {
    let q = parse_query(query);
    if q.is_empty() {
        return Vec::new();
    }
    let archived = q
        .iter()
        .any(|c| !c.negate && c.term == Term::Is(Is::Archived));
    let mut found: Vec<(u32, usize, usize, &Card, CardRef)> = Vec::new();
    for (ki, k) in set.iter().enumerate() {
        let order: Vec<_> = k.lists.values().filter(|l| !l.meta.deleted).collect();
        let x = Ctx { k, now, zone, me };
        for card in k.cards.values().filter(|c| !c.meta.deleted) {
            let list_alive = k.lists.get(&card.list).is_some_and(|l| !l.meta.deleted);
            if !list_alive || (card.archived && !archived) {
                continue;
            }
            if let Some(s) = score(&q, card, &x) {
                let li = order
                    .iter()
                    .filter(|l| (&l.pos, l.id) < (&k.lists[&card.list].pos, card.list))
                    .count();
                let r = CardRef {
                    kanban: k.id,
                    card: card.id,
                };
                found.push((s, ki, li, card, r));
            }
        }
    }
    found.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then(a.1.cmp(&b.1))
            .then(a.2.cmp(&b.2))
            .then(a.3.pos.cmp(&b.3.pos))
            .then(a.4.card.cmp(&b.4.card))
    });
    found
        .into_iter()
        .map(|(score, .., card)| Hit { card, score })
        .collect()
}

#[cfg(test)]
#[path = "search_tests.rs"]
mod tests;
