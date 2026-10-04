//! A malha posada do recorte ([`super`]): a ARTE em repouso, a grelha de baldes e as contas de
//! triângulo. Irmão pelo tecto de LOC.

use ph2d_vec_scene::VecPath;

use super::avalia;

/// ⭐⭐ **A ARTE em repouso** — os contornos fechados achatados e a regra de preenchimento da forma.
/// ⛔ A malha do campo passa um pouco ALÉM do desenho (as células da borda), e na dobra essa margem
/// caía sobre o traço de trás e apagava-o (MEDIDO, A6: a `110°` o traço de cima da cópia de baixo
/// sumia sob a margem da cópia da frente, triângulos de centro `(28,2; 17,5)`). Um triângulo só
/// tapa onde o que ele lá põe é ARTE.
#[derive(Default)]
pub(super) struct Arte {
    aneis: Vec<Vec<[f64; 2]>>,
    regra: ph2d_vec_scene::FillRule,
}

impl Arte {
    pub(super) fn de(fonte: &VecPath) -> Self {
        let aneis = (0..fonte.contour_count())
            .filter_map(|c| fonte.contour(c))
            .filter(|(v, fechado)| *fechado && v.len() > 1)
            .map(|(v, _)| {
                (0..v.len())
                    .flat_map(|k| {
                        let c = [
                            v[k].anchor,
                            v[k].out_handle,
                            v[(k + 1) % v.len()].in_handle,
                            v[(k + 1) % v.len()].anchor,
                        ];
                        (0..16).map(move |i| avalia(&c, f64::from(i) / 16.0))
                    })
                    .collect()
            })
            .collect();
        Self {
            aneis,
            regra: fonte.fill_rule,
        }
    }

    /// `p` é arte? — o número de voltas pela regra da forma; sem anéis, sim.
    pub(super) fn tem(&self, p: [f64; 2]) -> bool {
        if self.aneis.is_empty() {
            return true;
        }
        let voltas: i32 = self
            .aneis
            .iter()
            .flat_map(|a| (0..a.len()).map(move |i| (a[i], a[(i + 1) % a.len()])))
            .map(|(a, b)| {
                let corta = (a[1] > p[1]) != (b[1] > p[1])
                    && (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0] > p[0];
                match (corta, b[1] > a[1]) {
                    (false, _) => 0,
                    (true, true) => 1,
                    (true, false) => -1,
                }
            })
            .sum();
        match self.regra {
            ph2d_vec_scene::FillRule::EvenOdd => voltas % 2 != 0,
            ph2d_vec_scene::FillRule::NonZero => voltas != 0,
        }
    }
}

/// Baldes de triângulos posados por célula — a consulta de um ponto só vê os do balde dele.
pub(super) struct Grelha {
    pub(super) origem: [f64; 2],
    pub(super) lado: f64,
    pub(super) dim: [usize; 2],
    pub(super) baldes: Vec<Vec<u32>>,
}

/// Coordenadas baricêntricas `(u, v)` de `p` em `abc` (sinal livre); `None` num triângulo nulo.
pub(super) fn bari(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> Option<(f64, f64)> {
    let den = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
    if den.abs() < 1e-12 {
        return None;
    }
    let u = ((p[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (p[1] - a[1])) / den;
    let v = ((b[0] - a[0]) * (p[1] - a[1]) - (p[0] - a[0]) * (b[1] - a[1])) / den;
    Some((u, v))
}

pub(super) fn dentro(uv: Option<(f64, f64)>) -> bool {
    uv.is_some_and(|(u, v)| u >= -1e-9 && v >= -1e-9 && u + v <= 1.0 + 1e-9)
}

impl Grelha {
    pub(super) fn nova(pos: &[[f64; 2]], tris: &[[u32; 3]]) -> Self {
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for p in pos {
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        #[expect(clippy::cast_precision_loss, reason = "contagem de triângulos")]
        let n = tris.len().max(1) as f64;
        let area = ((hi[0] - lo[0]) * (hi[1] - lo[1])).max(1e-12);
        let lado = (area / n).sqrt() * 2.0;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "dimensão da grelha, finita e positiva"
        )]
        let dim = [0, 1].map(|k| (((hi[k] - lo[k]) / lado).floor() as usize + 1).min(4096));
        let mut g = Self {
            origem: lo,
            lado,
            dim,
            baldes: vec![Vec::new(); dim[0] * dim[1]],
        };
        for (i, t) in tris.iter().enumerate() {
            let ps = t.map(|v| pos[v as usize]);
            let c0 = g.celula([
                ps[0][0].min(ps[1][0]).min(ps[2][0]),
                ps[0][1].min(ps[1][1]).min(ps[2][1]),
            ]);
            let c1 = g.celula([
                ps[0][0].max(ps[1][0]).max(ps[2][0]),
                ps[0][1].max(ps[1][1]).max(ps[2][1]),
            ]);
            for y in c0[1]..=c1[1] {
                for x in c0[0]..=c1[0] {
                    #[expect(clippy::cast_possible_truncation, reason = "índice de triângulo u32")]
                    g.baldes[y * dim[0] + x].push(i as u32);
                }
            }
        }
        g
    }

    pub(super) fn celula(&self, p: [f64; 2]) -> [usize; 2] {
        [0, 1].map(|k| {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "célula da grelha, presa ao intervalo"
            )]
            let c = ((p[k] - self.origem[k]) / self.lado).floor().max(0.0) as usize;
            c.min(self.dim[k] - 1)
        })
    }

    pub(super) fn balde(&self, p: [f64; 2]) -> &[u32] {
        let c = self.celula(p);
        &self.baldes[c[1] * self.dim[0] + c[0]]
    }
}
