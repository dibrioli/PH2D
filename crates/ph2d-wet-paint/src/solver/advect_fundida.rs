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

use super::{Ctx, momentum_row, outflow_row, prepare_row, split_vel, transport_row};
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

/// `> 0` (o produto) deixa o `momentum` num passe à parte; `0` funde-o nos blocos (só `rf = 1`). Medido na
/// rodada 8 (4K, bloco 16, 16 fios): dentro 51,27 × fora 50,30 ms/passo — não ganha; fica atrás do override.
#[doc(hidden)]
pub static MOMENTO_FORA: AtomicUsize = AtomicUsize::new(1);

/// O `momentum` corre dentro dos blocos?
pub(super) fn momento_no_bloco(g: &Grid) -> bool {
    g.flow.is_identity() && MOMENTO_FORA.load(Relaxed) == 0
}

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

/// Células da faixa a partir das quais a fundida ganha (rodada 8, 16 fios, mínimo de 8 rondas, fases ×
/// fundida bloco 16): 10,5 M → 52,1 × 50,3 ms · 2,6 M → 13,03 × 13,16 · 1,3 M → 6,50 × 6,89 · 0,4 M → 2,58 ×
/// 2,73. Abaixo da cache cheia as fases já moram na L3 e o halo recalculado só custa.
pub const MIN_CELULAS_FUNDIDA: usize = 4 << 20;

/// A rota do produto: fundida só nas faixas GRANDES em `Serial`/`Parallel` (a `Medida` é de janela média).
pub(super) fn usa_fundida(mode: Rows, celulas: usize) -> bool {
    ADVECT_FASES.load(Relaxed) == 0
        && celulas >= MIN_CELULAS_FUNDIDA
        && !matches!(mode, Rows::Medida(_))
}

/// Um bloco: índice, os três planos de destino e (grade de fluxo idêntica) as duas linhas de velocidade.
struct Item<'a> {
    k: usize,
    fr: &'a mut [f32],
    sr: &'a mut [f32],
    cr: &'a mut [[f32; 3]],
    vx: &'a mut [f32],
    vy: &'a mut [f32],
}

/// `(momentum +) prepare + outflow + transport` por blocos; devolve o `vmax` (o mesmo do
/// [`super::prepare_rows`]). O `momentum` entra no bloco só com a grade de fluxo IDÊNTICA à fina (`rf = 1`,
/// a do wash): aí a linha de fluxo `y` é escrita pela linha fina `y` e o bloco fino é o bloco de fluxo.
pub(super) fn fused_rows(
    g: &mut Grid,
    p: &Params,
    mode: Rows,
    grav: (f64, f64),
    linhas: usize,
    complemento: bool,
) -> f64 {
    let momento = momento_no_bloco(g);
    let (c, scratch, vel_x, vel_y) = split_vel(g, p);
    let s = c.s;
    let n = linhas.max(1) * s;
    let band = c.by0 as usize * s..(c.by1 as usize + 1) * s;
    let com_momentum = momento && c.fb.0 <= c.fb.2;
    let SolverScratch {
        dst_film,
        dst_susp,
        dst_rgb,
        ..
    } = scratch;
    let mut vxs: Vec<&mut [f32]> = Vec::new();
    let mut vys: Vec<&mut [f32]> = Vec::new();
    if com_momentum {
        debug_assert_eq!(c.geom.s, s);
        vxs = vel_x[band.clone()].chunks_mut(n).collect();
        vys = vel_y[band.clone()].chunks_mut(n).collect();
    }
    let (mut vxs, mut vys) = (vxs.into_iter(), vys.into_iter());
    // `[..by0)`, a faixa, `(by1..]` — as pontas só as copia o complemento ([`super::commit::so_troca`]).
    let (pre_f, resto_f) = dst_film.split_at_mut(band.start);
    let (band_f, pos_f) = resto_f.split_at_mut(band.len());
    let (pre_s, resto_s) = dst_susp.split_at_mut(band.start);
    let (band_s, pos_s) = resto_s.split_at_mut(band.len());
    let (pre_c, resto_c) = dst_rgb.split_at_mut(band.start);
    let (band_c, pos_c) = resto_c.split_at_mut(band.len());
    let itens: Vec<Item> = band_f
        .chunks_mut(n)
        .zip(band_s.chunks_mut(n))
        .zip(band_c.chunks_mut(n))
        .enumerate()
        .map(|(k, ((fr, sr), cr))| Item {
            k,
            fr,
            sr,
            cr,
            vx: vxs.next().unwrap_or_default(),
            vy: vys.next().unwrap_or_default(),
        })
        .collect();
    let job = |it: Item| {
        let a = c.by0 + (it.k * n / s) as i32;
        if com_momentum {
            for (r, (vx, vy)) in it.vx.chunks_mut(s).zip(it.vy.chunks_mut(s)).enumerate() {
                momentum_row(&c, a + r as i32, vx, vy, grav.0, grav.1);
            }
        }
        let v = BLOCOS.with(|t| bloco(&c, a, it.fr, it.sr, it.cr, &mut t.borrow_mut()));
        if complemento {
            let rows = it
                .fr
                .chunks_mut(s)
                .zip(it.sr.chunks_mut(s))
                .zip(it.cr.chunks_mut(s));
            for (r, ((fr, sr), cr)) in rows.enumerate() {
                super::commit::complemento_linha(&c, a + r as i32, fr, sr, cr);
            }
        }
        v
    };
    let nucleo = || match mode {
        Rows::Serial => itens.into_iter().map(job).fold(0.0f64, f64::max),
        Rows::Parallel => itens.into_par_iter().map(job).reduce(|| 0.0f64, f64::max),
        Rows::Medida(piso) => crate::par::medida_reduce(itens, piso, 0.0f64, job, f64::max),
    };
    if !complemento {
        return nucleo();
    }
    // As linhas fora da faixa vão inteiras do grid para o rascunho, ao lado do núcleo.
    let (fim, ini) = (band.end, band.start);
    let mut pontas = || {
        // Pedaços de 64 linhas: a ponta de baixo chega a ~2 M células, um `memcpy` só seria um fio só.
        let k = 64 * s;
        let fora = [
            (&mut *pre_f, &mut *pre_s, &mut *pre_c, 0),
            (&mut *pos_f, &mut *pos_s, &mut *pos_c, fim),
        ];
        for (f, sv, cv, o) in fora {
            f.par_chunks_mut(k)
                .zip(sv.par_chunks_mut(k))
                .zip(cv.par_chunks_mut(k))
                .enumerate()
                .for_each(|(j, ((f, sv), cv))| {
                    let (a, b) = (o + j * k, o + j * k + f.len());
                    f.copy_from_slice(&c.film[a..b]);
                    sv.copy_from_slice(&c.susp[a..b]);
                    cv.copy_from_slice(&c.susp_rgb[a..b]);
                });
        }
        debug_assert_eq!(ini, pre_f.len());
    };
    match mode {
        Rows::Parallel => rayon::join(nucleo, pontas).0,
        _ => {
            pontas();
            nucleo()
        }
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
    if super::saida::SAIDA_BLOCO.load(Relaxed) > 0 {
        let (i_lo, i_hi) = (o_lo.max(c.by0), o_hi.min(c.by1));
        for y in (o_lo..=o_hi).filter(|&y| y < i_lo || y > i_hi) {
            let o = (y - o_lo) as usize * s;
            of[o..o + s].fill(0.0);
        }
        if i_lo <= i_hi {
            let (oa, ob) = ((i_lo - o_lo) as usize * s, (i_hi - o_lo + 1) as usize * s);
            super::saida::outflow_block(c, i_lo, &mut of[oa..ob], uv, bt, ubase, u_lo, u_hi);
        }
    } else {
        for y in o_lo..=o_hi {
            let o = (y - o_lo) as usize * s;
            let row = &mut of[o..o + s];
            if y < c.by0 || y > c.by1 {
                row.fill(0.0);
            } else {
                outflow_row(c, y, row, uv, bt, ubase, u_lo, u_hi);
            }
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
    if !momento_no_bloco(g) {
        super::momentum_rows(g, p, gx, gy, mode);
    }
    let no_bloco = super::commit::commit_no_bloco(g);
    let vmax = fused_rows(g, p, mode, (gx, gy), linhas, no_bloco);
    if no_bloco {
        super::commit::so_troca(g);
    } else {
        super::commit_rows(g, mode);
    }
    vmax
}
