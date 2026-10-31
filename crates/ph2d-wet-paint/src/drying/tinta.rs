//! O TEMPO pela lei única da tinta (F4-2 E8, ADR-0197; `docs/Painter/Arquitetura/E8_tempo.md`): com
//! [`crate::sim::LeiDaTinta::Pigmento`] assentar move a fracção da SOMA que a massa move (sem a inversa de
//! Newton: o ecrã junta filme e aberto, logo não muda), re-molhar devolve-a pela abertura `o`, e o aberto
//! cura `o ← o·e^{−Δt/τ(m)}` até FECHAR sobre `r_g`. Invariante: `filme.v·M_UM == susp`, `aberto.v·M_UM == sett`.

use super::edge_window::EdgePlane;
use super::{DryConst, dry_cell};
use crate::grid::Grid;
use crate::grid::tinta::{CelulaTinta, srgb_0_255};
use ph2d_paint_tinta::{Bandas, Soma, camada, fecha, optica_celula, s_fator, transfere};
use rayon::prelude::*;

/// A cura do óleo puro (`m = 1`), em segundos do relógio da simulação (F2 §2: «τ(1) ≈ horas», escala artística).
pub const TAU_OLEO_S: f32 = 3600.0;

/// Abaixo desta abertura o aberto FECHA (`e^{−4,6}`: o pick-up `k_u·o` já não leva 1 %).
pub const O_FECHA: f32 = 0.01;

/// De quantos passos (40 Hz) em quantos o relógio da cura anda: a cura varre a grade INTEIRA (a tinta seca fora
/// da caixa molhada também cura) e `e^{−a}·e^{−b} = e^{−(a+b)}` deixa juntar o tempo.
pub const PASSOS_DA_CURA: u64 = 40;

/// `τ(m)`: só o veículo que OXIDA (a fracção `m` de óleo) cura, à taxa `m/τ_óleo`. A goma da aguarela re-dissolve
/// sempre ⇒ `τ(0) = ∞` (a aguarela não fecha; levantá-la seca é a lei dela).
#[must_use]
pub fn tau(m: f32) -> f32 {
    if m > 0.0 {
        TAU_OLEO_S / m
    } else {
        f32::INFINITY
    }
}

/// A abertura do aberto que recebe `chega` de volume fresco (`o = 1`): média pelo volume (`o = 1` fica `1` ao bit).
#[inline]
fn abre_pelo_volume(c: &mut CelulaTinta, antes: f32, chega: f32) {
    let v = antes + chega;
    if v > 0.0 {
        c.o += (1.0 - c.o) * (chega / v);
    }
}

/// Assentar: a fracção `f` do filme passa ao aberto (a mesma que `susp → sett`); o espelho `sett_rgb` segue.
#[inline]
pub(crate) fn assenta(c: &mut CelulaTinta, f: f64, sett_rgb: &mut [f32; 3]) {
    let antes = c.aberto.v;
    let movido = transfere(&mut c.filme, &mut c.aberto, f as f32);
    abre_pelo_volume(c, antes, movido.v);
    if let Some(rgb) = srgb_0_255(&c.aberto) {
        *sett_rgb = rgb;
    }
}

/// Re-molhar (a porta de [`super::lift_settled`] na lei nova): a fracção `b·o` do aberto volta ao filme, na MASSA
/// com a aritmética de `lift_settled` e nas somas por [`transfere`]. Com `o = 1` a massa é a da lei velha ao bit.
#[inline]
pub(crate) fn levanta(
    c: &mut CelulaTinta,
    b: f64,
    susp: &mut f32,
    sett: &mut f32,
    susp_rgb: &mut [f32; 3],
) {
    let b = b * f64::from(c.o);
    let st = f64::from(*sett);
    let lift = st * b;
    *susp = (f64::from(*susp) + lift) as f32;
    *sett = (st - lift) as f32;
    transfere(&mut c.aberto, &mut c.filme, b as f32);
    if let Some(rgb) = srgb_0_255(&c.filme) {
        *susp_rgb = rgb;
    }
}

/// A secagem independente de ordem com as somas: o MESMO [`dry_cell`] de [`super::dry_jacobi`], com a 6.ª faixa
/// (`Grid::tinta`) — o `walk_rows5` não a leva.
pub(super) fn dry_jacobi<P: EdgePlane>(
    g: &mut Grid,
    c: &DryConst,
    mode: crate::par::Rows,
    soft: &[f64],
) {
    let s = g.s;
    let (by0, by1) = (g.by0, g.by1);
    let Grid {
        scratch,
        film,
        susp,
        sett,
        susp_rgb,
        sett_rgb,
        tinta,
        paper,
        vela,
        aparencia,
        oleo,
        row_lo,
        row_hi,
        spans_enabled,
        bx0,
        bx1,
        ..
    } = g;
    let (spans_enabled, bx0, bx1) = (*spans_enabled, *bx0, *bx1);
    let edge: &[u8] = &scratch.edge;
    let (paper, vela): (&[f32], &[f32]) = (paper, vela);
    let aparencia: &[crate::grid::aparencia::AparenciaCelula] = aparencia;
    let oleo: &[crate::grid::oleo_celula::OleoCelula] = oleo;
    let (row_lo, row_hi): (&[i32], &[i32]) = (row_lo, row_hi);
    let band = by0 as usize * s..(by1 as usize + 1) * s;
    let corpo = |ri: usize,
                 fr: &mut [f32],
                 sr: &mut [f32],
                 tr: &mut [f32],
                 cr: &mut [[f32; 3]],
                 dr: &mut [[f32; 3]],
                 kr: &mut [CelulaTinta]| {
        let y = by0 + ri as i32;
        let (lo, hi) = crate::grid::span_x_of(row_lo, row_hi, spans_enabled, bx0, bx1, y);
        if lo > hi {
            return;
        }
        let mut out = [0.0f64; 3];
        let base = y as usize * s;
        for x in lo as usize..=hi as usize {
            if fr[x] <= 0.0 && sr[x] <= 0.0 {
                continue;
            }
            let i = base + x;
            dry_cell(
                c,
                paper[i],
                P::nb(edge, soft, i),
                vela[i],
                crate::grid::aparencia::vidrado(aparencia, i),
                crate::grid::aparencia::opaca(aparencia, i),
                crate::grid::oleo_celula::da_celula(oleo, i),
                &mut fr[x],
                &mut sr[x],
                &mut tr[x],
                &mut cr[x],
                &mut dr[x],
                &mut out,
                Some(&mut kr[x]),
            );
        }
    };
    let linhas: Vec<_> = film[band.clone()]
        .chunks_mut(s)
        .zip(susp[band.clone()].chunks_mut(s))
        .zip(sett[band.clone()].chunks_mut(s))
        .zip(susp_rgb[band.clone()].chunks_mut(s))
        .zip(sett_rgb[band.clone()].chunks_mut(s))
        .zip(tinta[band].chunks_mut(s))
        .collect();
    let serial = matches!(mode, crate::par::Rows::Serial);
    let roda =
        |(ri, (((((fr, sr), tr), cr), dr), kr)): (usize, _)| corpo(ri, fr, sr, tr, cr, dr, kr);
    if serial {
        linhas.into_iter().enumerate().for_each(roda);
    } else {
        linhas.into_par_iter().enumerate().for_each(roda);
    }
}

/// A CURA de `dt_s` segundos em toda a grade: `o ← o·e^{−dt/τ(m)}` onde há aberto; `o < O_FECHA` com a célula SECA
/// (sem água e sem filme — o fecho é então invisível ao bit, `transfere.rs fecha`) ⇒ o aberto vira `r_g`, o volume
/// dele vai para `altura_fechada` e a massa `sett` sai da grade (o Óleo Real soma-a de volta pela
/// `render::tinta::massa_da_altura`).
pub fn cura(g: &mut Grid, dt_s: f32) {
    if g.tinta.is_empty() {
        return;
    }
    let s = g.s;
    let Grid {
        tinta, sett, film, ..
    } = g;
    let film: &[f32] = film;
    let rows = tinta.len() / s;
    let mode = crate::par::Rows::pick_medido(rows, s, crate::par::MIN_CELLS_DRYING);
    crate::par::walk_rows2(mode, tinta, sett, s, |ri, kr, tr| {
        for (x, c) in kr.iter_mut().enumerate() {
            if c.aberto.v <= 0.0 || c.m <= 0.0 {
                continue;
            }
            c.o *= (-dt_s / tau(c.m)).exp();
            let i = ri * s + x;
            if c.o < O_FECHA && film[i] <= 0.0 && c.filme.v <= 0.0 {
                fecha_celula(c, &mut tr[x]);
            }
        }
    });
}

/// O fecho de UMA célula (sem as condições de [`cura`]): o aberto passa a fundo e a massa dele sai de `sett`.
pub(crate) fn fecha_celula(c: &mut CelulaTinta, sett: &mut f32) {
    c.altura_fechada += c.aberto.v;
    let s = s_fator(c.m, false);
    // O fundo sobre o preto fecha com a MESMA óptica (a cobertura do fechado = a de antes, ao bit).
    c.r_g0 = camada(
        &optica_celula(&Soma::default(), &c.aberto, s),
        &Bandas(c.r_g0),
    )
    .0;
    let mut rg = Bandas(c.r_g);
    fecha(&mut rg, &mut c.aberto, s);
    c.r_g = rg.0;
    c.o = 1.0;
    *sett = 0.0;
}

#[cfg(test)]
#[path = "tinta_tests.rs"]
mod tests;
