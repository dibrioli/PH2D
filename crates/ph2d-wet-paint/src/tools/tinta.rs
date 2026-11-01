//! F4-2 E6b: as ferramentas que RELAXAM a massa para uma média (Blend, Smear) na lei
//! [`Pigmento`](crate::sim::LeiDaTinta::Pigmento). A soma segue a massa célula a célula: o que a fatia PERDE sai
//! com a cor dela ([`transfere`] pela mesma fracção); o que GANHA chega com a cor média, pelo volume, das somas da
//! pegada antes da ferramenta (média de somas, F4_2 §4). O invariante `v · M_UM == massa` fica exacto.
//! Doc: `docs/Painter/Arquitetura/E6b_pickup_e_ferramentas.md`.

use crate::grid::Grid;
use crate::grid::tinta::{M_UM, garante};
use ph2d_paint_tinta::{Espectro, Soma, transfere};

/// A pegada antes da ferramenta: o rectângulo `(x0, y0, x1, y1)` inclusivo, as massas e a soma de cada fatia.
pub(crate) struct Foto {
    rect: (usize, usize, usize, usize),
    susp: Vec<f32>,
    sett: Vec<f32>,
    filme: Soma,
    aberto: Soma,
    /// O `Espectro` total de cada fatia (E10c); `None` sem o lado espectral.
    lado: Option<[Espectro; 2]>,
}

impl Foto {
    /// Fotografa o rectângulo (recortado à grade); `None` com a lei velha.
    pub(crate) fn tira(g: &mut Grid, p: &crate::sim::Params, rect: [i64; 4]) -> Option<Self> {
        if p.tinta != crate::sim::LeiDaTinta::Pigmento {
            return None;
        }
        garante(g);
        let cl = |v: i64, n: usize| v.clamp(0, n as i64 - 1) as usize;
        let (x0, y0, x1, y1) = (
            cl(rect[0], g.w),
            cl(rect[1], g.h),
            cl(rect[2], g.w),
            cl(rect[3], g.h),
        );
        let (mut susp, mut sett) = (Vec::new(), Vec::new());
        let (mut f, mut a) = (Soma::default(), Soma::default());
        let mut lado = (!g.pigmento.is_empty()).then(<[Espectro; 2]>::default);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let i = y * g.s + x;
                susp.push(g.susp[i]);
                sett.push(g.sett[i]);
                f = f.junta(g.tinta[i].filme);
                a = a.junta(g.tinta[i].aberto);
                if let Some(l) = lado.as_mut() {
                    *l = [0, 1].map(|k| l[k].junta(g.pigmento[i][k]));
                }
            }
        }
        Some(Self {
            rect: (x0, y0, x1, y1),
            susp,
            sett,
            filme: f,
            aberto: a,
            lado,
        })
    }

    /// Depois da ferramenta: cada fatia da pegada segue a sua massa.
    pub(crate) fn segue(&self, g: &mut Grid) {
        let (x0, y0, x1, y1) = self.rect;
        let mut k = 0;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let i = y * g.s + x;
                let c = &mut g.tinta[i];
                let (f0, a0) = (c.filme, c.aberto);
                fatia(&mut c.filme, self.susp[k], g.susp[i], &self.filme);
                fatia(&mut c.aberto, self.sett[k], g.sett[i], &self.aberto);
                if let (Some(t), Some(l)) = (self.lado, g.pigmento.get_mut(i)) {
                    let (fl, al) = (self.susp[k], self.sett[k]);
                    fatia_e(&mut l[0], (f0, fl, g.susp[i]), (&self.filme, &t[0]));
                    fatia_e(&mut l[1], (a0, al, g.sett[i]), (&self.aberto, &t[1]));
                }
                k += 1;
            }
        }
    }
}

/// A ferramenta `f` com a [`Foto`] da caixa de raio `raio` em volta de `(x, y)` (a caixa tem de cobrir TODA célula
/// cuja massa a ferramenta escreve); a lei velha não paga nada.
pub(crate) fn com_foto<R>(
    g: &mut Grid,
    p: &crate::sim::Params,
    (x, y, raio): (f64, f64, f64),
    f: impl FnOnce(&mut Grid) -> R,
) -> R {
    let caixa = [x - raio, y - raio, x + raio, y + raio];
    let foto = Foto::tira(
        g,
        p,
        [
            caixa[0].floor() as i64 - 1,
            caixa[1].floor() as i64 - 1,
            caixa[2].ceil() as i64 + 1,
            caixa[3].ceil() as i64 + 1,
        ],
    );
    let r = f(g);
    if let Some(foto) = foto {
        foto.segue(g);
    }
    r
}

/// A fatia `s` cuja massa passou de `antes` a `depois`.
fn fatia(s: &mut Soma, antes: f32, depois: f32, chega: &Soma) {
    if depois < antes {
        let f = if antes > 0.0 {
            1.0 - depois / antes
        } else {
            1.0
        };
        transfere(s, &mut Soma::default(), f);
    } else if depois > antes {
        let dv = (depois - antes) / M_UM;
        // Chega a tinta média da pegada (cor e meio); sem ela (não deve), a da própria fatia.
        let de = if chega.v > 0.0 { *chega } else { *s };
        if de.v > 0.0 {
            *s = s.junta(de.escala(dv / de.v));
        } else {
            s.v += dv;
        }
    }
}

/// O par de [`fatia`] no `Espectro` (E10c): a MESMA fracção que sai, e o que chega na composição da pegada.
fn fatia_e(
    e: &mut Espectro,
    (s, antes, depois): (Soma, f32, f32),
    (chega, chega_e): (&Soma, &Espectro),
) {
    if depois < antes {
        let f = if antes > 0.0 {
            1.0 - depois / antes
        } else {
            1.0
        };
        let _ = Espectro::transfere(e, &mut Espectro::default(), f);
    } else if depois > antes {
        let dv = (depois - antes) / M_UM;
        if chega.v > 0.0 {
            *e = e.junta(chega_e.escala(dv / chega.v));
        } else if s.v > 0.0 {
            *e = e.junta(e.escala(dv / s.v));
        }
    }
}

#[cfg(test)]
#[path = "tinta_invariante_tests.rs"]
mod invariante_tests;
