//! §6.6 Pressure projection + §6.7 boundaries (child of [`super`] — split for
//! the workspace file-LOC cap; pure code motion, fingerprint-pinned).

use crate::grid::Grid;
use crate::sim::Params;
use crate::tuning::Knob;

/// One cheap Jacobi relaxation toward incompressibility: kills piling-up but
/// leaves the uniform downward drift intact so drips survive. The divergence
/// and pressure scratch fields reuse the transient-flow arrays (rebuilt next
/// frame anyway).
pub fn project(g: &mut Grid, p: &Params) {
    let s = g.s;
    let w = g.w as i32;
    let h = g.h as i32;
    let projection = p.k(Knob::Projection);
    let (gbx0, gbx1, gby0, gby1) = (g.bx0, g.bx1, g.by0, g.by1);
    let spans_on = g.spans_enabled;
    let Grid {
        row_lo,
        row_hi,
        flow_x,
        flow_y,
        vel_x,
        vel_y,
        active,
        ..
    } = g;
    let div = flow_x;
    let prs = flow_y;
    // A faixa desta linha, com a margem de 1 que o estêncil pede.
    let span1 = |y: i32| -> (i32, i32) {
        let (lo, hi) = crate::grid::span_x_of(row_lo, row_hi, spans_on, gbx0, gbx1, y);
        if lo > hi {
            (1, 0)
        } else {
            ((lo - 1).max(0), (hi + 1).min(s as i32 - 1))
        }
    };
    let zy0 = (gby0 - 1).max(0);
    let zy1 = (gby1 + 1).min(h + 1);
    for y in zy0..=zy1 {
        // neighbours of active cells must read 0 — e "vizinhos de células
        // ativas" é exatamente a faixa viva com margem 1: `div`/`prs` só são
        // LIDOS em `i±1` / `i±s` a partir de uma célula ativa (laços 3 e 4),
        // e `active ⊆ faixa`. O que sobra do rascunho fora daí não é lido por
        // ninguém — o `flow` é reconstruído todo frame nas células ativas e o
        // fingerprint da sessão não o inclui, por ser transiente.
        let (zx0, zx1) = span1(y);
        if zx0 > zx1 {
            continue;
        }
        let base = y as usize * s;
        div[base + zx0 as usize..base + zx1 as usize + 1].fill(0.0);
        prs[base + zx0 as usize..base + zx1 as usize + 1].fill(0.0);
    }
    for y in gby0..=gby1 {
        let (bx0, bx1) = crate::grid::span_x_of(row_lo, row_hi, spans_on, gbx0, gbx1, y);
        if bx0 > bx1 {
            continue;
        }
        let mut i = bx0 as usize + y as usize * s;
        for _x in bx0..=bx1 {
            if active[i] != 0 {
                div[i] = (vel_x[i - 1] as f64 - vel_x[i + 1] as f64 + vel_y[i - s] as f64
                    - vel_y[i + s] as f64) as f32;
            }
            i += 1;
        }
    }
    for y in gby0..=gby1 {
        let (bx0, bx1) = crate::grid::span_x_of(row_lo, row_hi, spans_on, gbx0, gbx1, y);
        if bx0 > bx1 {
            continue;
        }
        let mut i = bx0 as usize + y as usize * s;
        for _x in bx0..=bx1 {
            if active[i] != 0 {
                prs[i] = ((div[i] as f64
                    + 0.25
                        * (div[i - 1] as f64
                            + div[i + 1] as f64
                            + div[i - s] as f64
                            + div[i + s] as f64))
                    * 0.25) as f32;
            }
            i += 1;
        }
    }
    for y in gby0..=gby1 {
        let (bx0, bx1) = crate::grid::span_x_of(row_lo, row_hi, spans_on, gbx0, gbx1, y);
        if bx0 > bx1 {
            continue;
        }
        let mut i = bx0 as usize + y as usize * s;
        for x in bx0..=bx1 {
            if active[i] == 0 {
                i += 1;
                continue;
            }
            let mut nvx =
                vel_x[i] as f64 - 0.5 * (prs[i + 1] as f64 - prs[i - 1] as f64) * projection;
            let mut nvy =
                vel_y[i] as f64 - 0.5 * (prs[i + s] as f64 - prs[i - s] as f64) * projection;
            // Guard: a corrected velocity whose back-trace leaves the sheet
            // is zeroed.
            if x as f64 - nvx < 1.0 || x as f64 - nvx > w as f64 {
                nvx = 0.0;
            }
            if y as f64 - nvy < 1.0 || y as f64 - nvy > h as f64 {
                nvy = 0.0;
            }
            vel_x[i] = nvx as f32;
            vel_y[i] = nvy as f32;
            i += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// §6.7 Boundaries — the drain
// ---------------------------------------------------------------------------

/// The drain band is two cells wide on every side: the pad ring plus the
/// first interior ring. Mass that advected onto it is deleted (the drip ran
/// off the paper); wetness reads the faintest non-zero byte; velocities get
/// their tangential component zeroed first, then the normal written OUTWARD
/// at 0.1 (corners end up carrying the normal value). The soft outward bias
/// is what keeps the solver stable at the edge.
pub fn apply_boundaries(g: &mut Grid, velocity_only: bool) {
    let s = g.s;
    let w = g.w;
    let h = g.h;
    let rows = g.rows;
    let top_rows = [0usize, 1];
    let bot_rows = [h, h + 1];
    let left_cols = [0usize, 1];
    let right_cols = [w, w + 1];
    if !velocity_only {
        for y in top_rows.iter().chain(bot_rows.iter()) {
            let b = y * s;
            g.film[b..b + s].fill(0.0);
            g.susp[b..b + s].fill(0.0);
            g.wet[b..b + s].fill(1);
        }
        for y in 0..rows {
            let b = y * s;
            for x in left_cols.iter().chain(right_cols.iter()) {
                g.film[b + x] = 0.0;
                g.susp[b + x] = 0.0;
                g.wet[b + x] = 1;
            }
        }
    }
    // Tangential first, on every band...
    for y in top_rows.iter().chain(bot_rows.iter()) {
        let b = y * s;
        g.vel_x[b..b + s].fill(0.0);
        g.flow_x[b..b + s].fill(0.0);
    }
    for y in 0..rows {
        let b = y * s;
        for x in left_cols.iter().chain(right_cols.iter()) {
            g.vel_y[b + x] = 0.0;
            g.flow_y[b + x] = 0.0;
        }
    }
    // ...then the normal component, written outward.
    for y in top_rows {
        let b = y * s;
        g.vel_y[b..b + s].fill(-0.1);
        g.flow_y[b..b + s].fill(-0.1);
    }
    for y in bot_rows {
        let b = y * s;
        g.vel_y[b..b + s].fill(0.1);
        g.flow_y[b..b + s].fill(0.1);
    }
    for y in 0..rows {
        let b = y * s;
        for x in left_cols {
            g.vel_x[b + x] = -0.1;
            g.flow_x[b + x] = -0.1;
        }
        for x in right_cols {
            g.vel_x[b + x] = 0.1;
            g.flow_x[b + x] = 0.1;
        }
    }
}
