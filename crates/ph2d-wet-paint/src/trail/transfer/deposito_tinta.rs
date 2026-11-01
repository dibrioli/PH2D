//! F4-2 E6a: o depósito da lei [`LeiDaTinta::Pigmento`](crate::sim::LeiDaTinta): a massa que pousa no `susp`/`sett`
//! entra nas SOMAS da célula ([`crate::grid::tinta`]) com a cor e a transparência da ponta; o que a massa perde, a
//! soma perde pela mesma fracção. A massa segue a lei velha ao bit, salvo a tinta que cobre (R19: `WetCover`/
//! `DryCover`), que não corre — a cobertura é a da óptica. Sem `mix.mix` (R9): misturar é somar.
//! Doc: `docs/Painter/Arquitetura/E6a_deposito.md`.

use super::remolha;
use crate::grid::Grid;
use crate::grid::tinta::{soma_da_massa, srgb_0_255};
use crate::opacity::alpha_of_mass;
use ph2d_paint_tinta::{Soma, transfere};

/// Tira a fracção `1 − fica` da fatia (a massa que sai da grade: o tecto das 3000).
fn tira(s: &mut Soma, fica: f64) {
    if fica < 1.0 {
        transfere(s, &mut Soma::default(), (1.0 - fica.max(0.0)) as f32);
    }
}

/// `susp_rgb`/`sett_rgb` passam a ser o ESPELHO das somas: os leitores ainda da lei velha (o bico, o ecrã antes
/// da E5) vêem a cor misturada em Oklab; ninguém da lei nova os lê.
fn espelha(g: &mut Grid, i: usize) {
    let c = g.tinta[i];
    if let Some(rgb) = srgb_0_255(&c.filme) {
        g.susp_rgb[i] = rgb;
    }
    if let Some(rgb) = srgb_0_255(&c.aberto) {
        g.sett_rgb[i] = rgb;
    }
}

/// O tecto das 3000 no assentado (o mesmo da lei velha), com o aberto a acompanhar.
fn corta_sett(g: &mut Grid, i: usize, shed_pig: f64) {
    let old = f64::from(g.sett[i]);
    if old > 3000.0 && !crate::trail::sonda::sem_teto() {
        let ns = (old - shed_pig).max(0.0);
        g.sett[i] = ns as f32;
        tira(&mut g.tinta[i].aberto, ns / old);
    }
}

/// Os parâmetros de UM transfer (os mesmos de todas as células da janela).
pub(super) struct Pigmento {
    pub vela: f32,
    pub dissolve: f64,
    pub shed_pig: f64,
    pub shed_water: f64,
    pub water_cap: f64,
    pub filme: bool,
    pub mix: crate::colorops::ColorMix,
    pub glaze: f64,
    /// O meio do pincel (`0` água · `1` óleo): o Real Oil até a E10 lhe dar um controlo.
    pub meio: f32,
}

/// O que UMA célula da janela pousa.
pub(super) struct Pouso {
    /// A massa.
    pub v: f64,
    /// A água da janela.
    pub agua: f64,
    /// A cor da ponta, sRGB `0..255`.
    pub ponta: [f32; 3],
    /// A Transparency do material que pousa.
    pub transparencia: f32,
    /// A ponta seca sobre a célula seca: pousa já no assentado.
    pub seco: bool,
    /// O 1.º toque deste traço na tinta molhada de outro (com Glaze).
    pub primeiro: bool,
}

impl Pigmento {
    /// A porta da célula `i` (o par de `if seco { … } else { … }` da lei velha em `transfer_paint_impl`).
    pub(super) fn pousa(
        &self,
        g: &mut Grid,
        i: usize,
        x: &Pouso,
        inicio: &mut remolha::Inicio,
        out: &mut [f64; 3],
    ) {
        let v = x.v;
        let nova = soma_da_massa(x.ponta, x.transparencia, self.meio, v as f32);
        if x.seco {
            let old = f64::from(g.sett[i]);
            let c = &mut g.tinta[i];
            if old <= 0.0 {
                c.aberto = nova;
                g.sett[i] = v as f32;
            } else {
                c.aberto = c.aberto.junta(nova);
                g.sett[i] = (old + v) as f32;
            }
            corta_sett(g, i, self.shed_pig);
            espelha(g, i);
            return;
        }
        let in_a = alpha_of_mass(v);
        if self.dissolve > 0.0 {
            let antes = f64::from(g.sett[i]);
            remolha::remolha(g, i, self.dissolve, in_a, inicio, self.mix, self.glaze, out);
            if antes > 0.0 {
                let f = 1.0 - f64::from(g.sett[i]) / antes;
                let c = &mut g.tinta[i];
                transfere(&mut c.aberto, &mut c.filme, f as f32);
            }
        }
        // O Glaze: a tinta molhada de outro traço assenta (a massa da lei velha; o ecrã não muda: filme e aberto
        // somam-se na óptica).
        if self.vela > 0.0 && g.susp[i] > 0.0 {
            let d = if x.primeiro {
                1.0
            } else {
                crate::colorops::outra_tinta(g.susp_rgb[i].map(f64::from), x.ponta.map(f64::from))
            };
            let dm = f64::from(g.susp[i]) * f64::from(self.vela) * d;
            if dm > 0.0 {
                let f = dm / f64::from(g.susp[i]);
                g.film[i] = (f64::from(g.film[i]) * (1.0 - f)) as f32;
                g.sett[i] = (f64::from(g.sett[i]) + dm) as f32;
                g.susp[i] = (f64::from(g.susp[i]) - dm) as f32;
                let c = &mut g.tinta[i];
                transfere(&mut c.filme, &mut c.aberto, f as f32);
            }
        }
        let old = f64::from(g.susp[i]);
        if old > 0.0 {
            let nm = if old < 3000.0 || crate::trail::sonda::sem_teto() {
                old + v
            } else {
                old + v - self.shed_pig
            };
            g.susp[i] = nm.max(0.0) as f32;
            let c = &mut g.tinta[i];
            c.filme = c.filme.junta(nova);
            tira(&mut c.filme, f64::from(g.susp[i]) / (old + v));
        } else {
            g.susp[i] = v as f32;
            g.tinta[i].filme = nova;
            if !self.filme {
                g.film[i] = (f64::from(g.film[i]) + x.agua) as f32;
            }
        }
        corta_sett(g, i, self.shed_pig);
        let film_i = f64::from(g.film[i]);
        let nf = if film_i < self.water_cap || self.filme {
            film_i + x.agua
        } else {
            film_i + x.agua - self.shed_water
        };
        g.film[i] = (if nf < 0.00001 { 0.00001 } else { nf }) as f32;
        espelha(g, i);
    }
}

/// O arrasto da tinta (`Drag`): as somas da célula `i` vão para as de `si` pela mesma fracção que a massa.
pub(super) fn arrasta(g: &mut Grid, i: usize, si: usize, drag: f64) {
    let d = drag as f32;
    let lerp = |a: f32, b: f32| a + (b - a) * d;
    let fatia = |a: Soma, b: Soma| Soma {
        v: lerp(a.v, b.v).max(0.0),
        lab: [0, 1, 2].map(|k| lerp(a.lab[k], b.lab[k])),
        t: lerp(a.t, b.t),
        m: lerp(a.m, b.m).max(0.0),
    };
    let (a, b) = (g.tinta[i], g.tinta[si]);
    g.tinta[i].filme = fatia(a.filme, b.filme);
    g.tinta[i].aberto = fatia(a.aberto, b.aberto);
    espelha(g, i);
}

/// Lei VELHA. **Pousa `v` de pigmento da ponta SECA na camada assente** da célula `i` — a mesma composição de
/// opacidade e a mesma mistura do pigmento suspenso, sobre `sett`.
pub(super) fn pousa_seco(
    g: &mut Grid,
    i: usize,
    v: f64,
    tip: [f32; 3],
    p: &crate::sim::Params,
    out: &mut [f64; 3],
) {
    let old = g.sett[i] as f64;
    if old <= 0.0 {
        g.sett_rgb[i] = tip;
        g.sett[i] = v as f32;
        return;
    }
    let (in_a, a_old) = (alpha_of_mass(v), alpha_of_mass(old));
    let e9 = a_old * (1.0 - in_a);
    if e9 + in_a > 0.0 {
        let wgt = in_a / (e9 + in_a);
        let uc = g.sett_rgb[i].map(f64::from);
        let tip = tip.map(f64::from);
        p.mix
            .mix(uc[0], uc[1], uc[2], tip[0], tip[1], tip[2], wgt, out);
        crate::colorops::velatura(out, wgt, p.glaze, a_old, uc, tip);
        g.sett_rgb[i] = [out[0] as f32, out[1] as f32, out[2] as f32];
    }
    g.sett[i] = (old + v) as f32;
}

#[cfg(test)]
#[path = "deposito_tinta_tests.rs"]
mod tests;
