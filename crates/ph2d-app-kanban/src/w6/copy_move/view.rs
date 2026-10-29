//! A FOLHA de cada diálogo (o que se vê): os seletores do destino, o «Keep…», os modelos.

use super::{Dialog, Kind, keep_get, rows_in};
use crate::edit::Field;
use crate::w6::sheet::{self, Item, Sheet};
use crate::w6::{Act, Dlg, Pick};
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_filter::SortBy;
use ph2d_kanban_model::{Aging, CardRef, Entry, KanbanSet};

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

fn select(
    s: &mut Sheet,
    d: &Dialog,
    p: Pick,
    label: &str,
    value: String,
    options: Vec<(String, Dlg, bool)>,
) {
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
    let kname = set
        .get(d.kanban)
        .map(|k| k.header.name.clone())
        .unwrap_or_default();
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
        select(
            s,
            d,
            Pick::Position,
            "kanban.w6.pick.position",
            (d.index + 1).to_string(),
            opts,
        );
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
            s.push(
                Item::Note(tr("kanban.w6.mirror_note").to_owned()),
                Act::Inside,
            );
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
            let here =
                d.kanban == d.here && set.get(d.here).is_some_and(|k| k.lists.contains_key(&l));
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
            s.push(
                Item::Note(tr("kanban.w6.aging_note").to_owned()),
                Act::Inside,
            );
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
            let kname = set
                .get(d.kanban)
                .map(|k| k.header.name.clone())
                .unwrap_or_default();
            select(
                &mut s,
                d,
                Pick::Kanban,
                "kanban.w6.pick.source",
                kname,
                kanbans,
            );
            s.check(
                d.keep_cards,
                tr("kanban.w6.keep_cards"),
                Act::Dlg(Dlg::KeepCards),
            );
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
                    .map(move |c| {
                        (
                            CardRef {
                                kanban: k.id,
                                card: c.id,
                            },
                            c.title.clone(),
                        )
                    })
            })
            .collect();
        if all.is_empty() {
            s.push(
                Item::Note(tr("kanban.w6.no_templates").to_owned()),
                Act::Inside,
            );
            s.push(
                Item::Note(tr("kanban.w6.no_templates_how").to_owned()),
                Act::Inside,
            );
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
        k.checklists
            .values()
            .filter(|x| x.card == r.card && !x.meta.deleted)
            .count(),
        c.labels.len(),
        c.members.len(),
        k.attachments
            .values()
            .filter(|x| x.card == r.card && !x.meta.deleted)
            .count(),
        k.comments
            .values()
            .filter(|x| x.card == r.card && !x.meta.deleted)
            .count(),
    ]
}
