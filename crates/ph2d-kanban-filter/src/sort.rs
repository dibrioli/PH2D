//! **Ordenar uma lista** (o «Ordenar por…» do menu da lista do Trello): data de criação (mais nova
//! ou mais antiga primeiro), nome, data de entrega. Cartões e espelhos ordenam-se juntos (o espelho
//! pelo original).
//!
//! Só se reescreve o MÍNIMO: fica parada a maior subsequência que já está na ordem pedida, e só as
//! outras linhas ganham posição nova — menos escritas, menos conflitos com quem edita ao lado.

use std::cmp::Ordering;

use ph2d_kanban_model::{Card, Entry, FracKey, KanbanId, KanbanOp, KanbanSet, ListId};

use crate::text::normalize;

/// O critério.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SortBy {
    /// Data de criação, a mais nova primeiro.
    Newest,
    /// Data de criação, a mais antiga primeiro.
    Oldest,
    /// Nome, alfabético (sem caixa nem acentos).
    Name,
    /// Data de entrega, a mais próxima primeiro; sem data no fim.
    Due,
}

fn cmp(by: SortBy, a: &Card, b: &Card) -> Ordering {
    match by {
        SortBy::Newest => b.created.cmp(&a.created),
        SortBy::Oldest => a.created.cmp(&b.created),
        SortBy::Name => normalize(&a.title)
            .cmp(&normalize(&b.title))
            .then_with(|| a.title.cmp(&b.title)),
        SortBy::Due => match (a.due, b.due) {
            (Some(x), Some(y)) => x.at.cmp(&y.at),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        },
    }
}

/// As posições novas das linhas que mudam (só essas), pela ordem pedida.
#[must_use]
pub fn sort_keys(
    set: &KanbanSet,
    kanban: KanbanId,
    list: ListId,
    by: SortBy,
) -> Vec<(Entry, FracKey)> {
    let Some(k) = set.get(kanban) else {
        return Vec::new();
    };
    let rows: Vec<(Entry, FracKey, &Card)> = set
        .entries_in(kanban, list)
        .into_iter()
        .filter_map(|e| Some((e, k.entry_pos(e)?.clone(), set.card(e.shows(kanban))?)))
        .collect();
    // A ordem pedida; o empate guarda a ordem de agora (estável).
    let mut want: Vec<usize> = (0..rows.len()).collect();
    want.sort_by(|&i, &j| cmp(by, rows[i].2, rows[j].2));
    let mut rank = vec![0; rows.len()];
    for (r, &i) in want.iter().enumerate() {
        rank[i] = r;
    }
    let keep = longest_kept(&rows.iter().map(|r| &r.1).collect::<Vec<_>>(), &rank);
    let mut out = Vec::new();
    let mut lo: Option<FracKey> = None;
    for (r, &i) in want.iter().enumerate() {
        if keep[i] {
            lo = Some(rows[i].1.clone());
            continue;
        }
        let hi = want[r + 1..]
            .iter()
            .find(|&&j| keep[j])
            .map(|&j| &rows[j].1);
        let pos = FracKey::between(lo.as_ref(), hi);
        lo = Some(pos.clone());
        out.push((rows[i].0, pos));
    }
    out
}

/// Quais linhas ficam: a maior subsequência (na ordem de agora) com `rank` e posição a subir.
fn longest_kept(pos: &[&FracKey], rank: &[usize]) -> Vec<bool> {
    let n = rank.len();
    let mut len = vec![1_usize; n];
    let mut prev = vec![usize::MAX; n];
    for i in 0..n {
        for j in 0..i {
            if rank[j] < rank[i] && pos[j] < pos[i] && len[j] + 1 > len[i] {
                len[i] = len[j] + 1;
                prev[i] = j;
            }
        }
    }
    let mut keep = vec![false; n];
    let mut at = (0..n).max_by_key(|&i| (len[i], std::cmp::Reverse(i)));
    while let Some(i) = at {
        keep[i] = true;
        at = (prev[i] != usize::MAX).then(|| prev[i]);
    }
    keep
}

/// O lote que ordena a lista (um passo de desfazer); vazio = já está ordenada.
#[must_use]
pub fn sort_list(set: &KanbanSet, kanban: KanbanId, list: ListId, by: SortBy) -> Vec<KanbanOp> {
    let Some(k) = set.get(kanban) else {
        return Vec::new();
    };
    sort_keys(set, kanban, list, by)
        .into_iter()
        .filter_map(|(e, pos)| k.entry_moved(e, list, pos))
        .collect()
}

#[cfg(test)]
#[path = "sort_tests.rs"]
mod tests;
