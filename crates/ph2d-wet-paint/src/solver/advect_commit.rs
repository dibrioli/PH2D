//! O *commit* do [`super`] (o rascunho vira o grid) e o caminhante de dois planos com redução — aqui
//! pelo tecto de LOC do pai.

use super::*;
use crate::grid::faixa_tiles::{span_runs, tiles_row_of};

/// **O rascunho vira o grid.** Com a faixa viva a cobrir quase a folha (o wash), TROCA os três planos com o
/// rascunho e copia de volta só o COMPLEMENTO (o que a faixa não reescreveu); senão copia a faixa. As duas
/// rotas dão o mesmo grid ao bit (`flow_bit_tests`, `advect_bit_tests`) — o rascunho é derivado.
/// A rota lê [`crate::flow::COMMIT_ROTA`] (0 = pela faixa, o produto; 1/2 forçam, gates e sonda).
pub(super) fn commit_rows(g: &mut Grid, mode: Rows) {
    let s = g.s;
    let (tw, tiles_on) = (g.tile_words, g.spans_enabled && g.tiles_enabled);
    let (by0, by1) = (g.by0, g.by1);
    let viva: usize = (by0..=by1)
        .map(|y| {
            let (lo, hi) = span_x_of(&g.row_lo, &g.row_hi, g.spans_enabled, g.bx0, g.bx1, y);
            // Linha sem faixa: as sentinelas `i32::MAX`/`MIN` fazem `hi − lo` transbordar.
            if lo > hi { 0 } else { (hi - lo + 1) as usize }
        })
        .sum();
    let auto = (g.cells - viva) * 4 <= viva;
    let rota = crate::flow::COMMIT_ROTA.load(std::sync::atomic::Ordering::Relaxed);
    if !match rota {
        1 => false,
        2 => true,
        _ => auto,
    } {
        commit_copia(g, mode);
        return;
    }
    let Grid {
        scratch,
        film,
        susp,
        susp_rgb,
        row_lo,
        row_hi,
        row_tiles,
        spans_enabled,
        bx0,
        bx1,
        ..
    } = g;
    std::mem::swap(film, &mut scratch.dst_film);
    std::mem::swap(susp, &mut scratch.dst_susp);
    std::mem::swap(susp_rgb, &mut scratch.dst_rgb);
    let (spans_enabled, bx0, bx1) = (*spans_enabled, *bx0, *bx1);
    let row_tiles: &[u64] = row_tiles;
    let (of, os, oc): (&[f32], &[f32], &[[f32; 3]]) =
        (&scratch.dst_film, &scratch.dst_susp, &scratch.dst_rgb);
    par::walk_rows3(mode, film, susp, susp_rgb, s, |r, fr, sr, cr| {
        let y = r as i32;
        let base = r * s;
        let (lo, hi) = if y < by0 || y > by1 {
            (1, 0)
        } else {
            span_x_of(row_lo, row_hi, spans_enabled, bx0, bx1, y)
        };
        let mut volta = |a: usize, b: usize| {
            fr[a..b].copy_from_slice(&of[base + a..base + b]);
            sr[a..b].copy_from_slice(&os[base + a..base + b]);
            cr[a..b].copy_from_slice(&oc[base + a..base + b]);
        };
        // O COMPLEMENTO dos troços que o advect reescreveu.
        let mut ate = 0usize;
        if lo <= hi {
            for (a, b) in span_runs(tiles_row_of(row_tiles, tw, tiles_on, y), lo, hi) {
                volta(ate, a as usize);
                ate = b as usize + 1;
            }
        }
        volta(ate, s);
    });
}

/// A rota de cópia do [`commit_rows`]: três cópias contíguas de plano por linha da faixa.
fn commit_copia(g: &mut Grid, mode: Rows) {
    let s = g.s;
    let (tw, tiles_on) = (g.tile_words, g.spans_enabled && g.tiles_enabled);
    let (by0, by1) = (g.by0, g.by1);
    let Grid {
        scratch,
        film,
        susp,
        susp_rgb,
        row_lo,
        row_hi,
        row_tiles,
        spans_enabled,
        bx0,
        bx1,
        ..
    } = g;
    let (spans_enabled, bx0, bx1) = (*spans_enabled, *bx0, *bx1);
    let row_tiles: &[u64] = row_tiles;
    let (df, ds, dc): (&[f32], &[f32], &[[f32; 3]]) =
        (&scratch.dst_film, &scratch.dst_susp, &scratch.dst_rgb);
    let band = by0 as usize * s..(by1 as usize + 1) * s;
    par::walk_rows3(
        mode,
        &mut film[band.clone()],
        &mut susp[band.clone()],
        &mut susp_rgb[band],
        s,
        |ri, fr, sr, cr| {
            let y = by0 + ri as i32;
            let (lo, hi) = span_x_of(row_lo, row_hi, spans_enabled, bx0, bx1, y);
            for (lo, hi) in span_runs(tiles_row_of(row_tiles, tw, tiles_on, y), lo, hi) {
                let (lo, hi) = (lo as usize, hi as usize);
                let (a, b) = (y as usize * s + lo, y as usize * s + hi);
                fr[lo..=hi].copy_from_slice(&df[a..=b]);
                sr[lo..=hi].copy_from_slice(&ds[a..=b]);
                cr[lo..=hi].copy_from_slice(&dc[a..=b]);
            }
        },
    );
}

/// [`par::walk_rows_reduce`] sobre DOIS planos (a mesma lei da redução) — aqui e não em `par` pelo tecto
/// de LOC daquele ficheiro.
pub(super) fn walk_rows2_reduce<T: Send, U: Send, R: Send + Sync + Copy>(
    mode: Rows,
    a: &mut [T],
    b: &mut [U],
    stride: usize,
    ident: R,
    f: impl Fn(usize, &mut [T], &mut [U]) -> R + Send + Sync,
    red: impl Fn(R, R) -> R + Send + Sync,
) -> R {
    debug_assert_eq!(a.len(), b.len(), "os dois planos tem faixas diferentes");
    let g = |(r, (ra, rb)): (usize, (&mut [T], &mut [U]))| f(r, ra, rb);
    match mode {
        Rows::Serial => a
            .chunks_mut(stride)
            .zip(b.chunks_mut(stride))
            .enumerate()
            .map(g)
            .fold(ident, &red),
        Rows::Parallel => a
            .par_chunks_mut(stride)
            .zip(b.par_chunks_mut(stride))
            .enumerate()
            .map(g)
            .reduce(|| ident, &red),
        Rows::Medida(piso) => par::medida_reduce(
            a.chunks_mut(stride)
                .zip(b.chunks_mut(stride))
                .enumerate()
                .collect(),
            piso,
            ident,
            g,
            red,
        ),
    }
}
