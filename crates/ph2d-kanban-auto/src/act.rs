//! **As acções** — cada uma vira ops sobre uma CÓPIA de trabalho do projecto ([`Work`]), que as
//! aplica logo (a acção seguinte vê a anterior) e as junta ao [`Plan`] que o executor devolve.

use ph2d_kanban_dates::{add_days, day_of, instant};
use ph2d_kanban_filter::{SortBy, sort_list};
use ph2d_kanban_model::{
    Card, CardId, CardRef, CheckItem, CheckItemId, Checklist, ChecklistId, Comment, CommentId,
    CopyKeep, Due, FracKey, KanbanId, KanbanOp, KanbanSet, Member, Meta, Moment, Place, Plan,
    Record, copy_card, mirror_card, move_card, remove_card,
};

use crate::cond::Cx;
use crate::event::{Event, Seen};
use crate::spec::{Action, DateRule, Dest, End, Sort};
use crate::vars::expand;

/// O cartão em que uma acção de cartão age (segue-o se ele muda de Kanban).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Target {
    pub kanban: KanbanId,
    pub card: CardId,
}

/// A cópia de trabalho e o que já se fez nela.
pub(crate) struct Work {
    pub set: KanbanSet,
    pub plan: Plan,
}

impl Work {
    /// Aplica `p` à cópia, junta-o ao plano e devolve o que mudou (em nome do Butler).
    fn commit(&mut self, p: Plan) -> Vec<Event> {
        let seen = Seen::plan(&self.set, &p);
        for (k, ops) in p.into_steps() {
            self.plan.extend(k, ops.clone());
            self.set.apply(k, ops);
        }
        seen.events(&self.set, &Member::butler())
    }

    pub(crate) fn put(&mut self, k: KanbanId, r: Record) -> Vec<Event> {
        let mut p = Plan::default();
        p.push(k, KanbanOp::put(r));
        self.commit(p)
    }

    fn card(&self, t: Target) -> Option<Card> {
        self.set
            .get(t.kanban)?
            .cards
            .get(&t.card)
            .filter(|c| !c.meta.deleted)
            .cloned()
    }

    fn edit(&mut self, t: Target, f: impl FnOnce(&mut Card)) -> Vec<Event> {
        let Some(mut c) = self.card(t) else {
            return Vec::new();
        };
        f(&mut c);
        self.put(t.kanban, Record::card(c))
    }

    fn place(&self, home: KanbanId, d: Dest, at: End) -> Option<Place> {
        let kanban = d.kanban.unwrap_or(home);
        let k = self.set.get(kanban)?;
        k.lists.get(&d.list).filter(|l| !l.meta.deleted)?;
        let pos = match at {
            End::Bottom => k.pos_after_cards(d.list),
            End::Top => {
                let first = self.set.entries_in(kanban, d.list).first().copied();
                FracKey::between(None, first.and_then(|e| k.entry_pos(e)))
            }
        };
        Some(Place {
            kanban,
            list: d.list,
            pos,
        })
    }
}

fn date(r: DateRule, cx: &Cx) -> Moment {
    let ms = match r.at {
        None => add_days(cx.now.0, i64::from(r.days), cx.zone),
        Some(t) => {
            let d = day_of(cx.now.0, cx.zone).add_days(i64::from(r.days));
            instant(d, t.hour, t.minute, cx.zone).unwrap_or(cx.now.0)
        }
    };
    Moment(ms)
}

fn sort_by(s: Sort) -> SortBy {
    match s {
        Sort::Newest => SortBy::Newest,
        Sort::Oldest => SortBy::Oldest,
        Sort::Name => SortBy::Name,
        Sort::Due => SortBy::Due,
    }
}

/// O cartão que um `move_card` entre Kanbans criou no destino.
fn moved_to(p: &Plan, kanban: KanbanId) -> Option<CardId> {
    p.ops(kanban).iter().find_map(|op| match op {
        KanbanOp::Put(r) => match &**r {
            Record::Card(c) => Some(c.id),
            _ => None,
        },
        _ => None,
    })
}

/// Corre a acção de QUADRO `a` (uma vez) no Kanban `home`.
pub(crate) fn board(w: &mut Work, a: &Action, home: KanbanId, cx: &Cx) -> Vec<Event> {
    match a {
        Action::SortList { list, by } => {
            let mut p = Plan::default();
            p.extend(home, sort_list(&w.set, home, *list, sort_by(*by)));
            w.commit(p)
        }
        Action::CreateCard { to, title, at } => {
            let Some(place) = w.place(home, *to, *at) else {
                return Vec::new();
            };
            let Some(k) = w.set.get(place.kanban) else {
                return Vec::new();
            };
            let title = expand(title, &w.set, home, None, cx);
            let c = Card::new(CardId(k.mint()), place.list, place.pos, &title, cx.now);
            w.put(place.kanban, Record::card(c))
        }
        _ => Vec::new(),
    }
}

/// Corre a acção de CARTÃO `a` no cartão `t` (que pode mudar de Kanban).
#[allow(clippy::too_many_lines)]
pub(crate) fn card(w: &mut Work, a: &Action, t: &mut Target, cx: &Cx) -> Vec<Event> {
    let src = CardRef {
        kanban: t.kanban,
        card: t.card,
    };
    let now = cx.now;
    match a {
        Action::MoveTo { to, at } => {
            let Some(place) = w.place(t.kanban, *to, *at) else {
                return Vec::new();
            };
            let Some(p) = move_card(&w.set, src, &place) else {
                return Vec::new();
            };
            if place.kanban != t.kanban
                && let Some(c) = moved_to(&p, place.kanban)
            {
                *t = Target {
                    kanban: place.kanban,
                    card: c,
                };
            }
            w.commit(p)
        }
        Action::CopyTo { to, at } => {
            let (Some(place), Some(c)) = (w.place(t.kanban, *to, *at), w.card(*t)) else {
                return Vec::new();
            };
            let keep = CopyKeep::default();
            match copy_card(&w.set, src, &place, keep, &c.title, now) {
                Some(p) => w.commit(p),
                None => Vec::new(),
            }
        }
        Action::MirrorTo { to } => {
            let Some(place) = w.place(t.kanban, *to, End::Bottom) else {
                return Vec::new();
            };
            match mirror_card(&w.set, src, &place) {
                Some(p) => w.commit(p),
                None => Vec::new(),
            }
        }
        Action::Archive => match remove_card(&w.set, src, true) {
            Some(p) => w.commit(p),
            None => Vec::new(),
        },
        Action::Unarchive => w.edit(*t, |c| c.archived = false),
        Action::AddLabel(l) => w.edit(*t, |c| {
            if !c.labels.contains(l) {
                c.labels.push(*l);
            }
        }),
        Action::RemoveLabel(l) => w.edit(*t, |c| c.labels.retain(|x| x != l)),
        Action::ClearLabels => w.edit(*t, |c| c.labels.clear()),
        Action::AddMember(m) => w.edit(*t, |c| add_member(c, m)),
        Action::JoinMe => w.edit(*t, |c| add_member(c, &cx.me)),
        Action::RemoveMember(m) => w.edit(*t, |c| c.members.retain(|x| x != m)),
        Action::ClearMembers => w.edit(*t, |c| c.members.clear()),
        Action::SetDue(r) => {
            let at = date(*r, cx);
            w.edit(*t, |c| {
                let remind = c.due.and_then(|d| d.remind_before_min);
                c.due = Some(Due {
                    at,
                    done: false,
                    remind_before_min: remind,
                });
            })
        }
        Action::ClearDue => w.edit(*t, |c| c.due = None),
        Action::SetStart(r) => {
            let at = date(*r, cx);
            w.edit(*t, |c| c.start = Some(at))
        }
        Action::ClearStart => w.edit(*t, |c| c.start = None),
        Action::MarkDue { done } => w.edit(*t, |c| {
            if let Some(d) = &mut c.due {
                d.done = *done;
            }
        }),
        Action::AddChecklist { title, items } => {
            let title = expand(title, &w.set, t.kanban, Some(t.card), cx);
            add_checklist(w, *t, &title, items)
        }
        Action::CopyChecklist { from } => {
            let Some(k) = w.set.get(t.kanban) else {
                return Vec::new();
            };
            let Some(src) = k.checklists.get(from).filter(|c| !c.meta.deleted) else {
                return Vec::new();
            };
            let mut items: Vec<&CheckItem> = k
                .items
                .values()
                .filter(|i| i.checklist == *from && !i.meta.deleted)
                .collect();
            items.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
            let texts: Vec<String> = items.into_iter().map(|i| i.text.clone()).collect();
            let title = src.title.clone();
            add_checklist(w, *t, &title, &texts)
        }
        Action::CheckAll { done } => {
            let Some(k) = w.set.get(t.kanban) else {
                return Vec::new();
            };
            let lists: Vec<ChecklistId> = k
                .checklists
                .values()
                .filter(|c| c.card == t.card && !c.meta.deleted)
                .map(|c| c.id)
                .collect();
            let mut p = Plan::default();
            for i in k.items.values() {
                if lists.contains(&i.checklist) && !i.meta.deleted && i.done != *done {
                    let item = CheckItem {
                        done: *done,
                        ..i.clone()
                    };
                    p.push(t.kanban, KanbanOp::put(Record::CheckItem(item)));
                }
            }
            w.commit(p)
        }
        Action::Comment(text) => {
            let Some(k) = w.set.get(t.kanban) else {
                return Vec::new();
            };
            let c = Comment {
                id: CommentId(k.mint()),
                card: t.card,
                author: Member::butler(),
                at: now,
                text: expand(text, &w.set, t.kanban, Some(t.card), cx),
                edited: None,
                reactions: Vec::new(),
                meta: Meta::default(),
            };
            w.put(t.kanban, Record::Comment(c))
        }
        Action::SetField { field, value } => w.edit(*t, |c| match value {
            Some(v) => {
                c.fields.insert(*field, v.clone());
            }
            None => {
                c.fields.remove(field);
            }
        }),
        Action::SortList { .. } | Action::CreateCard { .. } => Vec::new(),
    }
}

fn add_member(c: &mut Card, m: &Member) {
    if !c.members.contains(m) {
        c.members.push(m.clone());
    }
}

fn add_checklist(w: &mut Work, t: Target, title: &str, items: &[String]) -> Vec<Event> {
    let Some(k) = w.set.get(t.kanban) else {
        return Vec::new();
    };
    let ids = k.mint_n(1 + items.len());
    let last = k
        .checklists
        .values()
        .filter(|c| c.card == t.card && !c.meta.deleted)
        .map(|c| &c.pos)
        .max();
    let list = ChecklistId(ids[0]);
    let mut p = Plan::default();
    p.push(
        t.kanban,
        KanbanOp::put(Record::Checklist(Checklist {
            id: list,
            card: t.card,
            pos: FracKey::between(last, None),
            title: title.to_owned(),
            hide_checked: false,
            meta: Meta::default(),
        })),
    );
    let mut pos: Option<FracKey> = None;
    for (text, id) in items.iter().zip(&ids[1..]) {
        let here = FracKey::between(pos.as_ref(), None);
        p.push(
            t.kanban,
            KanbanOp::put(Record::CheckItem(CheckItem {
                id: CheckItemId(*id),
                checklist: list,
                pos: here.clone(),
                text: text.clone(),
                done: false,
                member: None,
                due: None,
                meta: Meta::default(),
            })),
        );
        pos = Some(here);
    }
    w.commit(p)
}
