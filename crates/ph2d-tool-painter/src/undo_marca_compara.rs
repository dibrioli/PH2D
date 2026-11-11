//! **O que mudou entre dois retratos** — filho de [`super`]: a comparação por texel que a lei da W9 usa
//! quando o passo não traz a força (B52.R4, movido à letra de `undo_marca.rs`).

use super::ModelSnapshot;
use super::marca;
use crate::compositor::Region;
use crate::layers::LayerId as RtLayerId;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Os píxeis RGBA da camada `id` de `s` (a ativa vive na tela).
fn rgba(s: &ModelSnapshot, id: RtLayerId) -> Option<&[u8]> {
    if s.layers.active() == Some(id) {
        Some(s.canvas_rgba.as_slice())
    } else {
        s.images.get(&id).map(|i| i.rgba8.as_slice())
    }
}

/// Liga em `out` os texels onde os planos `a` e `b` (de `k` elementos por texel, `n` texels) diferem,
/// dentro das linhas `linhas` e das colunas `colunas` (de largura `w`). Um plano só de um lado: os
/// texels dele que não são o de fábrica.
#[expect(clippy::too_many_arguments, reason = "a janela e a forma do plano")]
fn compara<T: PartialEq + Default>(
    a: Option<&[T]>,
    b: Option<&[T]>,
    k: usize,
    w: usize,
    n: usize,
    linhas: std::ops::Range<usize>,
    colunas: std::ops::Range<usize>,
    out: &mut [u64],
) {
    let zero = T::default();
    let difere = |i: usize| -> bool {
        match (a, b) {
            (Some(a), Some(b)) => a[i * k..(i + 1) * k] != b[i * k..(i + 1) * k],
            (Some(x), None) | (None, Some(x)) => x[i * k..(i + 1) * k].iter().any(|v| *v != zero),
            (None, None) => false,
        }
    };
    match (a, b) {
        (Some(a), Some(b)) if std::ptr::eq(a.as_ptr(), b.as_ptr()) && a.len() == b.len() => return,
        (Some(a), Some(b)) if a.len() != n * k || b.len() != n * k => return,
        (Some(x), None) | (None, Some(x)) if x.len() != n * k => return,
        _ => {}
    }
    for y in linhas {
        for x in colunas.clone() {
            let i = y * w + x;
            if difere(i) {
                marca::liga(out, i);
            }
        }
    }
}

/// Um plano preguiçoso por camada (`None` = nunca escrito).
fn de<T>(m: &BTreeMap<RtLayerId, Arc<Vec<T>>>, id: RtLayerId) -> Option<&[T]> {
    m.get(&id).map(|v| v.as_slice())
}

/// Liga em `out` os texels onde algum plano de alguma camada difere de `b` para `a`; a tela da camada
/// ativa dos dois lados só dentro de `hint` (a janela que as escritas declararam desde o último registo).
pub(super) fn texels_mudados(
    b: &ModelSnapshot,
    a: &ModelSnapshot,
    hint: Option<Region>,
    out: &mut [u64],
) {
    let (w, h) = (a.canvas_size.0 as usize, a.canvas_size.1 as usize);
    let n = w * h;
    let mut ids: BTreeSet<RtLayerId> = b.images.keys().chain(a.images.keys()).copied().collect();
    ids.extend(b.layers.active());
    ids.extend(a.layers.active());
    let tudo = (0..h, 0..w);
    for &id in &ids {
        let mesma_tela = b.layers.active() == Some(id) && a.layers.active() == Some(id);
        let (linhas, colunas) = match hint.filter(|_| mesma_tela) {
            Some(r) => (
                r.y as usize..((r.y + r.h) as usize).min(h),
                r.x as usize..((r.x + r.w) as usize).min(w),
            ),
            None => tudo.clone(),
        };
        compara(rgba(b, id), rgba(a, id), 4, w, n, linhas, colunas, out);
    }
    // O relevo descrito sem ser segurado não testemunha (`elide`).
    if b.relief_elided.is_empty() && a.relief_elided.is_empty() {
        let ids: BTreeSet<RtLayerId> = [&b.heights, &a.heights]
            .into_iter()
            .flat_map(BTreeMap::keys)
            .chain(b.covers.keys().chain(a.covers.keys()))
            .chain(b.mats.keys().chain(a.mats.keys()))
            .copied()
            .collect();
        for id in ids {
            let (l, c) = tudo.clone();
            compara(de(&b.heights, id), de(&a.heights, id), 1, w, n, l, c, out);
            let (l, c) = tudo.clone();
            compara(de(&b.covers, id), de(&a.covers, id), 1, w, n, l, c, out);
            let (l, c) = tudo.clone();
            compara(de(&b.mats, id), de(&a.mats, id), 1, w, n, l, c, out);
        }
    }
}
