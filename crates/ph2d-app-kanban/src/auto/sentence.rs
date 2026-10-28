//! ⭐ **A frase em peças** — a frase do Butler (`ph2d_kanban_auto::describe`) partida em secções
//! (gatilho, cada condição, cada acção, a agenda, o alcance) e cada peça com o alvo que a muda: a
//! cabeça da secção escolhe o TIPO (outro gatilho, outra acção), um nome escolhe o valor (lista,
//! etiqueta, membro), um texto/número/hora escreve-se, e certas palavras alternam (topo/fundo).
//! A frase sai SEMPRE de `describe` — o construtor nunca escreve uma frase própria.

use super::{Sec, Slot};
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_auto::{Condition, Kind, Part, Spec, Word, describe};
use ph2d_kanban_model::{FieldValue, Kanban, KanbanSet};

/// Uma peça da frase e o alvo que a muda (`None` = só texto).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Seg {
    pub part: Part,
    pub slot: Option<Slot>,
}

/// As palavras de moldura (nunca chips).
fn frame(w: Word) -> bool {
    matches!(
        w,
        Word::When
            | Word::And
            | Word::ForCardsIn
            | Word::CardButton
            | Word::BoardButton
            | Word::By
            | Word::Except
            | Word::AtTime
            | Word::OtherKanban
    )
}

/// As palavras que alternam no lugar (o clique troca o valor).
pub(crate) fn toggles(w: Word) -> bool {
    matches!(
        w,
        Word::With
            | Word::Without
            | Word::AtTop
            | Word::AtBottom
            | Word::SortNewest
            | Word::SortOldest
            | Word::SortName
            | Word::SortDue
            | Word::Days
            | Word::Hours
            | Word::DueNoDates
            | Word::DueOverdue
            | Word::DueNextDay
            | Word::DueNextWeek
            | Word::DueNextMonth
            | Word::FieldSet
            | Word::FieldEmpty
    )
}

/// As condições da automação (as da regra ou as do alcance).
pub(crate) fn conditions(s: &Spec) -> &[Condition] {
    match &s.kind {
        Kind::Rule { conditions, .. } => conditions,
        Kind::BoardButton { scope }
        | Kind::Calendar { scope, .. }
        | Kind::DueDate { scope, .. } => &scope.conditions,
        Kind::CardButton => &[],
    }
}

pub(crate) fn conditions_mut(s: &mut Spec) -> Option<&mut Vec<Condition>> {
    match &mut s.kind {
        Kind::Rule { conditions, .. } => Some(conditions),
        Kind::BoardButton { scope }
        | Kind::Calendar { scope, .. }
        | Kind::DueDate { scope, .. } => Some(&mut scope.conditions),
        Kind::CardButton => None,
    }
}

/// As palavras que estão no lugar de um valor («any list», «every list», «to now»): escolhem-se
/// como o valor.
fn stands_for_a_value(w: Word) -> bool {
    matches!(
        w,
        Word::AnyList
            | Word::AnyLabel
            | Word::AnyMember
            | Word::AnyField
            | Word::AllLists
            | Word::Now
    )
}

/// Marca as peças de uma secção: a 1.ª palavra que não é moldura nem alterna é a cabeça.
fn mark(sec: Sec, parts: &[Part], out: &mut Vec<Seg>) {
    let mut head = false;
    for (n, p) in parts.iter().enumerate() {
        let n = u8::try_from(n).unwrap_or(u8::MAX);
        let slot = match p {
            Part::Word(w) if frame(*w) => None,
            Part::Word(w) if stands_for_a_value(*w) => Some(Slot::Arg(sec, n)),
            Part::Word(w) if toggles(*w) => Some(Slot::Word(sec, n)),
            Part::Word(_) if !head && !matches!(sec, Sec::Scope | Sec::Name) => {
                head = true;
                Some(Slot::Head(sec))
            }
            Part::Word(_) | Part::Kanban(_) | Part::Checklist(_) | Part::Value(_) => None,
            _ => Some(Slot::Arg(sec, n)),
        };
        out.push(Seg {
            part: p.clone(),
            slot,
        });
    }
}

fn plain(parts: impl IntoIterator<Item = Part>, out: &mut Vec<Seg>) {
    out.extend(parts.into_iter().map(|part| Seg { part, slot: None }));
}

/// ⭐ A frase de `s` em peças com alvos.
pub(crate) fn segs(s: &Spec) -> Vec<Seg> {
    let mut bare = s.clone();
    bare.actions.clear();
    if let Some(c) = conditions_mut(&mut bare) {
        c.clear();
    }
    let head = describe(&bare);
    let mut out = Vec::new();
    let split = head
        .iter()
        .position(|p| *p == Part::Word(Word::ForCardsIn))
        .unwrap_or(head.len());
    match &s.kind {
        Kind::Rule { .. } => {
            plain(head.first().cloned(), &mut out);
            mark(Sec::Trigger, head.get(1..).unwrap_or(&[]), &mut out);
        }
        Kind::CardButton | Kind::BoardButton { .. } => {
            plain(head.first().cloned(), &mut out);
            mark(Sec::Name, head.get(1..split).unwrap_or(&[]), &mut out);
            mark(Sec::Scope, &head[split..], &mut out);
        }
        Kind::Calendar { .. } | Kind::DueDate { .. } => {
            mark(Sec::Schedule, &head[..split], &mut out);
            mark(Sec::Scope, &head[split..], &mut out);
        }
    }
    for (i, c) in conditions(s).iter().enumerate() {
        let mut one = bare.clone();
        if let Some(v) = conditions_mut(&mut one) {
            v.push(c.clone());
        }
        let all = describe(&one);
        let sec = Sec::Cond(u16::try_from(i).unwrap_or(u16::MAX));
        mark(sec, all.get(head.len()..).unwrap_or(&[]), &mut out);
    }
    for (i, a) in s.actions.iter().enumerate() {
        if i > 0 {
            plain([Part::Word(Word::And)], &mut out);
        }
        let one = Spec {
            name: String::new(),
            enabled: true,
            owner: s.owner.clone(),
            kind: Kind::CardButton,
            actions: vec![a.clone()],
        };
        let all = describe(&one);
        let sec = Sec::Action(u16::try_from(i).unwrap_or(u16::MAX));
        mark(sec, all.get(2..).unwrap_or(&[]), &mut out);
    }
    out
}

/// O nome que o documento dá a cada id (a lista apagada lê-se «(deleted)»).
pub(crate) struct Names<'a> {
    pub k: &'a Kanban,
    pub set: &'a KanbanSet,
}

impl Names<'_> {
    /// O texto de uma peça.
    pub(crate) fn text(&self, p: &Part) -> String {
        let gone = || tr("kanban.auto.ui.deleted").to_owned();
        let k = self.k;
        match p {
            Part::Word(w) => tr(w.key()).to_owned(),
            Part::List(l) => k.lists.get(l).map_or_else(gone, |l| l.title.clone()),
            Part::Kanban(id) => self
                .set
                .get(*id)
                .map_or_else(gone, |k| k.header.name.clone()),
            Part::Label(l) => match k.labels.get(l) {
                Some(l) if !l.name.trim().is_empty() => l.name.clone(),
                Some(_) => tr("kanban.auto.ui.unnamed").to_owned(),
                None => gone(),
            },
            Part::Member(m) => m.0.clone(),
            Part::Field(f) => k.fields.get(f).map_or_else(gone, |f| f.name.clone()),
            Part::Checklist(c) => k.checklists.get(c).map_or_else(gone, |c| c.title.clone()),
            Part::Text(t) => tr_with("kanban.auto.ui.quoted", &[("t", t)]),
            Part::Number(n) => n.to_string(),
            Part::Time(t) => crate::back::dates::time_text(t.hour, t.minute),
            Part::Weekdays(m) => weekdays(*m),
            Part::Date { month: None, day } => tr_with("kanban.auto.ui.day_n", &[("n", day)]),
            Part::Date {
                month: Some(mo),
                day,
            } => tr_with(
                "kanban.back.day",
                &[
                    ("mon", &ph2d_kanban_views::datepick::month_name(*mo, false)),
                    ("d", day),
                ],
            ),
            Part::Value(v) => match v {
                FieldValue::Text(s) => s.clone(),
                FieldValue::Number(n) => n.to_string(),
                FieldValue::Date(d) => crate::back::dates::short(*d, *d),
                FieldValue::Checkbox(b) => tr(if *b {
                    "kanban.auto.ui.check"
                } else {
                    "kanban.auto.ui.uncheck"
                })
                .to_owned(),
                FieldValue::Option(o) => format!("#{}", o.0),
            },
        }
    }

    /// A frase inteira num texto (as sondas, a sugestão).
    pub(crate) fn line(&self, s: &Spec) -> String {
        describe(s)
            .iter()
            .map(|p| self.text(p))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Os dias de uma máscara (bit 0 = segunda): «Mon–Fri», «Sat and Sun» ou «Mo, We».
pub(crate) fn weekdays(m: u8) -> String {
    match m & 0x7F {
        0x1F => tr("kanban.auto.ui.weekdays.work").to_owned(),
        0x60 => tr("kanban.auto.ui.weekdays.weekend").to_owned(),
        m => (0..7u8)
            .filter(|i| m & (1 << i) != 0)
            .map(|i| tr(&format!("kanban.back.wd.{i}")).to_owned())
            .collect::<Vec<_>>()
            .join(", "),
    }
}
