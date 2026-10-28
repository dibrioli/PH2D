//! **Envelhecimento** (o Power-Up «Card Aging» do Trello): o cartão parado desbota (modo regular)
//! ou amarelece e rasga (modo pirata, [`ph2d_kanban_model::Aging`]). O nível é o tempo desde a
//! última atividade do cartão.
//!
//! Os limiares não estão na documentação da Atlassian (só «com a inatividade»); usam-se 1, 2 e 4
//! semanas — as mesmas fronteiras do filtro «Atividade» do Trello, para o cartão que desbota ser
//! o mesmo que o filtro chama «sem atividade».

use std::collections::BTreeMap;

use ph2d_kanban_model::{Card, CardId, Kanban, Moment, Subject};

/// Dias sem atividade para os níveis 1, 2 e 3.
pub const AGE_DAYS: [i64; 3] = [7, 14, 28];

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

/// A última atividade do cartão: nascer, cada entrada da atividade, cada comentário (e edição) e
/// cada anexo — o `dateLastActivity` do Trello.
#[must_use]
pub fn last_activity(card: &Card, k: &Kanban) -> Moment {
    let acts = k
        .activities
        .values()
        .filter(|a| a.subject == Subject::Card(card.id) && !a.meta.deleted)
        .map(|a| a.at);
    let comments = k
        .comments
        .values()
        .filter(|c| c.card == card.id && !c.meta.deleted)
        .map(|c| c.edited.unwrap_or(c.at).max(c.at));
    let atts = k
        .attachments
        .values()
        .filter(|a| a.card == card.id && !a.meta.deleted)
        .map(|a| a.at);
    acts.chain(comments)
        .chain(atts)
        .fold(card.created, Moment::max)
}

/// [`last_activity`] de todos os cartões de `k` numa passagem (para contar um Kanban inteiro).
#[must_use]
pub fn last_activity_index(k: &Kanban) -> BTreeMap<CardId, Moment> {
    let mut m: BTreeMap<CardId, Moment> = k.cards.values().map(|c| (c.id, c.created)).collect();
    let mut bump = |id: CardId, at: Moment| {
        if let Some(t) = m.get_mut(&id) {
            *t = (*t).max(at);
        }
    };
    for a in k.activities.values().filter(|a| !a.meta.deleted) {
        if let Subject::Card(id) = a.subject {
            bump(id, a.at);
        }
    }
    for c in k.comments.values().filter(|c| !c.meta.deleted) {
        bump(c.card, c.edited.unwrap_or(c.at).max(c.at));
    }
    for a in k.attachments.values().filter(|a| !a.meta.deleted) {
        bump(a.card, a.at);
    }
    m
}

/// O nível de envelhecimento `0..=3` do cartão em `now` (0 = fresco).
#[must_use]
pub fn age_level(card: &Card, k: &Kanban, now: Moment) -> u8 {
    level(last_activity(card, k), now)
}

pub(crate) fn level(last: Moment, now: Moment) -> u8 {
    let idle = now.0.saturating_sub(last.0);
    AGE_DAYS.iter().filter(|d| idle >= *d * DAY_MS).count() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn age_level_climbs_at_one_two_and_four_weeks() {
        let d = |n: i64| Moment(n * DAY_MS);
        let cases = [
            (0, 0),
            (6, 0),
            (7, 1),
            (13, 1),
            (14, 2),
            (27, 2),
            (28, 3),
            (400, 3),
        ];
        for (days, want) in cases {
            assert_eq!(level(d(10), d(10 + days)), want, "{days} dias");
        }
        assert_eq!(level(d(10), d(5)), 0, "um relógio para trás não envelhece");
    }
}
