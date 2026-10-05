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

/// Uma grelha triangular posada: `(repouso, posado)` de cada vértice.
type Grade = Vec<([f64; 2], [f64; 2])>;

/// ⭐ **A malha fina, refinada SOB PEDIDO:** só os [`cobridores`] entram, cada um na grelha de
/// baldes pela caixa alargada da sua folga, e cada um só se parte (até `tol`) na 1.ª vez que um ponto
/// lhe cai na caixa ([`Fina::cobre`]) — os triângulos longe do traço de trás nunca se posam além do
/// 1.º degrau.
pub(super) struct Fina {
    tol: f64,
    adapta: bool,
    /// Baldes dos cobridores (índice = triângulo do campo), pela caixa alargada.
    pub(super) grelha: Grelha,
    caixa: Vec<[f64; 4]>,
    /// O lado e a grelha de cada triângulo, e se ela já está FINAL (desvio `≤ tol` ou lado máximo).
    grades: std::cell::RefCell<Vec<(usize, Grade, bool)>>,
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
) -> Grade {
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

/// Os triângulos da malha do campo, com a chave de osso de cada um.
pub(super) struct Campo<'a> {
    pub(super) tris: &'a [[u32; 3]],
    pub(super) chave: &'a [f64],
}

/// ⭐ **Quem pode TAPAR alguém** — o triângulo de chave maior de um par sem vértice comum cujas
/// caixas, cada uma alargada pela sua `folga` (o quanto a pele exacta sai da recta), se tocam. Um
/// triângulo fora desta lista não tapa nada e não entra na malha fina. Varrimento em `x`.
fn cobridores(
    c: &Campo<'_>,
    grades: &[(usize, Grade, bool)],
    folga: &[f64],
) -> (Vec<bool>, Vec<[f64; 4]>) {
    // A caixa dos pontos EXACTOS da grelha do 1.º degrau, alargada pela folga.
    let caixa: Vec<[f64; 4]> = grades
        .iter()
        .zip(folga)
        .map(|((_, g, _), &m)| {
            let (lo, hi) = g
                .iter()
                .fold(([f64::MAX; 2], [f64::MIN; 2]), |(lo, hi), (_, p)| {
                    (
                        [lo[0].min(p[0]), lo[1].min(p[1])],
                        [hi[0].max(p[0]), hi[1].max(p[1])],
                    )
                });
            [lo[0] - m, lo[1] - m, hi[0] + m, hi[1] + m]
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
    (cobre, caixa)
}

impl Fina {
    /// O 1.º degrau da malha fina dos triângulos de `campo`: `posa(k, u, v)` dá o repouso e o ponto
    /// posado no baricentro `(u, v)` do triângulo `k`. `fixo` força o lado de todos (gates e
    /// sondas, todos entram); senão a grelha de lado `2` mede a folga de cada um e só os
    /// [`cobridores`] entram (com `tol` não finito ou `≤ 0`, lado `1`). Um triângulo `rigido(k)` (a
    /// mesma linha de pesos nos três cantos, sem correcções) é posado por UMA transformação afim:
    /// a recta é a pele exacta.
    pub(super) fn nova(
        campo: &Campo<'_>,
        tol: f64,
        fixo: Option<usize>,
        rigido: impl Fn(usize) -> bool,
        mut posa: impl FnMut(usize, f64, f64) -> ([f64; 2], [f64; 2]),
    ) -> Self {
        let n = campo.tris.len();
        let adapta = fixo.is_none() && tol > 0.0 && tol.is_finite();
        let mut grades: Vec<(usize, Grade, bool)> = Vec::with_capacity(n);
        let mut folga = vec![0.0; n];
        for k in 0..n {
            grades.push(if let Some(d) = fixo {
                (d.max(1), grelha_de(d.max(1), k, None, &mut posa), true)
            } else if !adapta || rigido(k) {
                (1, grelha_de(1, k, None, &mut posa), true)
            } else {
                let g = grelha_de(1, k, None, &mut posa);
                let g2 = grelha_de(2, k, Some(&g), &mut posa);
                let dv = desvio(1, &g2, &g);
                // ⚠️ O desvio é AMOSTRADO nos meios das arestas: a folga dobra-o (o máximo
                // amostrado erra para baixo).
                folga[k] = 2.0 * dv;
                (2, g2, dv <= tol)
            });
        }
        let (cobre, caixa) = cobridores(campo, &grades, &folga);
        let quais: Vec<usize> = (0..n).filter(|&k| !adapta || cobre[k]).collect();
        let grelha = Grelha::de_caixas(&caixa, &quais);
        Self {
            tol,
            adapta,
            grelha,
            caixa,
            grades: std::cell::RefCell::new(grades),
        }
    }

    /// ⭐ O ponto do repouso que o triângulo `k` põe em `q` (posado), pela pele exacta até `tol`;
    /// `None` se `q` não cai nele. Parte o triângulo na 1.ª vez que é preciso.
    pub(super) fn cobre(
        &self,
        k: usize,
        q: [f64; 2],
        mut posa: impl FnMut(usize, f64, f64) -> ([f64; 2], [f64; 2]),
    ) -> Option<[f64; 2]> {
        let c = self.caixa[k];
        if q[0] < c[0] || q[0] > c[2] || q[1] < c[1] || q[1] > c[3] {
            return None;
        }
        let mut grades = self.grades.borrow_mut();
        let (d, g, final_) = &mut grades[k];
        if !*final_ && self.adapta {
            loop {
                let g2 = grelha_de(2 * *d, k, Some(g), &mut posa);
                let ok = desvio(*d, &g2, g) <= self.tol;
                (*d, *g) = (2 * *d, g2);
                if ok || *d >= MAX_DIV {
                    break;
                }
            }
            *final_ = true;
        }
        let (d, g) = (*d, &*g);
        let em = |[a, b, c]: [usize; 3]| {
            let uv = super::malha::bari(q, g[a].1, g[b].1, g[c].1);
            super::malha::dentro(uv).then(|| {
                let (u, v) = uv.unwrap_or_default();
                [0, 1].map(|x| (1.0 - u - v) * g[a].0[x] + u * g[b].0[x] + v * g[c].0[x])
            })
        };
        for j in 0..d {
            for i in 0..d - j {
                if let Some(r) = em([ix(d, i, j), ix(d, i + 1, j), ix(d, i, j + 1)]) {
                    return Some(r);
                }
                if i + j + 1 < d {
                    if let Some(r) = em([ix(d, i + 1, j), ix(d, i + 1, j + 1), ix(d, i, j + 1)]) {
                        return Some(r);
                    }
                }
            }
        }
        None
    }

    /// Quantos vértices finos há agora (sondas).
    #[cfg(test)]
    pub(super) fn vertices(&self) -> usize {
        self.grades.borrow().iter().map(|(_, g, _)| g.len()).sum()
    }
}
