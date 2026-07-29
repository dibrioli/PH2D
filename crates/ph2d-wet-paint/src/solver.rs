//! Fluid solver passes (port of `solver.js`; SPEC §6.1, §6.3–6.7, plus the
//! flow-side gated extensions of §17). All passes iterate only the active
//! bounding box.
//!
//! The velocity design in one paragraph: gravity accumulates in the
//! PERSISTENT field (vel_x/vel_y); every frame the TRANSIENT flow
//! (flow_x/flow_y) is rebuilt from it — with leveling + capillary + the
//! look-ahead absorbency brake on one frame in four, and as a cheap unbraked
//! smoothing on the other three. Mass then advects along the transient flow.
//! Because the brake only bites 1-in-4, a drip that would stall under
//! constant braking keeps advancing on the free frames: that asymmetry is
//! why drips run.

use crate::grid::{Grid, wet_byte_from_paper};
use crate::par::{self, Rows};
use crate::rng::hash2_signed;
use crate::sim::Params;
use crate::tuning::Knob;

const BLOOM_SEED: u32 = 0x600d;

#[inline]
fn clamp_sym(v: f64, m: f64) -> f64 {
    if v > m {
        m
    } else if v < -m {
        -m
    } else {
        v
    }
}

// ---------------------------------------------------------------------------
// §6.3 Flow-field build (every 4th frame — the braked frame)
// ---------------------------------------------------------------------------

pub fn build_flow_field(g: &mut Grid, p: &Params, gx: f64, gy: f64, ext_bypass: bool) {
    let s = g.s;
    let cells = g.cells as i64;
    let max_v = p.k(Knob::MaxVelocity);
    let g_mag = (gx * gx + gy * gy).sqrt();
    let fingering = !ext_bypass && p.k(Knob::ExtFingering) > 0.0 && g_mag > 0.0;
    let (mut gnx, mut gny, mut wave_len, mut t_axis_x, mut t_axis_y) = (0.0, 0.0, 1.0, 0.0, 0.0);
    if fingering {
        gnx = gx / g_mag;
        gny = gy / g_mag;
        wave_len = (g.w as f64 / p.k(Knob::ExtRivulets)).max(4.0);
        t_axis_x = -gny;
        t_axis_y = gnx; // transverse axis
    }
    let backrun = !ext_bypass && p.k(Knob::ExtBackrun) > 0.0;
    let ext_backrun = p.k(Knob::ExtBackrun);
    let ext_fingering = p.k(Knob::ExtFingering);
    let mix = p.mix;
    let mut out = [0.0f64; 3];
    // Hoist every knob out of the hot loop.
    let level_k = p.k(Knob::Leveling);
    let level_clamp = p.k(Knob::LevelClamp);
    let cap_k = p.k(Knob::Capillary);
    let cap_gate = p.k(Knob::CapillaryGate);
    let visc_threshold = p.k(Knob::Viscosity);
    let brake_bias = p.k(Knob::Brake);
    let brake_reach = p.k(Knob::BrakeReach);
    let (gbx0, gbx1, by0, by1) = (g.bx0, g.bx1, g.by0, g.by1);
    let spans_on = g.spans_enabled;
    // Split-borrow the fields once — the loop body reads/writes through
    // locals instead of re-projecting `g.` per access.
    let Grid {
        row_lo,
        row_hi,
        film,
        susp,
        susp_rgb,
        sett,
        sett_rgb,
        vel_x,
        vel_y,
        flow_x,
        flow_y,
        wet,
        paper,
        active,
        bloom,
        ..
    } = g;

    // Per-row subslices of the READ-ONLY arrays (film/paper/vel/active are
    // never written by this pass on any path) let the compiler prove the
    // stencil indices in-range and drop the bounds checks — the measured
    // cost of this loop was dominated by checked loads. The mutable arrays
    // (susp/sett/colors via backrun, wet, flow) stay checked full-array:
    // one door, no duplicated physics.
    let film_full: &[f32] = film;
    let paper_full: &[f32] = paper;
    let velx_full: &[f32] = vel_x;
    let vely_full: &[f32] = vel_y;
    let active_full: &[u8] = active;
    for y in by0..=by1 {
        // A faixa viva desta linha (ver `Grid::row_lo`): fora dela `active` é
        // 0, e o corpo do laço já era um `continue`.
        let (bx0, bx1) = crate::grid::span_x_of(row_lo, row_hi, spans_on, gbx0, gbx1, y);
        if bx0 > bx1 {
            continue;
        }
        let off = (bx0 - 1) as usize; // bx0 >= 1
        let span = (bx1 - bx0 + 3) as usize; // stencil reach 1 on both sides
        let base = y as usize * s;
        let row = base + off;
        // Middle / up / down row windows, all exactly `span` long.
        let film_m = &film_full[row..row + span];
        let film_u = &film_full[row - s..row - s + span];
        let film_d = &film_full[row + s..row + s + span];
        let paper_m = &paper_full[row..row + span];
        let paper_u = &paper_full[row - s..row - s + span];
        let paper_d = &paper_full[row + s..row + s + span];
        let velx_m = &velx_full[row..row + span];
        let velx_u = &velx_full[row - s..row - s + span];
        let velx_d = &velx_full[row + s..row + s + span];
        let vely_m = &vely_full[row..row + span];
        let vely_u = &vely_full[row - s..row - s + span];
        let vely_d = &vely_full[row + s..row + s + span];
        let active_m = &active_full[row..row + span];
        let mut i = bx0 as usize + base;
        for x in bx0..=bx1 {
            let k = (x - bx0) as usize + 1; // 1..=span-2
            if active_m[k] == 0 {
                i += 1;
                continue;
            }
            let f = film_m[k] as f64;
            let mut ex = velx_m[k] as f64;
            let mut ey = vely_m[k] as f64;

            // Leveling: water flows thick -> thin, clamped per axis.
            let mut lx = (film_m[k - 1] as f64 - film_m[k + 1] as f64) * level_k;
            if lx > level_clamp {
                lx = level_clamp;
            } else if lx < -level_clamp {
                lx = -level_clamp;
            }
            let mut ly = (film_u[k] as f64 - film_d[k] as f64) * level_k;
            if ly > level_clamp {
                ly = level_clamp;
            } else if ly < -level_clamp {
                ly = -level_clamp;
            }
            ex += lx;
            ey += ly;

            // Capillary: only thin paint follows the tooth. Steepest-descent
            // pull — the asymmetric form picks the drop ACROSS the cell in
            // the direction of the steeper fall, which channels water into
            // grain rivulets.
            if (susp[i] as f64 + sett[i] as f64) < cap_gate {
                let pc = paper_m[k] as f64;
                let pl = paper_m[k - 1] as f64;
                let pr = paper_m[k + 1] as f64;
                ex += if pl > pc {
                    if pc < pr { pl - pr } else { pc - pr }
                } else if pc < pr {
                    pl - pc
                } else {
                    pl - pr
                } * cap_k;
                let pu = paper_u[k] as f64;
                let pd = paper_d[k] as f64;
                ey += if pu > pc {
                    if pc < pd { pu - pd } else { pc - pd }
                } else if pc < pd {
                    pu - pc
                } else {
                    pu - pd
                } * cap_k;
            }

            // Viscosity: deep water drags its persistent-field neighbours
            // along. (The knob is the film THRESHOLD; the 0.2 blend weights
            // are fixed.)
            if f > visc_threshold {
                ex = 0.2 * ex
                    + 0.2
                        * (velx_m[k - 1] as f64
                            + velx_m[k + 1] as f64
                            + velx_u[k] as f64
                            + velx_d[k] as f64);
                ey = 0.2 * ey
                    + 0.2
                        * (vely_m[k - 1] as f64
                            + vely_m[k + 1] as f64
                            + vely_u[k] as f64
                            + vely_d[k] as f64);
            }

            // Wetness stamp: deep film marks the sheet damp, valleys damper —
            // future flow keeps running along established wet channels.
            if f > 3.0 {
                wet[i] = wet_byte_from_paper(paper_m[k] as f64);
            }

            // Fingering (extension, PRE-brake): at a drip's leading edge, add
            // a transverse sinusoidal ripple — push down at the peaks plus a
            // slight sideways component; the brake then self-selects rivulet
            // columns.
            if fingering && f > 0.1 {
                let j = i as i64 + js_round_i64(gnx) + js_round_i64(gny) * s as i64;
                if j >= 0 && j < cells && film_full[j as usize] as f64 <= 0.01 {
                    let phase =
                        (2.0 * std::f64::consts::PI * (x as f64 * t_axis_x + y as f64 * t_axis_y))
                            / wave_len;
                    let push = ext_fingering * f.min(2.0);
                    let sn = libm::sin(phase);
                    let cs = libm::cos(phase);
                    ex += gnx * push * sn + t_axis_x * push * 0.3 * cs;
                    ey += gny * push * sn + t_axis_y * push * 0.3 * cs;
                }
            }

            // Look-ahead absorbency brake: probe a few px downstream; wet or
            // flooded ground ahead lets flow keep running, dry ground stalls
            // it. Linear index arithmetic, no 2-D clamp: a probe past the
            // array end skips the brake entirely (flow at the sheet edge runs
            // off it).
            let s_len = brake_reach / ((ex * ex + ey * ey).sqrt() + 0.01);
            let probe =
                i as i64 + (ex * s_len).trunc() as i64 + (ey * s_len).trunc() as i64 * s as i64;
            if probe >= 0 && probe < cells {
                let pu = probe as usize;
                let mut brake = film_full[pu] as f64 + (3.0 / 255.0) * wet[pu] as f64 - brake_bias;
                if brake < 0.05 {
                    brake = 0.05;
                } else if brake > 1.0 {
                    brake = 1.0;
                }
                ex *= brake;
                ey *= brake;
            }

            // Backrun / bloom (extension, POST-brake): where this cell is
            // much wetter than a neighbour holding settled pigment, shove
            // flow toward it and lift some of its settled mass back into
            // suspension. A per-cell budget (max 6 blooming build-frames per
            // fresh front) stops sloshing from pumping thin lines;
            // integer-hash jitter crenellates the rim.
            if backrun {
                let thr = 0.8 + 0.2 * hash2_signed(x, y, BLOOM_SEED);
                let mut met = false;
                let can_bloom = bloom[i] < 6;
                for n_idx in 0..4 {
                    let dxn: i64 = match n_idx {
                        0 => -1,
                        1 => 1,
                        _ => 0,
                    };
                    let dyn_: i64 = match n_idx {
                        2 => -1,
                        3 => 1,
                        _ => 0,
                    };
                    let nb = (i as i64 + dxn + dyn_ * s as i64) as usize;
                    let gap = f - film_full[nb] as f64;
                    if gap > thr && sett[nb] as f64 > 0.0 {
                        met = true;
                        if can_bloom {
                            let push = ext_backrun * gap.min(1.5) * 0.5;
                            if dxn != 0 {
                                ex += dxn as f64 * push;
                            } else {
                                ey += dyn_ as f64 * push;
                            }
                            let lift = sett[nb] as f64 * 0.1;
                            let w = lift / (susp[nb] as f64 + lift);
                            let uc = susp_rgb[nb];
                            let sc = sett_rgb[nb];
                            mix.mix(
                                uc[0] as f64,
                                uc[1] as f64,
                                uc[2] as f64,
                                sc[0] as f64,
                                sc[1] as f64,
                                sc[2] as f64,
                                w,
                                &mut out,
                            );
                            susp_rgb[nb] = [out[0] as f32, out[1] as f32, out[2] as f32];
                            susp[nb] = (susp[nb] as f64 + lift) as f32;
                            sett[nb] = (sett[nb] as f64 - lift) as f32;
                        }
                    }
                }
                if met {
                    if can_bloom {
                        bloom[i] += 1;
                    }
                } else {
                    bloom[i] = 0;
                }
            }

            flow_x[i] = clamp_sym(ex, max_v) as f32;
            flow_y[i] = clamp_sym(ey, max_v) as f32;
            i += 1;
        }
    }
}

/// `Math.round` of a unit-vector component, as the JS fingering does.
#[inline]
fn js_round_i64(v: f64) -> i64 {
    (v + 0.5).floor() as i64
}

// ---------------------------------------------------------------------------
// §6.4 Velocity smoothing (the other 3 frames — never braked)
// ---------------------------------------------------------------------------

pub fn smooth_velocity(g: &mut Grid, p: &Params) {
    let rows = (g.by1 - g.by0 + 1).max(0) as usize;
    let span = (g.bx1 - g.bx0 + 1).max(0) as usize;
    smooth_velocity_rows(g, p, Rows::pick(rows, span, par::MIN_CELLS_GATHER));
}

/// [`smooth_velocity`] com a rota de caminhada FORÇADA — a porta dos gates de
/// identidade (ADR-0145). O produto chama sempre o [`smooth_velocity`].
///
/// **É um GATHER puro:** escreve `flow_x`/`flow_y` no próprio índice e lê
/// `vel_x`/`vel_y` (inclusive das linhas vizinhas), `film` e `active` — nenhum
/// deles tocado por este passe. Sem redução, sem transcendental, sem RNG.
pub fn smooth_velocity_rows(g: &mut Grid, p: &Params, mode: Rows) {
    let s = g.s;
    let max_v = p.k(Knob::MaxVelocity);
    let (gbx0, gbx1, by0, by1) = (g.bx0, g.bx1, g.by0, g.by1);
    if by1 < by0 {
        return;
    }
    let spans_on = g.spans_enabled;
    let Grid {
        row_lo,
        row_hi,
        film,
        vel_x,
        vel_y,
        flow_x,
        flow_y,
        active,
        ..
    } = g;
    let row_lo: &[i32] = row_lo;
    let row_hi: &[i32] = row_hi;
    let film: &[f32] = film;
    let velx: &[f32] = vel_x;
    let vely: &[f32] = vel_y;
    let active: &[u8] = active;
    let b = by0 as usize * s..(by1 as usize + 1) * s;
    par::walk_rows2(
        mode,
        &mut flow_x[b.clone()],
        &mut flow_y[b],
        s,
        |r, fxr, fyr| {
            let y = by0 + r as i32;
            // Faixa viva: fora dela `active` é 0 e o corpo já era um `continue`.
            let (bx0, bx1) = crate::grid::span_x_of(row_lo, row_hi, spans_on, gbx0, gbx1, y);
            if bx0 > bx1 {
                return;
            }
            let base = y as usize * s;
            for x in bx0..=bx1 {
                let i = x as usize + base;
                if active[i] == 0 {
                    continue;
                }
                let (mut fx, mut fy);
                if film[i] as f64 > 0.05 {
                    fx = 0.2 * velx[i] as f64
                        + 0.2
                            * (velx[i - 1] as f64
                                + velx[i + 1] as f64
                                + velx[i - s] as f64
                                + velx[i + s] as f64);
                    fy = 0.2 * vely[i] as f64
                        + 0.2
                            * (vely[i - 1] as f64
                                + vely[i + 1] as f64
                                + vely[i - s] as f64
                                + vely[i + s] as f64);
                } else {
                    // Whatever gravity the persistent field carries passes
                    // straight through.
                    fx = velx[i] as f64;
                    fy = vely[i] as f64;
                }
                if fx > max_v {
                    fx = max_v;
                } else if fx < -max_v {
                    fx = -max_v;
                }
                if fy > max_v {
                    fy = max_v;
                } else if fy < -max_v {
                    fy = -max_v;
                }
                fxr[x as usize] = fx as f32;
                fyr[x as usize] = fy as f32;
            }
        },
    );
}

/// Diffusion (extension, smoothing frames): Fickian spread of suspended
/// pigment through a still wet film. Symmetric flux to the +x/+y neighbours
/// only (each edge visited once => mass-conserving); color rides
/// mass-weighted.
pub fn diffusion_pass(g: &mut Grid, p: &Params) {
    let s = g.s;
    let mut out = [0.0f64; 3];
    let rate_knob = p.k(Knob::ExtDiffusion);
    for y in g.by0..=g.by1 {
        // Faixa viva: fora dela `active` é 0 e o corpo já era um `continue`.
        let (bx0, bx1) = g.span_x(y);
        if bx0 > bx1 {
            continue;
        }
        let mut i = bx0 as usize + y as usize * s;
        for _x in bx0..=bx1 {
            if g.active[i] == 0 || g.film[i] as f64 <= 0.1 {
                i += 1;
                continue;
            }
            let rate = rate_knob * (g.film[i] as f64 / 1.5).min(1.0);
            for e in 0..2 {
                let nb = if e == 0 { i + 1 } else { i + s };
                let flux = rate * (g.susp[i] as f64 - g.susp[nb] as f64);
                let (from, to) = if flux > 0.0 { (i, nb) } else { (nb, i) };
                let mut dm = flux.abs();
                if dm <= 0.0 {
                    continue;
                }
                if dm > g.susp[from] as f64 {
                    dm = g.susp[from] as f64;
                }
                let w = dm / (g.susp[to] as f64 + dm);
                let tc = g.susp_rgb[to];
                let fc = g.susp_rgb[from];
                p.mix.mix(
                    tc[0] as f64,
                    tc[1] as f64,
                    tc[2] as f64,
                    fc[0] as f64,
                    fc[1] as f64,
                    fc[2] as f64,
                    w,
                    &mut out,
                );
                g.susp_rgb[to] = [out[0] as f32, out[1] as f32, out[2] as f32];
                g.susp[from] = (g.susp[from] as f64 - dm) as f32;
                g.susp[to] = (g.susp[to] as f64 + dm) as f32;
            }
            i += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// §6.5 advection and §6.6/§6.7 projection + boundaries live in child modules
// (workspace file-LOC cap); the `solver::` paths are re-exported unchanged.
// ---------------------------------------------------------------------------

mod active_region;
mod advect;
mod project;
pub use active_region::{rebuild_active_region, rebuild_active_region_rows};
pub use advect::advect;
pub use project::{apply_boundaries, project, project_rows};
