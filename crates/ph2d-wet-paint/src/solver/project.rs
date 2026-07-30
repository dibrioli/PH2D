//! §6.6 Pressure projection + §6.7 boundaries (child of [`super`] — split for
//! the workspace file-LOC cap; pure code motion, fingerprint-pinned).

use crate::grid::Grid;
use crate::par::{self, Rows};
use crate::sim::Params;
use crate::tuning::Knob;

/// One cheap Jacobi relaxation toward incompressibility: kills piling-up but
/// leaves the uniform downward drift intact so drips survive. The divergence
/// and pressure scratch fields reuse the transient-flow arrays (rebuilt next
/// frame anyway).
pub fn project(g: &mut Grid, p: &Params) {
    let (fbx0, fby0, fbx1, fby1) = crate::flow::flow_bbox((g.bx0, g.by0, g.bx1, g.by1), g.flow.rf);
    let rows = (fby1 - fby0 + 1).max(0) as usize;
    let span = (fbx1 - fbx0 + 1).max(0) as usize;
    project_rows(g, p, Rows::pick(rows, span, par::MIN_CELLS_JACOBI));
}

/// [`project`] com a rota de caminhada FORÇADA — a porta dos gates de
/// identidade (ADR-0145). O produto chama sempre o [`project`].
///
/// **Os quatro laços são row-paralelos e o motivo é o mesmo em todos: cada um
/// lê um buffer e escreve OUTRO** (é o que faz deste passe um Jacobi, e não um
/// Gauss-Seidel). Laço a laço: o rascunho é zerado (escreve `div`/`prs`, não lê
/// nada) · a divergência lê `vel` e escreve `div` · a pressão lê `div` e
/// escreve `prs` · a correção lê `prs` e escreve `vel` **no próprio índice**.
/// `active` é só lido. Nenhuma redução, nenhum transcendental, nenhum RNG.
pub fn project_rows(g: &mut Grid, p: &Params, mode: Rows) {
    let fine_s = g.s;
    let (gw, gh) = (g.w as i32, g.h as i32);
    let geom = g.flow;
    let s = geom.s;
    // ⚠️ O guard de back-trace mede a folha em células de FLUXO, porque `x`/`y`
    // aqui são coordenadas de fluxo — mas a velocidade está em células FINAS
    // por frame, então ela cruza a mesma escala do resto: `nvx` fino sobre uma
    // grade `rf` vezes menor anda `rf` vezes mais em índice.
    let w = geom.w as i32;
    let h = geom.h as i32;
    let projection = p.k(Knob::Projection);
    // A regra de unidade: as DUAS diferenças finitas deste passe (a divergência
    // e o gradiente de pressão) são tomadas sobre vizinhos de FLUXO.
    let dif = crate::flow::diff_scale(geom.rf);
    let (fbx0, fby0, fbx1, fby1) = crate::flow::flow_bbox((g.bx0, g.by0, g.bx1, g.by1), geom.rf);
    let (gbx0, gbx1, gby0, gby1) = (fbx0, fbx1, fby0, fby1);
    g.refresh_flow_spans();
    let spans_on = g.spans_enabled;
    let Grid {
        frow_lo,
        frow_hi,
        flow_x,
        flow_y,
        vel_x,
        vel_y,
        active,
        ..
    } = g;
    let div = flow_x;
    let prs = flow_y;
    let row_lo: &[i32] = frow_lo;
    let row_hi: &[i32] = frow_hi;
    let active: &[u8] = active;
    // A faixa desta linha, com a margem de 1 que o estêncil pede.
    let span1 = |y: i32| -> (i32, i32) {
        let (lo, hi) = crate::grid::span_x_of(row_lo, row_hi, spans_on, gbx0, gbx1, y);
        if lo > hi {
            (1, 0)
        } else {
            ((lo - 1).max(0), (hi + 1).min(s as i32 - 1))
        }
    };
    let live =
        |y: i32| -> (i32, i32) { crate::grid::span_x_of(row_lo, row_hi, spans_on, gbx0, gbx1, y) };
    let zy0 = (gby0 - 1).max(0);
    let zy1 = (gby1 + 1).min(h + 1);
    // ⚠️ A faixa VAZIA é `y1 < y0`, e ela acontece: `empty_bbox` deixa
    // `by1 = -1`, então os laços 2-4 recebem `0..=-1`. Um `for` sobre isso é um
    // no-op, mas `y1 as usize` sobre `-1` é um `usize` gigante — o range TEM de
    // ser construído vazio antes da conversão.
    let band = |y0: i32, y1: i32| -> std::ops::Range<usize> {
        if y1 < y0 {
            return 0..0;
        }
        y0 as usize * s..(y1 as usize + 1) * s
    };
    {
        let z = band(zy0, zy1);
        // neighbours of active cells must read 0 — e "vizinhos de células
        // ativas" é exatamente a faixa viva com margem 1: `div`/`prs` só são
        // LIDOS em `i±1` / `i±s` a partir de uma célula ativa (laços 3 e 4),
        // e `active ⊆ faixa`. O que sobra do rascunho fora daí não é lido por
        // ninguém — o `flow` é reconstruído todo frame nas células ativas e o
        // fingerprint da sessão não o inclui, por ser transiente.
        par::walk_rows2(mode, &mut div[z.clone()], &mut prs[z], s, |r, dr, pr| {
            let y = zy0 + r as i32;
            let (zx0, zx1) = span1(y);
            if zx0 > zx1 {
                return;
            }
            dr[zx0 as usize..=zx1 as usize].fill(0.0);
            pr[zx0 as usize..=zx1 as usize].fill(0.0);
        });
    }
    {
        let velx: &[f32] = vel_x;
        let vely: &[f32] = vel_y;
        par::walk_rows(mode, &mut div[band(gby0, gby1)], s, |r, dr| {
            let y = gby0 + r as i32;
            let (bx0, bx1) = live(y);
            if bx0 > bx1 {
                return;
            }
            let base = y as usize * s;
            for x in bx0..=bx1 {
                let i = x as usize + base;
                if active[crate::flow::probe_idx(x, y, gw, gh, fine_s, geom.rf)] != 0 {
                    dr[x as usize] = ((velx[i - 1] as f64 - velx[i + 1] as f64
                        + vely[i - s] as f64
                        - vely[i + s] as f64)
                        * dif) as f32;
                }
            }
        });
    }
    {
        let div_ro: &[f32] = div;
        par::walk_rows(mode, &mut prs[band(gby0, gby1)], s, |r, pr| {
            let y = gby0 + r as i32;
            let (bx0, bx1) = live(y);
            if bx0 > bx1 {
                return;
            }
            let base = y as usize * s;
            for x in bx0..=bx1 {
                let i = x as usize + base;
                // A relaxação de Jacobi é uma MÉDIA ponderada, adimensional —
                // ela não leva o fator de diferença finita.
                if active[crate::flow::probe_idx(x, y, gw, gh, fine_s, geom.rf)] != 0 {
                    pr[x as usize] = ((div_ro[i] as f64
                        + 0.25
                            * (div_ro[i - 1] as f64
                                + div_ro[i + 1] as f64
                                + div_ro[i - s] as f64
                                + div_ro[i + s] as f64))
                        * 0.25) as f32;
                }
            }
        });
    }
    {
        let prs_ro: &[f32] = prs;
        let b = band(gby0, gby1);
        par::walk_rows2(
            mode,
            &mut vel_x[b.clone()],
            &mut vel_y[b],
            s,
            |r, rx, ry| {
                let y = gby0 + r as i32;
                let (bx0, bx1) = live(y);
                if bx0 > bx1 {
                    return;
                }
                let base = y as usize * s;
                for x in bx0..=bx1 {
                    let i = x as usize + base;
                    if active[crate::flow::probe_idx(x, y, gw, gh, fine_s, geom.rf)] == 0 {
                        continue;
                    }
                    let k = x as usize;
                    let mut nvx = rx[k] as f64
                        - 0.5 * (prs_ro[i + 1] as f64 - prs_ro[i - 1] as f64) * dif * projection;
                    let mut nvy = ry[k] as f64
                        - 0.5 * (prs_ro[i + s] as f64 - prs_ro[i - s] as f64) * dif * projection;
                    // Guard: a corrected velocity whose back-trace leaves the
                    // sheet is zeroed.
                    if x as f64 - nvx < 1.0 || x as f64 - nvx > w as f64 {
                        nvx = 0.0;
                    }
                    if y as f64 - nvy < 1.0 || y as f64 - nvy > h as f64 {
                        nvy = 0.0;
                    }
                    rx[k] = nvx as f32;
                    ry[k] = nvy as f32;
                }
            },
        );
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
    // ⚠️ As duas metades deste passe moram em GRADES DIFERENTES: a massa
    // (film/susp/wet) é FINA, a velocidade é de FLUXO. Em `rf = 1` as duas
    // geometrias coincidem e o código é o que sempre rodou.
    if !velocity_only {
        let s = g.s;
        let (w, h, rows) = (g.w, g.h, g.rows);
        for y in [0usize, 1, h, h + 1] {
            let b = y * s;
            g.film[b..b + s].fill(0.0);
            g.susp[b..b + s].fill(0.0);
            g.wet[b..b + s].fill(1);
        }
        for y in 0..rows {
            let b = y * s;
            for x in [0usize, 1, w, w + 1] {
                g.film[b + x] = 0.0;
                g.susp[b + x] = 0.0;
                g.wet[b + x] = 1;
            }
        }
    }
    let geom = g.flow;
    let s = geom.s;
    let (w, h, rows) = (geom.w, geom.h, geom.rows);
    let top_rows = [0usize, 1];
    let bot_rows = [h, h + 1];
    let left_cols = [0usize, 1];
    let right_cols = [w, w + 1];
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
