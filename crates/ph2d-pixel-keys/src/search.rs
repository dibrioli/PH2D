//! A busca aproximada do Run Command (Aseprite O10: subsequência, `flp` acha Flip).

use crate::PixelAction;

/// O casamento de `query` (sem espaços, sem caixa) como subsequência de `name`; `None` = não casa.
/// Mais alto = melhor: letra no início de palavra e letras seguidas valem mais, saltos custam.
#[must_use]
pub fn score(query: &str, name: &str) -> Option<i32> {
    let q: Vec<char> = query
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect();
    if q.is_empty() {
        return Some(0);
    }
    let n: Vec<char> = name.chars().flat_map(char::to_lowercase).collect();
    let mut s = 0;
    let mut qi = 0;
    let mut last: Option<usize> = None;
    for (i, &ch) in n.iter().enumerate() {
        if qi == q.len() {
            break;
        }
        if ch != q[qi] {
            continue;
        }
        let word_start = i == 0 || !n[i - 1].is_alphanumeric();
        s += 10;
        if word_start {
            s += 8;
        }
        match last {
            Some(l) if l + 1 == i => s += 6,
            Some(l) => s -= i32::try_from((i - l - 1).min(20)).unwrap_or(20),
            None => s -= i32::try_from(i.min(20)).unwrap_or(20),
        }
        last = Some(i);
        qi += 1;
    }
    (qi == q.len()).then_some(s)
}

/// As acções que casam, melhor primeiro; empates pela secção e pela ordem do diálogo.
#[must_use]
pub fn search(query: &str) -> Vec<(PixelAction, i32)> {
    let mut v: Vec<(usize, PixelAction, i32)> = PixelAction::ALL
        .iter()
        .enumerate()
        .filter_map(|(i, &a)| score(query, a.name()).map(|s| (i, a, s)))
        .collect();
    v.sort_by(|a, b| {
        b.2.cmp(&a.2)
            .then(a.1.section().cmp(&b.1.section()))
            .then(a.0.cmp(&b.0))
    });
    v.into_iter().map(|(_, a, s)| (a, s)).collect()
}
