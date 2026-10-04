//! ⭐⭐ **A ponta de um corte do traço acerta no CRUZAMENTO desenhado** (A9, 2026-10-04) — numa
//! dobra extrema sobravam tiques de `0,1`–`0,9` larguras de traço para lá do cruzamento com a borda
//! da frente. Quem decide «tapado» é a malha posada em triângulos RECTOS, e o desenho segue a pele
//! exacta: MEDIDO, o ponto afasta-se da malha até `0,19` aresta a `75°` e `1`–`2` arestas na dobra.
//! Onde o traço de trás deixa de se ver, ele CRUZA a borda desenhada da frente; a menos de uma
//! largura do traço (mais perto, o resto é um tique — a régua da F52) a ponta vai para lá. Só os
//! contornos FECHADOS são borda (as riscas tocam o contorno de propósito).

use super::frente::{avalia, cubica};
use ph2d_vec_scene::VecPath;

#[cfg(test)]
thread_local! {
    /// Os gates desligam o encaixe para o CONTROLO.
    pub(super) static SEM_ENCAIXE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Amostras por segmento do assado na polilinha das bordas.
const AMOSTRAS: usize = 16;

/// As bordas desenhadas (os contornos FECHADOS do assado) em polilinha, com uma grelha de troços.
pub(super) struct Bordas {
    /// Por contorno do assado: os pontos da volta (o parâmetro do ponto `i` é `i / AMOSTRAS`);
    /// vazio num aberto.
    pl: Vec<Vec<[f64; 2]>>,
    lado: f64,
    /// `(célula, contorno, troço)` ordenado pela célula.
    celulas: Vec<([i64; 2], u32, u32)>,
}

fn cruza(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> Option<f64> {
    let (r, s) = ([b[0] - a[0], b[1] - a[1]], [d[0] - c[0], d[1] - c[1]]);
    let den = r[0] * s[1] - r[1] * s[0];
    if den.abs() < 1e-18 {
        return None;
    }
    let q = [c[0] - a[0], c[1] - a[1]];
    let (t, u) = (
        (q[0] * s[1] - q[1] * s[0]) / den,
        (q[0] * r[1] - q[1] * r[0]) / den,
    );
    ((0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)).then_some(t)
}

impl Bordas {
    pub(super) fn de(d: &VecPath, lado: f64) -> Self {
        let pl: Vec<Vec<[f64; 2]>> = (0..d.contour_count())
            .filter_map(|c| d.contour(c))
            .map(|(v, fechado)| {
                if !fechado || v.len() < 2 {
                    return Vec::new();
                }
                let mut w = v.to_vec();
                w.push(v[0]);
                (0..v.len())
                    .flat_map(|k| {
                        let c = cubica(&w, k);
                        #[expect(clippy::cast_precision_loss, reason = "amostra")]
                        (0..AMOSTRAS).map(move |i| avalia(&c, i as f64 / AMOSTRAS as f64))
                    })
                    .collect()
            })
            .collect();
        let mut celulas = Vec::new();
        let celula = |p: [f64; 2]| {
            #[expect(clippy::cast_possible_truncation, reason = "célula da grelha, finita")]
            [0, 1].map(|k| (p[k] / lado).floor() as i64)
        };
        for (c, l) in pl.iter().enumerate() {
            for j in 0..l.len() {
                let (a, b) = (l[j], l[(j + 1) % l.len()]);
                let (c0, c1) = (
                    celula([a[0].min(b[0]), a[1].min(b[1])]),
                    celula([a[0].max(b[0]), a[1].max(b[1])]),
                );
                for y in c0[1]..=c1[1] {
                    for x in c0[0]..=c1[0] {
                        #[expect(clippy::cast_possible_truncation, reason = "índices u32")]
                        celulas.push(([x, y], c as u32, j as u32));
                    }
                }
            }
        }
        celulas.sort_unstable();
        Self { pl, lado, celulas }
    }

    /// O parâmetro `u` (no contorno `c` do assado, `k + t`, sem dar a volta) levado ao cruzamento
    /// com outra borda mais perto AO LONGO do contorno, a menos de `alcance` de arco; sem nenhum, `u`.
    pub(super) fn encaixa(&self, c: usize, u: f64, alcance: f64) -> f64 {
        #[cfg(test)]
        if SEM_ENCAIXE.with(std::cell::Cell::get) {
            return u;
        }
        let Some(l) = self.pl.get(c).filter(|l| l.len() > 2) else {
            return u;
        };
        let n = l.len();
        #[expect(clippy::cast_precision_loss, reason = "amostra")]
        let s = u * AMOSTRAS as f64;
        let i0 = s.floor();
        #[expect(clippy::cast_possible_truncation, reason = "índice da polilinha")]
        let ponto = |i: f64| l[(i as i64).rem_euclid(n as i64) as usize];
        let p = {
            let (a, b, f) = (ponto(i0), ponto(i0 + 1.0), s - i0);
            [
                (b[0] - a[0]).mul_add(f, a[0]),
                (b[1] - a[1]).mul_add(f, a[1]),
            ]
        };
        let mut melhor: Option<(f64, f64)> = None;
        for passo in [1.0, -1.0] {
            // O troço `(a → b)` da janela, de parâmetros `(sa, sb)` em amostras e arco inicial `arco`.
            let (mut a, mut sa, mut arco) = (p, s, 0.0);
            let mut sb = if passo > 0.0 { i0 + 1.0 } else { i0 };
            if sb == sa {
                sb += passo;
            }
            for _ in 0..n {
                if arco > alcance {
                    break;
                }
                let b = ponto(sb);
                let troco = sa.min(sb).floor();
                if let Some(t) = self.cruzamento(c, troco, a, b) {
                    let em = arco + t * (b[0] - a[0]).hypot(b[1] - a[1]);
                    if em <= alcance && melhor.is_none_or(|(m, _)| em < m) {
                        melhor = Some((em, (sb - sa).mul_add(t, sa)));
                    }
                }
                arco += (b[0] - a[0]).hypot(b[1] - a[1]);
                (a, sa, sb) = (b, sb, sb + passo);
            }
        }
        #[expect(clippy::cast_precision_loss, reason = "amostra")]
        melhor.map_or(u, |(_, sx)| sx / AMOSTRAS as f64)
    }

    /// O primeiro cruzamento (fracção em `a → b`) do troço `troco` do contorno `c` com uma borda
    /// que não lhe é vizinha.
    fn cruzamento(&self, c: usize, troco: f64, a: [f64; 2], b: [f64; 2]) -> Option<f64> {
        let n = self.pl[c].len() as i64;
        #[expect(clippy::cast_possible_truncation, reason = "índice da polilinha")]
        let proprio = (troco as i64).rem_euclid(n);
        #[expect(clippy::cast_possible_truncation, reason = "célula da grelha, finita")]
        let celula = |p: [f64; 2]| [0, 1].map(|k| (p[k] / self.lado).floor() as i64);
        let (c0, c1) = (
            celula([a[0].min(b[0]), a[1].min(b[1])]),
            celula([a[0].max(b[0]), a[1].max(b[1])]),
        );
        let mut melhor: Option<f64> = None;
        for y in c0[1]..=c1[1] {
            for x in c0[0]..=c1[0] {
                let de = self.celulas.partition_point(|e| e.0 < [x, y]);
                for &(_, c2, j) in self.celulas[de..].iter().take_while(|e| e.0 == [x, y]) {
                    let (c2, j) = (c2 as usize, i64::from(j));
                    if c2 == c
                        && ((j - proprio).rem_euclid(n) <= 1 || (proprio - j).rem_euclid(n) <= 1)
                    {
                        continue;
                    }
                    let l = &self.pl[c2];
                    #[expect(
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss,
                        reason = "índice"
                    )]
                    let (p, q) = (l[j as usize], l[(j as usize + 1) % l.len()]);
                    if let Some(t) = cruza(a, b, p, q) {
                        melhor = Some(melhor.map_or(t, |m: f64| m.min(t)));
                    }
                }
            }
        }
        melhor
    }
}

#[cfg(test)]
#[path = "skin_desenho_camadas_cruza_tests.rs"]
mod tests;
