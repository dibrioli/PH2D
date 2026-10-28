//! **O filtro do quadro** (o painel «Filtro» do Trello, `F`; a maquete `Filter.dc.html`).
//!
//! As leis, da ajuda do Trello («Filtering for cards on a board»):
//! - A **palavra-chave** procura no título e nos campos personalizados do cartão; a maquete (e o
//!   texto do Trello, «Search cards, members, labels and more») junta os nomes das etiquetas e dos
//!   membros. Palavras soltas casam QUALQUER uma; entre aspas, a frase. A descrição e os comentários
//!   são da pesquisa global ([`crate::search`]), não do filtro. Com palavra-chave, ela é sempre
//!   exigida.
//! - Entre categorias (membros, estado, entrega, atividade, etiquetas) vale o **E**. Dentro de uma
//!   categoria, **qualquer correspondência** ([`MatchMode::Any`]) pede uma das opções marcadas e
//!   **correspondência exacta** ([`MatchMode::Exact`]) pede todas.
//! - Entregas: «atrasado» e as janelas só contam cartões NÃO concluídos; o dia são 24 h, a semana
//!   7 dias locais, o mês um mês do calendário local, sempre a contar de agora.

use std::collections::BTreeSet;

use ph2d_kanban_dates::{CivilDate, Zone, add_days, civil, instant};
use ph2d_kanban_model::{
    Card, Entry, FieldKind, FieldValue, Kanban, KanbanId, KanbanSet, LabelId, ListId, Member,
    Moment,
};

use crate::age::{last_activity, last_activity_index};
use crate::text::{needles, normalize};

/// Uma janela da data de entrega.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DueWindow {
    NoDates,
    Overdue,
    NextDay,
    NextWeek,
    NextMonth,
}

/// Uma janela da atividade.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ActivityWindow {
    LastWeek,
    LastTwoWeeks,
    LastFourWeeks,
    /// Sem atividade nas últimas 4 semanas.
    IdleFourWeeks,
}

/// Como as opções de uma categoria se combinam.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MatchMode {
    #[default]
    Any,
    Exact,
}

/// O filtro de UMA vista (nunca do documento).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Filter {
    pub keyword: String,
    pub members: BTreeSet<Member>,
    pub no_members: bool,
    /// «Cartões atribuídos a mim».
    pub mine: bool,
    /// «Marcado como concluído».
    pub complete: bool,
    /// «Não marcado como concluído».
    pub incomplete: bool,
    pub due: BTreeSet<DueWindow>,
    pub activity: BTreeSet<ActivityWindow>,
    pub labels: BTreeSet<LabelId>,
    pub no_labels: bool,
    pub mode: MatchMode,
}

impl Filter {
    /// Quantas escolhas estão feitas (o número no botão «Filtros»; a palavra-chave conta uma).
    #[must_use]
    pub fn selected(&self) -> usize {
        usize::from(!self.keyword.trim().is_empty())
            + self.members.len()
            + usize::from(self.no_members)
            + usize::from(self.mine)
            + usize::from(self.complete)
            + usize::from(self.incomplete)
            + self.due.len()
            + self.activity.len()
            + self.labels.len()
            + usize::from(self.no_labels)
    }

    /// Nada escolhido: tudo passa (o «Limpar tudo», `X`).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.selected() == 0
    }
}

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

/// A mesma hora local um mês depois (o dia preso ao fim do mês: 31/jan → 28 ou 29/fev).
fn month_later(now: i64, zone: &Zone) -> i64 {
    let c = civil(now, zone);
    let (y, m) = if c.date.month == 12 {
        (c.date.year + 1, 1)
    } else {
        (c.date.year, c.date.month + 1)
    };
    let date = (1..=c.date.day)
        .rev()
        .find_map(|d| CivilDate::new(y, m, d))
        .unwrap_or(c.date);
    instant(date, c.hour, c.minute, zone).unwrap_or(now + 31 * DAY_MS)
}

/// O cartão cai na janela `w` em `now`?
#[must_use]
pub fn due_in(w: DueWindow, card: &Card, now: Moment, zone: &Zone) -> bool {
    let Some(due) = card.due else {
        return w == DueWindow::NoDates;
    };
    if due.done {
        return false;
    }
    let (at, now) = (due.at.0, now.0);
    let horizon = match w {
        DueWindow::NoDates => return false,
        DueWindow::Overdue => return at < now,
        DueWindow::NextDay => now + DAY_MS,
        DueWindow::NextWeek => add_days(now, 7, zone),
        DueWindow::NextMonth => month_later(now, zone),
    };
    now <= at && at <= horizon
}

fn active_in(w: ActivityWindow, last: Moment, now: Moment) -> bool {
    let weeks = |n: i64| now.0 - n * 7 * DAY_MS;
    match w {
        ActivityWindow::LastWeek => last.0 >= weeks(1),
        ActivityWindow::LastTwoWeeks => last.0 >= weeks(2),
        ActivityWindow::LastFourWeeks => last.0 >= weeks(4),
        ActivityWindow::IdleFourWeeks => last.0 < weeks(4),
    }
}

/// Uma categoria: vazia passa; senão uma (Any) ou todas (Exact) as opções valem.
fn category(mode: MatchMode, opts: impl IntoIterator<Item = bool>) -> bool {
    let mut it = opts.into_iter().peekable();
    if it.peek().is_none() {
        return true;
    }
    match mode {
        MatchMode::Any => it.any(|b| b),
        MatchMode::Exact => it.all(|b| b),
    }
}

/// As etiquetas do cartão como `(nome, cor)` normalizado — para casar etiquetas de outro Kanban.
fn label_key(k: &Kanban, id: LabelId) -> Option<(String, String)> {
    let l = k.labels.get(&id).filter(|l| !l.meta.deleted)?;
    Some((normalize(&l.name), format!("{:?}", l.color)))
}

fn has_label(card: &Card, ck: &Kanban, view: &Kanban, want: LabelId) -> bool {
    if ck.id == view.id {
        return card.labels.contains(&want);
    }
    let Some(w) = label_key(view, want) else {
        return false;
    };
    card.labels
        .iter()
        .any(|l| label_key(ck, *l).as_ref() == Some(&w))
}

/// Os textos onde a palavra-chave procura, normalizados.
fn haystack(card: &Card, k: &Kanban) -> Vec<String> {
    let mut v = vec![normalize(&card.title)];
    for (f, val) in &card.fields {
        let Some(def) = k.fields.get(f).filter(|d| !d.meta.deleted) else {
            continue;
        };
        match (val, &def.kind) {
            (FieldValue::Text(t), _) => v.push(normalize(t)),
            (FieldValue::Number(n), _) => v.push(n.to_string()),
            (FieldValue::Option(o), FieldKind::Dropdown(opts)) => {
                if let Some(o) = opts.iter().find(|x| x.id == *o) {
                    v.push(normalize(&o.text));
                }
            }
            _ => {}
        }
    }
    v.extend(
        card.labels
            .iter()
            .filter_map(|l| label_key(k, *l))
            .map(|l| l.0),
    );
    v.extend(card.members.iter().map(|m| normalize(&m.0)));
    v
}

#[allow(clippy::too_many_arguments)]
fn matches_with(
    f: &Filter,
    card: &Card,
    ck: &Kanban,
    view: &Kanban,
    last: Moment,
    now: Moment,
    zone: &Zone,
    me: &Member,
) -> bool {
    let words = needles(&f.keyword);
    if !words.is_empty() {
        let hay = haystack(card, ck);
        if !words
            .iter()
            .any(|w| hay.iter().any(|h| h.contains(w.as_str())))
        {
            return false;
        }
    }
    let mode = f.mode;
    let members = f
        .members
        .iter()
        .map(|m| card.members.contains(m))
        .chain(f.no_members.then(|| card.members.is_empty()))
        .chain(f.mine.then(|| card.members.contains(me)));
    let done = card.due.is_some_and(|d| d.done);
    let status = f
        .complete
        .then_some(done)
        .into_iter()
        .chain(f.incomplete.then_some(!done));
    let due = f.due.iter().map(|w| due_in(*w, card, now, zone));
    let activity = f.activity.iter().map(|w| active_in(*w, last, now));
    let labels = f
        .labels
        .iter()
        .map(|l| has_label(card, ck, view, *l))
        .chain(f.no_labels.then(|| card.labels.is_empty()));
    category(mode, members)
        && category(mode, status)
        && category(mode, due)
        && category(mode, activity)
        && category(mode, labels)
}

/// O cartão `card` (do Kanban `k`) passa no filtro em `now`, no fuso `zone`, para a pessoa `me`?
#[must_use]
pub fn matches(f: &Filter, card: &Card, k: &Kanban, now: Moment, zone: &Zone, me: &Member) -> bool {
    let last = last_activity(card, k);
    matches_with(f, card, k, k, last, now, zone, me)
}

/// Quantos cartões à vista de `k` passam no filtro.
#[must_use]
pub fn count(f: &Filter, k: &Kanban, now: Moment, zone: &Zone, me: &Member) -> usize {
    let last = last_activity_index(k);
    k.cards
        .values()
        .filter(|c| !c.meta.deleted && !c.archived)
        .filter(|c| {
            let l = last.get(&c.id).copied().unwrap_or(c.created);
            matches_with(f, c, k, k, l, now, zone, me)
        })
        .count()
}

/// As linhas da lista que o filtro deixa ver (cartões e espelhos, pela ordem); um espelho passa
/// se o original passa — as etiquetas de outro Kanban casam pelo nome e cor.
#[must_use]
pub fn filter_entries(
    f: &Filter,
    set: &KanbanSet,
    kanban: KanbanId,
    list: ListId,
    now: Moment,
    zone: &Zone,
    me: &Member,
) -> Vec<Entry> {
    let Some(view) = set.get(kanban) else {
        return Vec::new();
    };
    set.entries_in(kanban, list)
        .into_iter()
        .filter(|e| {
            let r = e.shows(kanban);
            let (Some(ck), Some(card)) = (set.get(r.kanban), set.card(r)) else {
                return false;
            };
            let last = last_activity(card, ck);
            matches_with(f, card, ck, view, last, now, zone, me)
        })
        .collect()
}

#[cfg(test)]
#[path = "filter_tests.rs"]
mod tests;
