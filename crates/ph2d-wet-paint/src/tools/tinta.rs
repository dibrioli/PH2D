//! F4-2 E6b: as ferramentas que RELAXAM a massa para uma média (Blend, Smear) na lei
//! [`Pigmento`](crate::sim::LeiDaTinta::Pigmento). A soma segue a massa célula a célula: o que a fatia PERDE sai
//! com a cor dela ([`transfere`] pela mesma fracção); o que GANHA chega com a cor média, pelo volume, das somas da
//! pegada antes da ferramenta (média de somas, F4_2 §4). O invariante `v · M_UM == massa` fica exacto.
//! Doc: `docs/Painter/Arquitetura/E6b_pickup_e_ferramentas.md`.

use crate::grid::Grid;
use crate::grid::tinta::{M_UM, garante};
use ph2d_paint_tinta::{Soma, Tinta, transfere};

/// A pegada antes da ferramenta: o rectângulo `(x0, y0, x1, y1)` inclusivo, as massas e a cor média das fatias.
pub(crate) struct Foto {
    rect: (usize, usize, usize, usize),
    susp: Vec<f32>,
    sett: Vec<f32>,
    filme: Option<Tinta>,
    aberto: Option<Tinta>,
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
        for y in y0..=y1 {
            for x in x0..=x1 {
                let i = y * g.s + x;
                susp.push(g.susp[i]);
                sett.push(g.sett[i]);
                f = f.junta(g.tinta[i].filme);
                a = a.junta(g.tinta[i].aberto);
            }
        }
        Some(Self {
            rect: (x0, y0, x1, y1),
            susp,
            sett,
            filme: f.cor(),
            aberto: a.cor(),
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
                fatia(&mut c.filme, self.susp[k], g.susp[i], self.filme);
                fatia(&mut c.aberto, self.sett[k], g.sett[i], self.aberto);
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
fn fatia(s: &mut Soma, antes: f32, depois: f32, chega: Option<Tinta>) {
    if depois < antes {
        let f = if antes > 0.0 {
            1.0 - depois / antes
        } else {
            1.0
        };
        transfere(s, &mut Soma::default(), f);
    } else if depois > antes {
        let dv = (depois - antes) / M_UM;
        match chega {
            Some(t) => *s = s.junta(Soma::de(t, dv)),
            // Pegada sem tinta antes: a massa nasceu do nada (não deve); a cor da própria fatia.
            None => match s.cor() {
                Some(t) => *s = s.junta(Soma::de(t, dv)),
                None => s.v += dv,
            },
        }
    }
}

#[cfg(test)]
#[path = "tinta_invariante_tests.rs"]
mod invariante_tests;
