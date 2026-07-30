//! §6.5 Advection + gravity (child of [`super`] — split for the workspace
//! file-LOC cap; pure code motion, the session fingerprint pins byte-identity).

use crate::colorops::km_weighted_mean_color;
use crate::grid::Grid;
use crate::sim::Params;
use crate::tuning::Knob;

/// Semi-Lagrangian CONSERVATIVE GATHER: each cell back-traces along its
/// transient flow, pulls water + suspended pigment from the 4 bilinear source
/// corners (subtracting there, clamped so no corner goes negative), and adds
/// the total here. The destination's suspended color is REPLACED by the
/// incoming mean whenever any mass arrives — a fast rivulet that delivers
/// even a little mass takes the cell's color, which is what makes color
/// fronts move. No caps: fronts pile up (rim / backrun raw material).
/// Gravity lands UNBRAKED in the persistent field, scaled by the local film.
/// Returns the max |velocity component| seen (drives the drying cadence).
pub fn advect(g: &mut Grid, p: &Params, gx: f64, gy: f64) -> f64 {
    let s = g.s;
    let w = g.w as f64;
    let h = g.h as f64;
    let geom = g.flow;
    let (gw, gh) = (g.w as i32, g.h as i32);
    let max_v = p.k(Knob::MaxVelocity);
    let (by0, by1) = (g.by0, g.by1);
    let km_mean = p.km_mixing; // route the incoming mean through K–M
    let mut km_colors = [0.0f64; 12];
    let mut km_weights = [0.0f64; 4];
    let mut km_out = [0.0f64; 3];
    let mut vmax = 0.0f64;
    for y in by0..=by1 {
        // ⚠️ Este é o ÚNICO passe cujo ramo inativo NÃO é um `continue` puro:
        // ele ZERA a velocidade persistente da célula que deixou de ser água.
        // Restringi-lo à faixa viva não é, portanto, byte-idêntico "por
        // construção" como os outros — apoia-se num invariante:
        //
        //   **fora da faixa, `vel` já é zero.**
        //
        // Ele se sustenta porque `vel` só fica ≠ 0 em célula ATIVA (advect e
        // project escrevem lá; o anel é escrito pelo `apply_boundaries`), e
        // uma célula que acabou de sair do ativo continua a ≤1 célula da água
        // que a deixou — dentro da dilatação de [`crate::grid::SPAN_PAD`] = 5,
        // com `maxVelocity` = 0,2 célula/frame e rebuild a cada 2 frames.
        //
        // O invariante é o que o net de debug [`crate::grid::verify_spans`]
        // afirma a cada passo, e o gate diferencial (mesma sessão, faixa ON e
        // OFF, fingerprint idêntico) é o oráculo do produto.
        let (bx0, bx1) = g.span_x(y);
        if bx0 > bx1 {
            continue;
        }
        // ⚠️ **A posse do bloco é caminhada, não dividida.** O 1º corte
        // perguntava `is_probe_cell` por célula, o que são DUAS divisões
        // inteiras no laço mais quente do motor — medido, o `advect` foi a
        // **0,81×** e comeu sozinho o ganho de 10,25× do `build_flow_field`.
        // A linha é fixa dentro do laço e a coluna anda em bloco: uma divisão
        // por LINHA, e o resto é soma e comparação.
        let identity = geom.is_identity();
        let rfi = geom.rf as i32;
        let cy = crate::flow::fine_to_flow(y, geom.rf);
        let probe_row = identity || crate::flow::flow_probe(cy, gh, geom.rf) == y;
        let mut cx = crate::flow::fine_to_flow(bx0, geom.rf);
        let mut blk_end = cx * rfi; // última coluna FINA do bloco `cx`
        let mut px = crate::flow::flow_probe(cx, gw, geom.rf);
        let mut i = bx0 as usize + y as usize * s;
        for x in bx0..=bx1 {
            if x > blk_end {
                cx += 1;
                blk_end = cx * rfi;
                px = crate::flow::flow_probe(cx, gw, geom.rf);
            }
            // ⚠️ **A velocidade mora na grade de FLUXO, e quem a escreve é a
            // célula PROBE do bloco** — a mesma que os passes de fluxo leem
            // (plano 30). Em `rf = 1` toda célula é o próprio probe, então este
            // laço faz exatamente o que sempre fez.
            //
            // A alternativa — extrair o momento para um passe grosso próprio —
            // foi DESCARTADA por byte-identidade: o `advect` é uma varredura
            // SEQUENCIAL cujas escritas de `film` alcançam as células ainda por
            // visitar, então o `f` que a gravidade multiplica depende de onde o
            // laço está. Um passe separado leria o film de ANTES de qualquer
            // advecção, e a rede de segurança de `rf = 1` cairia junto.
            let owns_flow = probe_row && (identity || x == px);
            let fi = if identity { i } else { geom.idx(cx, cy) };
            if g.active[i] == 0 {
                if owns_flow {
                    g.vel_x[fi] = 0.0;
                    g.vel_y[fi] = 0.0;
                }
                i += 1;
                continue;
            }
            let (ux, uy) = crate::flow::flow_at_cell(&g.flow_x, &g.flow_y, &geom, x, y, i);
            let axv = ux.abs();
            let ayv = uy.abs();
            if axv > vmax {
                vmax = axv;
            }
            if ayv > vmax {
                vmax = ayv;
            }
            let sx = x as f64 - ux;
            let sy = y as f64 - uy;
            // A back-trace that leaves the sheet keeps its momentum and moves
            // nothing: a drip reaching the edge must not lose its velocity.
            if sx < 1.0 || sx > w || sy < 1.0 || sy > h {
                i += 1;
                continue;
            }
            let x0 = sx as i64; // positive, so trunc == floor
            let y0 = sy as i64;
            let fx = sx - x0 as f64;
            let fy = sy - y0 as f64;
            let i00 = x0 as usize + y0 as usize * s;
            let i10 = i00 + 1;
            let i01 = i00 + s;
            let i11 = i01 + 1;
            let w00 = (1.0 - fx) * (1.0 - fy);
            let w10 = fx * (1.0 - fy);
            let w01 = (1.0 - fx) * fy;
            let w11 = fx * fy;

            // Persistent velocity = transient flow sampled at the source,
            // then gravity injected (unbraked) scaled by the water here.
            let f = g.film[i] as f64;
            if owns_flow {
                let (sfx, sfy) = crate::flow::flow_at_point(&g.flow_x, &g.flow_y, &geom, sx, sy);
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
                g.vel_x[fi] = nvx as f32;
                g.vel_y[fi] = nvy as f32;
            }

            // Pigment gather (pre-clamp weights drive the incoming color
            // mean). The "reduce p_k by the shortfall" clamp cannot bite
            // here: each pull p_k = susp[corner] * w_k with w_k in [0,1] is
            // computed and subtracted atomically within this cell, so a
            // corner can never go negative.
            let m00 = g.susp[i00] as f64;
            let m10 = g.susp[i10] as f64;
            let m01 = g.susp[i01] as f64;
            let m11 = g.susp[i11] as f64;
            let p00 = m00 * w00;
            let p10 = m10 * w10;
            let p01 = m01 * w01;
            let p11 = m11 * w11;
            let want = p00 + p10 + p01 + p11;
            if want >= 0.00001 {
                let inv = 1.0 / want;
                let (r_in, g_in, b_in);
                let c00 = g.susp_rgb[i00];
                let c10 = g.susp_rgb[i10];
                let c01 = g.susp_rgb[i01];
                let c11 = g.susp_rgb[i11];
                if km_mean {
                    // Pigment-mixing checkbox ON: corner mean in K/S space.
                    km_colors[0] = c00[0] as f64;
                    km_colors[1] = c00[1] as f64;
                    km_colors[2] = c00[2] as f64;
                    km_colors[3] = c10[0] as f64;
                    km_colors[4] = c10[1] as f64;
                    km_colors[5] = c10[2] as f64;
                    km_colors[6] = c01[0] as f64;
                    km_colors[7] = c01[1] as f64;
                    km_colors[8] = c01[2] as f64;
                    km_colors[9] = c11[0] as f64;
                    km_colors[10] = c11[1] as f64;
                    km_colors[11] = c11[2] as f64;
                    km_weights[0] = p00;
                    km_weights[1] = p10;
                    km_weights[2] = p01;
                    km_weights[3] = p11;
                    km_weighted_mean_color(&km_colors, &km_weights, 4, inv, &mut km_out);
                    r_in = km_out[0];
                    g_in = km_out[1];
                    b_in = km_out[2];
                } else {
                    r_in = (c00[0] as f64 * p00
                        + c10[0] as f64 * p10
                        + c01[0] as f64 * p01
                        + c11[0] as f64 * p11)
                        * inv;
                    g_in = (c00[1] as f64 * p00
                        + c10[1] as f64 * p10
                        + c01[1] as f64 * p01
                        + c11[1] as f64 * p11)
                        * inv;
                    b_in = (c00[2] as f64 * p00
                        + c10[2] as f64 * p10
                        + c01[2] as f64 * p01
                        + c11[2] as f64 * p11)
                        * inv;
                }
                g.susp[i00] = (m00 - p00) as f32;
                g.susp[i10] = (m10 - p10) as f32;
                g.susp[i01] = (m01 - p01) as f32;
                g.susp[i11] = (m11 - p11) as f32;
                g.susp[i] = (g.susp[i] as f64 + want) as f32;
                g.susp_rgb[i] = [r_in as f32, g_in as f32, b_in as f32]; // REPLACE
            }
            // Water moves the same way (no color, no threshold).
            let f00 = g.film[i00] as f64;
            let f10 = g.film[i10] as f64;
            let f01 = g.film[i01] as f64;
            let f11 = g.film[i11] as f64;
            let q00 = f00 * w00;
            let q10 = f10 * w10;
            let q01 = f01 * w01;
            let q11 = f11 * w11;
            g.film[i00] = (f00 - q00) as f32;
            g.film[i10] = (f10 - q10) as f32;
            g.film[i01] = (f01 - q01) as f32;
            g.film[i11] = (f11 - q11) as f32;
            // JS `film[i] += q00 + q10 + q01 + q11`: the RHS sums FIRST, then
            // adds to the cell — a different f64 rounding than left-to-right
            // from the cell (port-verify finding, bit-parity).
            let q_sum = q00 + q10 + q01 + q11;
            g.film[i] = (g.film[i] as f64 + q_sum) as f32;
            i += 1;
        }
    }
    vmax
}
