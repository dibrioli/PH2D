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
    let max_v = p.k(Knob::MaxVelocity);
    let (bx0, bx1, by0, by1) = (g.bx0, g.bx1, g.by0, g.by1);
    let km_mean = p.km_mixing; // route the incoming mean through K–M
    let mut km_colors = [0.0f64; 12];
    let mut km_weights = [0.0f64; 4];
    let mut km_out = [0.0f64; 3];
    let mut vmax = 0.0f64;
    for y in by0..=by1 {
        let mut i = bx0 as usize + y as usize * s;
        for x in bx0..=bx1 {
            if g.active[i] == 0 {
                g.vel_x[i] = 0.0;
                g.vel_y[i] = 0.0;
                i += 1;
                continue;
            }
            let ux = g.flow_x[i] as f64;
            let uy = g.flow_y[i] as f64;
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
            let mut nvx = g.flow_x[i00] as f64 * w00
                + g.flow_x[i10] as f64 * w10
                + g.flow_x[i01] as f64 * w01
                + g.flow_x[i11] as f64 * w11
                + gx * f;
            let mut nvy = g.flow_y[i00] as f64 * w00
                + g.flow_y[i10] as f64 * w10
                + g.flow_y[i01] as f64 * w01
                + g.flow_y[i11] as f64 * w11
                + gy * f;
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
            g.vel_x[i] = nvx as f32;
            g.vel_y[i] = nvy as f32;

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
