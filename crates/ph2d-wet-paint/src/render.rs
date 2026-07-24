//! Renderer (port of `render.js`, SPEC §13): composites the interior of the
//! layer grids into an RGBA byte image (cell (x,y) -> pixel (x-1, y-1)).
//! Pure — the shell owns pixels; the export path reuses this.
//!
//! The look, per pixel: a paper-tinted near-white base; each pigment layer
//! alpha-composited settled-then-suspended, with the tooth offset
//! (granulation) and a SIGNED mass-gradient emboss added INTO the pigment
//! color channels so grain shows through the paint; an optional damp overlay
//! (cool darkening + a meniscus glint on the film's rim); optional K–M glaze
//! stacking instead of alpha-over. A render-only "paper visibility" master
//! fades the tooth in the appearance without ever touching the physics array.

use crate::colorops::km_glaze_channel_linear;
use crate::colorops::transfer::{srgb255_of_linear, srgb255_to_linear};
use crate::grid::Grid;
use crate::jsmath::clamp_u8;
use crate::opacity::alpha_of_mass;
use crate::rng::hash2;
use crate::sim::Params;
use crate::tuning::Knob;

const INV_SQRT2: f64 = 0.707_106_781_186_547_6;
const LIGHT_X: f64 = -INV_SQRT2; // light from top-left
const LIGHT_Y: f64 = -INV_SQRT2;

#[inline]
fn smoothstep01(t: f64) -> f64 {
    if t <= 0.0 {
        0.0
    } else if t >= 1.0 {
        1.0
    } else {
        t * t * (3.0 - 2.0 * t)
    }
}

/// One layer as the renderer sees it. The slice passed to [`render_region`]
/// holds ALL layers in stack order (visibility filtered inside — layer 0 is
/// the paper source even when hidden).
pub struct RenderLayer<'a> {
    pub grid: &'a Grid,
    pub opacity: f64,
    pub visible: bool,
}

/// Render a sub-rectangle (cell coords, inclusive, [1..W] x [1..H]) of the
/// layer stack into `out` (RGBA8, W*H*4).
#[allow(clippy::too_many_arguments)]
pub fn render_region(
    p: &Params,
    layers: &[RenderLayer<'_>],
    active: &Grid,
    show_wet: bool,
    km_glaze: bool,
    ext_bypass: bool,
    out: &mut [u8],
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
) {
    let base0 = layers[0].grid; // paper source (baked identically everywhere)
    let s = base0.s;
    let w = base0.w;
    let paper_vis = p.k(Knob::PaperVisibility);
    let grain_vis = p.k(Knob::VisualGrain);
    let emboss_k = p.k(Knob::Emboss);
    let ext = !ext_bypass;
    let rake = if ext { p.k(Knob::ExtRakeLight) } else { 0.0 };
    let valley = if ext { p.k(Knob::ExtValleyGran) } else { 0.0 };
    let sheen = if ext { p.k(Knob::ExtWetSheen) } else { 0.0 };
    let dither = if ext { p.k(Knob::ExtGrainDither) } else { 0.0 };
    let edge_tint = if ext {
        (p.k(Knob::ExtEdgeTint) - 0.5) * 2.0
    } else {
        0.0
    }; // signed, 0 = neutral

    for cy in y0..=y1 {
        let mut i = x0 as usize + cy as usize * s;
        let mut o = ((cy - 1) as usize * w + (x0 - 1) as usize) * 4;
        for cx in x0..=x1 {
            let pap_raw = base0.paper[i] as f64;
            let pap = 0.5 + (pap_raw - 0.5) * paper_vis;
            // Base: the sheet, paper-tinted near-white.
            let mut r = 255.0 + (pap * 30.0 - 30.0);
            let mut g = r;
            let mut b = r;
            // Rake light (extension): grazing light across the paper relief.
            if rake > 0.0 {
                let gx = (base0.paper[i + 1] as f64 - base0.paper[i - 1] as f64) * 0.5;
                let gy = (base0.paper[i + s] as f64 - base0.paper[i - s] as f64) * 0.5;
                let mut lit = (gx * LIGHT_X + gy * LIGHT_Y) * 90.0 * rake;
                lit = lit.clamp(-12.0, 12.0);
                r += lit;
                g += lit;
                b += lit;
            }
            // Optional K–M glaze stacking works in linear reflectance. The
            // sheet is entered into that space LAZILY — a pixel no layer
            // paints must not pay a transfer round trip to arrive back at the
            // colour it already had. Measured on a bare sheet, that round
            // trip was 4x the whole render (1.264 ms eager, 0.323 ms lazy).
            //
            // ⚠️ It is a PERF fix and only a perf fix: the round trip is
            // accurate enough that the bytes were already identical, so no
            // comparison of output can tell the two apart. The gate for this
            // is `the_glaze_checkbox_is_free_where_there_is_no_paint`, and it
            // has to time rather than compare.
            //
            // ⚠️ And it is a fix to THIS renderer, which today no product
            // path calls — the live composite goes through
            // [`render_pigment_region_visual`], whose glaze arm sits behind
            // `la <= 0.0 { continue }` and so was never eager. The artist's
            // glaze got faster from the tabulated transfer, not from here.
            // Keep them consistent anyway: this is the SPEC §13 reference
            // look, and the two must not drift into different answers about
            // what an unpainted pixel costs or shows.
            let (mut lr, mut lg, mut lb) = (0.0, 0.0, 0.0);
            let mut in_linear = false;

            for layer in layers.iter().filter(|l| l.visible) {
                let lgrd = layer.grid;
                let op = layer.opacity;
                let s_mass = lgrd.sett[i] as f64;
                let f_mass = lgrd.susp[i] as f64;
                let total = s_mass + f_mass;
                if total <= 0.0 || op <= 0.0 {
                    continue;
                }
                // Granulation offset: the tooth relief printed into the
                // pigment, fading out linearly as total mass goes 1000 ->
                // 3000 (thick paint hides the grain).
                let mut fade = 1.0;
                if total > 1000.0 {
                    fade = if total >= 3000.0 {
                        0.0
                    } else {
                        (3000.0 - total) / 2000.0
                    };
                }
                let mut v = (pap * 100.0 - 40.0) * grain_vis * fade;
                // Signed emboss from the mass gradient: one side lightens,
                // the other darkens — a soft bevel, not an outline. Vertical
                // weighted 2x.
                let m_l = lgrd.sett[i - 1] as f64 + lgrd.susp[i - 1] as f64;
                let m_r = lgrd.sett[i + 1] as f64 + lgrd.susp[i + 1] as f64;
                let m_u = lgrd.sett[i - s] as f64 + lgrd.susp[i - s] as f64;
                let m_d = lgrd.sett[i + s] as f64 + lgrd.susp[i + s] as f64;
                let mut emb = ((m_r - m_l) * 0.5 + (m_d - m_u) * 1.0) * 0.008 * emboss_k;
                emb = emb.clamp(-40.0, 40.0);
                v += emb;
                // Valley granulation (extension): settled reads denser in
                // troughs.
                let mut sett_for_alpha = s_mass;
                if valley > 0.0 {
                    sett_for_alpha = s_mass * (1.0 + valley * 3.0 * (0.5 - pap).max(0.0));
                }
                let a_s = alpha_of_mass(sett_for_alpha) * op;
                let a_f = alpha_of_mass(f_mass) * op;
                let sc = lgrd.sett_rgb[i];
                let uc = lgrd.susp_rgb[i];
                if km_glaze {
                    // Stack by reflectance: paper -> settled -> suspended.
                    if (a_s > 0.0 || a_f > 0.0) && !in_linear {
                        lr = srgb255_to_linear(r);
                        lg = srgb255_to_linear(g);
                        lb = srgb255_to_linear(b);
                        in_linear = true;
                    }
                    if a_s > 0.0 {
                        lr = km_glaze_channel_linear(lr, srgb255_to_linear(sc[0] as f64 + v), a_s);
                        lg = km_glaze_channel_linear(lg, srgb255_to_linear(sc[1] as f64 + v), a_s);
                        lb = km_glaze_channel_linear(lb, srgb255_to_linear(sc[2] as f64 + v), a_s);
                    }
                    if a_f > 0.0 {
                        lr = km_glaze_channel_linear(lr, srgb255_to_linear(uc[0] as f64 + v), a_f);
                        lg = km_glaze_channel_linear(lg, srgb255_to_linear(uc[1] as f64 + v), a_f);
                        lb = km_glaze_channel_linear(lb, srgb255_to_linear(uc[2] as f64 + v), a_f);
                    }
                } else {
                    // Alpha-over, settled then suspended; the tooth offset v
                    // rides inside the pigment color so grain shows THROUGH
                    // the color.
                    if a_s > 0.0 {
                        r = r * (1.0 - a_s) + (sc[0] as f64 + v) * a_s;
                        g = g * (1.0 - a_s) + (sc[1] as f64 + v) * a_s;
                        b = b * (1.0 - a_s) + (sc[2] as f64 + v) * a_s;
                    }
                    if a_f > 0.0 {
                        r = r * (1.0 - a_f) + (uc[0] as f64 + v) * a_f;
                        g = g * (1.0 - a_f) + (uc[1] as f64 + v) * a_f;
                        b = b * (1.0 - a_f) + (uc[2] as f64 + v) * a_f;
                    }
                }
                // Edge tint (extension): bidirectional darken/lighten on mass
                // fronts.
                // Under glaze the tint has never been applied (it is an sRGB
                // offset and the stack is in reflectance), so the gradient is
                // not computed either — same output, minus a dead `sqrt` per
                // layer per pixel.
                if edge_tint != 0.0 && !km_glaze {
                    let gmx = (m_r - m_l) * 0.5;
                    let gmy = (m_d - m_u) * 0.5;
                    let gm = (gmx * gmx + gmy * gmy).sqrt();
                    let wgt = smoothstep01((gm - 100.0) / 1300.0);
                    let tint = edge_tint * 130.0 * wgt;
                    r -= tint;
                    g -= tint;
                    b -= tint;
                }
            }
            if in_linear {
                r = srgb255_of_linear(lr);
                g = srgb255_of_linear(lg);
                b = srgb255_of_linear(lb);
            }

            // Show-wet overlay (active layer): a realistic damp look — darken
            // cool plus a meniscus glint on the film's rim.
            if show_wet {
                let film = active.film[i] as f64;
                let mut wet_sig = active.wet[i] as f64 / 255.0;
                if film > 0.02 {
                    let sv = smoothstep01((film / 2.5).min(1.0));
                    if sv > wet_sig {
                        wet_sig = sv;
                    }
                }
                if wet_sig > 0.01 {
                    r *= 1.0 - 2.4 * 0.04 * wet_sig;
                    g *= 1.0 - 1.6 * 0.04 * wet_sig;
                    b *= 1.0 - 0.8 * 0.04 * wet_sig;
                    let gx = (active.film[i + 1] as f64 - active.film[i - 1] as f64) * 0.5;
                    let gy = (active.film[i + s] as f64 - active.film[i - s] as f64) * 0.5;
                    let mag = (gx * gx + gy * gy).sqrt();
                    if mag > 0.01 {
                        let rim = (mag * 0.7).min(1.0);
                        let shine = 18.3
                            * rim
                            * rim
                            * (0.5 + 0.5 * ((gx / mag) * LIGHT_X + (gy / mag) * LIGHT_Y));
                        r += shine;
                        g += shine;
                        b += shine;
                    }
                }
            }
            // Wet sheen (extension): specular keyed on the paper gradient x
            // light.
            if sheen > 0.0 {
                let film = active.film[i] as f64;
                if film > 0.05 {
                    let win = smoothstep01((film - 0.05) / 2.45);
                    let gx = (base0.paper[i + 1] as f64 - base0.paper[i - 1] as f64) * 0.5;
                    let gy = (base0.paper[i + s] as f64 - base0.paper[i - s] as f64) * 0.5;
                    let spec = (gx * LIGHT_X + gy * LIGHT_Y).max(0.0) * 160.0 * sheen * win;
                    r += spec;
                    g += spec;
                    b += spec;
                }
            }
            // Static grain dither (extension): per-pixel hash, ±0.5 * knob.
            if dither > 0.0 {
                let d = (hash2(cx, cy, 0xd17e) - 0.5) * dither;
                r += d;
                g += d;
                b += d;
            }
            out[o] = clamp_u8(r);
            out[o + 1] = clamp_u8(g);
            out[o + 2] = clamp_u8(b);
            out[o + 3] = 255;
            i += 1;
            o += 4;
        }
    }
}

/// Pigment-only export (SPEC §15b): straight-alpha over transparent.
/// alpha = combined coverage aS + aF(1-aS); RGB = settled-then-suspended
/// straight-alpha compositing, over-composited across visible layers.
pub fn render_pigment_only(layers: &[RenderLayer<'_>], out: &mut [u8]) {
    let g0 = layers[0].grid;
    render_pigment_only_region(layers, 1, 1, g0.w, g0.h, out);
}

/// The region form of [`render_pigment_only`] (the product's dirty-rect
/// composite). Thin wrapper over [`render_pigment_region_visual`] with the
/// visual terms OFF — byte-identical by construction (every visual term is
/// branch-gated, so the off path runs exactly the historical float sequence)
/// and there is exactly ONE per-cell body to diverge. Cell coords,
/// inclusive, [1..W] x [1..H]; `out` is the FULL W*H*4 canvas — only the
/// rect's pixels are written.
pub fn render_pigment_only_region(
    layers: &[RenderLayer<'_>],
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    out: &mut [u8],
) {
    render_pigment_region_visual(None, layers, PigmentVisual::default(), x0, y0, x1, y1, out);
}

/// The product composite's VISUAL terms — the model's paper-into-the-paint
/// half, layer-honest: no SHEET (a layer is not the app's background — the
/// paper-tinted near-white base of [`render_region`] would paint an opaque
/// page over the art under it). Alpha is ALWAYS pure pigment coverage.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct PigmentVisual {
    /// Print the tooth into the pigment: the granulation offset `v`
    /// (`(pap·100−40)·visualGrain·fade`) plus the signed mass-gradient
    /// emboss, both riding INSIDE the pigment colour channels exactly as in
    /// [`render_region`], scaled by the `PaperVisibility` master. "The
    /// texture becomes visually part of the painting."
    pub paper: bool,
    /// Stack suspended over settled by K–M reflectance
    /// ([`km_glaze_channel_linear`]) instead of alpha-over — the
    /// layer-integration half of the model's glaze (there the base is the
    /// sheet; here the wet film glazes the dried paint).
    pub km_glaze: bool,
}

/// [`render_pigment_only_region`]'s single per-cell body, with the optional
/// visual terms. `p` is only read when a visual flag is on (the wrapper
/// passes `None`; the tool passes the session's live params).
#[allow(clippy::too_many_arguments)]
pub fn render_pigment_region_visual(
    p: Option<&Params>,
    layers: &[RenderLayer<'_>],
    visual: PigmentVisual,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    out: &mut [u8],
) {
    let g0 = layers[0].grid;
    let s = g0.s;
    let w = g0.w;
    let paper_on = visual.paper && p.is_some();
    let glaze_on = visual.km_glaze;
    let (paper_vis, grain_vis, emboss_k) = match (paper_on, p) {
        (true, Some(p)) => (
            p.k(Knob::PaperVisibility),
            p.k(Knob::VisualGrain),
            p.k(Knob::Emboss),
        ),
        _ => (0.0, 0.0, 0.0),
    };
    for cy in y0..=y1 {
        let mut i = x0 + cy * s;
        let mut o = ((cy - 1) * w + (x0 - 1)) * 4;
        for _cx in x0..=x1 {
            // Accumulate premultiplied, then unpremultiply at the end.
            let (mut pr, mut pg, mut pb, mut pa) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
            for layer in layers.iter().filter(|l| l.visible) {
                let lg = layer.grid;
                let op = layer.opacity;
                let a_s = alpha_of_mass(lg.sett[i] as f64) * op;
                let a_f = alpha_of_mass(lg.susp[i] as f64) * op;
                let la = a_s + a_f * (1.0 - a_s);
                if la <= 0.0 {
                    continue;
                }
                // Layer color: settled, then suspended over it (straight
                // alpha) — or by K–M reflectance under the glaze flag.
                let sc = lg.sett_rgb[i];
                let uc = lg.susp_rgb[i];
                let (mut cr, mut cg, mut cb) = (sc[0] as f64, sc[1] as f64, sc[2] as f64);
                if a_s <= 0.0 {
                    cr = uc[0] as f64;
                    cg = uc[1] as f64;
                    cb = uc[2] as f64;
                } else if a_f > 0.0 {
                    if glaze_on {
                        // The wet film GLAZES the dried paint: suspended over
                        // settled by reflectance, per channel, in linear
                        // light — [`render_region`]'s stacking law with the
                        // settled colour as the base.
                        // The 0..255 doors carry `clamp_byte(..)/255` inside
                        // the table lookup (`colorops::transfer`).
                        cr = srgb255_of_linear(km_glaze_channel_linear(
                            srgb255_to_linear(cr),
                            srgb255_to_linear(uc[0] as f64),
                            a_f,
                        ));
                        cg = srgb255_of_linear(km_glaze_channel_linear(
                            srgb255_to_linear(cg),
                            srgb255_to_linear(uc[1] as f64),
                            a_f,
                        ));
                        cb = srgb255_of_linear(km_glaze_channel_linear(
                            srgb255_to_linear(cb),
                            srgb255_to_linear(uc[2] as f64),
                            a_f,
                        ));
                    } else {
                        cr = cr * (1.0 - a_f) + uc[0] as f64 * a_f;
                        cg = cg * (1.0 - a_f) + uc[1] as f64 * a_f;
                        cb = cb * (1.0 - a_f) + uc[2] as f64 * a_f;
                    }
                }
                if paper_on {
                    // The tooth printed into the pigment — [`render_region`]'s
                    // exact `v`: paper faded by the master toward flat 0.5,
                    // granulation fading out as mass goes 1000 -> 3000, plus
                    // the signed emboss (vertical weighted 2x). Adding v to
                    // the combined colour equals adding it to settled AND
                    // suspended before combining (distributivity).
                    let pap_raw = g0.paper[i] as f64;
                    let pap = 0.5 + (pap_raw - 0.5) * paper_vis;
                    let total = lg.sett[i] as f64 + lg.susp[i] as f64;
                    let mut fade = 1.0;
                    if total > 1000.0 {
                        fade = if total >= 3000.0 {
                            0.0
                        } else {
                            (3000.0 - total) / 2000.0
                        };
                    }
                    let mut v = (pap * 100.0 - 40.0) * grain_vis * fade;
                    let m_l = lg.sett[i - 1] as f64 + lg.susp[i - 1] as f64;
                    let m_r = lg.sett[i + 1] as f64 + lg.susp[i + 1] as f64;
                    let m_u = lg.sett[i - s] as f64 + lg.susp[i - s] as f64;
                    let m_d = lg.sett[i + s] as f64 + lg.susp[i + s] as f64;
                    let mut emb = ((m_r - m_l) * 0.5 + (m_d - m_u) * 1.0) * 0.008 * emboss_k;
                    emb = emb.clamp(-40.0, 40.0);
                    v += emb;
                    cr += v;
                    cg += v;
                    cb += v;
                }
                // Over-composite onto the accumulator (premultiplied).
                pr = pr * (1.0 - la) + cr * la;
                pg = pg * (1.0 - la) + cg * la;
                pb = pb * (1.0 - la) + cb * la;
                pa += la * (1.0 - pa);
            }
            if pa > 0.0 {
                out[o] = clamp_u8(pr / pa);
                out[o + 1] = clamp_u8(pg / pa);
                out[o + 2] = clamp_u8(pb / pa);
                out[o + 3] = clamp_u8(pa * 255.0);
            } else {
                out[o] = 0;
                out[o + 1] = 0;
                out[o + 2] = 0;
                out[o + 3] = 0;
            }
            i += 1;
            o += 4;
        }
    }
}
