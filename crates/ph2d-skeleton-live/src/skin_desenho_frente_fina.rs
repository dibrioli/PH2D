//! ⭐⭐ **A MALHA FINA do recorte** (A10) — quem decide «tapado» era a malha do campo posada em
//! triângulos RECTOS, e o desenho segue a pele EXACTA: o ponto posado afasta-se do triângulo recto
//! até `0,19` aresta sem dobra e `0,6`–`2,1` arestas na dobra (F56), e na ponta do vinco a ponta do
//! traço passava `0,4`–`3` larguras. Cada triângulo do campo parte-se em `d × d` subtriângulos cujos
//! vértices são posados pela pele exacta, com `d` dobrado até o meio de cada aresta ficar a menos de
//! `tol` da recta (o desvio cai `4×` por dobra). Irmão de [`super`] pelo tecto de LOC.

use super::malha::Grelha;

#[cfg(test)]
#[path = "skin_desenho_frente_fina_tests.rs"]
mod tests;

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

/// A grelha posada de lado `d` do triângulo `k`: `(repouso, posado)` de cada vértice. Com a grelha
/// de lado `d / 2` em `antes`, os vértices dela (os pares) não se posam outra vez.
fn grelha_de(
    d: usize,
    k: usize,
    antes: Option<&[([f64; 2], [f64; 2])]>,
    posa: &mut impl FnMut(usize, f64, f64) -> ([f64; 2], [f64; 2]),
) -> Vec<([f64; 2], [f64; 2])> {
    let mut g = Vec::with_capacity((d + 1) * (d + 2) / 2);
    for j in 0..=d {
        for i in 0..=d - j {
            if let Some(a) = antes.filter(|_| i % 2 == 0 && j % 2 == 0) {
                g.push(a[ix(d / 2, i / 2, j / 2)]);
                continue;
            }
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

/// A malha do campo POSADA (recta), com a chave de osso de cada triângulo.
pub(super) struct Campo<'a> {
    pub(super) pos: &'a [[f64; 2]],
    pub(super) tris: &'a [[u32; 3]],
    pub(super) chave: &'a [f64],
}

/// ⭐ **Quem pode TAPAR alguém** — o triângulo de chave maior de um par sem vértice comum cujas
/// caixas, cada uma alargada pela sua `folga` (o quanto a pele exacta sai da recta), se tocam. Um
/// triângulo fora desta lista não tapa nada e não entra na malha fina. Varrimento em `x`.
fn cobridores(c: &Campo<'_>, folga: &[f64]) -> Vec<bool> {
    let caixa: Vec<[f64; 4]> = c
        .tris
        .iter()
        .zip(folga)
        .map(|(t, &m)| {
            let p = t.map(|v| c.pos[v as usize]);
            [
                p[0][0].min(p[1][0]).min(p[2][0]) - m,
                p[0][1].min(p[1][1]).min(p[2][1]) - m,
                p[0][0].max(p[1][0]).max(p[2][0]) + m,
                p[0][1].max(p[1][1]).max(p[2][1]) + m,
            ]
        })
        .collect();
    let mut ordem: Vec<usize> = (0..caixa.len()).collect();
    ordem.sort_by(|&a, &b| caixa[a][0].total_cmp(&caixa[b][0]));
    let mut cobre = vec![false; caixa.len()];
    for (x, &a) in ordem.iter().enumerate() {
        for &b in &ordem[x + 1..] {
            if caixa[b][0] > caixa[a][2] {
                break;
            }
            if caixa[b][1] > caixa[a][3] || caixa[b][3] < caixa[a][1] {
                continue;
            }
            if c.chave[a] == c.chave[b] || c.tris[a].iter().any(|v| c.tris[b].contains(v)) {
                continue;
            }
            cobre[if c.chave[a] > c.chave[b] { a } else { b }] = true;
        }
    }
    cobre
}

impl Fina {
    /// A malha fina dos triângulos de `campo`: `posa(k, u, v)` dá o repouso e o ponto posado no
    /// baricentro `(u, v)` do triângulo `k`. `fixo` força o lado de todos (gates e sondas); senão o
    /// lado dobra até o desvio ser `≤ tol` (com `tol` não finito ou `≤ 0`, lado `1`), e só nos
    /// [`cobridores`] — os outros ficam de fora. Um triângulo `rigido(k)` (a mesma linha de pesos
    /// nos três cantos, sem correcções) é posado por UMA transformação afim: a recta é a pele exacta.
    pub(super) fn nova(
        campo: &Campo<'_>,
        tol: f64,
        fixo: Option<usize>,
        rigido: impl Fn(usize) -> bool,
        mut posa: impl FnMut(usize, f64, f64) -> ([f64; 2], [f64; 2]),
    ) -> Self {
        let n = campo.tris.len();
        let adapta = fixo.is_none() && tol > 0.0 && tol.is_finite();
        // O 1.º degrau de todos: a grelha de lado 2 e o desvio dela à recta.
        let mut grelhas: Vec<(usize, Vec<([f64; 2], [f64; 2])>)> = Vec::with_capacity(n);
        let mut folga = vec![0.0; n];
        for k in 0..n {
            grelhas.push(if let Some(d) = fixo {
                (d.max(1), grelha_de(d.max(1), k, None, &mut posa))
            } else if !adapta || rigido(k) {
                (1, grelha_de(1, k, None, &mut posa))
            } else {
                let g = grelha_de(1, k, None, &mut posa);
                let g2 = grelha_de(2, k, Some(&g), &mut posa);
                // ⚠️ O desvio é AMOSTRADO nos meios das arestas: a folga dobra-o (o máximo
                // amostrado erra para baixo).
                folga[k] = 2.0 * desvio(1, &g2, &g);
                (2, g2)
            });
        }
        let cobre = if adapta {
            cobridores(campo, &folga)
        } else {
            vec![true; n]
        };
        let mut pos = Vec::new();
        let mut repouso = Vec::new();
        let mut tris = Vec::new();
        let mut pai = Vec::new();
        for (k, (mut d, mut g)) in grelhas.into_iter().enumerate() {
            if !cobre[k] {
                continue;
            }
            if adapta && d == 2 && folga[k] > 2.0 * tol {
                loop {
                    let g2 = grelha_de(2 * d, k, Some(&g), &mut posa);
                    let ok = desvio(d, &g2, &g) <= tol;
                    (d, g) = (2 * d, g2);
                    if ok || d >= MAX_DIV {
                        break;
                    }
                }
            }
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

