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

use crate::colorops::km_weighted_mean_color;
use crate::grid::{Grid, wet_byte_from_paper};
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
// §6.1 Active-region rebuild (every 2nd frame)
// ---------------------------------------------------------------------------

/// Rebuild the active mask + fresh bbox from the water map. Pass 1 marks
/// horizontal wet triples inside the previous (padded) bbox; pass 2 grows a
/// vertical "skirt" wherever a vertical triple sums to EXACTLY 1 — and the 2s
/// it writes count in later sums, which is load-bearing: an isolated front
/// gets a full skirt and can run 1 cell/frame, while a train of close stripes
/// starves its own skirt and waits to merge (keeps a wide front from
/// decomposing into permanent horizontal bands).
pub fn rebuild_active_region(g: &mut Grid) {
    let s = g.s;
    let w = g.w as i32;
    let h = g.h as i32;
    if !g.has_fluid || g.bx1 < g.bx0 {
        g.empty_bbox();
        return;
    }
    // Clear the mask over the previous bbox padded ±2. Extend the clear to
    // the pad ring when the box touches the sheet edge, so skirt 2s written
    // on the pad cannot go stale (the [1..W] clamp plus pad writes would
    // otherwise leak permanent actives on row 0 / row H+1).
    let px0 = (g.bx0 - 2).max(1);
    let px1 = (g.bx1 + 2).min(w);
    let py0 = (g.by0 - 2).max(1);
    let py1 = (g.by1 + 2).min(h);
    let cx0 = if px0 == 1 { 0 } else { px0 };
    let cx1 = if px1 == w { w + 1 } else { px1 };
    let cy0 = if py0 == 1 { 0 } else { py0 };
    let cy1 = if py1 == h { h + 1 } else { py1 };
    for y in cy0..=cy1 {
        let base = y as usize * s;
        g.active[base + cx0 as usize..base + cx1 as usize + 1].fill(0);
    }

    // Pass 1 — wet cells (one row/col inside the brushable area; the drain
    // ring only ever activates via the skirt).
    let mut fx0 = w + 1;
    let mut fx1 = 0;
    let mut fy0 = h + 1;
    let mut fy1 = 0;
    let sx0 = px0.max(2);
    let sx1 = px1.min(w - 1);
    let sy0 = py0.max(3);
    let sy1 = py1.min(h - 2);
    let mut fired = false;
    for y in sy0..=sy1 {
        let mut i = sx0 as usize + y as usize * s;
        for x in sx0..=sx1 {
            if g.film[i - 1] as f64 + g.film[i] as f64 + g.film[i + 1] as f64 > 0.0 {
                g.active[i - 1] = 1;
                g.active[i] = 1;
                g.active[i + 1] = 1;
                fired = true;
                if x < fx0 {
                    fx0 = x;
                }
                if x > fx1 {
                    fx1 = x;
                }
                if y < fy0 {
                    fy0 = y;
                }
                if y > fy1 {
                    fy1 = y;
                }
            }
            i += 1;
        }
    }
    if !fired {
        g.empty_bbox();
        return;
    }

    // Pass 2 — the skirt, scanned top-to-down so earlier 2s shape later sums.
    let kx0 = (fx0 - 2).max(1);
    let kx1 = (fx1 + 2).min(w);
    let ky0 = (fy0 - 2).max(1);
    let ky1 = (fy1 + 2).min(h);
    let mut nx0 = w + 1;
    let mut nx1 = 0;
    let mut ny0 = h + 1;
    let mut ny1 = 0;
    let mut any_fire = false;
    for y in ky0..=ky1 {
        let mut i = kx0 as usize + y as usize * s;
        for x in kx0..=kx1 {
            if g.active[i - s] as u32 + g.active[i] as u32 + g.active[i + s] as u32 == 1 {
                g.active[i - s] = 2;
                g.active[i] = 2;
                if g.active[i + s] == 0 {
                    g.active[i + s] = 2;
                }
                any_fire = true;
                if x < nx0 {
                    nx0 = x;
                }
                if x > nx1 {
                    nx1 = x;
                }
                if y < ny0 {
                    ny0 = y;
                }
                if y > ny1 {
                    ny1 = y; // the fire row itself
                }
            }
            i += 1;
        }
    }
    if !any_fire {
        // defensive: keep the pass-1 extent
        nx0 = fx0;
        nx1 = fx1;
        ny0 = fy0;
        ny1 = fy1;
    }
    g.bx0 = (nx0 - 5).max(1);
    g.bx1 = (nx1 + 5).min(w);
    g.by0 = (ny0 - 5).max(1);
    g.by1 = (ny1 + 5).min(h);
    g.has_fluid = true;
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
    let (bx0, bx1, by0, by1) = (g.bx0, g.bx1, g.by0, g.by1);
    // Split-borrow the fields once — the loop body reads/writes through
    // locals instead of re-projecting `g.` per access.
    let Grid {
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
    let off = (bx0 - 1) as usize; // bx0 >= 1
    let span = (bx1 - bx0 + 3) as usize; // stencil reach 1 on both sides
    for y in by0..=by1 {
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
    let s = g.s;
    let max_v = p.k(Knob::MaxVelocity);
    let (bx0, bx1, by0, by1) = (g.bx0, g.bx1, g.by0, g.by1);
    for y in by0..=by1 {
        let mut i = bx0 as usize + y as usize * s;
        for _x in bx0..=bx1 {
            if g.active[i] == 0 {
                i += 1;
                continue;
            }
            let (mut fx, mut fy);
            if g.film[i] as f64 > 0.05 {
                fx = 0.2 * g.vel_x[i] as f64
                    + 0.2
                        * (g.vel_x[i - 1] as f64
                            + g.vel_x[i + 1] as f64
                            + g.vel_x[i - s] as f64
                            + g.vel_x[i + s] as f64);
                fy = 0.2 * g.vel_y[i] as f64
                    + 0.2
                        * (g.vel_y[i - 1] as f64
                            + g.vel_y[i + 1] as f64
                            + g.vel_y[i - s] as f64
                            + g.vel_y[i + s] as f64);
            } else {
                // Whatever gravity the persistent field carries passes
                // straight through.
                fx = g.vel_x[i] as f64;
                fy = g.vel_y[i] as f64;
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
            g.flow_x[i] = fx as f32;
            g.flow_y[i] = fy as f32;
            i += 1;
        }
    }
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
        let mut i = g.bx0 as usize + y as usize * s;
        for _x in g.bx0..=g.bx1 {
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
// §6.5 Advection + gravity (every frame)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// §6.6 Pressure projection (every 3rd frame)
// ---------------------------------------------------------------------------

/// One cheap Jacobi relaxation toward incompressibility: kills piling-up but
/// leaves the uniform downward drift intact so drips survive. The divergence
/// and pressure scratch fields reuse the transient-flow arrays (rebuilt next
/// frame anyway).
pub fn project(g: &mut Grid, p: &Params) {
    let s = g.s;
    let w = g.w as i32;
    let h = g.h as i32;
    let projection = p.k(Knob::Projection);
    let Grid { flow_x, flow_y, vel_x, vel_y, active, .. } = g;
    let div = flow_x;
    let prs = flow_y;
    let zx0 = (g.bx0 - 1).max(0);
    let zx1 = (g.bx1 + 1).min(w + 1);
    let zy0 = (g.by0 - 1).max(0);
    let zy1 = (g.by1 + 1).min(h + 1);
    for y in zy0..=zy1 {
        // neighbours of active cells must read 0
        let base = y as usize * s;
        div[base + zx0 as usize..base + zx1 as usize + 1].fill(0.0);
        prs[base + zx0 as usize..base + zx1 as usize + 1].fill(0.0);
    }
    for y in g.by0..=g.by1 {
        let mut i = g.bx0 as usize + y as usize * s;
        for _x in g.bx0..=g.bx1 {
            if active[i] != 0 {
                div[i] = (vel_x[i - 1] as f64 - vel_x[i + 1] as f64 + vel_y[i - s] as f64
                    - vel_y[i + s] as f64) as f32;
            }
            i += 1;
        }
    }
    for y in g.by0..=g.by1 {
        let mut i = g.bx0 as usize + y as usize * s;
        for _x in g.bx0..=g.bx1 {
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
    for y in g.by0..=g.by1 {
        let mut i = g.bx0 as usize + y as usize * s;
        for x in g.bx0..=g.bx1 {
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
