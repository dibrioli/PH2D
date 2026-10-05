//! ⭐⭐ **A MALHA FINA do recorte** (A10) — quem decide «tapado» era a malha do campo posada em
//! triângulos RECTOS, e o desenho segue a pele EXACTA: o ponto posado afasta-se do triângulo recto
//! até `0,19` aresta sem dobra e `0,6`–`2,1` arestas na dobra (F56), e na ponta do vinco a ponta do
//! traço passava `0,4`–`3` larguras. Cada triângulo do campo parte-se em `d × d` subtriângulos cujos
//! vértices são posados pela pele exacta, com `d` dobrado até o meio de cada aresta ficar a menos de
//! `tol` da recta (o desvio cai `4×` por dobra). Irmão de [`super`] pelo tecto de LOC.

use super::malha::Grelha;

/// Subdivisões por aresta no máximo — tecto de RECURSO (memória e tempo por quadro), não de
/// qualidade: ver a fila §F60 para o preço medido.
pub(super) const MAX_DIV: usize = 16;

/// A malha fina posada: cada subtriângulo sabe o triângulo do campo de que nasceu (`pai`), que dá a
/// chave de osso e a vizinhança.
pub(super) struct Fina {
    pub(super) pos: Vec<[f64; 2]>,
    pub(super) repouso: Vec<[f64; 2]>,
    pub(super) tris: Vec<[u32; 3]>,
    pub(super) pai: Vec<u32>,
    pub(super) grelha: Grelha,
}

/// O índice do vértice `(i, j)` (`i + j ≤ d`) da grelha triangular de lado `d`.
const fn ix(d: usize, i: usize, j: usize) -> usize {
    // As linhas antes de `j` têm `d + 1 - j'` vértices cada.
    j * (d + 1) - j * j.saturating_sub(1) / 2 + i
}

/// A grelha posada de lado `d` do triângulo `k`: `(repouso, posado)` de cada vértice.
fn grelha_de(
    d: usize,
    k: usize,
    posa: &impl Fn(usize, f64, f64) -> ([f64; 2], [f64; 2]),
) -> Vec<([f64; 2], [f64; 2])> {
    let mut g = Vec::with_capacity((d + 1) * (d + 2) / 2);
    for j in 0..=d {
        for i in 0..=d - j {
            #[expect(clippy::cast_precision_loss, reason = "subdivisão pequena")]
            let (u, v) = (i as f64 / d as f64, j as f64 / d as f64);
            g.push(posa(k, u, v));
        }
    }
    g
}

/// O maior desvio entre a grelha `2d` e a recta da grelha `d` (os vértices ímpares da `2d` são os
/// meios das arestas da `d`).
fn desvio(d: usize, fina: &[([f64; 2], [f64; 2])], grossa: &[([f64; 2], [f64; 2])]) -> f64 {
    let e = 2 * d;
    let mut m = 0.0_f64;
    for j in 0..=e {
        for i in 0..=e - j {
            let (a, b) = match (i % 2, j % 2) {
                (0, 0) => continue,
                (1, 0) => (((i - 1) / 2, j / 2), ((i + 1) / 2, j / 2)),
                (0, 1) => ((i / 2, (j - 1) / 2), (i / 2, (j + 1) / 2)),
                _ => (((i - 1) / 2, (j + 1) / 2), ((i + 1) / 2, (j - 1) / 2)),
            };
            let (pa, pb) = (grossa[ix(d, a.0, a.1)].1, grossa[ix(d, b.0, b.1)].1);
            let p = fina[ix(e, i, j)].1;
            m = m.max((p[0] - 0.5 * (pa[0] + pb[0])).hypot(p[1] - 0.5 * (pa[1] + pb[1])));
        }
    }
    m
}

impl Fina {
    /// A malha fina de `n` triângulos: `posa(k, u, v)` dá o repouso e o ponto posado no baricentro
    /// `(u, v)` do triângulo `k`. `fixo` força o lado de todos (gates e sondas); senão o lado dobra
    /// até o desvio ser `≤ tol` (com `tol` não finito ou `≤ 0`, lado `1`: a malha do campo).
    pub(super) fn nova(
        n: usize,
        tol: f64,
        fixo: Option<usize>,
        posa: impl Fn(usize, f64, f64) -> ([f64; 2], [f64; 2]),
    ) -> Self {
        let mut pos = Vec::new();
        let mut repouso = Vec::new();
        let mut tris = Vec::new();
        let mut pai = Vec::new();
        for k in 0..n {
            let (d, g) = if let Some(d) = fixo {
                (d.max(1), grelha_de(d.max(1), k, &posa))
            } else if !(tol > 0.0 && tol.is_finite()) {
                (1, grelha_de(1, k, &posa))
            } else {
                let mut d = 1;
                let mut g = grelha_de(1, k, &posa);
                loop {
                    let g2 = grelha_de(2 * d, k, &posa);
                    let ok = desvio(d, &g2, &g) <= tol;
                    (d, g) = (2 * d, g2);
                    if ok || d >= MAX_DIV {
                        break (d, g);
                    }
                }
            };
            #[expect(clippy::cast_possible_truncation, reason = "índice de vértice u32")]
            let base = pos.len() as u32;
            for (r, p) in g {
                repouso.push(r);
                pos.push(p);
            }
            #[expect(clippy::cast_possible_truncation, reason = "índice de vértice u32")]
            let v = |i: usize, j: usize| base + ix(d, i, j) as u32;
            #[expect(clippy::cast_possible_truncation, reason = "índice de triângulo u32")]
            let ku = k as u32;
            for j in 0..d {
                for i in 0..d - j {
                    tris.push([v(i, j), v(i + 1, j), v(i, j + 1)]);
                    pai.push(ku);
                    if i + j + 1 < d {
                        tris.push([v(i + 1, j), v(i + 1, j + 1), v(i, j + 1)]);
                        pai.push(ku);
                    }
                }
            }
        }
        let grelha = Grelha::nova(&pos, &tris);
        Self {
            pos,
            repouso,
            tris,
            pai,
            grelha,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ix;

    #[test]
    fn o_indice_da_grelha_triangular_e_a_ordem_de_construcao() {
        for d in 1..=8 {
            let mut n = 0;
            for j in 0..=d {
                for i in 0..=d - j {
                    assert_eq!(ix(d, i, j), n, "d={d} i={i} j={j}");
                    n += 1;
                }
            }
        }
    }
}
