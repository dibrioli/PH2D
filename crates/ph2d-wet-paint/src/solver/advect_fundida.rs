//! **A advecção FUNDIDA por blocos de linhas** (rodada 7, 31/10 —
//! `docs/Painter/49_realshader/MEDIDA_31_10_RODADA7.md`).
//!
//! As fases `prepare → outflow → transport` do gather são as MESMAS funções por linha
//! ([`super::prepare_row`], [`super::outflow_row`], [`super::transport_row`]); aqui cada bloco de
//! [`BLOCO_LINHAS`] linhas corre as três seguidas, com `uv`/`bt`/`outflow` em planos PEQUENOS por fio
//! (ficam na cache) em vez de planos da grelha inteira que cada fase escrevia e a seguinte relia da RAM.
//!
//! O halo deriva do alcance `R` = [`super::destination_reach`]: o transporte de `y` lê a saída das
//! linhas `y0, y0+1` com `|y − y0| ≤ R` ⇒ saída em `[a−R, b+R]` (até a linha de borda `h+1`, zero); a saída de `y` lê `uv`/`bt` das linhas
//! `[y−R, y+R]` ⇒ `uv`/`bt` em `[a−2R, b+2R]`, recortado à faixa `[by0, by1]`. Fora da faixa a rota de
//! fases lia o rascunho velho, que só multiplica massa zero (ver o módulo pai); aqui conta zero.
//! O `momentum` fica fora: escreve só `vel_x/vel_y`, que nenhuma das três lê.

use super::{Ctx, outflow_row, prepare_row, split, transport_row};
use crate::grid::{Grid, SolverScratch};
use crate::par::Rows;
use crate::sim::Params;
use rayon::prelude::*;
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

/// `> 0` força a rota de FASES (os gates que hasheiam o rascunho da grelha inteira incrementam e
/// decrementam — um contador, para não se atropelarem); `0` = a fundida, o produto.
#[doc(hidden)]
pub static ADVECT_FASES: AtomicUsize = AtomicUsize::new(0);
/// Linhas por bloco da rota fundida: 16 (rodada 7, 16 fios, mínimo de 8 rondas: 16 → 50,9 · 32 → 52,5 · 64 → 54,2 ms/passo).
#[doc(hidden)]
pub static BLOCO_LINHAS: AtomicUsize = AtomicUsize::new(16);

/// Os planos do bloco, um conjunto por fio, reutilizados de passo a passo.
#[derive(Default)]
struct Blocos {
    uv: Vec<[f32; 2]>,
    bt: Vec<i8>,
    of: Vec<f32>,
}

thread_local! {
    static BLOCOS: RefCell<Blocos> = RefCell::new(Blocos::default());
}

/// A rota fundida corre em `Serial`/`Parallel`; a `Medida` (janelas médias) fica nas fases.
pub(super) fn usa_fundida(mode: Rows) -> bool {
    ADVECT_FASES.load(Relaxed) == 0 && !matches!(mode, Rows::Medida(_))
}

type Bloco<'a> = (usize, ((&'a mut [f32], &'a mut [f32]), &'a mut [[f32; 3]]));

/// `prepare + outflow + transport` por blocos; devolve o `vmax` (o mesmo do [`super::prepare_rows`]).
pub(super) fn fused_rows(g: &mut Grid, p: &Params, mode: Rows, linhas: usize) -> f64 {
    let (c, scratch) = split(g, p);
    let s = c.s;
    let n = linhas.max(1) * s;
    let band = c.by0 as usize * s..(c.by1 as usize + 1) * s;
    let SolverScratch {
        dst_film,
        dst_susp,
        dst_rgb,
        ..
    } = scratch;
    let job = |(k, ((fr, sr), cr)): Bloco| {
        let a = c.by0 + (k * n / s) as i32;
        BLOCOS.with(|t| bloco(&c, a, fr, sr, cr, &mut t.borrow_mut()))
    };
    match mode {
        Rows::Serial => dst_film[band.clone()]
            .chunks_mut(n)
            .zip(dst_susp[band.clone()].chunks_mut(n))
            .zip(dst_rgb[band].chunks_mut(n))
            .enumerate()
            .map(job)
            .fold(0.0f64, f64::max),
        _ => dst_film[band.clone()]
            .par_chunks_mut(n)
            .zip(dst_susp[band.clone()].par_chunks_mut(n))
            .zip(dst_rgb[band].par_chunks_mut(n))
            .enumerate()
            .map(job)
            .reduce(|| 0.0f64, f64::max),
    }
}

/// Um bloco: as linhas `a..a + fr.len()/s` do destino.
fn bloco(
    c: &Ctx,
    a: i32,
    fr: &mut [f32],
    sr: &mut [f32],
    cr: &mut [[f32; 3]],
    t: &mut Blocos,
) -> f64 {
    let s = c.s;
    let b = a + (fr.len() / s) as i32 - 1;
    let r = c.reach;
    let (o_lo, o_hi) = ((a - r).max(1), (b + r).min(c.gh + 1));
    let (u_lo, u_hi) = ((o_lo - r).max(c.by0), (o_hi + r).min(c.by1));
    let nu = (u_hi - u_lo + 1) as usize * s;
    let no = (o_hi - o_lo + 1) as usize * s;
    if t.uv.len() < nu {
        t.uv.resize(nu, [0.0; 2]);
        t.bt.resize(nu, SolverScratch::BT_NADA);
    }
    if t.of.len() < no {
        t.of.resize(no, 0.0);
    }
    let Blocos { uv, bt, of } = t;
    let (ubase, obase) = (u_lo as usize * s, o_lo as usize * s);
    let mut vmax = 0.0f64;
    for y in u_lo..=u_hi {
        let o = (y - u_lo) as usize * s;
        let v = prepare_row(c, y, &mut uv[o..o + s], &mut bt[o..o + s]);
        if y >= a && y <= b {
            vmax = f64::max(vmax, v);
        }
    }
    for y in o_lo..=o_hi {
        let o = (y - o_lo) as usize * s;
        let row = &mut of[o..o + s];
        if y < c.by0 || y > c.by1 {
            row.fill(0.0);
        } else {
            outflow_row(c, y, row, uv, bt, ubase, u_lo, u_hi);
        }
    }
    for y in a..=b {
        let o = (y - a) as usize * s;
        transport_row(
            c,
            y,
            &mut fr[o..o + s],
            &mut sr[o..o + s],
            &mut cr[o..o + s],
            of,
            obase,
            uv,
            ubase,
        );
    }
    vmax
}

/// A advecção inteira pela rota fundida, com o bloco dado — a porta do gate de identidade (sem depender
/// do [`ADVECT_FASES`] global, que outros gates seguram).
#[cfg(test)]
pub(super) fn advect_fundida_rows(
    g: &mut Grid,
    p: &Params,
    gx: f64,
    gy: f64,
    mode: Rows,
    linhas: usize,
) -> f64 {
    if g.by0 > g.by1 {
        return 0.0;
    }
    g.scratch.ensure(g.cells);
    super::momentum_rows(g, p, gx, gy, mode);
    let vmax = fused_rows(g, p, mode, linhas);
    super::commit_rows(g, mode);
    vmax
}
