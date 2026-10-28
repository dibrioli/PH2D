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
    // As linhas que mudam vêm em CORRIDAS entre duas que ficam; cada corrida reparte o vão dela.
    let mut out = Vec::new();
    let mut lo: Option<&FracKey> = None;
    let mut run: Vec<usize> = Vec::new();
    for &i in &want {
        if keep[i] {
            let hi = &rows[i].1;
            let keys = spread(lo, Some(hi), run.len());
            out.extend(run.drain(..).zip(keys).map(|(j, p)| (rows[j].0, p)));
            lo = Some(hi);
        } else {
            run.push(i);
        }
    }
    let n = run.len();
    out.extend(
        run.into_iter()
            .zip(spread(lo, None, n))
            .map(|(j, p)| (rows[j].0, p)),
    );
    out
}

/// `n` chaves por ordem entre `lo` e `hi`. Entre duas, por BISSECÇÃO: o comprimento cresce com
/// log(n); a fila «logo acima da anterior» cresceria um carácter a cada ~6 (milhares a 20 000).
fn spread(lo: Option<&FracKey>, hi: Option<&FracKey>, n: usize) -> Vec<FracKey> {
    let mut v = Vec::with_capacity(n);
    match (lo, hi) {
        (Some(a), Some(b)) => {
            if n > 0 {
                let mid = FracKey::between(Some(a), Some(b));
                v = spread(Some(a), Some(&mid), n / 2);
                v.push(mid.clone());
                v.extend(spread(Some(&mid), Some(b), n - n / 2 - 1));
            }
        }
        (None, Some(b)) => {
            let mut at = b.clone();
            for _ in 0..n {
                at = FracKey::between(None, Some(&at));
                v.push(at.clone());
            }
            v.reverse();
        }
        (lo, None) => {
            let mut at = lo.cloned();
            for _ in 0..n {
                let k = FracKey::between(at.as_ref(), None);
                v.push(k.clone());
                at = Some(k);
            }
        }
    }
    v
}

/// Uma árvore de Fenwick do MÁXIMO por prefixo de `rank`.
struct MaxTree(Vec<usize>);

impl MaxTree {
    /// O maior valor posto num rank `< r`.
    fn below(&self, r: usize) -> usize {
        let (mut p, mut best) = (r, 0);
        while p > 0 {
            best = best.max(self.0[p]);
            p &= p - 1;
        }
        best
    }

    fn put(&mut self, r: usize, v: usize) {
        let mut p = r + 1;
        while p < self.0.len() {
            self.0[p] = self.0[p].max(v);
            p += p.isolate_lowest_one();
        }
    }
}

/// As linhas com a mesma cadeia de comprimento `L`, por ordem de índice, em BLOCOS de posição igual.
/// De um bloco para o seguinte os ranks descem (senão a de trás teria cadeia `L + 1`).
#[derive(Default)]
struct Level {
    idx: Vec<usize>,
    /// O menor rank desde o começo do bloco até cada linha.
    min: Vec<usize>,
    /// `(começo em idx, posição)` de cada bloco.
    blocks: Vec<(usize, usize)>,
}

impl Level {
    fn push(&mut self, i: usize, rank: usize, group: usize) {
        let carry = match self.blocks.last() {
            Some(&(_, g)) if g == group => self.min.last().copied(),
            _ => {
                self.blocks.push((self.idx.len(), group));
                None
            }
        };
        self.idx.push(i);
        self.min.push(carry.map_or(rank, |m| m.min(rank)));
    }

    /// A linha de MENOR índice com rank `< r` (a que a versão O(n²) escolhia).
    fn first_below(&self, r: usize) -> Option<usize> {
        let end = |b: usize| self.blocks.get(b + 1).map_or(self.idx.len(), |x| x.0);
        // Os mínimos dos blocos descem: os primeiros blocos não têm rank `< r`.
        let (mut b, mut top) = (0, self.blocks.len());
        while b < top {
            let m = (b + top) / 2;
            if self.min[end(m) - 1] >= r {
                b = m + 1;
            } else {
                top = m;
            }
        }
        let (start, end) = (self.blocks.get(b)?.0, end(b));
        let at = start + self.min[start..end].partition_point(|&m| m >= r);
        self.idx.get(at).copied()
    }
}

/// Quais linhas ficam: a maior subsequência (na ordem de agora) com `rank` e posição a subir; no
/// empate de comprimento, a de menor índice (início e antecessores). `pos` vem por ordem (a de
/// `entries_in`), `rank` é uma permutação. O(n log n): Fenwick no rank, posições iguais em lote.
fn longest_kept(pos: &[&FracKey], rank: &[usize]) -> Vec<bool> {
    let n = rank.len();
    debug_assert!(
        pos.windows(2).all(|w| w[0] <= w[1]),
        "as linhas vêm por posição"
    );
    let mut len = vec![0_usize; n];
    let mut prev = vec![usize::MAX; n];
    let mut tree = MaxTree(vec![0; n + 1]);
    let mut levels: Vec<Level> = Vec::new();
    let mut g = 0;
    while g < n {
        let end = g + pos[g..].iter().take_while(|&&p| p == pos[g]).count();
        for i in g..end {
            let best = tree.below(rank[i]);
            len[i] = best + 1;
            if best > 0 {
                prev[i] = levels[best - 1].first_below(rank[i]).unwrap_or(usize::MAX);
            }
        }
        for i in g..end {
            tree.put(rank[i], len[i]);
            if levels.len() < len[i] {
                levels.push(Level::default());
            }
            levels[len[i] - 1].push(i, rank[i], g);
        }
        g = end;
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
