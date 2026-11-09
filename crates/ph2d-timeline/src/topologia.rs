//! ⭐ **A topologia de uma forma animada: o alinhamento de hoje com ontem** — puro, parametrizado por
//! (família, campos de âncora); dois chamadores: o vetor (`ph2d-app-vec::canais_topologia`, 8 campos por
//! vértice) e os pontos do Outline (`ph2d-physics-outline::canais_pontos`, 2). Lei do dono (19–20/10):
//! acrescentar/apagar ponto vale em TODAS as chaves. Vive aqui porque ambos já dependem desta crate.

use std::collections::BTreeMap;

use ph2d_ecs::FieldKey;

/// Os contornos de uma leitura: `i` → os pontos em ordem de `j`, cada um com os `N` campos
/// `primeiro ..= primeiro + N - 1` da `familia` (o que falta fica `0`).
pub fn contornos<const N: usize>(
    lidos: impl IntoIterator<Item = (FieldKey, f32)>,
    familia: u16,
    primeiro: u16,
) -> BTreeMap<u16, Vec<[f32; N]>> {
    let mut out: BTreeMap<u16, BTreeMap<u16, [f32; N]>> = BTreeMap::new();
    for (kk, v) in lidos {
        let Some(c) = kk.campo.checked_sub(primeiro).map(usize::from) else {
            continue;
        };
        if kk.familia == familia && c < N {
            out.entry(kk.i).or_default().entry(kk.j).or_insert([0.0; N])[c] = v;
        }
    }
    out.into_iter()
        .map(|(i, vs)| (i, vs.into_values().collect()))
        .collect()
}

/// A maior subsequência comum de âncoras EXACTAS (os campos `0` e `1`): os pares `(índice de ontem,
/// índice de hoje)`.
#[must_use]
pub fn alinhar<const N: usize>(antes: &[[f32; N]], agora: &[[f32; N]]) -> Vec<(usize, usize)> {
    let (n, m) = (antes.len(), agora.len());
    let igual = |a: usize, b: usize| antes[a][0] == agora[b][0] && antes[a][1] == agora[b][1];
    let mut t = vec![vec![0u32; m + 1]; n + 1];
    for a in (0..n).rev() {
        for b in (0..m).rev() {
            t[a][b] = if igual(a, b) {
                t[a + 1][b + 1] + 1
            } else {
                t[a + 1][b].max(t[a][b + 1])
            };
        }
    }
    let (mut a, mut b, mut pares) = (0, 0, Vec::new());
    while a < n && b < m {
        if igual(a, b) {
            pares.push((a, b));
            a += 1;
            b += 1;
        } else if t[a + 1][b] >= t[a][b + 1] {
            a += 1;
        } else {
            b += 1;
        }
    }
    pares
}
