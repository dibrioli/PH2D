//! A ESCRITA de uma escolha: aplicar o que o chip ofereceu, escrever o texto/número/hora, e as
//! palavras que alternam no lugar.

use super::{Pick, Typed};
use crate::auto::sentence::conditions_mut;
use crate::auto::{Sec, Slot};
use ph2d_kanban_auto::{
    Action, Condition, DueIn, DueWhen, End, Every, FieldTest, Kind, Sort, Spec, Test, TimeOfDay,
    Trigger, Unit,
};

fn sec_index(sec: Sec) -> Option<usize> {
    match sec {
        Sec::Cond(i) | Sec::Action(i) => Some(usize::from(i)),
        _ => None,
    }
}

/// ⭐ Aplica a escolha `pick` do chip `slot` a `s`.
pub(crate) fn apply(s: &mut Spec, slot: Slot, pick: Pick) {
    match (slot, pick) {
        (Slot::Head(Sec::Trigger), Pick::Trigger(t)) => {
            if let Kind::Rule { trigger, .. } = &mut s.kind {
                *trigger = t;
            }
        }
        (Slot::Head(Sec::Cond(i)), p) => {
            let Some(cs) = conditions_mut(s) else { return };
            let i = usize::from(i);
            match p {
                Pick::Test(t) if i < cs.len() => cs[i].test = t,
                Pick::Remove if i < cs.len() => {
                    cs.remove(i);
                }
                _ => {}
            }
        }
        (Slot::Head(Sec::Action(i)), p) => {
            let i = usize::from(i);
            match p {
                Pick::Action(a) if i < s.actions.len() => s.actions[i] = a,
                Pick::Remove if i < s.actions.len() => {
                    s.actions.remove(i);
                }
                _ => {}
            }
        }
        (Slot::Head(Sec::Schedule), Pick::Every(e)) => {
            if let Kind::Calendar { every, .. } = &mut s.kind {
                *every = e;
            }
        }
        (Slot::Head(Sec::Schedule), Pick::DueWhen(w)) => {
            if let Kind::DueDate { when, .. } = &mut s.kind {
                *when = w;
            }
        }
        (Slot::Arg(sec, _), p) => arg(s, sec, &p),
        _ => {}
    }
}

/// Um valor escolhido vai para o campo do MESMO tipo na secção `sec`.
fn arg(s: &mut Spec, sec: Sec, p: &Pick) {
    match sec {
        Sec::Trigger => {
            let Kind::Rule { trigger, .. } = &mut s.kind else {
                return;
            };
            match (trigger, p) {
                (Trigger::CardAdded { list, .. } | Trigger::MovedOut { list }, Pick::List(l)) => {
                    *list = *l;
                }
                (Trigger::Label { label, .. }, Pick::Label(l)) => *label = *l,
                (Trigger::Member { member, .. }, Pick::Member(m)) => member.clone_from(m),
                (Trigger::FieldChanged { field }, Pick::Field(f)) => *field = *f,
                _ => {}
            }
        }
        Sec::Scope | Sec::Name => {
            if let (
                Kind::BoardButton { scope }
                | Kind::Calendar { scope, .. }
                | Kind::DueDate { scope, .. },
                Pick::List(l),
            ) = (&mut s.kind, p)
            {
                scope.lists = l.iter().copied().collect();
            }
        }
        Sec::Schedule => {
            if let (Kind::Calendar { every, .. }, Pick::Weekdays(m)) = (&mut s.kind, p) {
                *every = Every::Weekdays(*m);
            }
        }
        Sec::Cond(i) => {
            let Some(c) = conditions_mut(s).and_then(|cs| cs.get_mut(usize::from(i))) else {
                return;
            };
            match (&mut c.test, p) {
                (Test::Labels { labels, .. }, Pick::Label(Some(l))) => *labels = vec![*l],
                (Test::Members { members, .. }, Pick::Member(Some(m))) => {
                    *members = vec![m.clone()]
                }
                (Test::InList(ls), Pick::List(Some(l))) => *ls = vec![*l],
                (Test::Field { field, .. }, Pick::Field(Some(f))) => *field = *f,
                _ => {}
            }
        }
        Sec::Action(i) => {
            let Some(a) = s.actions.get_mut(usize::from(i)) else {
                return;
            };
            match (a, p) {
                (
                    Action::MoveTo { to, .. }
                    | Action::CopyTo { to, .. }
                    | Action::MirrorTo { to }
                    | Action::CreateCard { to, .. },
                    Pick::List(Some(l)),
                ) => to.list = *l,
                (Action::SortList { list, .. }, Pick::List(Some(l))) => *list = *l,
                (Action::AddLabel(x) | Action::RemoveLabel(x), Pick::Label(Some(l))) => *x = *l,
                (Action::AddMember(x) | Action::RemoveMember(x), Pick::Member(Some(m))) => {
                    x.clone_from(m);
                }
                (Action::SetField { field, .. }, Pick::Field(Some(f))) => *field = *f,
                _ => {}
            }
        }
    }
}

/// O texto escrito na caixa do chip `slot` (o que não se lê não muda nada).
pub(crate) fn write(s: &mut Spec, slot: Slot, t: Typed, text: &str) {
    let Slot::Arg(sec, _) = slot else { return };
    let number = || text.trim().parse::<i64>().ok();
    match (sec, t) {
        (Sec::Name, Typed::Text) => text.trim().clone_into(&mut s.name),
        (Sec::Trigger, Typed::Text) => {
            if let Kind::Rule {
                trigger: Trigger::Commented { contains },
                ..
            } = &mut s.kind
            {
                *contains = Some(text.to_owned());
            }
        }
        (Sec::Cond(i), Typed::Text) => {
            if let Some(Condition {
                test: Test::Text(x),
                ..
            }) = conditions_mut(s).and_then(|c| c.get_mut(usize::from(i)))
            {
                text.clone_into(x);
            }
        }
        (Sec::Action(_), _) => action_text(s, sec_index(sec).unwrap_or(0), t, text),
        (Sec::Schedule, Typed::Number) => {
            let Some(n) = number().and_then(|n| u32::try_from(n).ok()) else {
                return;
            };
            if let Kind::DueDate {
                when: DueWhen::Before { n: x, .. } | DueWhen::After { n: x, .. },
                ..
            } = &mut s.kind
            {
                *x = n;
            }
        }
        (Sec::Schedule, Typed::Time) => {
            let Some((hour, minute)) = ph2d_kanban_views::datepick::parse_time(text) else {
                return;
            };
            let t = TimeOfDay { hour, minute };
            match &mut s.kind {
                Kind::Calendar { at, .. }
                | Kind::DueDate {
                    when: DueWhen::OnDay { at },
                    ..
                } => *at = t,
                _ => {}
            }
        }
        (Sec::Schedule, Typed::Day) => {
            let n: Vec<u8> = text
                .split(['/', '-', '.'])
                .filter_map(|x| x.trim().parse().ok())
                .collect();
            if let Kind::Calendar { every, .. } = &mut s.kind {
                match (every, n.as_slice()) {
                    (Every::MonthDay(d), [x]) if (1..=31).contains(x) => *d = *x,
                    (Every::YearDay { month, day }, [m, d])
                        if (1..=12).contains(m) && (1..=31).contains(d) =>
                    {
                        (*month, *day) = (*m, *d);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

fn action_text(s: &mut Spec, i: usize, t: Typed, text: &str) {
    let Some(a) = s.actions.get_mut(i) else {
        return;
    };
    match (a, t) {
        (
            Action::Comment(x)
            | Action::AddChecklist { title: x, .. }
            | Action::CreateCard { title: x, .. },
            Typed::Text,
        ) => text.clone_into(x),
        (Action::SetDue(r) | Action::SetStart(r), Typed::Number) => {
            if let Ok(n) = text.trim().parse::<i32>() {
                r.days = n;
            }
        }
        (Action::SetDue(r) | Action::SetStart(r), Typed::Time) => {
            r.at = ph2d_kanban_views::datepick::parse_time(text)
                .map(|(hour, minute)| TimeOfDay { hour, minute });
        }
        _ => {}
    }
}

/// A palavra `n` da secção `sec` roda para a seguinte (topo ↔ fundo, a ordenação, com ↔ sem…).
pub(crate) fn toggle(s: &mut Spec, sec: Sec) {
    match sec {
        Sec::Cond(i) => {
            if let Some(c) = conditions_mut(s).and_then(|c| c.get_mut(usize::from(i))) {
                toggle_test(c);
            }
        }
        Sec::Action(i) => {
            if let Some(a) = s.actions.get_mut(usize::from(i)) {
                match a {
                    Action::MoveTo { at, .. }
                    | Action::CopyTo { at, .. }
                    | Action::CreateCard { at, .. } => {
                        *at = if *at == End::Top {
                            End::Bottom
                        } else {
                            End::Top
                        };
                    }
                    Action::SortList { by, .. } => {
                        *by = match by {
                            Sort::Newest => Sort::Oldest,
                            Sort::Oldest => Sort::Name,
                            Sort::Name => Sort::Due,
                            Sort::Due => Sort::Newest,
                        };
                    }
                    _ => {}
                }
            }
        }
        Sec::Schedule => {
            if let Kind::DueDate {
                when: DueWhen::Before { unit, .. } | DueWhen::After { unit, .. },
                ..
            } = &mut s.kind
            {
                *unit = if *unit == Unit::Days {
                    Unit::Hours
                } else {
                    Unit::Days
                };
            }
        }
        _ => {}
    }
}

/// Numa condição a palavra que alterna é o «with/without» (o 1.º clique) ou o valor da janela.
fn toggle_test(c: &mut Condition) {
    match &mut c.test {
        Test::Due(d) => {
            *d = match d {
                DueIn::NoDates => DueIn::Overdue,
                DueIn::Overdue => DueIn::NextDay,
                DueIn::NextDay => DueIn::NextWeek,
                DueIn::NextWeek => DueIn::NextMonth,
                DueIn::NextMonth => DueIn::NoDates,
            };
        }
        Test::Field { is, .. } if *is == FieldTest::Set => *is = FieldTest::Empty,
        Test::Field { is, .. } if *is == FieldTest::Empty => *is = FieldTest::Set,
        _ => {}
    }
}

/// A palavra `n` (o «if/unless» é sempre a 1.ª peça da condição).
pub(crate) fn toggle_word(s: &mut Spec, sec: Sec, n: u8) {
    if let (Sec::Cond(i), 0) = (sec, n) {
        if let Some(c) = conditions_mut(s).and_then(|c| c.get_mut(usize::from(i))) {
            c.not = !c.not;
        }
        return;
    }
    toggle(s, sec);
}
