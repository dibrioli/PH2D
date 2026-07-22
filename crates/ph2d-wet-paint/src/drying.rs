//! Drying / settling / re-wetting (port of `drying.js`, SPEC §6.2) + fast dry
//! (SPEC §12).
//!
//! This pass is where the classic watercolor rim comes from: cells at the
//! edge of a wash (few suspended neighbours => low edge factor e) evaporate
//! up to ~26x faster than the interior, so pigment carried there by the flow
//! settles first and darkens the boundary. Once a cell has settled >= 1000
//! the edge boost stops — saturated washes stop over-darkening.

use crate::colorops::ColorMix;
use crate::grid::Grid;
use crate::jsmath::clamp01;
use crate::opacity::alpha_of_mass;
use crate::sim::Params;
use crate::solver::rebuild_active_region;
use crate::tuning::Knob;

/// Staining extension: bidirectional lift multiplier, exactly 1 at 0.5.
/// `pub(crate)`: the doc-23 active lifts (Wet/Smear/Blend tools) apply the
/// SAME law, so staining means one thing everywhere.
#[inline]
pub(crate) fn staining_multiplier(s: f64) -> f64 {
    if s < 0.5 {
        1.0 + (0.5 - s) * 14.0 // 0 -> 8x
    } else {
        (1.0 - s) * 2.0 // 1 -> 0x
    }
}

/// The LIFT DOOR (SPEC §6.2 re-wet arithmetic, extracted verbatim): move
/// `sett · b` back into suspension with the opacity-composited color. ONE
/// arithmetic for every rewetter — the passive drying pass below and the
/// doc-23 active tools (Wet/Smear/Blend) — so passive and active lift can
/// never disagree about what "dissolving" means. Callers own `b` (clamped
/// 0..1) and the staining multiplier.
#[inline]
pub(crate) fn lift_settled(
    b: f64,
    susp: &mut f32,
    sett: &mut f32,
    susp_rgb: &mut [f32; 3],
    sett_rgb: [f32; 3],
    mix: ColorMix,
    out: &mut [f64; 3],
) {
    let st = *sett as f64;
    let lift = st * b;
    let a_in = alpha_of_mass(st) * b;
    let u = alpha_of_mass(*susp as f64) * (1.0 - a_in);
    if u + a_in > 0.0 {
        let w = a_in / (u + a_in);
        mix.mix(
            susp_rgb[0] as f64,
            susp_rgb[1] as f64,
            susp_rgb[2] as f64,
            sett_rgb[0] as f64,
            sett_rgb[1] as f64,
            sett_rgb[2] as f64,
            w,
            out,
        );
        *susp_rgb = [out[0] as f32, out[1] as f32, out[2] as f32];
    } else {
        *susp_rgb = sett_rgb;
    }
    *susp = (*susp as f64 + lift) as f32;
    *sett = (st - lift) as f32;
}

/// One drying pass over every bbox cell holding water or suspended pigment
/// (deliberately NOT filtered by the active mask — paint dries everywhere).
/// `evap_base` is the cadence-adaptive scale (or 1000 for a force-dry pass);
/// `rewet_base` the cadence-adaptive lift floor.
pub fn drying_pass(g: &mut Grid, p: &Params, evap_base: f64, rewet_base: f64, ext_bypass: bool) {
    let s = g.s;
    let mut out = [0.0f64; 3];
    let mix = p.mix;
    let retention = p.k(Knob::Retention);
    let edge_darkening = p.k(Knob::EdgeDarkening);
    let base_evaporation = p.k(Knob::BaseEvaporation);
    let ext_granulation = p.k(Knob::ExtGranulation);
    let ext_staining = p.k(Knob::ExtStaining);
    // Hot-loop discipline: every same-cell value lives in an f32 LOCAL that
    // mirrors what the JS Float32Array holds after each store (the f32
    // rounding IS the semantics — a later read sees the rounded value), and
    // is written back once at the end. Neighbour reads compare in f32
    // directly: widening is exact, so `susp > 10.0f32` == `(f64)susp > 10.0`.
    for y in g.by0..=g.by1 {
        let mut i = g.bx0 as usize + y as usize * s;
        for _x in g.bx0..=g.bx1 {
            let film0 = g.film[i];
            let susp0 = g.susp[i];
            if film0 <= 0.0 && susp0 <= 0.0 {
                i += 1;
                continue;
            }
            let f = film0 as f64;
            let s_mass = susp0 as f64;
            // Post-store mirrors of the cell's mutable fields.

            let mut susp_c = susp0;
            let mut sett_c = g.sett[i];
            let mut susp_rgb = g.susp_rgb[i];
            let mut sett_rgb = g.sett_rgb[i];

            // Edge factor: fraction of the 3x3 neighbourhood carrying
            // pigment. A full block (9/9) reads e=1 (no boost); a rim cell
            // reads e<1.
            let mut e = 1.0;
            if film0 > 0.0 && sett_c < 1000.0 {
                let up = i - s;
                let dn = i + s;
                let count = (g.susp[up - 1] > 10.0) as u32
                    + (g.susp[up] > 10.0) as u32
                    + (g.susp[up + 1] > 10.0) as u32
                    + (g.susp[i - 1] > 10.0) as u32
                    + (susp0 > 10.0) as u32
                    + (g.susp[i + 1] > 10.0) as u32
                    + (g.susp[dn - 1] > 10.0) as u32
                    + (g.susp[dn] > 10.0) as u32
                    + (g.susp[dn + 1] > 10.0) as u32;
                e = if count == 9 { 1.0 } else { count as f64 / 9.0 };
            }

            // Evaporate: retention leak + edge-boosted linear loss.
            let mut new_film =
                f * retention - evap_base * ((1.0 - e) * edge_darkening + base_evaporation);
            if new_film < 0.0001 {
                new_film = 0.0;
            }
            let lost = if f > 0.0 {
                1.0 - clamp01(new_film / f)
            } else {
                1.0
            };
            let film_c: f32 = new_film as f32;

            // Settle: the fraction of water lost carries the same fraction of
            // suspended pigment onto the paper (opacity-composited color).
            if lost > 0.0 && s_mass > 0.0 {
                let mut dm = s_mass * lost;
                if !ext_bypass {
                    // Physical granulation (extension): settle biased toward
                    // valleys.
                    let mut bias = 1.0 + ext_granulation * 0.6 * (0.5 - g.paper[i] as f64) * 2.0;
                    if bias < 0.3 {
                        bias = 0.3;
                    } else if bias > 1.7 {
                        bias = 1.7;
                    }
                    dm *= bias;
                    if dm > s_mass {
                        dm = s_mass;
                    }
                }
                // SPEC §6.2 step 3 (the shared settle composite, on locals).
                let a_sett = alpha_of_mass(sett_c as f64);
                let a_in = alpha_of_mass(dm);
                if a_sett > 0.0 {
                    let u = a_sett * (1.0 - a_in);
                    let w = a_in / (u + a_in);
                    mix.mix(
                        sett_rgb[0] as f64,
                        sett_rgb[1] as f64,
                        sett_rgb[2] as f64,
                        susp_rgb[0] as f64,
                        susp_rgb[1] as f64,
                        susp_rgb[2] as f64,
                        w,
                        &mut out,
                    );
                    sett_rgb = [out[0] as f32, out[1] as f32, out[2] as f32];
                } else {
                    sett_rgb = susp_rgb;
                }
                sett_c = (sett_c as f64 + dm) as f32;
                let rem = s_mass - dm;
                susp_c = if rem < 0.0 { 0.0 } else { rem as f32 };
            }

            // Re-wet: standing water lifts a little settled pigment back into
            // suspension; the lift grows with the EXCESS water over what the
            // suspended pigment already occupies.
            let st = sett_c as f64;
            if film_c > 0.0 && st > 0.0 {
                let excess = (film_c as f64 - alpha_of_mass(susp_c as f64)).max(0.0);
                let mut b = rewet_base * (1.0 + excess * 50.0);
                if !ext_bypass {
                    b *= staining_multiplier(ext_staining);
                }
                b = clamp01(b);
                if b > 0.0 {
                    lift_settled(
                        b,
                        &mut susp_c,
                        &mut sett_c,
                        &mut susp_rgb,
                        sett_rgb,
                        mix,
                        &mut out,
                    );
                }
            }

            // Write the cell back once.
            g.film[i] = film_c;
            g.susp[i] = susp_c;
            g.sett[i] = sett_c;
            g.susp_rgb[i] = susp_rgb;
            g.sett_rgb[i] = sett_rgb;
            i += 1;
        }
    }
}

/// Fast dry (SPEC §12): repeatedly halve the water and run force-dry passes
/// until no fluid remains (guard <= 40 loops). No advection — it cannot
/// stall; edge darkening still forms because the settle step runs each loop.
pub fn fast_dry(g: &mut Grid, p: &Params, rewet_base: f64, ext_bypass: bool) {
    let s = g.s;
    let mut guard = 0;
    while guard < 40 && g.has_fluid {
        for y in g.by0..=g.by1 {
            let mut i = g.bx0 as usize + y as usize * s;
            for _x in g.bx0..=g.bx1 {
                g.film[i] = (g.film[i] as f64 * 0.5) as f32;
                i += 1;
            }
        }
        drying_pass(g, p, 1000.0, rewet_base, ext_bypass);
        rebuild_active_region(g);
        guard += 1;
    }
    if !g.has_fluid {
        g.empty_bbox();
    }
}
