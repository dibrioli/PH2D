//! ⭐ **`@` `#` `^` no título do cartão novo** (o Trello): `@pessoa` põe o membro, `#etiqueta` a
//! etiqueta, `^top` / `^bottom` / `^3` a posição na lista. Enquanto se escreve, a palavra do cursor
//! mostra as sugestões (`Tab` ou `Enter` completa, `↑`/`↓` escolhem); ao gravar, as palavras que
//! casam saem do título e viram membros, etiquetas e posição.

use super::sheet::{self, Item, Sheet};
use super::Act;
use crate::actions::Doc;
use crate::docs::KanbanDocs;
use crate::edit::Target;
use crate::tab::Tab;
use ph2d_editor_core::doc_surface::{Key, KeyInput, KeyOutcome};
use ph2d_kanban_filter::normalize;
use crate::metrics as m;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{CardId, FracKey, Kanban, LabelId, ListId, Member};

/// Quantas sugestões cabem por baixo do compositor: as linhas da folha (`ROW`) que ficam entre a
/// âncora e a borda da área (a folha que não cabe sobe e tapava o campo). Pelo menos uma. Medição
/// em `docs/TrelloClone/loop/MEDICOES.md` (T1).
pub(crate) fn rows_fit(anchor: Rect, area: Rect) -> usize {
    let below = anchor.y + anchor.h + m::LABEL_GAP;
    let room = area.y + area.h - sheet::PAD - below - sheet::PAD;
    ((room / sheet::ROW).floor().max(1.0)) as usize
}

fn fit(tab: &Tab) -> usize {
    tab.layout
        .lists
        .iter()
        .find_map(|l| l.composer)
        .map_or(usize::MAX, |at| rows_fit(at, tab.layout.area))
}

/// Onde o cartão novo cai.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Spot {
    Top,
    Bottom,
    At(usize),
}

/// O título lido: o texto que fica, os membros, as etiquetas e a posição.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Parsed {
    pub title: String,
    pub members: Vec<Member>,
    pub labels: Vec<LabelId>,
    pub spot: Option<Spot>,
}

fn key_of(s: &str) -> String {
    normalize(s).split_whitespace().collect()
}

fn person(people: &[Member], t: &str) -> Option<Member> {
    let t = key_of(t);
    people
        .iter()
        .find(|p| key_of(&p.0) == t)
        .or_else(|| {
            let mut m = people.iter().filter(|p| key_of(&p.0).starts_with(&t));
            let first = m.next();
            first.filter(|_| m.next().is_none())
        })
        .cloned()
}

fn label(k: &Kanban, t: &str) -> Option<LabelId> {
    let t = key_of(t);
    let live = || k.labels.values().filter(|l| !l.meta.deleted && !l.name.is_empty());
    live()
        .find(|l| key_of(&l.name) == t)
        .or_else(|| {
            let mut m = live().filter(|l| key_of(&l.name).starts_with(&t));
            let first = m.next();
            first.filter(|_| m.next().is_none())
        })
        .map(|l| l.id)
}

fn spot(t: &str) -> Option<Spot> {
    match normalize(t).as_str() {
        "top" => Some(Spot::Top),
        "bottom" => Some(Spot::Bottom),
        n => n.parse::<usize>().ok().filter(|n| *n > 0).map(|n| Spot::At(n - 1)),
    }
}

/// As pessoas que o `@` conhece: «eu», as equipas e as que estão nos cartões.
fn people_of(k: &Kanban, me: &Member, team: &[String]) -> Vec<Member> {
    let mut v = super::people(k, team);
    if !v.contains(me) {
        v.insert(0, me.clone());
    }
    v
}

/// ⭐ Lê o título `value` do cartão novo na vista `id`.
pub(crate) fn parse(docs: &KanbanDocs, id: u64, value: &str) -> Parsed {
    let set = docs.set.borrow();
    let Some(k) = docs.kanban_of(id).and_then(|kid| set.get(kid)) else {
        return Parsed {
            title: value.to_owned(),
            ..Parsed::default()
        };
    };
    let people = people_of(k, &docs.me, &docs.team);
    let mut out = Parsed::default();
    let mut words = Vec::new();
    for w in value.split_whitespace() {
        let mut ch = w.chars();
        let rest = &w[w.chars().next().map_or(0, char::len_utf8)..];
        let took = match ch.next() {
            Some('@') if !rest.is_empty() => person(&people, rest).map(|p| {
                if !out.members.contains(&p) {
                    out.members.push(p);
                }
            }),
            Some('#') if !rest.is_empty() => label(k, rest).map(|l| {
                if !out.labels.contains(&l) {
                    out.labels.push(l);
                }
            }),
            Some('^') if !rest.is_empty() => spot(rest).map(|s| out.spot = Some(s)),
            _ => None,
        };
        if took.is_none() {
            words.push(w);
        }
    }
    out.title = words.join(" ");
    if out.title.is_empty() {
        out.title = value.trim().to_owned();
    }
    out
}

/// ⭐ Cria o cartão lido (membros, etiquetas e posição no MESMO passo de desfazer).
pub(crate) fn add(doc: &mut Doc<'_>, list: ListId, after: Option<CardId>, p: Parsed) -> Option<CardId> {
    let pos = match p.spot {
        None => doc.pos_after(list, after)?,
        Some(s) => {
            let k = doc.kanban()?;
            let cards = k.cards_in(list);
            let i = match s {
                Spot::Top => 0,
                Spot::Bottom => cards.len(),
                Spot::At(i) => i.min(cards.len()),
            };
            let lo = i.checked_sub(1).map(|j| &cards[j].pos);
            FracKey::between(lo, cards.get(i).map(|c| &c.pos))
        }
    };
    let Parsed {
        title,
        members,
        labels,
        ..
    } = p;
    doc.add_card_at(list, pos, title, move |c| {
        c.members = members;
        c.labels = labels;
    })
}

/// A palavra no cursor do compositor (`(início em bytes, palavra)`), se começa por `@` `#` `^`.
fn word_at(text: &str, caret: usize) -> Option<(usize, &str)> {
    let head = text.get(..caret)?;
    let start = head.rfind(char::is_whitespace).map_or(0, |i| i + 1);
    let w = &head[start..];
    w.starts_with(['@', '#', '^']).then_some((start, w))
}

/// As sugestões para a palavra do cursor do compositor: `(texto a pôr, rótulo)`.
pub(crate) fn candidates(tab: &Tab, k: &Kanban, me: &Member, team: &[String]) -> Vec<(String, String)> {
    let Some(f) = tab.field.as_ref().filter(|f| matches!(f.target, Target::NewCard { .. })) else {
        return Vec::new();
    };
    let Some((_, w)) = word_at(&f.text, f.caret) else {
        return Vec::new();
    };
    let (sigil, rest) = w.split_at(1);
    let q = key_of(rest);
    let mut out: Vec<(String, String)> = match sigil {
        "@" => people_of(k, me, team)
            .into_iter()
            .filter(|p| key_of(&p.0).starts_with(&q))
            .map(|p| (format!("@{}", key_of(&p.0)), p.0))
            .collect(),
        "#" => k
            .labels
            .values()
            .filter(|l| !l.meta.deleted && !l.name.is_empty())
            .filter(|l| key_of(&l.name).starts_with(&q))
            .map(|l| (format!("#{}", key_of(&l.name)), l.name.clone()))
            .collect(),
        _ => {
            let n = match &f.target {
                Target::NewCard { list, .. } => k.cards_in(*list).len() + 1,
                _ => 1,
            };
            let mut v = vec![
                ("^top".to_owned(), ph2d_i18n::tr("kanban.w6.pos.top").to_owned()),
                ("^bottom".to_owned(), ph2d_i18n::tr("kanban.w6.pos.bottom").to_owned()),
            ];
            v.extend((1..=n).map(|i| (format!("^{i}"), ph2d_i18n::tr_with("kanban.w6.pos.n", &[("n", &i)]))));
            v.into_iter().filter(|(t, _)| t[1..].starts_with(&q)).collect()
        }
    };
    out.truncate(fit(tab));
    out
}

/// A folha das sugestões (vazia = nada a sugerir).
pub(crate) fn sheet(tab: &Tab, k: &Kanban, me: &Member, team: &[String]) -> Option<Sheet> {
    let c = candidates(tab, k, me, team);
    if c.is_empty() {
        return None;
    }
    let last = c.len() - 1;
    let mut s = Sheet::new(sheet::W);
    for (i, (_, label)) in c.into_iter().enumerate() {
        s.push(
            Item::Entry {
                text: label,
                icon: None,
                on: i == tab.w6.pick.min(last),
            },
            Act::Suggest(u8::try_from(i).unwrap_or(0)),
        );
    }
    Some(s)
}

/// Põe a sugestão `i` no lugar da palavra do cursor.
pub(crate) fn pick(docs: &mut KanbanDocs, id: u64, i: u8) {
    let team = docs.team.clone();
    let me = docs.me.clone();
    let set = docs.set();
    let set = set.borrow();
    let Some(t) = docs.tab_mut(id) else {
        return;
    };
    let Some(k) = set.get(t.view.kanban) else {
        return;
    };
    let c = candidates(t, k, &me, &team);
    let Some((put, _)) = c.get(usize::from(i)) else {
        return;
    };
    let Some(f) = t.field.as_mut() else {
        return;
    };
    let Some((start, _)) = word_at(&f.text, f.caret) else {
        return;
    };
    f.anchor = start;
    f.insert(&format!("{put} "));
    t.w6.pick = 0;
}

/// O teclado do compositor com sugestões: `Tab`/`Enter` completam, `↑`/`↓` escolhem.
pub(crate) fn key(docs: &mut KanbanDocs, id: u64, input: &KeyInput<'_>) -> Option<KeyOutcome> {
    if !input.pressed {
        return None;
    }
    let n = {
        let set = docs.set.borrow();
        let t = docs.tabs.iter().find(|t| t.id() == id)?;
        let k = set.get(t.view.kanban)?;
        candidates(t, k, &docs.me, &docs.team).len()
    };
    if n == 0 {
        if let Some(t) = docs.tab_mut(id) {
            t.w6.pick = 0;
        }
        return None;
    }
    let t = docs.tab_mut(id)?;
    match input.key {
        Key::Down => t.w6.pick = (t.w6.pick + 1).min(n - 1),
        Key::Up => t.w6.pick = t.w6.pick.saturating_sub(1),
        Key::Tab | Key::Enter => {
            let i = u8::try_from(t.w6.pick.min(n - 1)).unwrap_or(0);
            pick(docs, id, i);
        }
        _ => return None,
    }
    Some(super::taken())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_suggestions_that_fit_follow_the_room_under_the_field() {
        let area = |h| Rect::new(0.0, 0.0, 1200.0, h);
        let at = |y| Rect::new(40.0, y, 272.0, 36.0);
        // 1080 px de janela, o campo a meio: cabem as linhas do espaço, não 6.
        assert_eq!(rows_fit(at(500.0), area(1000.0)), 13);
        assert_eq!(rows_fit(at(100.0), area(1000.0)), 26);
        // Janela pequena: o campo perto do fundo deixa UMA linha (nunca zero).
        assert_eq!(rows_fit(at(250.0), area(420.0)), 3);
        assert_eq!(rows_fit(at(300.0), area(420.0)), 1);
    }
}
