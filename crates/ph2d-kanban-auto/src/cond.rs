//! **Gatilho e condições** — se uma regra casa um [`Event`], e se um cartão passa nas condições.
//! As condições do filtro vão por [`ph2d_kanban_filter::matches`] (a MESMA lei do painel Filtro);
//! lista, campo e checklists, que o filtro não tem, ficam aqui.

use ph2d_kanban_dates::Zone;
use ph2d_kanban_filter::{DueWindow, Filter, MatchMode, matches, normalize};
use ph2d_kanban_model::{Card, FieldValue, Kanban, Member, Moment};

use crate::event::{Event, What};
use crate::spec::{Added, By, Condition, DueIn, FieldTest, Test, Trigger};

/// O «agora», o fuso e quem fez o gesto (o «eu» das condições e de «juntar-me»).
#[derive(Clone, Debug)]
pub struct Cx<'a> {
    pub now: Moment,
    pub zone: &'a Zone,
    pub me: Member,
}

pub(crate) fn by_ok(by: &By, who: &Member) -> bool {
    match by {
        By::Anyone => true,
        By::Only(m) => m == who,
        By::Except(m) => m != who,
    }
}

/// `None` = qualquer.
fn any<T: PartialEq>(want: Option<T>, got: T) -> bool {
    want.is_none_or(|w| w == got)
}

/// A regra com o gatilho `t` dispara pelo acontecimento `e`? (`k` = o Kanban depois, para o texto
/// de um comentário.)
pub(crate) fn fires(t: &Trigger, e: &Event, k: &Kanban) -> bool {
    match (t, &e.what) {
        (Trigger::CardAdded { list, how }, What::Created { list: l }) => {
            *how != Added::MovedIn && any(*list, *l)
        }
        (Trigger::CardAdded { list, how }, What::MovedIn { list: l, .. }) => {
            *how != Added::Created && any(*list, *l)
        }
        (Trigger::MovedOut { list }, What::MovedOut { list: l }) => any(*list, *l),
        (Trigger::Label { label, added: true }, What::LabelAdded(l))
        | (
            Trigger::Label {
                label,
                added: false,
            },
            What::LabelRemoved(l),
        ) => any(*label, *l),
        (
            Trigger::Member {
                member,
                added: true,
            },
            What::MemberAdded(m),
        )
        | (
            Trigger::Member {
                member,
                added: false,
            },
            What::MemberRemoved(m),
        ) => member.as_ref().is_none_or(|w| w == m),
        (Trigger::DueSet, What::DueSet(_))
        | (Trigger::DueRemoved, What::DueRemoved)
        | (Trigger::ChecklistComplete, What::ChecklistComplete(_)) => true,
        (Trigger::DueDone { done }, What::DueDone(d))
        | (Trigger::ItemChecked { done }, What::ItemChecked { done: d, .. })
        | (Trigger::Archived { archived: done }, What::Archived(d)) => done == d,
        (Trigger::Commented { contains }, What::Commented(c)) => {
            let Some(text) = k.comments.get(c).map(|c| normalize(&c.text)) else {
                return false;
            };
            contains
                .as_ref()
                .is_none_or(|w| text.contains(normalize(w).as_str()))
        }
        (Trigger::FieldChanged { field }, What::FieldChanged(f)) => any(*field, *f),
        _ => false,
    }
}

fn window(d: DueIn) -> DueWindow {
    match d {
        DueIn::NoDates => DueWindow::NoDates,
        DueIn::Overdue => DueWindow::Overdue,
        DueIn::NextDay => DueWindow::NextDay,
        DueIn::NextWeek => DueWindow::NextWeek,
        DueIn::NextMonth => DueWindow::NextMonth,
    }
}

fn mode(all: bool) -> MatchMode {
    if all {
        MatchMode::Exact
    } else {
        MatchMode::Any
    }
}

/// O filtro que exprime `t`, quando o filtro o exprime.
fn as_filter(t: &Test) -> Option<Filter> {
    let mut f = Filter::default();
    match t {
        Test::Labels { labels, all } => {
            f.labels = labels.iter().copied().collect();
            f.mode = mode(*all);
        }
        Test::NoLabels => f.no_labels = true,
        Test::Members { members, all } => {
            f.members = members.iter().cloned().collect();
            f.mode = mode(*all);
        }
        Test::Mine => f.mine = true,
        Test::NoMembers => f.no_members = true,
        Test::Complete => f.complete = true,
        Test::Due(d) => {
            f.due.insert(window(*d));
        }
        Test::Text(s) => f.keyword.clone_from(s),
        Test::InList(_) | Test::Field { .. } | Test::ChecklistsDone => return None,
    }
    Some(f)
}

fn holds(c: &Condition, card: &Card, k: &Kanban, cx: &Cx) -> bool {
    let yes = match as_filter(&c.test) {
        Some(f) => matches(&f, card, k, cx.now, cx.zone, &cx.me),
        None => match &c.test {
            Test::InList(ls) => ls.contains(&card.list),
            Test::Field { field, is } => {
                let v = card.fields.get(field);
                match is {
                    FieldTest::Set => v.is_some_and(|v| !blank(v)),
                    FieldTest::Empty => v.is_none_or(blank),
                    FieldTest::Equals(want) => v == Some(want),
                }
            }
            Test::ChecklistsDone => {
                let lists: Vec<_> = k
                    .checklists
                    .values()
                    .filter(|c| c.card == card.id && !c.meta.deleted)
                    .map(|c| c.id)
                    .collect();
                !lists.is_empty()
                    && k.items
                        .values()
                        .filter(|i| lists.contains(&i.checklist) && !i.meta.deleted)
                        .all(|i| i.done)
            }
            _ => false,
        },
    };
    yes != c.not
}

fn blank(v: &FieldValue) -> bool {
    match v {
        FieldValue::Text(t) => t.trim().is_empty(),
        FieldValue::Checkbox(b) => !b,
        _ => false,
    }
}

/// O cartão passa em TODAS as condições.
pub(crate) fn all_hold(cs: &[Condition], card: &Card, k: &Kanban, cx: &Cx) -> bool {
    cs.iter().all(|c| holds(c, card, k, cx))
}
