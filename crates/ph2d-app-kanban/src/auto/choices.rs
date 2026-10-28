//! ⭐ **O que cada chip oferece e o que a escolha muda** — a cabeça de uma secção troca o TIPO
//! (gatilho, condição, acção, agenda; «Remove» tira a condição/acção), um valor troca-se pelo TIPO
//! da peça (uma lista vai para o campo-lista da secção, uma etiqueta para o campo-etiqueta…), e as
//! palavras que alternam rodam no lugar. Texto, número, hora e dia escrevem-se ([`Typed`]).

use super::sentence::{Seg, conditions_mut};
use super::{Sec, Slot};
use ph2d_kanban_auto::{
    Action, Added, Condition, DateRule, Dest, DueIn, DueWhen, End, Every, FieldTest, Kind, Part,
    Sort, Spec, Test, TimeOfDay, Trigger, Unit, Word, describe,
};
use ph2d_kanban_model::{FieldId, Kanban, LabelId, ListId, Member};

/// Uma escolha de um chip.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Pick {
    Trigger(Trigger),
    Test(Test),
    Action(Action),
    Every(Every),
    DueWhen(DueWhen),
    List(Option<ListId>),
    Label(Option<LabelId>),
    Member(Option<Member>),
    Field(Option<FieldId>),
    Weekdays(u8),
    Remove,
}

/// O que se escreve numa caixa (a peça que o chip mostra).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Typed {
    Text,
    Number,
    Time,
    Day,
}

/// O que a escolha mostra: a palavra da cabeça do modelo, ou o nome do valor.
pub(crate) enum Shown {
    Word(Word),
    Part(Part),
    /// «Remove» (tira a condição ou a acção).
    Remove,
}

/// Os valores por omissão de um modelo: a 1.ª lista, a 1.ª etiqueta, «eu».
struct Defaults {
    list: Option<ListId>,
    label: Option<LabelId>,
    me: Member,
}

impl Defaults {
    fn of(k: &Kanban, me: &Member) -> Self {
        Self {
            list: k.lists_in_order().first().map(|l| l.id),
            label: k.labels.values().find(|l| !l.meta.deleted).map(|l| l.id),
            me: me.clone(),
        }
    }
}

fn triggers(d: &Defaults) -> Vec<Trigger> {
    let mut v = vec![
        Trigger::CardAdded {
            list: d.list,
            how: Added::Any,
        },
        Trigger::CardAdded {
            list: d.list,
            how: Added::Created,
        },
        Trigger::CardAdded {
            list: d.list,
            how: Added::MovedIn,
        },
        Trigger::MovedOut { list: d.list },
    ];
    for added in [true, false] {
        v.push(Trigger::Label { label: None, added });
        v.push(Trigger::Member {
            member: None,
            added,
        });
    }
    v.extend([
        Trigger::DueSet,
        Trigger::DueRemoved,
        Trigger::DueDone { done: true },
        Trigger::DueDone { done: false },
        Trigger::ChecklistComplete,
        Trigger::ItemChecked { done: true },
        Trigger::ItemChecked { done: false },
        Trigger::Commented { contains: None },
        Trigger::Commented {
            contains: Some(String::new()),
        },
        Trigger::Archived { archived: true },
        Trigger::Archived { archived: false },
        Trigger::FieldChanged { field: None },
    ]);
    v
}

fn tests(d: &Defaults) -> Vec<Test> {
    let mut v = Vec::new();
    if let Some(l) = d.label {
        v.push(Test::Labels {
            labels: vec![l],
            all: false,
        });
    }
    v.extend([
        Test::NoLabels,
        Test::Members {
            members: vec![d.me.clone()],
            all: false,
        },
        Test::Mine,
        Test::NoMembers,
        Test::Complete,
        Test::Due(DueIn::Overdue),
        Test::Text(String::new()),
    ]);
    if let Some(l) = d.list {
        v.push(Test::InList(vec![l]));
    }
    v.push(Test::ChecklistsDone);
    v
}

fn actions(d: &Defaults) -> Vec<Action> {
    let mut v = Vec::new();
    if let Some(list) = d.list {
        let to = Dest { kanban: None, list };
        v.extend([
            Action::MoveTo {
                to,
                at: End::Bottom,
            },
            Action::CopyTo {
                to,
                at: End::Bottom,
            },
            Action::MirrorTo { to },
        ]);
    }
    v.extend([Action::Archive, Action::Unarchive]);
    if let Some(l) = d.label {
        v.extend([Action::AddLabel(l), Action::RemoveLabel(l)]);
    }
    v.extend([
        Action::ClearLabels,
        Action::AddMember(d.me.clone()),
        Action::RemoveMember(d.me.clone()),
        Action::JoinMe,
        Action::ClearMembers,
        Action::SetDue(DateRule { days: 1, at: None }),
        Action::ClearDue,
        Action::SetStart(DateRule { days: 0, at: None }),
        Action::ClearStart,
        Action::MarkDue { done: true },
        Action::MarkDue { done: false },
        Action::AddChecklist {
            title: String::new(),
            items: Vec::new(),
        },
        Action::CheckAll { done: true },
        Action::CheckAll { done: false },
        Action::Comment(String::new()),
    ]);
    if let Some(list) = d.list {
        v.push(Action::SortList {
            list,
            by: Sort::Name,
        });
        v.push(Action::CreateCard {
            to: Dest { kanban: None, list },
            title: String::new(),
            at: End::Bottom,
        });
    }
    v
}

const NINE: TimeOfDay = TimeOfDay { hour: 9, minute: 0 };

/// A palavra da cabeça de um modelo (o que a linha do menu diz).
fn head_word(spec: Spec) -> Option<Word> {
    let skip = describe(&spec).into_iter().skip(match spec.kind {
        Kind::Rule { .. } => 1,
        Kind::CardButton => 2,
        _ => 0,
    });
    skip.filter_map(|p| match p {
        Part::Word(w) if !matches!(w, Word::With | Word::Without) => Some(w),
        _ => None,
    })
    .next()
}

fn rule(trigger: Trigger, conditions: Vec<Condition>, me: &Member) -> Spec {
    Spec {
        name: String::new(),
        enabled: true,
        owner: me.clone(),
        kind: Kind::Rule {
            trigger,
            by: ph2d_kanban_auto::By::Anyone,
            conditions,
        },
        actions: Vec::new(),
    }
}

/// ⭐ As escolhas do chip `slot` em `s`: o que cada linha mostra e o que ela faz.
pub(crate) fn options(
    slot: Slot,
    s: &Spec,
    segs: &[Seg],
    k: &Kanban,
    people: &[Member],
    me: &Member,
) -> Vec<(Shown, Pick)> {
    let d = Defaults::of(k, me);
    let words = |v: Vec<(Option<Word>, Pick)>| {
        v.into_iter()
            .filter_map(|(w, p)| Some((Shown::Word(w?), p)))
            .collect::<Vec<_>>()
    };
    match slot {
        Slot::Head(Sec::Trigger) => words(
            triggers(&d)
                .into_iter()
                .map(|t| (head_word(rule(t.clone(), Vec::new(), me)), Pick::Trigger(t)))
                .collect(),
        ),
        Slot::Head(Sec::Cond(_)) => {
            let mut v = words(
                tests(&d)
                    .into_iter()
                    .map(|t| {
                        let c = vec![Condition::is(t.clone())];
                        let w = describe(&rule(Trigger::DueSet, c, me))
                            .get(3)
                            .and_then(word);
                        (w, Pick::Test(t))
                    })
                    .collect(),
            );
            v.push((Shown::Remove, Pick::Remove));
            v
        }
        Slot::Head(Sec::Action(_)) => {
            let mut v = words(
                actions(&d)
                    .into_iter()
                    .map(|a| {
                        let mut sp = rule(Trigger::DueSet, Vec::new(), me);
                        sp.kind = Kind::CardButton;
                        sp.actions = vec![a.clone()];
                        (head_word(sp), Pick::Action(a))
                    })
                    .collect(),
            );
            v.push((Shown::Remove, Pick::Remove));
            v
        }
        Slot::Head(Sec::Schedule) => match s.kind {
            Kind::Calendar { .. } => [
                (Word::EveryDay, Every::Day),
                (Word::EveryWeekdays, Every::Weekdays(0x1F)),
                (Word::EveryMonthDay, Every::MonthDay(1)),
                (Word::EveryYearDay, Every::YearDay { month: 1, day: 1 }),
            ]
            .into_iter()
            .map(|(w, e)| (Shown::Word(w), Pick::Every(e)))
            .collect(),
            _ => [
                (
                    Word::DueBefore,
                    DueWhen::Before {
                        n: 1,
                        unit: Unit::Days,
                    },
                ),
                (
                    Word::DueAfter,
                    DueWhen::After {
                        n: 1,
                        unit: Unit::Days,
                    },
                ),
                (Word::DueOnDay, DueWhen::OnDay { at: NINE }),
            ]
            .into_iter()
            .map(|(w, e)| (Shown::Word(w), Pick::DueWhen(e)))
            .collect(),
        },
        Slot::Head(_) | Slot::Word(..) => Vec::new(),
        Slot::Arg(sec, _) => {
            let part = segs
                .iter()
                .find(|g| g.slot == Some(slot))
                .map(|g| g.part.clone());
            let any = matches!(sec, Sec::Trigger | Sec::Scope);
            let mut v: Vec<(Shown, Pick)> = Vec::new();
            match part {
                Some(Part::List(_) | Part::Word(Word::AnyList | Word::AllLists)) => {
                    if any {
                        let w = if sec == Sec::Scope {
                            Word::AllLists
                        } else {
                            Word::AnyList
                        };
                        v.push((Shown::Word(w), Pick::List(None)));
                    }
                    v.extend(
                        k.lists_in_order()
                            .iter()
                            .map(|l| (Shown::Part(Part::List(l.id)), Pick::List(Some(l.id)))),
                    );
                }
                Some(Part::Label(_) | Part::Word(Word::AnyLabel)) => {
                    if any {
                        v.push((Shown::Word(Word::AnyLabel), Pick::Label(None)));
                    }
                    let mut ls: Vec<_> = k.labels.values().filter(|l| !l.meta.deleted).collect();
                    ls.sort_by_key(|l| (l.name.to_lowercase(), l.id));
                    v.extend(
                        ls.iter()
                            .map(|l| (Shown::Part(Part::Label(l.id)), Pick::Label(Some(l.id)))),
                    );
                }
                Some(Part::Member(_) | Part::Word(Word::AnyMember)) => {
                    if any {
                        v.push((Shown::Word(Word::AnyMember), Pick::Member(None)));
                    }
                    v.extend(people.iter().map(|m| {
                        (
                            Shown::Part(Part::Member(m.clone())),
                            Pick::Member(Some(m.clone())),
                        )
                    }));
                }
                Some(Part::Field(_) | Part::Word(Word::AnyField)) => {
                    if any {
                        v.push((Shown::Word(Word::AnyField), Pick::Field(None)));
                    }
                    v.extend(
                        k.fields
                            .values()
                            .filter(|f| !f.meta.deleted)
                            .map(|f| (Shown::Part(Part::Field(f.id)), Pick::Field(Some(f.id)))),
                    );
                }
                Some(Part::Weekdays(_)) => {
                    let masks = [0x1F, 0x60].into_iter().chain((0..7).map(|i| 1u8 << i));
                    v.extend(masks.map(|m| (Shown::Part(Part::Weekdays(m)), Pick::Weekdays(m))));
                }
                _ => {}
            }
            v
        }
    }
}

fn word(p: &Part) -> Option<Word> {
    match p {
        Part::Word(w) => Some(*w),
        _ => None,
    }
}

/// O que a peça do chip `slot` pede escrito (`None` = escolhe-se de uma lista).
pub(crate) fn typed(slot: Slot, segs: &[Seg]) -> Option<Typed> {
    let part = &segs.iter().find(|g| g.slot == Some(slot))?.part;
    Some(match part {
        Part::Text(_) => Typed::Text,
        Part::Number(_) | Part::Word(Word::Now) => Typed::Number,
        Part::Time(_) => Typed::Time,
        Part::Date { .. } => Typed::Day,
        _ => return None,
    })
}

/// O texto que a caixa abre (o valor de agora).
pub(crate) fn typed_now(slot: Slot, segs: &[Seg]) -> String {
    match segs.iter().find(|g| g.slot == Some(slot)).map(|g| &g.part) {
        Some(Part::Text(t)) => t.clone(),
        Some(Part::Number(n)) => n.to_string(),
        Some(Part::Time(t)) => crate::back::dates::time_text(t.hour, t.minute),
        Some(Part::Date {
            month: Some(m),
            day,
        }) => format!("{m}/{day}"),
        Some(Part::Date { month: None, day }) => day.to_string(),
        _ => String::new(),
    }
}

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

/// Uma condição nova (a 1.ª do menu) e uma acção nova (a 1.ª do menu).
pub(crate) fn new_condition(k: &Kanban, me: &Member) -> Condition {
    let d = Defaults::of(k, me);
    Condition::is(tests(&d).remove(0))
}

pub(crate) fn new_action(k: &Kanban, me: &Member) -> Action {
    let d = Defaults::of(k, me);
    actions(&d).remove(0)
}
