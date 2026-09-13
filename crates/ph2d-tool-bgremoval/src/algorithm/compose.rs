//! Final compose step: produce `scratch.output_rgba` from the
//! segmentation mask, optional soft alpha, and the input RGBA.
//!
//! Two paths:
//!
//! 1. **Guided filter ran** (`did_refine = true`): alpha comes from
//!    `scratch.alpha_f32` directly. Despill (when enabled) subtracts the
//!    detected bg chroma from soft-edge pixels.
//!
//! 2. **No refinement** (`did_refine = false`): alpha synthesised from
//!    `scratch.mask` + `scratch.delta_e` using the `[tolerance,
//!    tolerance + feather]` soft-band formula. Important:
//!    `scratch.delta_e[i]` is the **squared** Oklab distance (the chroma
//!    backend squares it once to avoid a per-pixel `sqrt` in its main
//!    loop), so comparisons happen against `tol_sq` / `(tol + feat)²`;
//!    the band-position fraction `t` linearises with `sqrt` once per
//!    soft-band pixel.
//!
//! Both paths assume the input is **straight-alpha RGBA8** (not
//! premultiplied) — enforced at the API boundary of
//! [`super::super::tool::BgRemovalTool::set_source_snapshot`].

use super::super::params::BgRemovalParams;
use super::super::scratch::BgRemovalScratch;
use super::SegmentResult;

/// Write the final RGBA into `scratch.output_rgba`.
///
/// `protect` is an optional freehand foreground-protection mask aligned
/// to `(w, h)` (one byte/pixel, `>= 128` = protected). Protected pixels
/// are forced fully opaque after the grow/shrink morphology and before
/// the edge bleed, so a painted "keep" region survives every backend +
/// refinement path. Pass `None` when nothing is painted.
///
/// `force_remove` is the symmetric destructive mask (Enio 2026-05-26):
/// when present (one byte/pixel, `> 0` = strength of removal), the
/// pixel's final alpha is lowered to `min(alpha, 255 - strength)`.
/// Applied AFTER `force_keep_protected`, so a force-remove dab wins
/// over a protect-brush dab AND over the silhouette auto-protect at
/// the same pixel (the user explicitly painted "remove this" last —
/// most-recent-intent wins). Used by the "Acrescentar Área" brush
/// shown when Detect Subject is on.
#[allow(clippy::too_many_arguments)]
pub fn write_output(
    rgba: &[u8],
    w: u32,
    h: u32,
    params: &BgRemovalParams,
    segment: &SegmentResult,
    did_refine: bool,
    protect: Option<&[u8]>,
    force_remove: Option<&[u8]>,
    scratch: &mut BgRemovalScratch,
) {
    let n = (w as usize) * (h as usize);
    debug_assert_eq!(rgba.len(), n * 4);
    debug_assert_eq!(scratch.output_rgba.len(), n * 4);

    if did_refine {
        // Path 1 — soft alpha from guided_filter.
        for i in 0..n {
            let a = (scratch.alpha_f32[i].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            let base = i * 4;
            scratch.output_rgba[base] = rgba[base];
            scratch.output_rgba[base + 1] = rgba[base + 1];
            scratch.output_rgba[base + 2] = rgba[base + 2];
            scratch.output_rgba[base + 3] = a;
        }
    } else {
        // Path 2 — hard mask + delta_e soft band. `delta_e` is SQUARED
        // Oklab distance — compare against `tol_sq` / `(tol + feat)²`,
        // linearise via `sqrt` once per soft-band pixel (P0 fix
        // 2026-05-16: previously compared raw `tol` against squared
        // `delta_e`, so the soft band never fired at expected positions).
        let tol = params.chroma.tolerance;
        let feat = params.chroma.feather.max(1e-6);
        let tol_sq = tol * tol;
        let outer = tol + feat;
        let outer_sq = outer * outer;
        for i in 0..n {
            let alpha = if scratch.mask[i] == 0 {
                // Hard bg from the connected-flood pass.
                0u8
            } else {
                let de_sq = scratch.delta_e[i];
                if de_sq >= outer_sq {
                    255
                } else if de_sq >= tol_sq {
                    let de = de_sq.max(0.0).sqrt();
                    let t = ((de - tol) / feat).clamp(0.0, 1.0);
                    (t * 255.0 + 0.5) as u8
                } else {
                    // ΔE below threshold but mask says fg → keep
                    // (e.g. flood-protected interior).
                    255
                }
            };
            let base = i * 4;
            scratch.output_rgba[base] = rgba[base];
            scratch.output_rgba[base + 1] = rgba[base + 1];
            scratch.output_rgba[base + 2] = rgba[base + 2];
            scratch.output_rgba[base + 3] = alpha;
        }
    }

    // Despill / foreground decontamination — when enabled and we have a
    // detected bg colour. Soft-edge pixels are a composite
    // `C = a·fg + (1−a)·bg`; we recover the true foreground
    // `fg = (C − (1−a)·bg) / a` and write it back, removing the colour
    // halo the background bleeds into anti-aliased edges (the canonical
    // green-screen "despill", generalised to any detected bg colour).
    // Only fractional-alpha pixels are touched: a==0 is invisible,
    // a==255 is already pure foreground.
    //
    // PROTECTED pixels are SKIPPED: the user painted them to keep as-is,
    // so the bg-over-fg assumption is wrong there. Despilling them turns
    // the fractional-alpha boundary between a kept (e.g. green) region and
    // the dark line-art into a magenta fringe — `(C − (1−a)·bg)/a` zeroes
    // green and amplifies red+blue. Leaving them untouched keeps the
    // painted region's true colour (and removes that fringe).
    if params.chroma.despill
        && let SegmentResult::Chroma { bg_oklab } = segment
    {
        let bg = super::chroma::oklab_to_srgb8(*bg_oklab);
        // When the user has picked extras, soft-edges adjacent to an
        // is_near_extra=1 pixel have a "bg side" whose true colour is
        // the extra pick, NOT the auto-detected bg. Applying the
        // `(C − (1−a)·bg)/a` formula with the wrong bg zeroes one
        // hue channel and amplifies the complementary pair → a
        // magenta/pink fringe (Enio 2026-05-26: "aparece um rosa
        // completamente estranho... como se o próprio algoritmo
        // usasse um rosa por trás da imagem"). Skip despill on any
        // soft-edge pixel within the 3×3 neighbourhood of an extras
        // pixel. Non-extras soft-edges keep the existing despill so
        // the auto-bg path is unchanged.
        let has_extras = !params.extra_bg_colors.is_empty();
        let wi = w as usize;
        let hi = h as usize;
        for i in 0..n {
            if protect.is_some_and(|pm| pm[i] > 0) {
                continue;
            }
            let base = i * 4;
            let a = scratch.output_rgba[base + 3];
            if a == 0 || a == 255 {
                continue;
            }
            if has_extras && is_near_extras_3x3(&scratch.is_near_extra, i, wi, hi) {
                continue;
            }
            let af = (a as f32) * (1.0 / 255.0);
            let inv = 1.0 - af;
            for (c, &bg_c) in bg.iter().enumerate() {
                let comp = scratch.output_rgba[base + c] as f32;
                let fg = (comp - inv * bg_c as f32) / af;
                scratch.output_rgba[base + c] = (fg + 0.5).clamp(0.0, 255.0) as u8;
            }
        }
    }

    // Grow / Shrink — morphology on the final alpha, BEFORE the edge
    // bleed (which only fills the resulting alpha==0 collar). Negative
    // `grow_px` erodes the matte to eat the residual background outline;
    // positive dilates it.
    grow_shrink_alpha(w, h, params.grow_px, scratch);

    // Foreground protection force-keep — the painted "keep" mask raises
    // each pixel's alpha to `max(alpha, strength)`, AFTER grow/shrink (so
    // a shrink can't eat it) and BEFORE the edge bleed (which only fills
    // the alpha==0 collar, never these). The soft falloff rim blends in.
    if let Some(pm) = protect {
        force_keep_protected(pm, n, scratch);
    }

    // Symmetric destructive mask — the user's "Acrescentar Área" brush
    // (Enio 2026-05-26). Applied AFTER force_keep so a force-remove dab
    // wins over a protect dab at the same pixel (the more-recent
    // intent), AND so it can override the silhouette auto-protect when
    // Detect Subject leaks background through narrow gaps (the original
    // use case: the area between the character's legs that the
    // silhouette walker couldn't separate from the foreground).
    // BEFORE the edge bleed for the same reason as force_keep — bleed
    // only fills alpha==0 collars, and a force-removed pixel is
    // intentionally alpha==0, so it joins the collar fill seamlessly.
    if let Some(fr) = force_remove {
        force_remove_painted(fr, n, scratch);
    }

    // Edge bleed — fix the Apply-vs-preview edge fringe. The on-canvas
    // preview is clean because Vello's `draw_image` premultiplies BEFORE
    // bilinear sampling; the wgpu sprite shader samples the STRAIGHT
    // texture bilinearly and premultiplies AFTER (`rgb * a`). So a fully
    // transparent texel that still holds the original bg colour bleeds
    // that colour into the anti-aliased edge → fringe. Propagating the
    // visible edge colour into the transparent collar makes the straight
    // texture safe to bilinear-sample. Only alpha==0 texels are written,
    // so the preview is byte-identical with or without this pass.
    bleed_edges(w, h, scratch);
}

/// Erode (`grow_px < 0`) or dilate (`grow_px > 0`) the alpha channel of
/// `scratch.output_rgba` in place by `|grow_px|` rounded pixels, using
/// `scratch.morph_alpha` as the per-pass read snapshot.
///
/// Grayscale morphology: each 1-px pass replaces every alpha with the
/// min (erode) or max (dilate) over its 3×3 (8-connected + self)
/// True iff pixel `i` (in a `w × h` grid) has any 3×3-neighbour with
/// `is_near_extra[n] != 0`. Cheap branch (≤9 reads) gated by
/// `!extra_colors.is_empty()` in the despill path, so non-extras runs
/// pay nothing.
fn is_near_extras_3x3(is_near_extra: &[u8], i: usize, w: usize, h: usize) -> bool {
    let x = i % w;
    let y = i / w;
    let x_min = x.saturating_sub(1);
    let x_max = (x + 1).min(w.saturating_sub(1));
    let y_min = y.saturating_sub(1);
    let y_max = (y + 1).min(h.saturating_sub(1));
    for yy in y_min..=y_max {
        let row = yy * w;
        for xx in x_min..=x_max {
            if is_near_extra[row + xx] != 0 {
                return true;
            }
        }
    }
    false
}

/// neighbourhood, repeated `radius` times ≈ a Chebyshev-disc of that
/// radius. RGB is untouched — eroded pixels simply become transparent;
/// the subsequent edge bleed re-colours the freshly transparent collar.
fn grow_shrink_alpha(w: u32, h: u32, grow_px: f32, scratch: &mut BgRemovalScratch) {
    let radius = grow_px.abs().round() as i32;
    if radius == 0 {
        return;
    }
    let dilate = grow_px > 0.0;
    let wi = w as usize;
    let hi = h as usize;
    if wi == 0 || hi == 0 {
        return;
    }
    for _pass in 0..radius {
        // Snapshot the current alpha so neighbour reads are unaffected
        // by writes earlier in this pass.
        for i in 0..wi * hi {
            scratch.morph_alpha[i] = scratch.output_rgba[i * 4 + 3];
        }
        for y in 0..hi {
            for x in 0..wi {
                let i = y * wi + x;
                let mut acc = scratch.morph_alpha[i];
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx < 0 || ny < 0 || nx >= wi as i32 || ny >= hi as i32 {
                            // Treat off-image as background (alpha 0):
                            // erosion shrinks at the border, dilation
                            // does not bleed past it.
                            if !dilate {
                                acc = 0;
                            }
                            continue;
                        }
                        let nv = scratch.morph_alpha[ny as usize * wi + nx as usize];
                        acc = if dilate { acc.max(nv) } else { acc.min(nv) };
                    }
                }
                scratch.output_rgba[i * 4 + 3] = acc;
            }
        }
    }
}

/// Apply the protection mask as a per-pixel keep-floor: the final alpha
/// is `max(alpha_seg, strength)` where `strength = protect[i]` (0..255).
/// A hard-painted core (255) becomes fully opaque; a soft falloff rim
/// raises the alpha proportionally, so a feathered brush blends into the
/// matte instead of a hard cutout. RGB is untouched — the source colour
/// is what the user wants to keep.
fn force_keep_protected(protect: &[u8], n: usize, scratch: &mut BgRemovalScratch) {
    debug_assert!(protect.len() >= n);
    for (i, &p) in protect.iter().enumerate().take(n) {
        let a = &mut scratch.output_rgba[i * 4 + 3];
        *a = (*a).max(p);
    }
}

/// Symmetric counterpart to [`force_keep_protected`]: lower each
/// pixel's alpha to `min(alpha, 255 - strength)`. A hard-painted core
/// (strength=255) zeroes the alpha; a soft falloff rim cuts the alpha
/// proportionally, so the brush's edge blends into the existing matte.
/// RGB is untouched — invisible alpha=0 pixels don't sample anyway, and
/// the partial-alpha rim keeps its existing colour (potentially
/// despilled by an earlier pass).
fn force_remove_painted(force_remove: &[u8], n: usize, scratch: &mut BgRemovalScratch) {
    debug_assert!(force_remove.len() >= n);
    for (i, &r) in force_remove.iter().enumerate().take(n) {
        if r == 0 {
            continue;
        }
        let a = &mut scratch.output_rgba[i * 4 + 3];
        let max_alpha = 255u8.saturating_sub(r);
        if *a > max_alpha {
            *a = max_alpha;
        }
    }
}

/// Number of 1-pixel rings of foreground colour to grow outward into the
/// transparent / low-alpha collar. A handful covers the bilinear
/// footprint at any reasonable on-canvas scale; capped so we never flood
/// the whole transparent background. // LITERAL-OK: bilinear footprint
const BLEED_RINGS: usize = 4;

/// Grow edge colour outward into the FULLY TRANSPARENT collar so the
/// straight-alpha texture is safe for the sprite shader's
/// bilinear-then-premultiply sampling. Operates in place on
/// `scratch.output_rgba` (RGB only; alpha is preserved) using
/// `scratch.bleed_valid` as the per-pixel state machine.
///
/// CRITICAL invariant: only pixels with `alpha == 0` are ever written.
/// They are invisible under premultiplied compositing, so the on-canvas
/// Vello preview (which premultiplies BEFORE sampling) is byte-identical
/// with or without this pass — the bleed exists purely to stop the wgpu
/// sprite shader from bilinear-sampling the original background colour
/// out of transparent texels. Sources are any pixel with `alpha > 0`
/// (the true, already-despilled edge colour), NOT just the opaque
/// interior — seeding only `== 255` would let a mis-classified opaque
/// background pocket spray its colour into the edge (the green-halo
/// regression). Overwriting the partial-alpha AA band (NOT just the
/// transparent collar) likewise corrupts comic line-art, where that
/// band IS the artwork — so this stays strictly an `alpha == 0` fill.
fn bleed_edges(w: u32, h: u32, scratch: &mut BgRemovalScratch) {
    let wi = w as usize;
    let hi = h as usize;
    let n = wi * hi;
    if n == 0 {
        return;
    }
    // Seed: any visible pixel (alpha > 0) is a colour source; only the
    // fully-transparent collar (alpha == 0) is a fill target.
    for i in 0..n {
        scratch.bleed_valid[i] = u8::from(scratch.output_rgba[i * 4 + 3] > 0);
    }
    for _ring in 0..BLEED_RINGS {
        let mut any = false;
        for y in 0..hi {
            for x in 0..wi {
                let i = y * wi + x;
                if scratch.bleed_valid[i] != 0 {
                    continue; // already a source (1) or filled this ring (2)
                }
                // Average the RGB of the 8-connected neighbours that
                // are sources as of the start of this ring (== 1).
                let (mut sr, mut sg, mut sb, mut cnt) = (0u32, 0u32, 0u32, 0u32);
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx < 0 || ny < 0 || nx >= wi as i32 || ny >= hi as i32 {
                            continue;
                        }
                        let ni = ny as usize * wi + nx as usize;
                        if scratch.bleed_valid[ni] == 1 {
                            let nb = ni * 4;
                            sr += scratch.output_rgba[nb] as u32;
                            sg += scratch.output_rgba[nb + 1] as u32;
                            sb += scratch.output_rgba[nb + 2] as u32;
                            cnt += 1;
                        }
                    }
                }
                if cnt > 0 {
                    let inv = 1.0 / cnt as f32;
                    let base = i * 4;
                    scratch.output_rgba[base] = (sr as f32 * inv) as u8;
                    scratch.output_rgba[base + 1] = (sg as f32 * inv) as u8;
                    scratch.output_rgba[base + 2] = (sb as f32 * inv) as u8;
                    // Filled this ring; promote to source only AFTER the
                    // ring so it isn't used as a source within it.
                    scratch.bleed_valid[i] = 2;
                    any = true;
                }
            }
        }
        if !any {
            break;
        }
        for v in scratch.bleed_valid.iter_mut() {
            if *v == 2 {
                *v = 1;
            }
        }
    }
}

#[cfg(test)]
#[path = "compose_tests.rs"]
mod tests;
