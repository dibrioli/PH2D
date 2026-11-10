// CÓPIA DE SONDA (commit base `341341b233`) de `src/solver/advect_jacobi.rs` (advect_jacobi_rows e
// os seus passes) e de `src/drying.rs` + `src/drying/edge_window.rs` (drying_pass_jacobi_rows,
// dry_cell, lift_settled, DryConst, staining_multiplier, EdgeWindow). Incluída por `include!` num
// módulo que define `type F`: `F = f64` reproduz a original (gate: hash igual); `F = f32` é a C.1.
// As leis de COR da crate (K–M, `assenta`/`levanta`, `velatura`, `alpha_of_mass`, `clamp01`, a
// amostragem do campo de fluxo) ficam f64 — chamadas da crate; só a aritmética destas rotinas muda.
// Todas as fns `#[inline(always)]` para que a cópia sob `#[target_feature]` (C.2) as re-gere.

use crate::par::{self, Mode};
use ph2d_wet_paint::assenta::Ecra;
use ph2d_wet_paint::colorops::{ColorMix, km_weighted_mean_color};
use ph2d_wet_paint::flow::{FlowRowSampler, flow_at_cell, flow_at_point, flow_bbox, flow_probe};
use ph2d_wet_paint::grid::{Grid, aparencia, oleo_celula, span_x_of};
use ph2d_wet_paint::jsmath::clamp01;
use ph2d_wet_paint::opacity::alpha_of_mass;
use ph2d_wet_paint::sim::Params;
use ph2d_wet_paint::tuning::Knob;

#[inline(always)]
fn w(x: f64) -> F {
    x as F
}
#[inline(always)]
fn d(x: F) -> f64 {
    x as f64
}
#[inline(always)]
fn fi(x: i32) -> F {
    x as F
}

#[inline(always)]
fn destination_reach(max_v: f64) -> i32 {
    let r = max_v.ceil() as i32;
    (if r < 1 { 1 } else { r }) + 1
}

#[inline(always)]
fn backtrace(x: i32, y: i32, ux: F, uy: F, wf: F, hf: F) -> Option<(i64, F, i64, F)> {
    let sx = fi(x) - ux;
    let sy = fi(y) - uy;
    if sx < 1.0 || sx > wf || sy < 1.0 || sy > hf {
        return None;
    }
    let x0 = sx as i64;
    let y0 = sy as i64;
    Some((x0, sx - x0 as F, y0, sy - y0 as F))
}

#[inline(always)]
fn corner_weight(c: i64, base: i64, frac: F) -> F {
    if c == base {
        1.0 - frac
    } else if c == base + 1 {
        frac
    } else {
        0.0
    }
}

#[inline(always)]
fn take_scale(outflow: F) -> F {
    if outflow > 1.0 { 1.0 / outflow } else { 1.0 }
}

#[inline(always)]
pub fn advect_jacobi_rows(g: &mut Grid, p: &Params, gx: f64, gy: f64, mode: Mode) -> f64 {
    if g.by0 > g.by1 {
        return 0.0;
    }
    g.scratch.ensure(g.cells);
    momentum_rows(g, p, w(gx), w(gy), mode);
    let vmax = prepare_rows(g, mode);
    outflow_rows(g, p, mode);
    transport_rows(g, p, mode);
    commit_rows(g, mode);
    d(vmax)
}

#[inline(always)]
fn prepare_rows(g: &mut Grid, mode: Mode) -> F {
    let geom = g.flow;
    let s = g.s;
    let (by0, by1) = (g.by0, g.by1);
    let Grid {
        scratch,
        flow_x,
        flow_y,
        active,
        row_lo,
        row_hi,
        spans_enabled,
        bx0,
        bx1,
        ..
    } = g;
    let (spans_enabled, bx0, bx1) = (*spans_enabled, *bx0, *bx1);
    let band = by0 as usize * s..(by1 as usize + 1) * s;
    par::walk_rows_reduce(
        mode,
        &mut scratch.uv[band],
        s,
        0.0 as F,
        |ri, row| {
            let y = by0 + ri as i32;
            let (lo, hi) = span_x_of(row_lo, row_hi, spans_enabled, bx0, bx1, y);
            if lo > hi {
                return 0.0;
            }
            let mut sampler = FlowRowSampler::new(&geom, y);
            let mut vmax: F = 0.0;
            let mut i = lo as usize + y as usize * s;
            for x in lo..=hi {
                let (ux, uy) = sampler.at(flow_x, flow_y, x, i, geom.s);
                let (ux, uy) = (w(ux), w(uy));
                row[x as usize] = [ux as f32, uy as f32];
                if active[i] != 0 {
                    let axv = ux.abs();
                    let ayv = uy.abs();
                    if axv > vmax {
                        vmax = axv;
                    }
                    if ayv > vmax {
                        vmax = ayv;
                    }
                }
                i += 1;
            }
            vmax
        },
        F::max,
    )
}

#[inline(always)]
fn momentum_rows(g: &mut Grid, p: &Params, gx: F, gy: F, mode: Mode) {
    let geom = g.flow;
    let s = g.s;
    let (gw, gh) = (g.w as i32, g.h as i32);
    let (wf, hf) = (g.w as F, g.h as F);
    let max_v = w(p.k(Knob::MaxVelocity));
    let (fbx0, fby0, fbx1, fby1) = flow_bbox((g.bx0, g.by0, g.bx1, g.by1), geom.rf);
    if fby0 > fby1 || fbx0 > fbx1 {
        return;
    }
    let (by0, by1) = (g.by0, g.by1);
    let Grid {
        vel_x,
        vel_y,
        flow_x,
        flow_y,
        film,
        active,
        row_lo,
        row_hi,
        spans_enabled,
        bx0,
        bx1,
        ..
    } = g;
    let (spans_enabled, bx0, bx1) = (*spans_enabled, *bx0, *bx1);
    let band = fby0 as usize * geom.s..(fby1 as usize + 1) * geom.s;
    par::walk_rows2(
        mode,
        &mut vel_x[band.clone()],
        &mut vel_y[band],
        geom.s,
        |r, vx, vy| {
            let cy = fby0 + r as i32;
            let py = flow_probe(cy, gh, geom.rf);
            if py < by0 || py > by1 {
                return;
            }
            let (lo, hi) = span_x_of(row_lo, row_hi, spans_enabled, bx0, bx1, py);
            if lo > hi {
                return;
            }
            for cx in fbx0..=fbx1 {
                let px = flow_probe(cx, gw, geom.rf);
                if px < lo || px > hi {
                    continue;
                }
                let i = px as usize + py as usize * s;
                let c = cx as usize;
                if active[i] == 0 {
                    vx[c] = 0.0;
                    vy[c] = 0.0;
                    continue;
                }
                let (ux, uy) = flow_at_cell(flow_x, flow_y, &geom, px, py, i);
                let (ux, uy) = (w(ux), w(uy));
                if backtrace(px, py, ux, uy, wf, hf).is_none() {
                    continue;
                }
                let sx = fi(px) - ux;
                let sy = fi(py) - uy;
                let f = film[i] as F;
                let (sfx, sfy) = flow_at_point(flow_x, flow_y, &geom, d(sx), d(sy));
                let mut nvx = w(sfx) + gx * f;
                let mut nvy = w(sfy) + gy * f;
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
                vx[c] = nvx as f32;
                vy[c] = nvy as f32;
            }
        },
    );
}

#[inline(always)]
fn outflow_rows(g: &mut Grid, p: &Params, mode: Mode) {
    let s = g.s;
    let gh = g.h as i32;
    let (wf, hf) = (g.w as F, g.h as F);
    let reach = destination_reach(p.k(Knob::MaxVelocity));
    let (by0, by1) = (g.by0, g.by1);
    let Grid {
        scratch,
        active,
        row_lo,
        row_hi,
        spans_enabled,
        bx0,
        bx1,
        ..
    } = g;
    let (spans_enabled, bx0, bx1) = (*spans_enabled, *bx0, *bx1);
    let uv: &[[f32; 2]] = &scratch.uv;
    let band = by0 as usize * s..(by1 as usize + 1) * s;
    par::walk_rows(mode, &mut scratch.outflow[band], s, |ri, row| {
        let y = by0 + ri as i32;
        let (lo, hi) = span_x_of(row_lo, row_hi, spans_enabled, bx0, bx1, y);
        if lo > hi {
            return;
        }
        let (lo_u, hi_u) = (lo as usize, hi as usize);
        row[lo_u..=hi_u].fill(0.0);
        for ny in (y - reach).max(1)..=(y + reach).min(gh) {
            let (slo, shi) = span_x_of(row_lo, row_hi, spans_enabled, bx0, bx1, ny);
            if slo > shi {
                continue;
            }
            let rowbase = ny as usize * s;
            for nx in slo..=shi {
                let di = nx as usize + rowbase;
                if active[di] == 0 {
                    continue;
                }
                let uvd = uv[di];
                let Some((x0, fx, y0, fy)) = backtrace(nx, ny, uvd[0] as F, uvd[1] as F, wf, hf)
                else {
                    continue;
                };
                let wy = corner_weight(i64::from(y), y0, fy);
                if wy == 0.0 {
                    continue;
                }
                let c0 = x0 as usize;
                if x0 >= lo as i64 && x0 <= hi as i64 {
                    row[c0] += ((1.0 - fx) * wy) as f32;
                }
                if x0 >= lo as i64 - 1 && x0 < hi as i64 {
                    row[c0 + 1] += (fx * wy) as f32;
                }
            }
        }
    });
}

#[inline(always)]
fn transport_rows(g: &mut Grid, p: &Params, mode: Mode) {
    let s = g.s;
    let (wf, hf) = (g.w as F, g.h as F);
    let km_mean = p.km_mixing;
    let (by0, by1) = (g.by0, g.by1);
    let Grid {
        scratch,
        film,
        susp,
        susp_rgb,
        active,
        row_lo,
        row_hi,
        spans_enabled,
        bx0,
        bx1,
        ..
    } = g;
    let (spans_enabled, bx0, bx1) = (*spans_enabled, *bx0, *bx1);
    let outflow: &[f32] = &scratch.outflow;
    let uv: &[[f32; 2]] = &scratch.uv;
    let band = by0 as usize * s..(by1 as usize + 1) * s;
    // Rascunho em planos (31/10, o produto deixou o `AdvCell` intercalado): os mesmos valores.
    let (dfm, dsm, dcm) = (
        &mut scratch.dst_film,
        &mut scratch.dst_susp,
        &mut scratch.dst_rgb,
    );
    par::walk_rows3(
        mode,
        &mut dfm[band.clone()],
        &mut dsm[band.clone()],
        &mut dcm[band],
        s,
        |ri, fr, sr, cr| {
            let y = by0 + ri as i32;
            let (lo, hi) = span_x_of(row_lo, row_hi, spans_enabled, bx0, bx1, y);
            if lo > hi {
                return;
            }
            let mut km_colors = [0.0f64; 12];
            let mut km_weights = [0.0f64; 4];
            let mut km_out = [0.0f64; 3];
            let mut i = lo as usize + y as usize * s;
            for x in lo..=hi {
                let f0 = film[i] as F;
                let m0 = susp[i] as F;
                let of = outflow[i] as F;
                let leave = if of > 1.0 { 1.0 } else { of };
                let mut new_film = f0 - f0 * leave;
                let mut new_susp = m0 - m0 * leave;
                let mut new_rgb = susp_rgb[i];
                if active[i] != 0 {
                    let uvc = uv[i];
                    let (ux, uy) = (uvc[0] as F, uvc[1] as F);
                    if let Some((x0, fx, y0, fy)) = backtrace(x, y, ux, uy, wf, hf) {
                        let i00 = x0 as usize + y0 as usize * s;
                        let i10 = i00 + 1;
                        let i01 = i00 + s;
                        let i11 = i01 + 1;
                        let w00 = (1.0 - fx) * (1.0 - fy);
                        let w10 = fx * (1.0 - fy);
                        let w01 = (1.0 - fx) * fy;
                        let w11 = fx * fy;
                        let k00 = w00 * take_scale(outflow[i00] as F);
                        let k10 = w10 * take_scale(outflow[i10] as F);
                        let k01 = w01 * take_scale(outflow[i01] as F);
                        let k11 = w11 * take_scale(outflow[i11] as F);
                        let p00 = susp[i00] as F * k00;
                        let p10 = susp[i10] as F * k10;
                        let p01 = susp[i01] as F * k01;
                        let p11 = susp[i11] as F * k11;
                        let want = p00 + p10 + p01 + p11;
                        if want > 0.0 {
                            let inv = 1.0 / want;
                            let c00 = susp_rgb[i00];
                            let c10 = susp_rgb[i10];
                            let c01 = susp_rgb[i01];
                            let c11 = susp_rgb[i11];
                            let (r_in, g_in, b_in): (F, F, F);
                            if km_mean {
                                for k in 0..3 {
                                    km_colors[k] = c00[k] as f64;
                                    km_colors[3 + k] = c10[k] as f64;
                                    km_colors[6 + k] = c01[k] as f64;
                                    km_colors[9 + k] = c11[k] as f64;
                                }
                                km_weights[0] = d(p00);
                                km_weights[1] = d(p10);
                                km_weights[2] = d(p01);
                                km_weights[3] = d(p11);
                                km_weighted_mean_color(
                                    &km_colors,
                                    &km_weights,
                                    4,
                                    d(inv),
                                    &mut km_out,
                                );
                                r_in = w(km_out[0]);
                                g_in = w(km_out[1]);
                                b_in = w(km_out[2]);
                            } else {
                                r_in = (c00[0] as F * p00
                                    + c10[0] as F * p10
                                    + c01[0] as F * p01
                                    + c11[0] as F * p11)
                                    * inv;
                                g_in = (c00[1] as F * p00
                                    + c10[1] as F * p10
                                    + c01[1] as F * p01
                                    + c11[1] as F * p11)
                                    * inv;
                                b_in = (c00[2] as F * p00
                                    + c10[2] as F * p10
                                    + c01[2] as F * p01
                                    + c11[2] as F * p11)
                                    * inv;
                            }
                            new_susp += want;
                            new_rgb = [r_in as f32, g_in as f32, b_in as f32];
                        }
                        new_film += film[i00] as F * k00
                            + film[i10] as F * k10
                            + film[i01] as F * k01
                            + film[i11] as F * k11;
                    }
                }
                fr[x as usize] = new_film as f32;
                sr[x as usize] = new_susp as f32;
                cr[x as usize] = new_rgb;
                i += 1;
            }
        },
    );
}

#[inline(always)]
fn commit_rows(g: &mut Grid, mode: Mode) {
    let s = g.s;
    let (by0, by1) = (g.by0, g.by1);
    let Grid {
        scratch,
        film,
        susp,
        susp_rgb,
        row_lo,
        row_hi,
        spans_enabled,
        bx0,
        bx1,
        ..
    } = g;
    let (spans_enabled, bx0, bx1) = (*spans_enabled, *bx0, *bx1);
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
            if lo > hi {
                return;
            }
            let base = y as usize * s;
            let (lo, hi) = (lo as usize, hi as usize);
            fr[lo..=hi].copy_from_slice(&df[base + lo..=base + hi]);
            sr[lo..=hi].copy_from_slice(&ds[base + lo..=base + hi]);
            cr[lo..=hi].copy_from_slice(&dc[base + lo..=base + hi]);
        },
    );
}

// ─────────────────────────── secagem (drying.rs) ───────────────────────────

const PIGMENT: f32 = 10.0;

struct EdgeWindow {
    ud: [u32; 3],
    m: [u32; 3],
}

impl EdgeWindow {
    #[inline(always)]
    fn seed(susp: &[f32], i: usize, s: usize) -> Self {
        let mut win = Self {
            ud: [0; 3],
            m: [0; 3],
        };
        for (k, slot) in (0..3).zip(0usize..) {
            let j = i + k - 1;
            win.ud[slot] = u32::from(susp[j - s] > PIGMENT) + u32::from(susp[j + s] > PIGMENT);
            win.m[slot] = u32::from(susp[j] > PIGMENT);
        }
        win
    }
    #[inline(always)]
    fn count(&self) -> u32 {
        self.ud[0] + self.ud[1] + self.ud[2] + self.m[0] + self.m[1] + self.m[2]
    }
    #[inline(always)]
    fn advance(&mut self, susp: &[f32], next_right: usize, s: usize) {
        self.ud[0] = self.ud[1];
        self.ud[1] = self.ud[2];
        self.m[0] = self.m[1];
        self.m[1] = self.m[2];
        self.ud[2] =
            u32::from(susp[next_right - s] > PIGMENT) + u32::from(susp[next_right + s] > PIGMENT);
        self.m[2] = u32::from(susp[next_right] > PIGMENT);
    }
}

#[inline(always)]
fn staining_multiplier(s: F) -> F {
    if s < 0.5 {
        1.0 + (0.5 - s) * 14.0
    } else {
        (1.0 - s) * 2.0
    }
}

#[inline(always)]
fn am(m: F) -> F {
    w(alpha_of_mass(d(m)))
}

#[inline(always)]
fn v3f(c: [f32; 3]) -> [f64; 3] {
    c.map(f64::from)
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn lift_settled(
    b: F,
    susp: &mut f32,
    sett: &mut f32,
    susp_rgb: &mut [f32; 3],
    sett_rgb: [f32; 3],
    mix: ColorMix,
    glaze: f64,
    ecra: Ecra,
    out: &mut [f64; 3],
) {
    let st = *sett as F;
    let lift = st * b;
    if ecra != Ecra::Porto && lift > 0.0 {
        let sf = *susp as F;
        *susp_rgb = ph2d_wet_paint::assenta::levanta(
            ecra,
            v3f(sett_rgb),
            v3f(*susp_rgb),
            d(am(st)),
            d(am(sf)),
            d(am(st - lift)),
            d(am(sf + lift)),
        )
        .map(|v| v as f32);
        *susp = (sf + lift) as f32;
        *sett = (st - lift) as f32;
        return;
    }
    let a_in = am(st) * b;
    let a_susp = am(*susp as F);
    let u = a_susp * (1.0 - a_in);
    if u + a_in > 0.0 {
        let wgt = a_in / (u + a_in);
        mix.mix(
            susp_rgb[0] as f64,
            susp_rgb[1] as f64,
            susp_rgb[2] as f64,
            sett_rgb[0] as f64,
            sett_rgb[1] as f64,
            sett_rgb[2] as f64,
            d(wgt),
            out,
        );
        ph2d_wet_paint::colorops::velatura(
            out,
            d(wgt),
            glaze,
            d(a_susp),
            v3f(*susp_rgb),
            v3f(sett_rgb),
        );
        *susp_rgb = [out[0] as f32, out[1] as f32, out[2] as f32];
    } else {
        *susp_rgb = sett_rgb;
    }
    *susp = (*susp as F + lift) as f32;
    *sett = (st - lift) as f32;
}

struct DryConst {
    retention: F,
    edge_darkening: F,
    base_evaporation: F,
    ext_granulation: F,
    ext_staining: F,
    evap_base: F,
    rewet_base: F,
    ext_bypass: bool,
    mix: ColorMix,
    glaze: f64,
    ecra: Ecra,
}

impl DryConst {
    #[inline(always)]
    fn new(p: &Params, evap_base: f64, rewet_base: f64, ext_bypass: bool) -> Self {
        DryConst {
            retention: w(p.k(Knob::Retention)),
            edge_darkening: w(p.k(Knob::EdgeDarkening)),
            base_evaporation: w(p.k(Knob::BaseEvaporation)),
            ext_granulation: w(p.k(Knob::ExtGranulation)),
            ext_staining: w(p.k(Knob::ExtStaining)),
            evap_base: w(evap_base),
            rewet_base: w(rewet_base),
            ext_bypass,
            mix: p.mix,
            glaze: p.glaze,
            ecra: p.ecra,
        }
    }
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn dry_cell(
    c: &DryConst,
    paper: f32,
    count: u32,
    vela: f32,
    vidrado: f32,
    oleo: oleo_celula::OleoCelula,
    film: &mut f32,
    susp: &mut f32,
    sett: &mut f32,
    susp_rgb: &mut [f32; 3],
    sett_rgb: &mut [f32; 3],
    out: &mut [f64; 3],
) {
    let film0 = *film;
    let f = film0 as F;
    let s_mass = *susp as F;
    let mut susp_c = *susp;
    let mut sett_c = *sett;
    let mut susp_rgb_c = *susp_rgb;
    let mut sett_rgb_c = *sett_rgb;
    let mix = c.mix;
    let ecra = c.ecra.com_vela(vela).com_vidrado(vidrado).com_oleo(oleo);

    let mut e: F = 1.0;
    if film0 > 0.0 && sett_c < 1000.0 {
        e = if count == 9 { 1.0 } else { count as F / 9.0 };
    }
    let mut new_film =
        f * c.retention - c.evap_base * ((1.0 - e) * c.edge_darkening + c.base_evaporation);
    if new_film < 0.0001 {
        new_film = 0.0;
    }
    let lost = if f > 0.0 {
        1.0 - w(clamp01(d(new_film / f)))
    } else {
        1.0
    };
    let film_c: f32 = new_film as f32;

    if lost > 0.0 && s_mass > 0.0 {
        let mut dm = s_mass * lost;
        if !c.ext_bypass {
            let mut bias = 1.0 + c.ext_granulation * 0.6 * (0.5 - paper as F) * 2.0;
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
        let a_sett = am(sett_c as F);
        let a_in = am(dm);
        if ecra != Ecra::Porto {
            sett_rgb_c = ph2d_wet_paint::assenta::assenta(
                ecra,
                v3f(sett_rgb_c),
                v3f(susp_rgb_c),
                d(a_sett),
                d(am(s_mass)),
                d(am(sett_c as F + dm)),
                d(am(s_mass - dm)),
            )
            .map(|v| v as f32);
        } else if a_sett > 0.0 {
            let u = a_sett * (1.0 - a_in);
            let wgt = a_in / (u + a_in);
            mix.mix(
                sett_rgb_c[0] as f64,
                sett_rgb_c[1] as f64,
                sett_rgb_c[2] as f64,
                susp_rgb_c[0] as f64,
                susp_rgb_c[1] as f64,
                susp_rgb_c[2] as f64,
                d(wgt),
                out,
            );
            ph2d_wet_paint::colorops::velatura(
                out,
                d(wgt),
                c.glaze,
                d(a_sett),
                v3f(sett_rgb_c),
                v3f(susp_rgb_c),
            );
            sett_rgb_c = [out[0] as f32, out[1] as f32, out[2] as f32];
        } else {
            sett_rgb_c = susp_rgb_c;
        }
        sett_c = (sett_c as F + dm) as f32;
        let rem = s_mass - dm;
        susp_c = if rem < 0.0 { 0.0 } else { rem as f32 };
    }

    let st = sett_c as F;
    if film_c > 0.0 && st > 0.0 {
        let excess = (film_c as F - am(susp_c as F)).max(0.0);
        let mut b = c.rewet_base * (1.0 + excess * 50.0);
        if !c.ext_bypass {
            b *= staining_multiplier(c.ext_staining);
        }
        b = w(clamp01(d(b)));
        if b > 0.0 {
            lift_settled(
                b,
                &mut susp_c,
                &mut sett_c,
                &mut susp_rgb_c,
                sett_rgb_c,
                mix,
                c.glaze,
                ecra,
                out,
            );
        }
    }

    *film = film_c;
    *susp = susp_c;
    *sett = sett_c;
    *susp_rgb = susp_rgb_c;
    *sett_rgb = sett_rgb_c;
}

#[inline(always)]
fn edge_rows(g: &mut Grid, mode: Mode) {
    let s = g.s;
    let (by0, by1) = (g.by0, g.by1);
    let Grid {
        scratch,
        susp,
        row_lo,
        row_hi,
        spans_enabled,
        bx0,
        bx1,
        ..
    } = g;
    let (spans_enabled, bx0, bx1) = (*spans_enabled, *bx0, *bx1);
    let susp: &[f32] = susp;
    let band = by0 as usize * s..(by1 as usize + 1) * s;
    par::walk_rows(mode, &mut scratch.edge[band], s, |ri, row| {
        let y = by0 + ri as i32;
        let (lo, hi) = span_x_of(row_lo, row_hi, spans_enabled, bx0, bx1, y);
        if lo > hi {
            return;
        }
        let mut i = lo as usize + y as usize * s;
        let mut win = EdgeWindow::seed(susp, i, s);
        for x in lo..=hi {
            if x > lo {
                win.advance(susp, i + 1, s);
            }
            row[x as usize] = win.count() as u8;
            i += 1;
        }
    });
}

#[inline(always)]
pub fn drying_pass_jacobi_rows(
    g: &mut Grid,
    p: &Params,
    evap_base: f64,
    rewet_base: f64,
    ext_bypass: bool,
    mode: Mode,
) {
    if g.by0 > g.by1 {
        return;
    }
    g.scratch.ensure(g.cells);
    edge_rows(g, mode);

    let s = g.s;
    let c = DryConst::new(p, evap_base, rewet_base, ext_bypass);
    let (by0, by1) = (g.by0, g.by1);
    let Grid {
        scratch,
        film,
        susp,
        sett,
        susp_rgb,
        sett_rgb,
        paper,
        vela,
        aparencia,
        oleo,
        row_lo,
        row_hi,
        spans_enabled,
        bx0,
        bx1,
        ..
    } = g;
    let (spans_enabled, bx0, bx1) = (*spans_enabled, *bx0, *bx1);
    let edge: &[u8] = &scratch.edge;
    let paper: &[f32] = paper;
    let vela: &[f32] = vela;
    let aparencia_c: &aparencia::CampoAparencia = aparencia;
    let oleo: &[oleo_celula::OleoCelula] = oleo;
    let band = by0 as usize * s..(by1 as usize + 1) * s;
    par::walk_rows5(
        mode,
        &mut film[band.clone()],
        &mut susp[band.clone()],
        &mut sett[band.clone()],
        &mut susp_rgb[band.clone()],
        &mut sett_rgb[band],
        s,
        |ri, fr, sr, tr, cr, dr| {
            let y = by0 + ri as i32;
            let (lo, hi) = span_x_of(row_lo, row_hi, spans_enabled, bx0, bx1, y);
            if lo > hi {
                return;
            }
            let mut out = [0.0f64; 3];
            let base = y as usize * s;
            for x in lo as usize..=hi as usize {
                if fr[x] <= 0.0 && sr[x] <= 0.0 {
                    continue;
                }
                let i = base + x;
                let (mut a, mut b, mut dd) = (fr[x], sr[x], tr[x]);
                let (mut e, mut f) = (cr[x], dr[x]);
                dry_cell(
                    &c,
                    paper[i],
                    u32::from(edge[i]),
                    vela[i],
                    aparencia::vidrado(aparencia_c, i),
                    oleo_celula::da_celula(oleo, i),
                    &mut a,
                    &mut b,
                    &mut dd,
                    &mut e,
                    &mut f,
                    &mut out,
                );
                fr[x] = a;
                sr[x] = b;
                tr[x] = dd;
                cr[x] = e;
                dr[x] = f;
            }
        },
    );
}
