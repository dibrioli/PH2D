//! A velocidade persistente do [`super`] — aqui pelo tecto de LOC do pai.

use super::*;

/// **A velocidade persistente**, na grade de FLUXO: o fluxo amostrado na
/// origem do retro-traço mais a gravidade, escalada pela água daqui.
///
/// Quem escreve uma célula de fluxo é a célula fina **PROBE** dela (plano 30),
/// e cada linha de fluxo tem exatamente uma linha fina probe ⇒ as linhas são
/// disjuntas.
pub(in crate::solver) fn momentum_rows(g: &mut Grid, p: &Params, gx: f64, gy: f64, mode: Rows) {
    let (c, _, vel_x, vel_y) = split_vel(g, p);
    let (_, fby0, _, fby1) = c.fb;
    if fby0 > fby1 || c.fb.0 > c.fb.2 {
        return;
    }
    let band = fby0 as usize * c.geom.s..(fby1 as usize + 1) * c.geom.s;
    par::walk_rows2(
        mode,
        &mut vel_x[band.clone()],
        &mut vel_y[band],
        c.geom.s,
        |r, vx, vy| momentum_row(&c, fby0 + r as i32, vx, vy, gx, gy),
    );
}

/// Uma linha de FLUXO `cy` do [`momentum_rows`]; `vx`/`vy` são a linha inteira. Lê `film`/`active` só na
/// linha fina probe dela e o fluxo no retro-traço — nada do que as outras fases escrevem.
pub(in crate::solver) fn momentum_row(
    c: &Ctx,
    cy: i32,
    vx: &mut [f32],
    vy: &mut [f32],
    gx: f64,
    gy: f64,
) {
    let (fbx0, _, fbx1, _) = c.fb;
    let (s, geom, max_v) = (c.s, &c.geom, c.max_v);
    let py = crate::flow::flow_probe(cy, c.gh, geom.rf);
    if py < c.by0 || py > c.by1 {
        return;
    }
    for (lo, hi) in c.runs(py) {
        // As colunas de fluxo cuja probe pode cair no troço (±1: o pad e o bloco parcial da borda).
        let cx0 = fbx0.max(crate::flow::fine_to_flow(lo, geom.rf) - 1);
        let cx1 = fbx1.min(crate::flow::fine_to_flow(hi, geom.rf) + 1);
        for cx in cx0..=cx1 {
            let px = crate::flow::flow_probe(cx, c.gw, geom.rf);
            if px < lo || px > hi {
                continue;
            }
            let i = px as usize + py as usize * s;
            let col = cx as usize;
            if c.active[i] == 0 {
                vx[col] = 0.0;
                vy[col] = 0.0;
                continue;
            }
            let (ux, uy) = flow_at_cell(c.flow_x, c.flow_y, geom, px, py, i);
            // Sai da folha ⇒ o serial faz `continue` ANTES de escrever a
            // velocidade: uma gota que alcança a borda mantém o momento.
            if backtrace(px, py, ux, uy, c.wf, c.hf).is_none() {
                continue;
            }
            let sx = f64::from(px) - ux;
            let sy = f64::from(py) - uy;
            let f = c.film[i] as f64;
            let (sfx, sfy) = flow_at_point(c.flow_x, c.flow_y, geom, sx, sy);
            let mut nvx = sfx + gx * f;
            let mut nvy = sfy + gy * f;
            if nvx > max_v {
                nvx = max_v;
            } else if nvx < -max_v {
                nvx = -max_v;
            }
            if nvy > max_v {
                nvy = max_v;
            } else if nvy < -max_v {
                nvy = -max_v;
            }
            vx[col] = nvx as f32;
            vy[col] = nvy as f32;
        }
    }
}
