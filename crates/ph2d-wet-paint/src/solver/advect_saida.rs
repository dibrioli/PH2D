//! **A saída (`outflow`) por BLOCO de linhas** (B13.6): o [`super::outflow_row`] varre, para cada linha de
//! saída `y`, as `2R+1` linhas-destino da vizinhança e descarta pelo `bt` as que não puxam dela — cada destino
//! é lido `2R+1` vezes e retro-traçado duas. Aqui um bloco de linhas de saída varre cada linha-destino UMA vez
//! (mais o halo de `R` de cada lado), retro-traça cada destino uma vez e deposita nas duas linhas-fonte
//! `y0`, `y0 + 1` que estiverem no bloco.
//!
//! ⚠️ **Ao bit:** numa linha de saída `y` os depósitos chegam na MESMA ordem da rota por linha (linha-destino
//! crescente, depois coluna), com os mesmos filtros (`|ny − y| ≤ R`, a faixa de `y`, `wy ≠ 0`) e a mesma
//! aritmética — a soma em `f32` é a mesma sequência. Os blocos são disjuntos em linhas de saída: o paralelo é
//! o mesmo particionamento por linhas da rota por linha, só mais grosso. Gate
//! `a_saida_por_bloco_da_a_saida_por_linha_ao_bit`.

use super::{Ctx, backtrace, corner_weight};
use crate::grid::SolverScratch;
use crate::par::Rows;
use rayon::prelude::*;
use std::sync::atomic::AtomicUsize;

/// Linhas de saída por bloco; `0` = a rota por linha de antes (sondas e o oráculo). Produto: 16.
#[doc(hidden)]
pub static SAIDA_BLOCO: AtomicUsize = AtomicUsize::new(16);

/// As linhas `ya..=yb` da saída em `out` (cada uma com `s` colunas); `uv`/`bt` guardam as linhas
/// `ny_lo..=ny_hi` a partir do índice global `ubase`. Mesmo contrato do [`super::outflow_row`] linha a linha.
#[allow(clippy::too_many_arguments)]
pub(super) fn outflow_block(
    c: &Ctx,
    ya: i32,
    out: &mut [f32],
    uv: &[[f32; 2]],
    bt: &[i8],
    ubase: usize,
    ny_lo: i32,
    ny_hi: i32,
) {
    let s = c.s;
    let yb = ya + (out.len() / s) as i32 - 1;
    let spans: Vec<(i32, i32)> = (ya..=yb).map(|y| c.span(y)).collect();
    for (k, &(lo, hi)) in spans.iter().enumerate() {
        if lo <= hi {
            let row = &mut out[k * s..(k + 1) * s];
            crate::grid::faixa_tiles::zera_os_trocos(
                row,
                c.runs(ya + k as i32),
                lo as usize,
                hi as usize,
            );
        }
    }
    for ny in (ya - c.reach).max(ny_lo)..=(yb + c.reach).min(ny_hi) {
        let rowbase = ny as usize * s - ubase;
        // As linhas de saída que este destino pode alimentar: no bloco e a `R` dele.
        let (alvo_lo, alvo_hi) = ((ny - c.reach).max(ya), (ny + c.reach).min(yb));
        for nx in c.runs(ny).flat_map(|(a, b)| a..=b) {
            let di = nx as usize + rowbase;
            let d = bt[di];
            if d == SolverScratch::BT_NADA {
                continue;
            }
            let yb0 = ny - i32::from(d);
            if yb0 + 1 < alvo_lo || yb0 > alvo_hi {
                continue;
            }
            let uvd = uv[di];
            let Some((x0, fx, y0, fy)) =
                backtrace(nx, ny, f64::from(uvd[0]), f64::from(uvd[1]), c.wf, c.hf)
            else {
                continue;
            };
            for y in [yb0, yb0 + 1] {
                if y < alvo_lo || y > alvo_hi {
                    continue;
                }
                let k = (y - ya) as usize;
                let (lo, hi) = spans[k];
                if lo > hi {
                    continue;
                }
                let wy = corner_weight(i64::from(y), y0, fy);
                if wy == 0.0 {
                    continue;
                }
                let row = &mut out[k * s..(k + 1) * s];
                let c0 = x0 as usize;
                if x0 >= lo as i64 && x0 <= hi as i64 {
                    row[c0] += ((1.0 - fx) * wy) as f32;
                }
                if x0 >= lo as i64 - 1 && x0 < hi as i64 {
                    row[c0 + 1] += (fx * wy) as f32;
                }
            }
        }
    }
}

/// A saída da faixa `[by0, by1]` por blocos de `linhas`, na rota `mode`.
pub(super) fn outflow_rows_bloco(
    c: &Ctx,
    out: &mut [f32],
    uv: &[[f32; 2]],
    bt: &[i8],
    mode: Rows,
    linhas: usize,
) {
    let n = linhas.max(1) * c.s;
    let itens: Vec<(usize, &mut [f32])> = out.chunks_mut(n).enumerate().collect();
    let job = |(k, o): (usize, &mut [f32])| {
        let ya = c.by0 + (k * n / c.s) as i32;
        outflow_block(c, ya, o, uv, bt, 0, c.by0, c.by1);
        0.0f64
    };
    match mode {
        Rows::Serial => itens.into_iter().for_each(|it| {
            job(it);
        }),
        Rows::Parallel => itens.into_par_iter().for_each(|it| {
            job(it);
        }),
        Rows::Medida(piso) => {
            crate::par::medida_reduce(itens, piso, 0.0f64, job, f64::max);
        }
    }
}
