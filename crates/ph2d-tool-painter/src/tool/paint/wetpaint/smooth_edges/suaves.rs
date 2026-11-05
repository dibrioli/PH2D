//! [`Suaves`]: o Smooth Edges de uma região, um [`Suave`] por degrau presente (filho de [`super`], LOC cap).

use super::*;

/// A caixa de cada degrau, `(x0, y0, x1, y1)`.
type CaixasDosDegraus = Vec<Option<(usize, usize, usize, usize)>>;

/// Conta a célula `(x, y)` na caixa do degrau `d` (o `0` não tem caixa).
#[inline]
fn alarga(c: &mut CaixasDosDegraus, d: u8, x: usize, y: usize) {
    if d > 0 {
        let c = &mut c[usize::from(d)];
        *c = Some(c.map_or((x, y, x, y), |(p, q, r, s)| {
            (p.min(x), q.min(y), r.max(x), s.max(y))
        }));
    }
}

fn junta(mut a: CaixasDosDegraus, b: CaixasDosDegraus) -> CaixasDosDegraus {
    for (x, y) in a.iter_mut().zip(b) {
        *x = match (*x, y) {
            (Some((p, q, r, s)), Some((t, u, v, w))) => {
                Some((p.min(t), q.min(u), r.max(v), s.max(w)))
            }
            (a, b) => a.or(b),
        };
    }
    a
}

/// **O Smooth Edges de uma região, pelo valor de cada célula** (dono 04/11): um [`Suave`] por degrau presente, só na
/// caixa dos píxeis desse degrau. A mancha de cada um é a de TODA a tinta (senão a tinta à volta de um traço a `0`
/// ganhava uma orla falsa), por isso dois traços só se vêem ao alcance do raio; o resultado de um pixel não depende de
/// mais nada da região. O papel e o véu sem mancha lêem o maior degrau cuja tinta os alcança.
pub(in super::super) struct Suaves {
    x0: usize,
    y0: usize,
    cw: usize,
    ch: usize,
    rr: usize,
    /// O degrau que cada célula da janela lê (`0` = o pigmento dela).
    quem: Vec<u8>,
    grupos: Vec<Option<Suave>>,
}

impl Suaves {
    /// `None` = nenhuma tinta com Smooth Edges na região: o caminho de antes.
    #[allow(clippy::too_many_arguments)]
    pub(in super::super) fn nova(
        sampler: SampleU,
        (pigment, stride): (&[u8], usize),
        (plano, gw, gh): (&[u8], usize, usize),
        (w, h): (usize, usize),
        (x0, y0, x1, y1): (usize, usize, usize, usize),
        rr: usize,
        linhas: Option<bool>,
        orla: bool,
    ) -> Option<Self> {
        use ph2d_wet_paint::grid::aparencia::{DEGRAUS_DO_SUAVE, raio_do_degrau};
        if plano.is_empty() {
            return None;
        }
        // A janela em células (base 0) dos píxeis da região e do raio máximo à volta.
        let a = alcance(RAIO_MAX);
        let (cx0, cy0) = (x0.saturating_sub(a) / rr, y0.saturating_sub(a) / rr);
        let (cx1, cy1) = (
            ((x1 + a) / rr).min(gw.saturating_sub(1)),
            ((y1 + a) / rr).min(gh.saturating_sub(1)),
        );
        let (cw, ch) = (cx1 + 1 - cx0, cy1 + 1 - cy0);
        let n = usize::from(DEGRAUS_DO_SUAVE) + 1;
        let g_stride = gw * 4;
        let mut quem = vec![0u8; cw * ch];
        let mut na_mancha = vec![false; cw * ch];
        let caixa = quem
            .par_chunks_mut(cw)
            .zip(na_mancha.par_chunks_mut(cw))
            .enumerate()
            .map(|(j, (linha, mancha))| {
                let y = cy0 + j;
                let mut c = vec![None; n];
                for (b, (q, m)) in linha.iter_mut().zip(mancha.iter_mut()).enumerate() {
                    let x = cx0 + b;
                    *m = f32::from(pigment[y * g_stride + x * 4 + 3]) / 255.0 >= SS0;
                    if *m {
                        *q = plano[y * gw + x];
                        alarga(&mut c, *q, x, y);
                    }
                }
                c
            })
            .reduce(|| vec![None; n], junta);
        if caixa.iter().all(Option::is_none) {
            return None;
        }
        // O papel lê o maior degrau cuja tinta o alcança: do maior para o menor, quem já leu fica.
        for d in (1..n).rev() {
            let Some((p, q, r, s)) = caixa[d] else {
                continue;
            };
            let al = (raio_do_degrau(d as u8) / rr as f32).ceil() as usize + 1;
            let (ax0, ay0) = (p.saturating_sub(al).max(cx0), q.saturating_sub(al).max(cy0));
            let (ax1, ay1) = ((r + al).min(cx1), (s + al).min(cy1));
            let bw = ax1 + 1 - ax0;
            // A distância horizontal à tinta deste degrau, depois a vertical sobre ela.
            let mut hz = vec![false; bw * (ay1 + 1 - ay0)];
            hz.par_chunks_mut(bw).enumerate().for_each(|(j, linha)| {
                let y = ay0 + j;
                let e = |x: usize| {
                    let i = (y - cy0) * cw + x - cx0;
                    na_mancha[i] && plano[y * gw + x] == d as u8
                };
                let mut desde = usize::MAX;
                for (b, v) in linha.iter_mut().enumerate() {
                    desde = if e(ax0 + b) {
                        0
                    } else {
                        desde.saturating_add(1)
                    };
                    *v = desde <= al;
                }
                desde = usize::MAX;
                for (b, v) in linha.iter_mut().enumerate().rev() {
                    desde = if e(ax0 + b) {
                        0
                    } else {
                        desde.saturating_add(1)
                    };
                    *v |= desde <= al;
                }
            });
            let bh = ay1 + 1 - ay0;
            quem[(ay0 - cy0) * cw..(ay1 + 1 - cy0) * cw]
                .par_chunks_mut(cw)
                .enumerate()
                .for_each(|(j, linha)| {
                    let (l0, l1) = (j.saturating_sub(al), (j + al).min(bh - 1));
                    let mancha = &na_mancha[(ay0 + j - cy0) * cw..(ay0 + j + 1 - cy0) * cw];
                    for b in 0..bw {
                        let x = ax0 - cx0 + b;
                        if !mancha[x] && linha[x] == 0 && (l0..=l1).any(|l| hz[l * bw + b]) {
                            linha[x] = d as u8;
                        }
                    }
                });
        }
        // Um desfoque por degrau que alguma célula lê, só na caixa dos píxeis dele dentro da região.
        let (lx0, ly0) = (x0 / rr, y0 / rr);
        let (lx1, ly1) = (
            (x1.saturating_sub(1) / rr).min(cx1),
            (y1.saturating_sub(1) / rr).min(cy1),
        );
        let caixa_px = quem
            .par_chunks(cw)
            .enumerate()
            .filter(|(j, _)| (ly0..=ly1).contains(&(cy0 + j)))
            .map(|(j, linha)| {
                let mut c = vec![None; n];
                for x in lx0.max(cx0)..=lx1 {
                    alarga(&mut c, linha[x - cx0], x, cy0 + j);
                }
                c
            })
            .reduce(|| vec![None; n], junta);
        let grupos: Vec<Option<Suave>> = caixa_px
            .iter()
            .enumerate()
            .map(|(d, c)| {
                c.map(|(p, q, r, s)| {
                    Suave::nova(
                        sampler,
                        (pigment, stride),
                        (w, h),
                        (
                            (p * rr).max(x0),
                            (q * rr).max(y0),
                            ((r + 1) * rr).min(x1),
                            ((s + 1) * rr).min(y1),
                        ),
                        raio_do_degrau(d as u8),
                        linhas,
                        orla,
                    )
                })
            })
            .collect();
        Some(Self {
            x0: cx0,
            y0: cy0,
            cw,
            ch,
            rr,
            quem,
            grupos,
        })
    }

    /// (Testes) [`raio_escalar`]: toda a região lê um só [`Suave`].
    #[cfg(test)]
    pub(in super::super) fn escalar(s: Suave, (w, h): (usize, usize)) -> Self {
        Self {
            x0: 0,
            y0: 0,
            cw: w,
            ch: h,
            rr: 1,
            quem: vec![1; w * h],
            grupos: vec![None, Some(s)],
        }
    }

    /// O pigmento que o pixel `(px, py)` mostra, pelo degrau da tinta dele ([`Suave::aplica_com`]).
    #[inline]
    pub(in super::super) fn aplica_com(
        &self,
        px: usize,
        py: usize,
        dele: ([f32; 3], f32),
        orla: bool,
    ) -> ([f32; 3], f32) {
        let (cx, cy) = (
            (px / self.rr).clamp(self.x0, self.x0 + self.cw - 1) - self.x0,
            (py / self.rr).clamp(self.y0, self.y0 + self.ch - 1) - self.y0,
        );
        match self.quem[cy * self.cw + cx] {
            0 => dele,
            d => match &self.grupos[usize::from(d)] {
                Some(s) => s.aplica_com(px, py, dele, orla),
                None => dele,
            },
        }
    }

    /// [`Self::aplica_com`] com a orla com que se montaram.
    #[inline]
    pub(in super::super) fn aplica(
        &self,
        px: usize,
        py: usize,
        dele: ([f32; 3], f32),
    ) -> ([f32; 3], f32) {
        let (cx, cy) = (
            (px / self.rr).clamp(self.x0, self.x0 + self.cw - 1) - self.x0,
            (py / self.rr).clamp(self.y0, self.y0 + self.ch - 1) - self.y0,
        );
        match self.quem[cy * self.cw + cx] {
            0 => dele,
            d => match &self.grupos[usize::from(d)] {
                Some(s) => s.aplica(px, py, dele),
                None => dele,
            },
        }
    }
}
