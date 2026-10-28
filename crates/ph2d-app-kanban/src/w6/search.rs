//! ⭐ **A pesquisa** (`/`, a caixa do cabeçalho): procura em TODOS os Kanbans do projecto com os
//! operadores do Trello (`@membro`, `#etiqueta`, `due:`, `is:`, `list:`, `board:`, `has:`, `-`);
//! os resultados agrupam-se por Kanban e lista. `Enter` ou o clique abre o cartão — na aba do
//! Kanban dele, se for outro.

use super::sheet::{self, Item, Sheet};
use super::{Act, Boxed, Over};
use crate::docs::KanbanDocs;
use crate::tab::Tab;
use ph2d_editor_core::IconId;
use ph2d_editor_core::doc_surface::{Key, KeyInput, KeyOutcome};
use ph2d_i18n::tr;
use ph2d_kanban_model::CardRef;

/// Os operadores que a folha vazia ensina.
const HELP: [&str; 6] = [
    "kanban.w6.search.op_member",
    "kanban.w6.search.op_label",
    "kanban.w6.search.op_due",
    "kanban.w6.search.op_is",
    "kanban.w6.search.op_where",
    "kanban.w6.search.op_not",
];

/// ⭐ A folha dos resultados (o grupo «Kanban · Lista» por cima dos cartões dele).
pub(crate) fn sheet(tab: &Tab, set: &ph2d_kanban_model::KanbanSet) -> Sheet {
    let w = &tab.w6;
    let mut s = Sheet::new(sheet::WIDE);
    s.title("kanban.w6.search.title");
    let field = match &w.input {
        Some((Boxed::Query, f)) => Some(f.clone()),
        _ => None,
    };
    s.push(
        Item::Input {
            field,
            text: w.query.clone(),
            placeholder: tr("kanban.w6.search").to_owned(),
        },
        Act::SearchBox,
    );
    if w.query.trim().is_empty() {
        s.caption("kanban.w6.search.ops");
        for k in HELP {
            s.push(Item::Note(tr(k).to_owned()), Act::Inside);
        }
        return s;
    }
    if w.found.is_empty() {
        s.push(Item::Note(tr("kanban.w6.search.none").to_owned()), Act::Inside);
        return s;
    }
    for (group, rows) in groups(tab, set) {
        s.push(Item::Caption(group), Act::Inside);
        for (i, r, title) in rows {
            s.push(
                Item::Entry {
                    text: title,
                    icon: Some(IconId::File),
                    on: i == w.pick,
                },
                Act::Result(r),
            );
        }
    }
    s
}

/// Os resultados agrupados: `(«Kanban · Lista», [(índice no resultado, cartão, título)])`, os
/// grupos pela ordem do melhor resultado de cada.
fn groups(
    tab: &Tab,
    set: &ph2d_kanban_model::KanbanSet,
) -> Vec<(String, Vec<(usize, CardRef, String)>)> {
    let names = names(set, &tab.w6.found);
    let mut out: Vec<(String, Vec<(usize, CardRef, String)>)> = Vec::new();
    for (i, h) in tab.w6.found.iter().enumerate() {
        let Some((group, title)) = names.get(&h.card) else {
            continue;
        };
        match out.iter_mut().find(|(g, _)| g == group) {
            Some((_, v)) => v.push((i, h.card, title.clone())),
            None => out.push((group.clone(), vec![(i, h.card, title.clone())])),
        }
    }
    out
}

/// O nome do grupo e o título de cada resultado (lidos do conjunto antes de pintar).
fn names(
    set: &ph2d_kanban_model::KanbanSet,
    found: &[ph2d_kanban_filter::Hit],
) -> std::collections::BTreeMap<CardRef, (String, String)> {
    let mut out = std::collections::BTreeMap::new();
    for h in found {
        let (Some(k), Some(c)) = (set.get(h.card.kanban), set.card(h.card)) else {
            continue;
        };
        let list = k.lists.get(&c.list).map_or("", |l| l.title.as_str());
        let group = ph2d_i18n::tr_with(
            "kanban.w6.search.group",
            &[("kanban", &k.header.name), ("list", &list)],
        );
        out.insert(h.card, (group, c.title.clone()));
    }
    out
}

/// Abre o cartão `r` (na aba do Kanban dele) e fecha a pesquisa.
pub(crate) fn open(docs: &mut KanbanDocs, id: u64, r: CardRef) {
    if let Some(t) = docs.tab_mut(id) {
        t.w6.close_top();
    }
    if docs.kanban_of(id) == Some(r.kanban) {
        crate::back::open(docs, id, r.card);
        return;
    }
    docs.open(r.kanban);
    crate::back::open(docs, r.kanban.0, r.card);
}

pub(crate) fn click(docs: &mut KanbanDocs, id: u64, a: Act) {
    match a {
        Act::SearchBox => {
            let open = docs
                .tab_mut(id)
                .is_some_and(|t| t.w6.over == Some(Over::Search));
            if open {
                if let Some(t) = docs.tab_mut(id) {
                    let q = t.w6.query.clone();
                    t.w6.focus(Boxed::Query, &q);
                }
            } else {
                super::toggle_over(docs, id, Over::Search);
            }
        }
        Act::Result(r) => open(docs, id, r),
        _ => {}
    }
}

/// O teclado da pesquisa: escrever procura; `↑`/`↓` escolhem; `Enter` abre; `Esc` fecha.
pub(crate) fn key(docs: &mut KanbanDocs, id: u64, input: &KeyInput<'_>) -> KeyOutcome {
    if !input.pressed {
        return super::taken();
    }
    let Some(t) = docs.tab_mut(id) else {
        return KeyOutcome::default();
    };
    let n = t.w6.found.len();
    match input.key {
        Key::Escape => {
            t.w6.close_top();
            super::taken()
        }
        Key::Down => {
            t.w6.pick = (t.w6.pick + 1).min(n.saturating_sub(1));
            super::taken()
        }
        Key::Up => {
            t.w6.pick = t.w6.pick.saturating_sub(1);
            super::taken()
        }
        Key::Enter => {
            let r = t.w6.found.get(t.w6.pick).map(|h| h.card);
            if let Some(r) = r {
                open(docs, id, r);
            }
            super::taken()
        }
        _ => super::outcome(super::type_into(docs, id, input)),
    }
}
