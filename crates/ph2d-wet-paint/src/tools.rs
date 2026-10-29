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

mod bordas; // a frente da água que empurra e a orla do secador — filho por LOC
pub use bordas::{apply_dry_edge, apply_wet_push};
mod marca; // o Smear leva a marca do nivelado (K9)
mod secador; // o secador seca a tinta já nivelada (J6) — filho por LOC
pub use secador::apply_dry_level;
mod smear; // o Smear: arrasta a tinta do dab anterior (Doc 23 P2) — filho por LOC
pub use smear::apply_smear;

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

/// O [`rect_around`] de quem o usa ANTES de pintar (a fotografia da Blow e da Smear): `None` quando o
/// carimbo cai inteiro fora do interior. Os dois cantos prendem-se por lados opostos, e aí o
/// rectângulo sai INVERTIDO — a largura negativa feita `usize` era um pânico em debug e um pedido de
/// memória astronómico em release (o rato fora da tela; gate `a_dab_outside_the_grid_touches_nothing_…`).
fn rect_around_nao_vazio(g: &Grid, x: f64, y: f64, r: f64) -> Option<TouchedRect> {
    let rr = rect_around(g, x, y, r);
    (rr.x1 >= rr.x0 && rr.y1 >= rr.y0).then_some(rr)
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
    sil: crate::brush::CellFn<'_>,
    grain: Option<crate::brush::CellFn<'_>>,
) -> Option<TouchedRect> {
    apply_erase_impl(g, p, tex, dab, erase_slider, Some(sil), grain)
}

fn apply_erase_impl(
    g: &mut Grid,
    p: &Params,
    tex: &[f32],
    dab: &Dab,
    erase_slider: f64,
    sil: Option<crate::brush::CellFn<'_>>,
    grain: Option<crate::brush::CellFn<'_>>,
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
            let r = dab.r * dab.reach;
            for_each_stamp_pixel_shaped(*s, *w, *h, tex, dab.x, dab.y, r, sil, grain, body);
        }
        None => {
            debug_assert!(grain.is_none(), "grain override requires the shaped path");
            for_each_stamp_pixel(*s, *w, *h, tex, dab.x, dab.y, dab.r, TOOL_HARDNESS, body);
        }
    }
    if touched {
        let r = rect_around(g, dab.x, dab.y, dab.r * dab.reach);
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
        vela,
        aparencia,
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
                        p.glaze,
                        p.ecra
                            .com_vela(vela[i])
                            .com_vidrado(crate::grid::aparencia::vidrado(aparencia, i)),
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
    let r = rect_around_nao_vazio(
        g,
        dab.x,
        dab.y,
        dab.r + off_x.abs().max(off_y.abs()) as f64 + 1.0,
    )?;
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
    let geom = g.flow;
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
                // ⚠️ O impulso vai para a célula de FLUXO, e só a célula PROBE
                // do bloco o entrega — as `rf²` células finas de um bloco
                // somariam `rf²` vezes o mesmo empurrão, e o sopro sairia `rf²`
                // vezes forte. É a mesma lei da posse do momento no `advect`.
                if crate::flow::is_probe_cell(x, y, w_i32, h_i32, geom.rf) {
                    let vi = if geom.is_identity() {
                        i
                    } else {
                        geom.idx(
                            crate::flow::fine_to_flow(x, geom.rf),
                            crate::flow::fine_to_flow(y, geom.rf),
                        )
                    };
                    vel_x[vi] = (vel_x[vi] as f64 + cdx * e) as f32;
                    vel_y[vi] = (vel_y[vi] as f64 + cdy * e) as f32;
                }
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
