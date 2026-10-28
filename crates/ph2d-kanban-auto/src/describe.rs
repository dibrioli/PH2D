//! **A frase** de uma automação (o Butler escreve as regras como frases: «quando um cartão entra na
//! lista "Done", marcar a entrega como concluída e arquivar o cartão»). Sai em [`Part`]s: as
//! palavras são [`Word`]s com chave da i18n (`kanban.auto.<palavra>`), os nomes (lista, etiqueta,
//! membro…) são ids que a UI resolve no documento. Nenhum texto fixo aqui.

use ph2d_kanban_model::{ChecklistId, FieldId, FieldValue, KanbanId, LabelId, ListId, Member};

use crate::spec::{
    Action, Added, By, Condition, DateRule, Dest, DueIn, DueWhen, End, Every, FieldTest, Kind,
    Scope, Sort, Spec, Test, TimeOfDay, Trigger, Unit,
};

macro_rules! words {
    ($($w:ident),* $(,)?) => {
        /// Uma palavra (ou locução) da frase.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Word { $($w),* }

        impl Word {
            /// Todas, pela ordem (para o gate de que cada uma tem tradução).
            pub const ALL: &'static [Word] = &[$(Word::$w),*];

            /// A chave da i18n: `kanban.auto.` e o nome da palavra.
            #[must_use]
            pub fn key(self) -> &'static str {
                match self { $(Word::$w => concat!("kanban.auto.", stringify!($w))),* }
            }
        }
    };
}

words! {
    // Molduras.
    When, And, With, Without, By, Except, CardButton, BoardButton, ForCardsIn, AllLists,
    // Gatilhos.
    CardAdded, CardCreated, CardMovedIn, AnyList, MovedOut, LabelAdded, LabelRemoved, AnyLabel,
    MemberAdded, MemberRemoved, AnyMember, DueSet, DueRemoved, DueDone, DueUndone,
    ChecklistComplete, ItemChecked, ItemUnchecked, Commented, CommentContains, Archived, Unarchived,
    FieldChanged, AnyField,
    // Condições.
    HasLabel, HasAllLabels, NoLabels, HasMember, HasAllMembers, Mine, NoMembers, Complete, Due,
    DueNoDates, DueOverdue, DueNextDay, DueNextWeek, DueNextMonth, Text, InList, Field, FieldSet,
    FieldEmpty, FieldEquals, ChecklistsDone,
    // Acções.
    MoveTo, CopyTo, MirrorTo, AtTop, AtBottom, OtherKanban, Archive, Unarchive, AddLabel,
    RemoveLabel, ClearLabels, AddMember, RemoveMember, JoinMe, ClearMembers, SetDue, ClearDue,
    SetStart, ClearStart, MarkDone, MarkUndone, AddChecklist, CopyChecklist, CheckAll, UncheckAll,
    Comment, SetField, ClearField, SortList, SortNewest, SortOldest, SortName, SortDue, CreateCard,
    Now, InDays, AtTime,
    // Agenda.
    EveryDay, EveryWeekdays, EveryMonthDay, EveryYearDay, DueBefore, DueAfter, DueOnDay, Days,
    Hours,
}

/// Um pedaço da frase.
#[derive(Clone, Debug, PartialEq)]
pub enum Part {
    Word(Word),
    List(ListId),
    Kanban(KanbanId),
    Label(LabelId),
    Member(Member),
    Field(FieldId),
    Checklist(ChecklistId),
    /// Texto do artista (comentário, título, palavra-chave).
    Text(String),
    Number(i64),
    Time(TimeOfDay),
    /// Bit 0 = segunda … bit 6 = domingo.
    Weekdays(u8),
    /// Dia do mês (e o mês, num anual).
    Date {
        month: Option<u8>,
        day: u8,
    },
    Value(FieldValue),
}

use Part::Word as W;

/// A frase inteira: moldura e gatilho/agenda, condições, e as acções unidas por [`Word::And`].
#[must_use]
pub fn describe(s: &Spec) -> Vec<Part> {
    let mut p = Vec::new();
    match &s.kind {
        Kind::Rule {
            trigger,
            by,
            conditions,
        } => {
            p.push(W(Word::When));
            trigger_parts(trigger, &mut p);
            match by {
                By::Anyone => {}
                By::Only(m) => p.extend([W(Word::By), Part::Member(m.clone())]),
                By::Except(m) => p.extend([W(Word::Except), Part::Member(m.clone())]),
            }
            conditions_parts(conditions, &mut p);
        }
        Kind::CardButton => p.extend([W(Word::CardButton), Part::Text(s.name.clone())]),
        Kind::BoardButton { scope } => {
            p.extend([W(Word::BoardButton), Part::Text(s.name.clone())]);
            scope_parts(scope, &mut p);
        }
        Kind::Calendar { every, at, scope } => {
            match every {
                Every::Day => p.push(W(Word::EveryDay)),
                Every::Weekdays(m) => p.extend([W(Word::EveryWeekdays), Part::Weekdays(*m)]),
                Every::MonthDay(d) => p.extend([
                    W(Word::EveryMonthDay),
                    Part::Date {
                        month: None,
                        day: *d,
                    },
                ]),
                Every::YearDay { month, day } => p.extend([
                    W(Word::EveryYearDay),
                    Part::Date {
                        month: Some(*month),
                        day: *day,
                    },
                ]),
            }
            p.extend([W(Word::AtTime), Part::Time(*at)]);
            scope_parts(scope, &mut p);
        }
        Kind::DueDate { when, scope } => {
            match when {
                DueWhen::Before { n, unit } => due_parts(Word::DueBefore, *n, *unit, &mut p),
                DueWhen::After { n, unit } => due_parts(Word::DueAfter, *n, *unit, &mut p),
                DueWhen::OnDay { at } => {
                    p.extend([W(Word::DueOnDay), W(Word::AtTime), Part::Time(*at)]);
                }
            }
            scope_parts(scope, &mut p);
        }
    }
    for (i, a) in s.actions.iter().enumerate() {
        if i > 0 {
            p.push(W(Word::And));
        }
        action_parts(a, &mut p);
    }
    p
}

fn due_parts(w: Word, n: u32, unit: Unit, p: &mut Vec<Part>) {
    let u = match unit {
        Unit::Days => Word::Days,
        Unit::Hours => Word::Hours,
    };
    p.extend([W(w), Part::Number(i64::from(n)), W(u)]);
}

fn scope_parts(s: &Scope, p: &mut Vec<Part>) {
    p.push(W(Word::ForCardsIn));
    if s.lists.is_empty() {
        p.push(W(Word::AllLists));
    }
    p.extend(s.lists.iter().map(|l| Part::List(*l)));
    conditions_parts(&s.conditions, p);
}

fn any<T>(v: Option<T>, any: Word, f: impl FnOnce(T) -> Part) -> Part {
    v.map_or(W(any), f)
}

fn trigger_parts(t: &Trigger, p: &mut Vec<Part>) {
    match t {
        Trigger::CardAdded { list, how } => {
            p.push(W(match how {
                Added::Any => Word::CardAdded,
                Added::Created => Word::CardCreated,
                Added::MovedIn => Word::CardMovedIn,
            }));
            p.push(any(*list, Word::AnyList, Part::List));
        }
        Trigger::MovedOut { list } => {
            p.extend([W(Word::MovedOut), any(*list, Word::AnyList, Part::List)]);
        }
        Trigger::Label { label, added } => {
            let w = if *added {
                Word::LabelAdded
            } else {
                Word::LabelRemoved
            };
            p.extend([W(w), any(*label, Word::AnyLabel, Part::Label)]);
        }
        Trigger::Member { member, added } => {
            let w = if *added {
                Word::MemberAdded
            } else {
                Word::MemberRemoved
            };
            p.extend([W(w), any(member.clone(), Word::AnyMember, Part::Member)]);
        }
        Trigger::DueSet => p.push(W(Word::DueSet)),
        Trigger::DueRemoved => p.push(W(Word::DueRemoved)),
        Trigger::DueDone { done } => {
            p.push(W(if *done {
                Word::DueDone
            } else {
                Word::DueUndone
            }));
        }
        Trigger::ChecklistComplete => p.push(W(Word::ChecklistComplete)),
        Trigger::ItemChecked { done } => {
            p.push(W(if *done {
                Word::ItemChecked
            } else {
                Word::ItemUnchecked
            }));
        }
        Trigger::Commented { contains } => match contains {
            None => p.push(W(Word::Commented)),
            Some(t) => p.extend([W(Word::CommentContains), Part::Text(t.clone())]),
        },
        Trigger::Archived { archived } => {
            p.push(W(if *archived {
                Word::Archived
            } else {
                Word::Unarchived
            }));
        }
        Trigger::FieldChanged { field } => {
            p.extend([
                W(Word::FieldChanged),
                any(*field, Word::AnyField, Part::Field),
            ]);
        }
    }
}

fn conditions_parts(cs: &[Condition], p: &mut Vec<Part>) {
    for c in cs {
        p.push(W(if c.not { Word::Without } else { Word::With }));
        match &c.test {
            Test::Labels { labels, all } => {
                p.push(W(if *all {
                    Word::HasAllLabels
                } else {
                    Word::HasLabel
                }));
                p.extend(labels.iter().map(|l| Part::Label(*l)));
            }
            Test::NoLabels => p.push(W(Word::NoLabels)),
            Test::Members { members, all } => {
                p.push(W(if *all {
                    Word::HasAllMembers
                } else {
                    Word::HasMember
                }));
                p.extend(members.iter().cloned().map(Part::Member));
            }
            Test::Mine => p.push(W(Word::Mine)),
            Test::NoMembers => p.push(W(Word::NoMembers)),
            Test::Complete => p.push(W(Word::Complete)),
            Test::Due(d) => p.extend([
                W(Word::Due),
                W(match d {
                    DueIn::NoDates => Word::DueNoDates,
                    DueIn::Overdue => Word::DueOverdue,
                    DueIn::NextDay => Word::DueNextDay,
                    DueIn::NextWeek => Word::DueNextWeek,
                    DueIn::NextMonth => Word::DueNextMonth,
                }),
            ]),
            Test::Text(t) => p.extend([W(Word::Text), Part::Text(t.clone())]),
            Test::InList(ls) => {
                p.push(W(Word::InList));
                p.extend(ls.iter().map(|l| Part::List(*l)));
            }
            Test::Field { field, is } => {
                p.extend([W(Word::Field), Part::Field(*field)]);
                match is {
                    FieldTest::Set => p.push(W(Word::FieldSet)),
                    FieldTest::Empty => p.push(W(Word::FieldEmpty)),
                    FieldTest::Equals(v) => {
                        p.extend([W(Word::FieldEquals), Part::Value(v.clone())])
                    }
                }
            }
            Test::ChecklistsDone => p.push(W(Word::ChecklistsDone)),
        }
    }
}

fn dest_parts(w: Word, d: Dest, at: Option<End>, p: &mut Vec<Part>) {
    p.extend([W(w), Part::List(d.list)]);
    if let Some(k) = d.kanban {
        p.extend([W(Word::OtherKanban), Part::Kanban(k)]);
    }
    match at {
        Some(End::Top) => p.push(W(Word::AtTop)),
        Some(End::Bottom) => p.push(W(Word::AtBottom)),
        None => {}
    }
}

fn date_parts(w: Word, r: DateRule, p: &mut Vec<Part>) {
    p.push(W(w));
    if r.days == 0 && r.at.is_none() {
        p.push(W(Word::Now));
    } else {
        p.extend([W(Word::InDays), Part::Number(i64::from(r.days))]);
    }
    if let Some(t) = r.at {
        p.extend([W(Word::AtTime), Part::Time(t)]);
    }
}

fn action_parts(a: &Action, p: &mut Vec<Part>) {
    match a {
        Action::MoveTo { to, at } => dest_parts(Word::MoveTo, *to, Some(*at), p),
        Action::CopyTo { to, at } => dest_parts(Word::CopyTo, *to, Some(*at), p),
        Action::MirrorTo { to } => dest_parts(Word::MirrorTo, *to, None, p),
        Action::Archive => p.push(W(Word::Archive)),
        Action::Unarchive => p.push(W(Word::Unarchive)),
        Action::AddLabel(l) => p.extend([W(Word::AddLabel), Part::Label(*l)]),
        Action::RemoveLabel(l) => p.extend([W(Word::RemoveLabel), Part::Label(*l)]),
        Action::ClearLabels => p.push(W(Word::ClearLabels)),
        Action::AddMember(m) => p.extend([W(Word::AddMember), Part::Member(m.clone())]),
        Action::RemoveMember(m) => p.extend([W(Word::RemoveMember), Part::Member(m.clone())]),
        Action::JoinMe => p.push(W(Word::JoinMe)),
        Action::ClearMembers => p.push(W(Word::ClearMembers)),
        Action::SetDue(r) => date_parts(Word::SetDue, *r, p),
        Action::ClearDue => p.push(W(Word::ClearDue)),
        Action::SetStart(r) => date_parts(Word::SetStart, *r, p),
        Action::ClearStart => p.push(W(Word::ClearStart)),
        Action::MarkDue { done } => {
            p.push(W(if *done {
                Word::MarkDone
            } else {
                Word::MarkUndone
            }));
        }
        Action::AddChecklist { title, .. } => {
            p.extend([W(Word::AddChecklist), Part::Text(title.clone())]);
        }
        Action::CopyChecklist { from } => {
            p.extend([W(Word::CopyChecklist), Part::Checklist(*from)]);
        }
        Action::CheckAll { done } => {
            p.push(W(if *done {
                Word::CheckAll
            } else {
                Word::UncheckAll
            }));
        }
        Action::Comment(t) => p.extend([W(Word::Comment), Part::Text(t.clone())]),
        Action::SetField { field, value } => match value {
            Some(v) => p.extend([
                W(Word::SetField),
                Part::Field(*field),
                Part::Value(v.clone()),
            ]),
            None => p.extend([W(Word::ClearField), Part::Field(*field)]),
        },
        Action::SortList { list, by } => p.extend([
            W(Word::SortList),
            Part::List(*list),
            W(match by {
                Sort::Newest => Word::SortNewest,
                Sort::Oldest => Word::SortOldest,
                Sort::Name => Word::SortName,
                Sort::Due => Word::SortDue,
            }),
        ]),
        Action::CreateCard { to, title, at } => {
            p.extend([W(Word::CreateCard), Part::Text(title.clone())]);
            dest_parts(Word::InList, *to, Some(*at), p);
        }
    }
}
