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

    pub(super) fn balde(&self, p: [f64; 2]) -> &[u32] {
        let c = self.celula(p);
        &self.baldes[c[1] * self.dim[0] + c[0]]
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
}

/// Os triângulos `a` e `b` sobrepõem-se ou tocam-se? — nenhuma das seis arestas os separa com
/// folga positiva (o teste dos eixos separadores; num triângulo nulo a aresta não separa).
pub(super) fn se_sobrepoem(a: [[f64; 2]; 3], b: [[f64; 2]; 3]) -> bool {
    for (p, q) in [(a, b), (b, a)] {
        for e in 0..3 {
            let (u, v, w) = (p[e], p[(e + 1) % 3], p[(e + 2) % 3]);
            let mut n = [v[1] - u[1], u[0] - v[0]];
            if n[0] * (w[0] - u[0]) + n[1] * (w[1] - u[1]) > 0.0 {
                n = [-n[0], -n[1]];
            }
            if n != [0.0, 0.0]
                && q.iter()
                    .all(|x| n[0] * (x[0] - u[0]) + n[1] * (x[1] - u[1]) > 0.0)
            {
                return false;
            }
        }
    }
    true
}

/// ⭐⭐ **Pode alguém TAPAR alguém?** (A7) — há um par de triângulos POSADOS que se sobrepõem, um
/// de chave maior que o outro e sem vértice comum? Pára no primeiro. Sem nenhum (o caso comum, sem
/// dobra) nada há a cortar: um cobridor só tapa o dono a que se sobrepõe ([`super::Posada::tapado`]).
pub(super) fn ha_sobreposicao(
    pos: &[[f64; 2]],
    tris: &[[u32; 3]],
    chave: &[f64],
    g: &Grelha,
) -> bool {
    let tri = |i: usize| tris[i].map(|v| pos[v as usize]);
    let caixa = |t: [[f64; 2]; 3]| -> [f64; 4] {
        let x = t.map(|p| p[0]);
        let y = t.map(|p| p[1]);
        [
            x[0].min(x[1]).min(x[2]),
            y[0].min(y[1]).min(y[2]),
            x[0].max(x[1]).max(x[2]),
            y[0].max(y[1]).max(y[2]),
        ]
    };
    let caixas: Vec<[f64; 4]> = (0..tris.len()).map(|i| caixa(tri(i))).collect();
    let mut visto = vec![u32::MAX; tris.len()];
    for (i, t) in tris.iter().enumerate() {
        let ci = caixas[i];
        let (c0, c1) = (g.celula([ci[0], ci[1]]), g.celula([ci[2], ci[3]]));
        #[expect(clippy::cast_possible_truncation, reason = "índice de triângulo u32")]
        let iu = i as u32;
        for y in c0[1]..=c1[1] {
            for x in c0[0]..=c1[0] {
                for &k in &g.baldes[y * g.dim[0] + x] {
                    let ku = k as usize;
                    if visto[ku] == iu || chave[ku] <= chave[i] {
                        continue;
                    }
                    visto[ku] = iu;
                    let ck = caixas[ku];
                    if ck[0] > ci[2] || ck[2] < ci[0] || ck[1] > ci[3] || ck[3] < ci[1] {
                        continue;
                    }
                    if !tris[ku].iter().any(|v| t.contains(v)) && se_sobrepoem(tri(i), tri(ku)) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Nada pode ficar tapado: nenhum par sobreposto e, se o avesso tapa, nenhum triângulo virado.
pub(super) fn nada_tapa(algum_par: bool, avesso: bool, virado: &[bool]) -> bool {
    !algum_par && !(avesso && virado.contains(&true))
}
