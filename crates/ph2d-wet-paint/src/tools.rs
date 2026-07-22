//! Direct tools (port of `tools.js`, SPEC §11): erase, wet, dry, blow, smear.
//! Each dispatches per dab (no trail), with the fixed tool hardness 4.8 and
//! stamp = min(1, falloff * texture * min(3, intensity)). All respect the
//! brushable area (the stamp iterator enforces it) and expand the active
//! bbox.

use crate::brush::{for_each_stamp_pixel, for_each_stamp_pixel_shaped};
use crate::drying::{lift_settled, staining_multiplier};
use crate::grid::{Grid, wet_byte_from_paper};
use crate::jsmath::{clamp01, js_round};
use crate::opacity::alpha_of_mass;
use crate::sim::Params;
use crate::trail::{Dab, TouchedRect};
use crate::tuning::Knob;

/// Doc 23: the shared active-lift gain — `wetLift · stainingMultiplier`.
/// Every active rewetter (Wet dissolve, wet Smear, wet Blend) derives its
/// per-pixel `b` from this ONE product, so "how re-soluble is dried paint"
/// has a single answer across tools. `wetLift = 0` or `staining = 1` turn
/// every active lift off exactly.
#[inline]
pub(crate) fn active_lift_gain(p: &Params) -> f64 {
    p.k(Knob::WetLift) * staining_multiplier(p.k(Knob::ExtStaining))
}

pub const TOOL_HARDNESS: f64 = 4.8;

fn rect_around(g: &Grid, x: f64, y: f64, r: f64) -> TouchedRect {
    TouchedRect {
        x0: ((x - r).floor() as i32 - 1).max(2),
        y0: ((y - r).floor() as i32 - 1).max(2),
        x1: ((x + r).ceil() as i32 + 1).min(g.w as i32 - 1),
        y1: ((y + r).ceil() as i32 + 1).min(g.h as i32 - 1),
    }
}

fn expand_to(g: &mut Grid, r: &TouchedRect) {
    g.expand_bbox(r.x0, r.y0, r.x1, r.y1);
}

/// erase: paper-gated multiplicative removal of both layers + water.
pub fn apply_erase(
    g: &mut Grid,
    p: &Params,
    tex: &[f32],
    dab: &Dab,
    erase_slider: f64,
) -> Option<TouchedRect> {
    apply_erase_impl(g, p, tex, dab, erase_slider, None, None)
}

/// [`apply_erase`] with the HOST's silhouette + optional grain (the shaped
/// product door, twin of `Trail::accumulate_paint_shaped`): `sil` replaces
/// the internal falloff + footprint, `grain` replaces the bristle. ONE pixel
/// body serves both faces — `apply_erase` delegates with `None`s.
pub fn apply_erase_shaped(
    g: &mut Grid,
    p: &Params,
    tex: &[f32],
    dab: &Dab,
    erase_slider: f64,
    sil: &mut dyn FnMut(i32, i32) -> f64,
    grain: Option<&mut dyn FnMut(i32, i32) -> f64>,
) -> Option<TouchedRect> {
    apply_erase_impl(g, p, tex, dab, erase_slider, Some(sil), grain)
}

fn apply_erase_impl(
    g: &mut Grid,
    p: &Params,
    tex: &[f32],
    dab: &Dab,
    erase_slider: f64,
    sil: Option<&mut dyn FnMut(i32, i32) -> f64>,
    grain: Option<&mut dyn FnMut(i32, i32) -> f64>,
) -> Option<TouchedRect> {
    let gate = p.k(Knob::PaperGate);
    let pig_cap = p.k(Knob::GateSaturation);
    let force = p.k(Knob::Eraser) * erase_slider;
    // Doc 23 P4 — dried, staining paint resists the eraser: the settled
    // layer takes a weaker force (`force · (1 − 0.5·staining)`; staining 0
    // erases like the wet layers, staining 1 at half force). Suspended
    // pigment and water keep the full force — they are not bound to the
    // fibre.
    let force_sett = force * (1.0 - 0.5 * p.k(Knob::ExtStaining));
    let clamp_i = dab.intensity.min(3.0);
    let mut touched = false;
    let Grid {
        s,
        w,
        h,
        susp,
        sett,
        film,
        paper,
        ..
    } = g;
    let _ = s;
    let body = |i: usize, _x: i32, _y: i32, fall: f64, texv: f64| {
        let mut stamp = fall * texv * clamp_i;
        if stamp > 1.0 {
            stamp = 1.0;
        }
        let tooth = if (susp[i] as f64 + sett[i] as f64) < pig_cap {
            paper[i] as f64
        } else {
            0.45
        };
        let mut gg = stamp - (1.0 - tooth) * gate;
        if gg <= 0.0 {
            return;
        }
        if gg > 1.0 {
            gg = 1.0;
        }
        let k = 1.0 - force * gg;
        if k <= 0.0 {
            return; // never wipe a cell outright
        }
        susp[i] = (susp[i] as f64 * k) as f32;
        film[i] = (film[i] as f64 * k) as f32;
        if (susp[i] as f64) < 1.0 {
            susp[i] = 0.0; // stale color bytes may remain; mass 0 hides them
        }
        let k_sett = 1.0 - force_sett * gg;
        sett[i] = (sett[i] as f64 * k_sett) as f32;
        if (sett[i] as f64) < 1.0 {
            sett[i] = 0.0;
        }
        touched = true;
    };
    match sil {
        Some(sil) => {
            for_each_stamp_pixel_shaped(*s, *w, *h, tex, dab.x, dab.y, dab.r, sil, grain, body);
        }
        None => {
            debug_assert!(grain.is_none(), "grain override requires the shaped path");
            for_each_stamp_pixel(
                *s,
                *w,
                *h,
                tex,
                dab.x,
                dab.y,
                dab.r,
                TOOL_HARDNESS,
                dab.shape,
                dab.dir_x,
                dab.dir_y,
                body,
            );
        }
    }
    if touched {
        let r = rect_around(g, dab.x, dab.y, dab.r);
        expand_to(g, &r);
        Some(r)
    } else {
        None
    }
}

/// wet: drip-feed water; gentle, accumulates over repeated passes — and
/// (doc 23 P1) an ACTIVE lift: the stamp dissolves settled pigment back
/// into suspension on the spot. Watercolor is re-soluble (the gum-arabic
/// binder re-dissolves; "lifting" is a standard technique) and Rebelle's
/// Re-wet is stroke-driven — the passive re-wet in the drying pass is
/// ~1%/s ambience and cannot serve a gesture. `wetLift = 0` restores the
/// water-only model to the byte.
pub fn apply_wet(
    g: &mut Grid,
    p: &Params,
    tex: &[f32],
    dab: &Dab,
    water_unit: f64,
) -> Option<TouchedRect> {
    let add = 2.0 * (water_unit * 0.2) * (water_unit * 0.2) * p.k(Knob::WaterPerDab);
    let cap = p.k(Knob::WaterCap);
    let lift_gain = active_lift_gain(p);
    let mix = p.mix;
    let mut out = [0.0f64; 3];
    let clamp_i = dab.intensity.min(3.0);
    let mut touched = false;
    let Grid {
        s,
        w,
        h,
        film,
        paper,
        wet,
        susp,
        sett,
        susp_rgb,
        sett_rgb,
        ..
    } = g;
    for_each_stamp_pixel(
        *s,
        *w,
        *h,
        tex,
        dab.x,
        dab.y,
        dab.r,
        TOOL_HARDNESS,
        dab.shape,
        dab.dir_x,
        dab.dir_y,
        |i, _x, _y, fall, texv| {
            let mut stamp = fall * texv * clamp_i;
            if stamp > 1.0 {
                stamp = 1.0;
            }
            if stamp <= 0.0 {
                return;
            }
            let mut f = film[i] as f64 + stamp * add;
            if f > cap {
                f = cap;
            }
            film[i] = f as f32;
            wet[i] = wet_byte_from_paper(paper[i] as f64);
            // The active lift, through the SAME door as the passive re-wet
            // (drying.rs) — the freshly suspended pigment then bleeds,
            // follows gravity and answers the Blow/Smear tools.
            if lift_gain > 0.0 && sett[i] > 0.0 {
                let b = clamp01(stamp * lift_gain);
                if b > 0.0 {
                    lift_settled(
                        b,
                        &mut susp[i],
                        &mut sett[i],
                        &mut susp_rgb[i],
                        sett_rgb[i],
                        mix,
                        &mut out,
                    );
                }
            }
            touched = true;
        },
    );
    if touched {
        let r = rect_around(g, dab.x, dab.y, dab.r);
        expand_to(g, &r);
        Some(r)
    } else {
        None
    }
}

/// dry: shrink the film (never to exact zero) and SEAL the paper (wet = 0).
pub fn apply_dry(g: &mut Grid, p: &Params, tex: &[f32], dab: &Dab) -> Option<TouchedRect> {
    let force = p.k(Knob::Dryer);
    let clamp_i = dab.intensity.min(3.0);
    let mut touched = false;
    let Grid {
        s, w, h, film, wet, ..
    } = g;
    for_each_stamp_pixel(
        *s,
        *w,
        *h,
        tex,
        dab.x,
        dab.y,
        dab.r,
        TOOL_HARDNESS,
        dab.shape,
        dab.dir_x,
        dab.dir_y,
        |i, _x, _y, fall, texv| {
            let mut stamp = fall * texv * clamp_i;
            if stamp > 1.0 {
                stamp = 1.0;
            }
            if stamp <= 0.0 {
                return;
            }
            let mut f = film[i] as f64 * (0.999 - stamp * force);
            if f < 0.001 {
                f = 0.001; // floor: the wet map must know the stroke passed
            }
            film[i] = f as f32;
            wet[i] = 0; // sealed: future bleed stops here
            touched = true;
        },
    );
    if touched {
        let r = rect_around(g, dab.x, dab.y, dab.r);
        expand_to(g, &r);
        Some(r)
    } else {
        None
    }
}

/// blow: needs the sim live. Injects the (asymmetrically clamped) pointer
/// displacement into the PERSISTENT velocity of wet cells, and drags the
/// wetness byte from the drag source through a read-before-write snapshot
/// (even across dry cells).
pub fn apply_blow(
    g: &mut Grid,
    p: &Params,
    tex: &[f32],
    dab: &Dab,
    prev_x: f64,
    prev_y: f64,
) -> Option<TouchedRect> {
    let force = p.k(Knob::Blow);
    let clamp_i = dab.intensity.min(3.0);
    let raw_dx = dab.x - prev_x;
    let raw_dy = dab.y - prev_y;
    // Weak, slightly anti-forward: each axis clamps to [-0.6, +0.3].
    let cdx = raw_dx.clamp(-0.6, 0.3);
    let cdy = raw_dy.clamp(-0.6, 0.3);
    let off_x = js_round(raw_dx) as i32; // UNCLAMPED source offset
    let off_y = js_round(raw_dy) as i32;
    // Snapshot the wetness in the affected region so the drag reads
    // pre-write values (sampling live bytes would compound the drag within
    // one dab).
    let r = rect_around(
        g,
        dab.x,
        dab.y,
        dab.r + off_x.abs().max(off_y.abs()) as f64 + 1.0,
    );
    let snap_w = (r.x1 - r.x0 + 1) as usize;
    let snap_h = (r.y1 - r.y0 + 1) as usize;
    let mut snap = vec![0u8; snap_w * snap_h];
    for y in r.y0..=r.y1 {
        for x in r.x0..=r.x1 {
            snap[(x - r.x0) as usize + (y - r.y0) as usize * snap_w] =
                g.wet[x as usize + y as usize * g.s];
        }
    }
    let mut touched = false;
    let w_i32 = g.w as i32;
    let h_i32 = g.h as i32;
    let Grid {
        s,
        w,
        h,
        film,
        paper,
        wet,
        vel_x,
        vel_y,
        ..
    } = g;
    let stride = *s;
    for_each_stamp_pixel(
        *s,
        *w,
        *h,
        tex,
        dab.x,
        dab.y,
        dab.r,
        TOOL_HARDNESS,
        dab.shape,
        dab.dir_x,
        dab.dir_y,
        |i, x, y, fall, texv| {
            let mut stamp = fall * texv * clamp_i;
            if stamp > 1.0 {
                stamp = 1.0;
            }
            if stamp <= 0.0 {
                return;
            }
            // Skip the whole pixel when the UNCLAMPED, rounded source leaves
            // the sheet.
            let sx = x - off_x;
            let sy = y - off_y;
            if sx < 1 || sx > w_i32 || sy < 1 || sy > h_i32 {
                return;
            }
            let e = stamp * force;
            if film[i] > 0.0 {
                vel_x[i] = (vel_x[i] as f64 + cdx * e) as f32;
                vel_y[i] = (vel_y[i] as f64 + cdy * e) as f32;
                wet[i] = wet_byte_from_paper(paper[i] as f64); // re-wet under the air jet
            }
            // Wetness dragged from the source wherever the stamp touches,
            // wet or dry — through the read-before-write snapshot.
            let src = if sx < r.x0 || sx > r.x1 || sy < r.y0 || sy > r.y1 {
                wet[sx as usize + sy as usize * stride]
            } else {
                snap[(sx - r.x0) as usize + (sy - r.y0) as usize * snap_w]
            };
            wet[i] = (src as f64 * force + wet[i] as f64 * (1.0 - force)) as u8;
            touched = true;
        },
    );
    if touched {
        let rr = rect_around(g, dab.x, dab.y, dab.r);
        expand_to(g, &rr);
        Some(rr)
    } else {
        None
    }
}

/// smear: drag suspended pigment + water from the previous-dab offset (the
/// reach grows with speed). The source is a mass-weighted bilinear sample
/// of a pre-dab snapshot (weighting each corner by its mass keeps edges
/// from dragging toward black).
///
/// Doc 23 P2 — the settled layer is no longer untouchable (the old note
/// said "matching real smudge behavior"; measured against Rebelle's Smudge
/// and real re-soluble watercolor, it wasn't): on a WET cell the pass
/// dissolves settled pigment through the lift door (the next dab's
/// snapshot then drags what this one freed — smudging through wet paint IS
/// dissolve-then-drag); on a DRY cell the settled layer itself is dragged,
/// like every raster smudge. Both are resisted by `1 − staining`, so
/// `staining = 1` restores the suspended-only model to the byte.
pub fn apply_smear(
    g: &mut Grid,
    p: &Params,
    tex: &[f32],
    dab: &Dab,
    prev_x: f64,
    prev_y: f64,
) -> Option<TouchedRect> {
    let force = p.k(Knob::Smear);
    let sett_gain = 1.0 - p.k(Knob::ExtStaining);
    let clamp_i = dab.intensity.min(3.0);
    let dx = dab.x - prev_x;
    let dy = dab.y - prev_y;
    let reach = dx.abs().max(dy.abs()).ceil();
    // Snapshot the affected region (+10 px pad): sampling live arrays would
    // compound the drag within one dab.
    let pad = 10.0 + reach;
    let r = rect_around(g, dab.x, dab.y, dab.r + pad);
    let snap_w = (r.x1 - r.x0 + 1) as usize;
    let snap_h = (r.y1 - r.y0 + 1) as usize;
    let n = snap_w * snap_h;
    let mut s_mass = vec![0.0f32; n];
    let mut s_rgb = vec![[0.0f32; 3]; n];
    let mut s_f = vec![0.0f32; n];
    let mut s_sett = vec![0.0f32; n];
    let mut s_sett_rgb = vec![[0.0f32; 3]; n];
    for y in r.y0..=r.y1 {
        for x in r.x0..=r.x1 {
            let i = x as usize + y as usize * g.s;
            let l = (x - r.x0) as usize + (y - r.y0) as usize * snap_w;
            s_mass[l] = g.susp[i];
            s_rgb[l] = g.susp_rgb[i];
            s_f[l] = g.film[i];
            s_sett[l] = g.sett[i];
            s_sett_rgb[l] = g.sett_rgb[i];
        }
    }
    let mut out = [0.0f64; 3];
    let mix = p.mix;
    let mut touched = false;
    let Grid {
        s,
        w,
        h,
        susp,
        susp_rgb,
        sett,
        sett_rgb,
        film,
        ..
    } = g;
    for_each_stamp_pixel(
        *s,
        *w,
        *h,
        tex,
        dab.x,
        dab.y,
        dab.r,
        TOOL_HARDNESS,
        dab.shape,
        dab.dir_x,
        dab.dir_y,
        |i, x, y, fall, texv| {
            let mut stamp = fall * texv * clamp_i;
            if stamp > 1.0 {
                stamp = 1.0;
            }
            let u = (stamp * force).min(1.0);
            if u <= 0.0 {
                return;
            }
            // Clamp the sample offset to the snapshot pad.
            let mut fx = x as f64 - dx;
            let mut fy = y as f64 - dy;
            if fx < r.x0 as f64 {
                fx = r.x0 as f64;
            }
            if fx > (r.x1 - 1) as f64 {
                fx = (r.x1 - 1) as f64;
            }
            if fy < r.y0 as f64 {
                fy = r.y0 as f64;
            }
            if fy > (r.y1 - 1) as f64 {
                fy = (r.y1 - 1) as f64;
            }
            let x0 = fx.floor() as i32;
            let y0 = fy.floor() as i32;
            let tx = fx - x0 as f64;
            let ty = fy - y0 as f64;
            let l00 = (x0 - r.x0) as usize + (y0 - r.y0) as usize * snap_w;
            let l10 = l00 + 1;
            let l01 = l00 + snap_w;
            let l11 = l01 + 1;
            let w00 = (1.0 - tx) * (1.0 - ty);
            let w10 = tx * (1.0 - ty);
            let w01 = (1.0 - tx) * ty;
            let w11 = tx * ty;
            // Mass-weighted color sample.
            let m00 = s_mass[l00] as f64 * w00;
            let m10 = s_mass[l10] as f64 * w10;
            let m01 = s_mass[l01] as f64 * w01;
            let m11 = s_mass[l11] as f64 * w11;
            let src_mass = m00 + m10 + m01 + m11;
            if src_mass > 0.0 {
                let inv = 1.0 / src_mass;
                let k00 = s_rgb[l00];
                let k10 = s_rgb[l10];
                let k01 = s_rgb[l01];
                let k11 = s_rgb[l11];
                let cr = (k00[0] as f64 * m00
                    + k10[0] as f64 * m10
                    + k01[0] as f64 * m01
                    + k11[0] as f64 * m11)
                    * inv;
                let cg = (k00[1] as f64 * m00
                    + k10[1] as f64 * m10
                    + k01[1] as f64 * m01
                    + k11[1] as f64 * m11)
                    * inv;
                let cbv = (k00[2] as f64 * m00
                    + k10[2] as f64 * m10
                    + k01[2] as f64 * m01
                    + k11[2] as f64 * m11)
                    * inv;
                let in_c = alpha_of_mass(src_mass) * u;
                let res = alpha_of_mass(susp[i] as f64) * (1.0 - in_c);
                if in_c + res > 0.0 {
                    let wgt = in_c / (in_c + res);
                    let uc = susp_rgb[i];
                    mix.mix(
                        uc[0] as f64,
                        uc[1] as f64,
                        uc[2] as f64,
                        cr,
                        cg,
                        cbv,
                        wgt,
                        &mut out,
                    );
                    susp_rgb[i] = [out[0] as f32, out[1] as f32, out[2] as f32];
                }
                susp[i] = (susp[i] as f64 + (src_mass - susp[i] as f64) * u) as f32;
            }
            // Water lerps unconditionally (plain bilinear).
            let src_film = s_f[l00] as f64 * w00
                + s_f[l10] as f64 * w10
                + s_f[l01] as f64 * w01
                + s_f[l11] as f64 * w11;
            film[i] = (film[i] as f64 + (src_film - film[i] as f64) * u) as f32;
            // Doc 23 P2 — the settled layer, resisted by staining.
            let u_sett = u * sett_gain;
            if u_sett > 0.0 {
                if film[i] as f64 > 0.001 {
                    // WET cell: dissolve through the lift door; the susp
                    // drag above carries it from the next dab on.
                    if sett[i] > 0.0 {
                        let b = clamp01(u_sett);
                        lift_settled(
                            b,
                            &mut susp[i],
                            &mut sett[i],
                            &mut susp_rgb[i],
                            sett_rgb[i],
                            mix,
                            &mut out,
                        );
                    }
                } else {
                    // DRY cell: drag the settled layer itself (same
                    // mass-weighted bilinear sample as the suspended pass).
                    let sm00 = s_sett[l00] as f64 * w00;
                    let sm10 = s_sett[l10] as f64 * w10;
                    let sm01 = s_sett[l01] as f64 * w01;
                    let sm11 = s_sett[l11] as f64 * w11;
                    let src_sett = sm00 + sm10 + sm01 + sm11;
                    if src_sett > 0.0 {
                        let inv = 1.0 / src_sett;
                        let k00 = s_sett_rgb[l00];
                        let k10 = s_sett_rgb[l10];
                        let k01 = s_sett_rgb[l01];
                        let k11 = s_sett_rgb[l11];
                        let cr = (k00[0] as f64 * sm00
                            + k10[0] as f64 * sm10
                            + k01[0] as f64 * sm01
                            + k11[0] as f64 * sm11)
                            * inv;
                        let cg = (k00[1] as f64 * sm00
                            + k10[1] as f64 * sm10
                            + k01[1] as f64 * sm01
                            + k11[1] as f64 * sm11)
                            * inv;
                        let cbv = (k00[2] as f64 * sm00
                            + k10[2] as f64 * sm10
                            + k01[2] as f64 * sm01
                            + k11[2] as f64 * sm11)
                            * inv;
                        let in_c = alpha_of_mass(src_sett) * u_sett;
                        let res = alpha_of_mass(sett[i] as f64) * (1.0 - in_c);
                        if in_c + res > 0.0 {
                            let wgt = in_c / (in_c + res);
                            let sc = sett_rgb[i];
                            mix.mix(
                                sc[0] as f64,
                                sc[1] as f64,
                                sc[2] as f64,
                                cr,
                                cg,
                                cbv,
                                wgt,
                                &mut out,
                            );
                            sett_rgb[i] = [out[0] as f32, out[1] as f32, out[2] as f32];
                        }
                    }
                    if src_sett > 0.0 || sett[i] > 0.0 {
                        sett[i] = (sett[i] as f64 + (src_sett - sett[i] as f64) * u_sett) as f32;
                    }
                }
            }
            touched = true;
        },
    );
    if touched {
        let rr = rect_around(g, dab.x, dab.y, dab.r);
        expand_to(g, &rr);
        Some(rr)
    } else {
        None
    }
}
