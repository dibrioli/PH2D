//! F4-2 E6b: o BICO na lei [`Pigmento`](crate::sim::LeiDaTinta::Pigmento). A ponta TROCA tinta com a tela pela
//! fracção `1 − k` que o bico velho já calcula ([`super::retido`]): leva `1 − k` do filme e `(1 − k)·assentado·o`
//! do aberto ([`pick_up`]) para a [`Soma`] do texel, e devolve `1 − k` do que levou onde a janela pousa — a massa
//! devolvida soma-se ao `pig` do texel e a cor da ponta passa a ser a da soma (devolvido ⊕ tinta do pincel), em
//! Oklab pelo volume. Tela + ponta conservam-se (fora a tinta nova do pincel e a auto-limpeza).
//! Doc: `docs/Painter/Arquitetura/E6b_pickup_e_ferramentas.md`.

use super::*;
use crate::grid::tinta::pigmento::{ABERTO, FILME};
use crate::grid::tinta::{M_UM, garante, srgb_0_255};
use ph2d_paint_tinta::{Soma, Tinta, pick_up, transfere};

impl Trail {
    /// O passo da ponta com `Pigmento` (depois do bico velho, que só deixa a cor; esta escreve a cor da soma).
    pub(super) fn ponta_pigmento(
        &mut self,
        g: &mut Grid,
        p: &Params,
        clean: f64,
        pickup: Option<(i32, i32, i32, i32)>,
    ) {
        let n = (self.size * self.size) as usize;
        if self.ponta_tinta.len() != n {
            self.ponta_tinta = vec![Soma::default(); n];
        }
        if clean > 0.0 {
            for s in &mut self.ponta_tinta {
                transfere(s, &mut Soma::default(), clean.min(1.0) as f32);
            }
        }
        let Some((lx0, ly0, lx1, ly1)) = pickup else {
            return;
        };
        garante(g);
        let reservatorio = p.k(Knob::TipReservoir);
        if reservatorio > 0.0 {
            self.mistura_as_somas_da_ponta();
        }
        let (tip_retain, dabs) = (1.0 - p.k(Knob::Pickup), self.dab_count.max(1) as i32);
        let assentado = assentado(p, reservatorio) as f32;
        // O que a ponta SOLTA onde pousa: a mesma fracção por janela que ela troca (também sobre o papel branco).
        let solta = (if reservatorio > 0.0 {
            reservatorio
        } else {
            1.0 - tip_retain
        })
        .clamp(0.0, 1.0) as f32;
        let base = Tinta::de_srgb(
            [self.base_r, self.base_g, self.base_b].map(|c| (c / 255.0) as f32),
            0.0,
        );
        let (s, w, h) = (g.s, g.w as i32, g.h as i32);
        let com_limpo = self.limpo.len() == self.tip_r.len();
        for ly in ly0..=ly1 {
            let cy = self.anchor_y + (ly - self.half);
            if cy < 2 || cy > h - 1 {
                continue;
            }
            for lx in lx0..=lx1 {
                let cx = self.anchor_x + (lx - self.half);
                if cx < 2 || cx > w - 1 {
                    continue;
                }
                let i = cx as usize + cy as usize * s;
                let l = (lx + ly * self.size) as usize;
                let (w_s, w_f) = pesos_da_recolha(g, i, f64::from(assentado));
                let fe = w_s + w_f;
                let f = if fe > 0.0 {
                    (1.0 - retido(fe, reservatorio, dabs, tip_retain)) as f32
                } else {
                    0.0
                };
                let pig = f64::from(self.pig[l]);
                let ponta = &mut self.ponta_tinta[l];
                let mut devolvido = Soma::default();
                if pig > 0.0 {
                    transfere(ponta, &mut devolvido, solta);
                }
                if f > 0.0 {
                    let c = &mut g.tinta[i];
                    let o = c.o;
                    let (filme, aberto) = (c.filme.v, c.aberto.v);
                    pick_up(&mut c.filme, ponta, f, 1.0, 1.0);
                    pick_up(&mut c.aberto, ponta, f, o, assentado);
                    leva(&mut g.susp[i], filme, c.filme.v);
                    leva(&mut g.sett[i], aberto, c.aberto.v);
                    // E10c: a tela perde a MESMA fracção do lado espectral (a ponta ainda não o leva: §6).
                    let (pf, pa) = ((i, FILME), (i, ABERTO));
                    crate::grid::tinta::pigmento::move_em(g, pf, None, f);
                    crate::grid::tinta::pigmento::move_em(g, pa, None, f * o * assentado);
                }
                if pig > 0.0 {
                    let mistura = devolvido.junta(
                        Soma::de(base, (pig as f32) / M_UM).com_meio(p.k(Knob::Medium) as f32),
                    );
                    let novo = pig + f64::from(devolvido.v * M_UM);
                    self.pig[l] = novo as f32;
                    if let Some(rgb) = srgb_0_255(&mistura) {
                        (self.tip_r[l], self.tip_g[l], self.tip_b[l]) = (rgb[0], rgb[1], rgb[2]);
                    }
                    if com_limpo {
                        self.limpo[l] = (pig / novo) as f32;
                    }
                }
            }
        }
    }

    /// O reservatório na lei nova (o par de `mistura_o_reservatorio`): cada texel da pegada (`pig > 0`) passa a
    /// MÉDIA das somas da pegada — soma ÷ n em cada componente extensiva, logo o total da ponta conserva-se.
    fn mistura_as_somas_da_ponta(&mut self) {
        let (lx0, lx1, size) = (self.lx0, self.lx1, self.size);
        let pegada: Vec<usize> = (self.ly0..=self.ly1)
            .flat_map(|ly| (lx0..=lx1).map(move |lx| (lx + ly * size) as usize))
            .filter(|&l| self.pig.get(l).is_some_and(|&v| v > 0.0) && l < self.ponta_tinta.len())
            .collect();
        if pegada.is_empty() {
            return;
        }
        let total = pegada
            .iter()
            .fold(Soma::default(), |a, &l| a.junta(self.ponta_tinta[l]));
        let k = 1.0 / pegada.len() as f32;
        let media = total.escala(k);
        for l in pegada {
            self.ponta_tinta[l] = media;
        }
    }
}

/// A massa da fatia segue o volume da soma (o invariante `v · M_UM == massa`): a mesma fracção.
#[inline]
fn leva(massa: &mut f32, antes: f32, depois: f32) {
    if antes > 0.0 {
        *massa = (f64::from(*massa) * f64::from(depois) / f64::from(antes)) as f32;
    }
}

#[cfg(test)]
#[path = "bico_tinta_tests.rs"]
mod tests;
