//! ⭐ **Os diálogos do Trello** — copiar, mover e espelhar um cartão (destino Kanban → lista →
//! posição; copiar com «Keep…» e o título), copiar/mover uma lista, mover todos os cartões dela,
//! ordená-la, o envelhecimento do Kanban, criar um cartão de um MODELO e um Kanban novo a partir de
//! outro. Cada um é uma folha; o «Confirm» chama a porta pura e aplica o gesto.

use super::sheet::{self, Item, Sheet};
use super::{Act, Boxed, Dlg, Pick};
use crate::docs::KanbanDocs;
use crate::edit::Field;
use ph2d_editor_core::doc_surface::{Key, KeyInput, KeyOutcome};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_filter::SortBy;
use ph2d_kanban_model::{
    Aging, CardId, CardRef, CopyKeep, Entry, FracKey, KanbanId, KanbanOp, KanbanSet, ListId,
    ListRef, MirrorId,
};

/// O que o diálogo faz.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    CopyCard(CardRef),
    /// Mover a linha `row` da vista (um cartão, ou um espelho `mirror`).
    MoveCard {
        src: CardRef,
        mirror: Option<MirrorId>,
    },
    MirrorCard(CardRef),
    CopyList(ListId),
    MoveList(ListId),
    MoveAll(ListId),
    Sort(ListId),
    Aging,
    FromTemplate(ListId),
    NewKanbanFrom,
}

/// Um diálogo aberto.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Dialog {
    pub kind: Kind,
    pub anchor: Rect,
    /// O Kanban da vista que o abriu.
    pub here: KanbanId,
    pub kanban: KanbanId,
    pub list: Option<ListId>,
    pub index: usize,
    pub keep: CopyKeep,
    pub keep_cards: bool,
    pub title: String,
    pub open: Option<Pick>,
    pub template: Option<CardRef>,
}

const KEEP: [&str; 5] = [
    "kanban.w6.keep.checklists",
    "kanban.w6.keep.labels",
    "kanban.w6.keep.members",
    "kanban.w6.keep.attachments",
    "kanban.w6.keep.comments",
];

const SORTS: [(SortBy, &str); 4] = [
    (SortBy::Newest, "kanban.w6.sort.newest"),
    (SortBy::Oldest, "kanban.w6.sort.oldest"),
    (SortBy::Name, "kanban.w6.sort.name"),
    (SortBy::Due, "kanban.w6.sort.due"),
];

fn keep_get(k: &CopyKeep, i: u8) -> bool {
    match i {
        0 => k.checklists,
        1 => k.labels,
        2 => k.members,
        3 => k.attachments,
        _ => k.comments,
    }
}

fn keep_flip(k: &mut CopyKeep, i: u8) {
    let v = !keep_get(k, i);
    match i {
        0 => k.checklists = v,
        1 => k.labels = v,
        2 => k.members = v,
        3 => k.attachments = v,
        _ => k.comments = v,
    }
}

/// As linhas da lista `list` do Kanban `kanban` (cartões e espelhos à vista).
fn rows_in(set: &KanbanSet, kanban: KanbanId, list: ListId) -> Vec<Entry> {
    set.entries_in(kanban, list)
}

/// ⭐ Abre o diálogo `kind` (ancorado em `anchor`) na vista `id`, com as escolhas do Trello.
pub(crate) fn open(docs: &mut KanbanDocs, id: u64, kind: Kind, anchor: Rect) {
    let Some(here) = docs.kanban_of(id) else {
        return;
    };
    let mut d = Dialog {
        kind,
        anchor,
        here,
        kanban: here,
        list: None,
        index: 0,
        keep: CopyKeep::default(),
        keep_cards: true,
        title: String::new(),
        open: None,
        template: None,
    };
    {
        let set = docs.set.borrow();
        let first_list = |k: KanbanId| {
            set.get(k)
                .and_then(|k| k.lists_in_order().first().map(|l| l.id))
        };
        match kind {
            Kind::CopyCard(src) | Kind::MoveCard { src, .. } | Kind::MirrorCard(src) => {
                let card = set.card(src);
                d.title = card.map(|c| c.title.clone()).unwrap_or_default();
                let (k, list, entry) = match kind {
                    Kind::MoveCard {
                        mirror: Some(m), ..
                    } => {
                        let l = set.get(here).and_then(|k| k.mirrors.get(&m)).map(|m| m.list);
                        (here, l, Some(Entry::Mirror { id: m, of: src }))
                    }
                    _ => (src.kanban, card.map(|c| c.list), Some(Entry::Card(src.card))),
                };
                d.kanban = k;
                d.list = list;
                let at = list
                    .map(|l| rows_in(&set, k, l))
                    .and_then(|v| v.iter().position(|e| Some(*e) == entry));
                d.index = match kind {
                    Kind::CopyCard(_) => at.map_or(0, |i| i + 1),
                    _ => at.unwrap_or(0),
                };
                if let Kind::MirrorCard(_) = kind {
                    // O espelho vai para OUTRA lista: a 1.ª que não é a do cartão.
                    d.list = set.get(k).and_then(|kk| {
                        kk.lists_in_order()
                            .iter()
                            .map(|l| l.id)
                            .find(|l| Some(*l) != list)
                    });
                    d.index = 0;
                }
            }
            Kind::CopyList(l) | Kind::MoveList(l) | Kind::MoveAll(l) | Kind::Sort(l) => {
                d.list = Some(l);
                d.title = set
                    .get(here)
                    .and_then(|k| k.lists.get(&l))
                    .map(|l| l.title.clone())
                    .unwrap_or_default();
                d.index = set
                    .get(here)
                    .and_then(|k| k.lists_in_order().iter().position(|x| x.id == l))
                    .unwrap_or(0);
                if let Kind::MoveAll(_) = kind {
                    d.list = set.get(here).and_then(|k| {
                        k.lists_in_order().iter().map(|x| x.id).find(|x| *x != l)
                    });
                }
            }
            Kind::FromTemplate(l) => {
                d.list = Some(l);
                d.keep = CopyKeep::template();
            }
            Kind::NewKanbanFrom => {
                d.title = set
                    .get(here)
                    .map(|k| tr_with("kanban.w6.copy_of", &[("name", &k.header.name)]))
                    .unwrap_or_default();
                d.keep_cards = false;
            }
            Kind::Aging => {}
        }
        if d.list.is_none() {
            d.list = first_list(d.kanban);
        }
    }
    let title = d.title.clone();
    let typed = matches!(
        kind,
        Kind::CopyCard(_) | Kind::CopyList(_) | Kind::NewKanbanFrom
    );
    if let Some(t) = docs.tab_mut(id) {
        t.w6.over = None;
        t.menu = None;
        t.w6.scroll = 0.0;
        t.w6.dialog = Some(d);
        if typed {
            t.w6.focus(Boxed::Title, &title);
        } else {
            t.w6.input = None;
        }
    }
}

fn select(s: &mut Sheet, d: &Dialog, p: Pick, label: &str, value: String, options: Vec<(String, Dlg, bool)>) {
    let open = d.open == Some(p);
    s.push(
        Item::Select {
            label: tr(label).to_owned(),
            value,
            open,
        },
        Act::Dlg(Dlg::Open(p)),
    );
    if open {
        for (text, a, on) in options {
            s.push(Item::Choice { on, text }, Act::Dlg(a));
        }
    }
}

/// Os três seletores do destino: Kanban, lista e (se `pos`) posição.
fn destination(s: &mut Sheet, d: &Dialog, set: &KanbanSet, lists: bool, pos: Option<usize>) {
    let kanbans: Vec<(String, Dlg, bool)> = set
        .iter()
        .map(|k| (k.header.name.clone(), Dlg::Kanban(k.id), k.id == d.kanban))
        .collect();
    let kname = set.get(d.kanban).map(|k| k.header.name.clone()).unwrap_or_default();
    select(s, d, Pick::Kanban, "kanban.w6.pick.kanban", kname, kanbans);
    if lists {
        let ls: Vec<(String, Dlg, bool)> = set
            .get(d.kanban)
            .map(|k| {
                k.lists_in_order()
                    .iter()
                    .map(|l| (l.title.clone(), Dlg::List(l.id), Some(l.id) == d.list))
                    .collect()
            })
            .unwrap_or_default();
        let lname = d
            .list
            .and_then(|l| set.get(d.kanban)?.lists.get(&l).map(|l| l.title.clone()))
            .unwrap_or_default();
        select(s, d, Pick::List, "kanban.w6.pick.list", lname, ls);
    }
    if let Some(n) = pos {
        let opts = (0..n)
            .map(|i| ((i + 1).to_string(), Dlg::Position(i), i == d.index))
            .collect();
        select(s, d, Pick::Position, "kanban.w6.pick.position", (d.index + 1).to_string(), opts);
    }
}

/// Quantas posições a lista de destino oferece (as linhas, mais o fim; sem a que se move).
fn positions(d: &Dialog, set: &KanbanSet) -> usize {
    let Some(l) = d.list else {
        return 1;
    };
    let n = rows_in(set, d.kanban, l).len();
    let moving_here = match d.kind {
        Kind::MoveCard { src, mirror } => {
            let (e, home) = match mirror {
                Some(m) => (Entry::Mirror { id: m, of: src }, d.here),
                None => (Entry::Card(src.card), src.kanban),
            };
            d.kanban == home && rows_in(set, d.kanban, l).contains(&e)
        }
        _ => false,
    };
    n + usize::from(!moving_here)
}

fn input(s: &mut Sheet, d: &Dialog, field: Option<&Field>, caption: &str) {
    s.caption(caption);
    s.push(
        Item::Input {
            field: field.cloned(),
            text: d.title.clone(),
            placeholder: String::new(),
        },
        Act::Dlg(Dlg::Title),
    );
}

fn button(s: &mut Sheet, key: &str) {
    s.push(
        Item::Button {
            text: tr(key).to_owned(),
            primary: true,
        },
        Act::Dlg(Dlg::Confirm),
    );
}

/// ⭐ A folha do diálogo.
pub(crate) fn sheet(d: &Dialog, set: &KanbanSet, field: Option<&Field>) -> Sheet {
    let mut s = Sheet::new(sheet::W);
    match d.kind {
        Kind::CopyCard(src) => {
            s.title("kanban.w6.copy_card");
            input(&mut s, d, field, "kanban.w6.title");
            s.caption("kanban.w6.keep");
            let counts = keep_counts(set, src);
            for (i, key) in KEEP.iter().enumerate() {
                let i = u8::try_from(i).unwrap_or(0);
                let text = tr_with(key, &[("n", &counts[usize::from(i)])]);
                s.check(keep_get(&d.keep, i), text, Act::Dlg(Dlg::Keep(i)));
            }
            s.caption("kanban.w6.copy_to");
            destination(&mut s, d, set, true, Some(positions(d, set)));
            button(&mut s, "kanban.w6.create_card");
        }
        Kind::MoveCard { .. } => {
            s.title("kanban.w6.move_card");
            s.caption("kanban.w6.destination");
            destination(&mut s, d, set, true, Some(positions(d, set)));
            button(&mut s, "kanban.w6.move");
        }
        Kind::MirrorCard(_) => {
            s.title("kanban.w6.mirror_card");
            s.push(Item::Note(tr("kanban.w6.mirror_note").to_owned()), Act::Inside);
            destination(&mut s, d, set, true, Some(positions(d, set)));
            button(&mut s, "kanban.w6.mirror");
        }
        Kind::CopyList(_) => {
            s.title("kanban.w6.copy_list");
            input(&mut s, d, field, "kanban.w6.name");
            destination(&mut s, d, set, false, None);
            button(&mut s, "kanban.w6.create_list");
        }
        Kind::MoveList(l) => {
            s.title("kanban.w6.move_list");
            let n = set.get(d.kanban).map_or(0, |k| k.lists_in_order().len());
            let here = d.kanban == d.here && set.get(d.here).is_some_and(|k| k.lists.contains_key(&l));
            destination(&mut s, d, set, false, Some(n + usize::from(!here)));
            button(&mut s, "kanban.w6.move");
        }
        Kind::MoveAll(_) => {
            s.title("kanban.w6.move_all");
            destination(&mut s, d, set, true, None);
            button(&mut s, "kanban.w6.move");
        }
        Kind::Sort(_) => {
            s.title("kanban.w6.sort");
            for (by, key) in SORTS {
                s.push(
                    Item::Entry {
                        text: tr(key).to_owned(),
                        icon: None,
                        on: false,
                    },
                    Act::Dlg(Dlg::Sort(by)),
                );
            }
        }
        Kind::Aging => {
            s.title("kanban.w6.aging");
            s.push(Item::Note(tr("kanban.w6.aging_note").to_owned()), Act::Inside);
            let now = set.get(d.here).and_then(|k| k.header.aging);
            for (i, key, on) in [
                (0, "kanban.w6.aging.off", now.is_none()),
                (1, "kanban.w6.aging.regular", now == Some(Aging::Regular)),
                (2, "kanban.w6.aging.pirate", now == Some(Aging::Pirate)),
            ] {
                s.push(
                    Item::Radio {
                        on,
                        text: tr(key).to_owned(),
                    },
                    Act::Dlg(Dlg::Aging(i)),
                );
            }
        }
        Kind::FromTemplate(_) => from_template(&mut s, d, set, field),
        Kind::NewKanbanFrom => {
            s.title("kanban.w6.new_from");
            input(&mut s, d, field, "kanban.w6.name");
            let kanbans: Vec<(String, Dlg, bool)> = set
                .iter()
                .map(|k| (k.header.name.clone(), Dlg::Kanban(k.id), k.id == d.kanban))
                .collect();
            let kname = set.get(d.kanban).map(|k| k.header.name.clone()).unwrap_or_default();
            select(&mut s, d, Pick::Kanban, "kanban.w6.pick.source", kname, kanbans);
            s.check(d.keep_cards, tr("kanban.w6.keep_cards"), Act::Dlg(Dlg::KeepCards));
            button(&mut s, "kanban.w6.create");
        }
    }
    s
}

fn from_template(s: &mut Sheet, d: &Dialog, set: &KanbanSet, field: Option<&Field>) {
    s.title("kanban.w6.from_template");
    let Some(tpl) = d.template else {
        let all: Vec<(CardRef, String)> = set
            .iter()
            .flat_map(|k| {
                k.cards
                    .values()
                    .filter(|c| c.template && !c.archived && !c.meta.deleted)
                    .map(move |c| (CardRef { kanban: k.id, card: c.id }, c.title.clone()))
            })
            .collect();
        if all.is_empty() {
            s.push(Item::Note(tr("kanban.w6.no_templates").to_owned()), Act::Inside);
            s.push(Item::Note(tr("kanban.w6.no_templates_how").to_owned()), Act::Inside);
        }
        for (r, title) in all {
            s.push(
                Item::Entry {
                    text: title,
                    icon: Some(ph2d_editor_core::IconId::File),
                    on: false,
                },
                Act::Dlg(Dlg::Template(r)),
            );
        }
        return;
    };
    input(s, d, field, "kanban.w6.title");
    s.caption("kanban.w6.keep");
    let counts = keep_counts(set, tpl);
    for i in 0..4u8 {
        let text = tr_with(KEEP[usize::from(i)], &[("n", &counts[usize::from(i)])]);
        s.check(keep_get(&d.keep, i), text, Act::Dlg(Dlg::Keep(i)));
    }
    button(s, "kanban.w6.create_card");
}

/// Quantas checklists, etiquetas, membros, anexos e comentários o cartão `r` tem.
fn keep_counts(set: &KanbanSet, r: CardRef) -> [usize; 5] {
    let (Some(k), Some(c)) = (set.get(r.kanban), set.card(r)) else {
        return [0; 5];
    };
    [
        k.checklists.values().filter(|x| x.card == r.card && !x.meta.deleted).count(),
        c.labels.len(),
        c.members.len(),
        k.attachments.values().filter(|x| x.card == r.card && !x.meta.deleted).count(),
        k.comments.values().filter(|x| x.card == r.card && !x.meta.deleted).count(),
    ]
}

/// ⭐ Um clique no diálogo.
pub(crate) fn click(docs: &mut KanbanDocs, id: u64, a: Dlg) {
    if a == Dlg::Confirm {
        confirm(docs, id);
        return;
    }
    if let Dlg::Sort(by) = a {
        sort(docs, id, by);
        return;
    }
    if let Dlg::Aging(n) = a {
        aging(docs, id, n);
        return;
    }
    let tpl_title = match a {
        Dlg::Template(r) => docs.set.borrow().card(r).map(|c| c.title.clone()),
        _ => None,
    };
    let first = |set: &KanbanSet, k: KanbanId| {
        set.get(k)
            .and_then(|k| k.lists_in_order().first().map(|l| l.id))
    };
    let set = docs.set();
    let Some(t) = docs.tab_mut(id) else {
        return;
    };
    let Some(d) = t.w6.dialog.as_mut() else {
        return;
    };
    match a {
        Dlg::Open(p) => d.open = if d.open == Some(p) { None } else { Some(p) },
        Dlg::Kanban(k) => {
            d.kanban = k;
            d.list = first(&set.borrow(), k);
            d.index = 0;
            d.open = None;
        }
        Dlg::List(l) => {
            d.list = Some(l);
            d.index = 0;
            d.open = None;
        }
        Dlg::Position(i) => {
            d.index = i;
            d.open = None;
        }
        Dlg::Keep(i) => keep_flip(&mut d.keep, i),
        Dlg::KeepCards => d.keep_cards = !d.keep_cards,
        Dlg::Title => {
            let title = d.title.clone();
            t.w6.focus(Boxed::Title, &title);
        }
        Dlg::Template(r) => {
            d.template = Some(r);
            d.title = tpl_title.unwrap_or_default();
            let title = d.title.clone();
            t.w6.focus(Boxed::Title, &title);
        }
        Dlg::Confirm | Dlg::Sort(_) | Dlg::Aging(_) => {}
    }
}

fn sort(docs: &mut KanbanDocs, id: u64, by: SortBy) {
    let Some(Kind::Sort(l)) = docs.tab_mut(id).and_then(|t| t.w6.dialog.as_ref()).map(|d| d.kind) else {
        return;
    };
    let Some(kid) = docs.kanban_of(id) else {
        return;
    };
    let ops = ph2d_kanban_filter::sort_list(&docs.set.borrow(), kid, l, by);
    docs.with_doc(id, |_, doc| doc.apply(ops));
    close(docs, id);
}

fn aging(docs: &mut KanbanDocs, id: u64, n: u8) {
    let to = match n {
        1 => Some(Aging::Regular),
        2 => Some(Aging::Pirate),
        _ => None,
    };
    docs.with_doc(id, |_, doc| {
        let Some(mut h) = doc.kanban().map(|k| k.header.clone()) else {
            return false;
        };
        if h.aging == to {
            return false;
        }
        h.aging = to;
        doc.apply(vec![KanbanOp::Header(h)])
    });
}

fn close(docs: &mut KanbanDocs, id: u64) {
    if let Some(t) = docs.tab_mut(id) {
        t.w6.dialog = None;
        t.w6.input = None;
        if t.w6.quick.take().is_some() {
            t.field = None;
        }
    }
}

/// A posição da lista nova/movida no índice `index` entre as listas de `to` (sem `skip`).
fn list_pos(set: &KanbanSet, to: KanbanId, index: usize, skip: Option<ListId>) -> Option<FracKey> {
    let k = set.get(to)?;
    let others: Vec<&FracKey> = k
        .lists_in_order()
        .into_iter()
        .filter(|l| Some(l.id) != skip)
        .map(|l| &l.pos)
        .collect();
    let i = index.min(others.len());
    let lo = i.checked_sub(1).map(|j| others[j]);
    Some(FracKey::between(lo, others.get(i).copied()))
}

/// ⭐ «Confirm»: a porta pura do diálogo e o gesto aplicado.
fn confirm(docs: &mut KanbanDocs, id: u64) {
    let Some(d) = docs.tab_mut(id).and_then(|t| t.w6.dialog.clone()) else {
        return;
    };
    let now = crate::actions::now();
    let title = d.title.trim().to_owned();
    if let Kind::NewKanbanFrom = d.kind {
        if title.is_empty() {
            return;
        }
        let made = docs.set.borrow_mut().create_from(d.kanban, &title, d.keep_cards);
        close(docs, id);
        if let Some(k) = made {
            docs.open(k);
        }
        return;
    }
    let plan = {
        let set = docs.set.borrow();
        let place = |skip| d.list.and_then(|l| set.place_at(d.kanban, l, d.index, skip));
        match d.kind {
            Kind::CopyCard(src) if !title.is_empty() => place(None)
                .and_then(|p| ph2d_kanban_model::copy_card(&set, src, &p, d.keep, &title, now)),
            Kind::MoveCard { src, mirror } => match mirror {
                Some(m) => place(Some(Entry::Mirror { id: m, of: src }))
                    .and_then(|p| ph2d_kanban_model::move_mirror(&set, d.here, m, &p)),
                None => {
                    let skip = (d.kanban == src.kanban).then_some(Entry::Card(src.card));
                    place(skip).and_then(|p| ph2d_kanban_model::move_card(&set, src, &p))
                }
            },
            Kind::MirrorCard(src) => {
                place(None).and_then(|p| ph2d_kanban_model::mirror_card(&set, src, &p))
            }
            Kind::CopyList(l) if !title.is_empty() => {
                let index = if d.kanban == d.here { d.index + 1 } else { usize::MAX };
                list_pos(&set, d.kanban, index, None).and_then(|pos| {
                    let src = ListRef { kanban: d.here, list: l };
                    ph2d_kanban_model::copy_list(&set, src, d.kanban, pos, &title, now)
                })
            }
            Kind::MoveList(l) => list_pos(&set, d.kanban, d.index, Some(l)).and_then(|pos| {
                ph2d_kanban_model::move_list(&set, ListRef { kanban: d.here, list: l }, d.kanban, pos)
            }),
            Kind::MoveAll(l) => d.list.and_then(|to| {
                let src = ListRef { kanban: d.here, list: l };
                ph2d_kanban_model::move_all_cards(&set, src, ListRef { kanban: d.kanban, list: to })
            }),
            Kind::FromTemplate(l) if !title.is_empty() => d.template.and_then(|tpl| {
                let p = set.place_at(d.here, l, usize::MAX, None)?;
                ph2d_kanban_model::card_from_template(&set, tpl, &p, d.keep, &title, now)
            }),
            _ => None,
        }
    };
    let moved_away = match d.kind {
        Kind::MoveCard { src, mirror: None } => d.kanban != src.kanban,
        _ => false,
    };
    if super::gesture::apply(docs, plan) {
        close(docs, id);
        if moved_away && let Some(t) = docs.tab_mut(id) {
            t.back = None;
        }
    }
}

/// O teclado do diálogo: o título escreve; `Enter` confirma; `Esc` fecha.
pub(crate) fn key(docs: &mut KanbanDocs, id: u64, input: &KeyInput<'_>) -> KeyOutcome {
    if !input.pressed {
        return super::taken();
    }
    match input.key {
        Key::Escape => {
            close_dialog(docs, id);
            super::taken()
        }
        Key::Enter => {
            confirm(docs, id);
            super::taken()
        }
        _ => {
            let typed = super::type_into(docs, id, input);
            match typed {
                None => super::taken(),
                o => super::outcome(o),
            }
        }
    }
}

/// Fecha só o diálogo (a edição rápida por baixo fica).
fn close_dialog(docs: &mut KanbanDocs, id: u64) {
    if let Some(t) = docs.tab_mut(id) {
        t.w6.close_top();
    }
}

/// O cartão de uma linha para o diálogo de mover: o original e, se é um espelho, o espelho.
pub(crate) fn move_kind(docs: &KanbanDocs, id: u64, row: CardId) -> Option<Kind> {
    let t = docs.tabs.iter().find(|t| t.id() == id)?;
    Some(match t.rows.mirrors.get(&row) {
        Some((m, of)) => Kind::MoveCard {
            src: *of,
            mirror: Some(*m),
        },
        None => Kind::MoveCard {
            src: CardRef {
                kanban: t.view.kanban,
                card: row,
            },
            mirror: None,
        },
    })
}
