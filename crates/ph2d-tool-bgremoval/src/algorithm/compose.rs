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
mod tests {
    use super::*;
    use crate::params::BgRemovalParams;

    fn fresh_scratch(w: u32, h: u32) -> BgRemovalScratch {
        let mut s = BgRemovalScratch::default();
        s.ensure(w, h, false);
        s
    }

    fn solid_rgba(w: u32, h: u32, color: [u8; 3]) -> Vec<u8> {
        let n = (w * h) as usize;
        let mut rgba = vec![0u8; n * 4];
        for i in 0..n {
            rgba[i * 4] = color[0];
            rgba[i * 4 + 1] = color[1];
            rgba[i * 4 + 2] = color[2];
            rgba[i * 4 + 3] = 255;
        }
        rgba
    }

    // --- Path 2 (Chroma, no refinement) — squared-unit correctness ----

    #[test]
    fn path2_chroma_soft_band_uses_squared_units_at_band_edges() {
        // Audit BUG A (2026-05-16): soft-band comparisons must use
        // squared units (delta_e is ΔE²). With tol = 0.10, feat = 0.04:
        // tol_sq = 0.01, (tol+feat)² = 0.0196.
        //
        // We pick delta_e values *inside* the band (not at the exact
        // boundary, where f32(0.01) ≠ f32(0.10)² by ~1 ULP) so the
        // test doesn't rely on bit-exact boundary semantics.
        let w = 4u32;
        let h = 1u32;
        let mut scratch = fresh_scratch(w, h);
        let rgba = solid_rgba(w, h, [200, 30, 30]);
        for v in scratch.mask.iter_mut().take(4) {
            *v = 255;
        }
        // pixel 0: below tol_sq          → keep as fg (255)
        // pixel 1: linear ΔE ≈ 0.11      → t ≈ 0.25, alpha ≈ 64
        // pixel 2: linear ΔE ≈ 0.12      → t = 0.50, alpha ≈ 128
        // pixel 3: above outer_sq        → 255 (saturated)
        scratch.delta_e[0] = 0.005;
        scratch.delta_e[1] = 0.11_f32 * 0.11_f32; // ~0.0121
        scratch.delta_e[2] = 0.12_f32 * 0.12_f32; // ~0.0144
        scratch.delta_e[3] = 0.025;

        let params = BgRemovalParams {
            chroma: crate::params::ChromaParams {
                tolerance: 0.10,
                feather: 0.04,
                ..crate::params::ChromaParams::default()
            },
            // Pin neutral grow so these tests isolate the compose path
            // (the default params now carry a slight erode).
            grow_px: 0.0,
            ..BgRemovalParams::default()
        };
        let segment = SegmentResult::Chroma { bg_oklab: [0.0; 3] };

        write_output(
            &rgba,
            w,
            h,
            &params,
            &segment,
            false,
            None,
            None,
            &mut scratch,
        );

        assert_eq!(scratch.output_rgba[3], 255, "below tol_sq must keep fg");
        // Quarter-band: t ≈ 0.25 ⇒ alpha ≈ 64.
        let q_alpha = scratch.output_rgba[7];
        assert!(
            (q_alpha as i32 - 64).abs() <= 3,
            "quarter-band alpha = {q_alpha}, expected ~64"
        );
        // Mid-band: t = 0.5 ⇒ alpha ≈ 128.
        let mid_alpha = scratch.output_rgba[11];
        assert!(
            (mid_alpha as i32 - 128).abs() <= 3,
            "mid-band alpha = {mid_alpha}, expected ~128"
        );
        assert_eq!(scratch.output_rgba[15], 255, "above outer_sq must give 255");
    }

    #[test]
    fn path2_mask_zero_gives_hard_bg_regardless_of_delta_e() {
        let w = 1u32;
        let h = 1u32;
        let mut scratch = fresh_scratch(w, h);
        let rgba = solid_rgba(w, h, [10, 20, 30]);
        scratch.mask[0] = 0;
        scratch.delta_e[0] = 0.05; // mid-band but mask says bg
        let params = BgRemovalParams {
            // Pin neutral grow so these tests isolate the compose path
            // (the default params now carry a slight erode).
            grow_px: 0.0,
            ..BgRemovalParams::default()
        };
        let segment = SegmentResult::Chroma { bg_oklab: [0.0; 3] };
        write_output(
            &rgba,
            w,
            h,
            &params,
            &segment,
            false,
            None,
            None,
            &mut scratch,
        );
        assert_eq!(scratch.output_rgba[3], 0, "mask=0 must produce alpha=0");
    }

    // --- Path 3 (GrabCut, no refinement) — no delta_e leak ------------

    #[test]
    fn path3_grabcut_no_refine_ignores_stale_delta_e() {
        // Audit BUG B (2026-05-16): GrabCut + radius=0 must NOT read
        // delta_e. We poison delta_e to non-zero values that, under
        // the old buggy code path, would cause some mask=255 pixels
        // to mistakenly be down-mapped via the chroma soft-band.
        let w = 4u32;
        let h = 1u32;
        let mut scratch = fresh_scratch(w, h);
        let rgba = solid_rgba(w, h, [10, 20, 30]);
        // Mask: 0, 255, 255, 0  (binary-only, GrabCut convention).
        scratch.mask[0] = 0;
        scratch.mask[1] = 255;
        scratch.mask[2] = 255;
        scratch.mask[3] = 0;
        // Stale delta_e from a prior chroma run — would corrupt
        // alpha under the old path-2 code.
        scratch.delta_e[0] = 0.01;
        scratch.delta_e[1] = 0.0144; // mid-soft-band under old code
        scratch.delta_e[2] = 0.005;
        scratch.delta_e[3] = 0.02;

        let params = BgRemovalParams {
            // Pin neutral grow so these tests isolate the compose path
            // (the default params now carry a slight erode).
            grow_px: 0.0,
            ..BgRemovalParams::default()
        };
        let segment = SegmentResult::Chroma { bg_oklab: [0.0; 3] };
        write_output(
            &rgba,
            w,
            h,
            &params,
            &segment,
            false,
            None,
            None,
            &mut scratch,
        );

        // Alpha must equal mask, byte-for-byte. No delta_e leak.
        assert_eq!(scratch.output_rgba[3], 0);
        assert_eq!(scratch.output_rgba[7], 255);
        assert_eq!(scratch.output_rgba[11], 255);
        assert_eq!(scratch.output_rgba[15], 0);
    }

    // --- Protection force-keep --------------------------------------

    #[test]
    fn protect_forces_protected_pixel_opaque_regardless_of_mask() {
        // 2×1: both pixels classified as background (mask 0). The first
        // is protected → must end fully opaque; the second is not →
        // stays transparent.
        let w = 2u32;
        let h = 1u32;
        let mut scratch = fresh_scratch(w, h);
        let rgba = solid_rgba(w, h, [10, 20, 30]);
        scratch.mask[0] = 0;
        scratch.mask[1] = 0;
        let params = BgRemovalParams {
            grow_px: 0.0,
            ..BgRemovalParams::default()
        };
        let segment = SegmentResult::Chroma { bg_oklab: [0.0; 3] };
        let protect = [255u8, 0u8];
        write_output(
            &rgba,
            w,
            h,
            &params,
            &segment,
            false,
            Some(&protect),
            None,
            &mut scratch,
        );
        assert_eq!(
            scratch.output_rgba[3], 255,
            "protected bg pixel must be forced opaque"
        );
        assert_eq!(
            scratch.output_rgba[7], 0,
            "unprotected bg pixel stays transparent"
        );
    }

    #[test]
    fn protect_soft_strength_blends_proportionally() {
        // A soft-falloff rim (strength 128) over a background pixel
        // (mask 0) must lift alpha to ~128, not slam it to 255 — the
        // feathered keep-edge blends into the matte.
        let w = 2u32;
        let h = 1u32;
        let mut scratch = fresh_scratch(w, h);
        let rgba = solid_rgba(w, h, [10, 20, 30]);
        scratch.mask[0] = 0;
        scratch.mask[1] = 255; // already opaque foreground
        let params = BgRemovalParams {
            grow_px: 0.0,
            ..BgRemovalParams::default()
        };
        let segment = SegmentResult::Chroma { bg_oklab: [0.0; 3] };
        let protect = [128u8, 64u8];
        write_output(
            &rgba,
            w,
            h,
            &params,
            &segment,
            false,
            Some(&protect),
            None,
            &mut scratch,
        );
        // bg pixel lifted to the strength; fg pixel keeps its higher alpha.
        assert_eq!(
            scratch.output_rgba[3], 128,
            "soft rim lifts bg alpha to strength"
        );
        assert_eq!(
            scratch.output_rgba[7], 255,
            "max keeps the higher seg alpha"
        );
    }

    #[test]
    fn despill_skips_protected_pixels_no_magenta_fringe() {
        // A dark line-art pixel at fractional alpha over a green bg would
        // despill to magenta (`(C−(1−a)·bg)/a` zeroes green, amplifies
        // red+blue). A PROTECTED pixel must be left untouched — this is
        // the fix for the pink outline around a painted keep-region.
        let w = 2u32;
        let h = 1u32;
        let bg_oklab = crate::algorithm::chroma::srgb_to_oklab(0, 200, 0);
        let params = BgRemovalParams {
            grow_px: 0.0,
            ..BgRemovalParams::default() // despill defaults on
        };
        let segment = SegmentResult::Chroma { bg_oklab };
        let rgba = solid_rgba(w, h, [20, 20, 20]); // dark line-art
        let make = || {
            let mut s = fresh_scratch(w, h);
            s.alpha_f32[0] = 0.5;
            s.alpha_f32[1] = 0.5;
            s
        };

        // Unprotected → despill rewrites the RGB (green removed).
        let mut s_un = make();
        write_output(&rgba, w, h, &params, &segment, true, None, None, &mut s_un);
        assert_ne!(
            &s_un.output_rgba[0..3],
            &[20, 20, 20],
            "unprotected fractional pixel is despilled"
        );

        // Protected → RGB untouched (no magenta).
        let mut s_pr = make();
        let protect = [255u8, 255u8];
        write_output(
            &rgba,
            w,
            h,
            &params,
            &segment,
            true,
            Some(&protect),
            None,
            &mut s_pr,
        );
        assert_eq!(
            &s_pr.output_rgba[0..3],
            &[20, 20, 20],
            "protected pixel keeps its true colour (despill skipped)"
        );
    }

    #[test]
    fn protect_survives_a_full_shrink() {
        // A protected pixel must survive even an aggressive erode: the
        // force-keep runs AFTER grow/shrink. 3×1 opaque row, shrink by 2,
        // centre pixel protected.
        let w = 3u32;
        let h = 1u32;
        let mut scratch = fresh_scratch(w, h);
        let rgba = solid_rgba(w, h, [200, 50, 50]);
        for i in 0..3 {
            scratch.mask[i] = 255;
            scratch.output_rgba[i * 4 + 3] = 255;
        }
        let params = BgRemovalParams {
            grow_px: -2.0,
            ..BgRemovalParams::default()
        };
        let segment = SegmentResult::Chroma { bg_oklab: [0.0; 3] };
        let protect = [0u8, 255u8, 0u8];
        write_output(
            &rgba,
            w,
            h,
            &params,
            &segment,
            false,
            Some(&protect),
            None,
            &mut scratch,
        );
        assert_eq!(
            scratch.output_rgba[7], 255,
            "protected centre pixel must survive the shrink"
        );
    }

    // --- Path 1 (refinement) — alpha_f32 round-trip -------------------

    #[test]
    fn path1_refined_alpha_f32_maps_to_u8_with_round_half_up() {
        let w = 4u32;
        let h = 1u32;
        let mut scratch = fresh_scratch(w, h);
        let rgba = solid_rgba(w, h, [100, 100, 100]);
        scratch.alpha_f32[0] = 0.0;
        scratch.alpha_f32[1] = 0.5;
        scratch.alpha_f32[2] = 1.0;
        scratch.alpha_f32[3] = 1.5; // out-of-range → clamp
        let params = BgRemovalParams {
            grow_px: 0.0, // isolate compose path from the default erode
            ..BgRemovalParams::default()
        };
        let segment = SegmentResult::Chroma { bg_oklab: [0.0; 3] };
        write_output(
            &rgba,
            w,
            h,
            &params,
            &segment,
            true,
            None,
            None,
            &mut scratch,
        );
        assert_eq!(scratch.output_rgba[3], 0);
        assert_eq!(scratch.output_rgba[7], 128); // 0.5*255+0.5 = 128
        assert_eq!(scratch.output_rgba[11], 255);
        assert_eq!(scratch.output_rgba[15], 255);
    }

    // --- Edge bleed -------------------------------------------------

    #[test]
    fn bleed_fills_transparent_neighbour_with_foreground_rgb() {
        // 3×1: opaque FG (green) | transparent (cream contamination) |
        // transparent. After bleed the transparent pixels' RGB must
        // become the FG green; alpha must stay 0.
        let w = 3u32;
        let h = 1u32;
        let mut scratch = fresh_scratch(w, h);
        // pixel 0: opaque green source
        scratch.output_rgba[0..4].copy_from_slice(&[0, 200, 0, 255]);
        // pixel 1: transparent but holds cream bg
        scratch.output_rgba[4..8].copy_from_slice(&[240, 230, 210, 0]);
        // pixel 2: transparent cream
        scratch.output_rgba[8..12].copy_from_slice(&[240, 230, 210, 0]);

        bleed_edges(w, h, &mut scratch);

        // Pixel 1 took the green source RGB; alpha preserved at 0.
        assert_eq!(&scratch.output_rgba[4..7], &[0, 200, 0]);
        assert_eq!(scratch.output_rgba[7], 0, "alpha must be untouched");
        // Pixel 2 filled from the now-valid pixel 1 on the second ring.
        assert_eq!(&scratch.output_rgba[8..11], &[0, 200, 0]);
        assert_eq!(scratch.output_rgba[11], 0);
    }

    // --- Grow / Shrink morphology -----------------------------------

    #[test]
    fn shrink_erodes_a_one_pixel_border_off_the_matte() {
        // 5×5 solid-opaque block. One erosion pass must zero the outer
        // ring (every border pixel has an off-image / interior min of 0
        // only at the frame edge) — concretely the 4 corners + edges
        // touching the frame go to 0; the 3×3 interior stays 255.
        let (w, h) = (5u32, 5u32);
        let mut scratch = fresh_scratch(w, h);
        for i in 0..(w * h) as usize {
            scratch.output_rgba[i * 4 + 3] = 255;
        }
        grow_shrink_alpha(w, h, -1.0, &mut scratch);
        // Interior 3×3 (rows/cols 1..=3) stays opaque.
        for y in 1..=3 {
            for x in 1..=3 {
                let a = scratch.output_rgba[(y * 5 + x) * 4 + 3];
                assert_eq!(a, 255, "interior ({x},{y}) should survive erosion");
            }
        }
        // Border pixels eroded to 0 (off-image counts as background).
        assert_eq!(scratch.output_rgba[3], 0, "corner (0,0) eroded");
        assert_eq!(scratch.output_rgba[(2 * 5) * 4 + 3], 0, "edge (0,2) eroded");
    }

    #[test]
    fn grow_dilates_into_a_transparent_neighbour() {
        // 3×1: opaque | transparent | transparent. One dilation pass
        // grows the opaque alpha into pixel 1.
        let (w, h) = (3u32, 1u32);
        let mut scratch = fresh_scratch(w, h);
        scratch.output_rgba[3] = 255; // pixel 0 opaque
        grow_shrink_alpha(w, h, 1.0, &mut scratch);
        assert_eq!(scratch.output_rgba[7], 255, "pixel 1 dilated to opaque");
    }

    #[test]
    fn grow_zero_is_a_noop() {
        let (w, h) = (3u32, 3u32);
        let mut scratch = fresh_scratch(w, h);
        let original: Vec<u8> = (0..36).map(|i| i as u8).collect();
        scratch.output_rgba[..36].copy_from_slice(&original);
        grow_shrink_alpha(w, h, 0.0, &mut scratch);
        assert_eq!(&scratch.output_rgba[..36], &original[..]);
    }

    #[test]
    fn bleed_never_touches_partial_alpha_edge_pixels() {
        // The partial-alpha AA band IS the artwork in comic line-art;
        // overwriting it corrupts the edge. The bleed must leave every
        // alpha<255 visible pixel byte-for-byte and only fill the fully
        // transparent collar.
        let w = 3u32;
        let h = 1u32;
        let mut scratch = fresh_scratch(w, h);
        scratch.output_rgba[0..4].copy_from_slice(&[0, 200, 0, 255]); // opaque FG
        scratch.output_rgba[4..8].copy_from_slice(&[180, 60, 90, 128]); // partial edge
        scratch.output_rgba[8..12].copy_from_slice(&[240, 230, 210, 0]); // transparent

        bleed_edges(w, h, &mut scratch);

        // Partial edge pixel: untouched.
        assert_eq!(&scratch.output_rgba[4..8], &[180, 60, 90, 128]);
        // Transparent pixel: filled from a visible neighbour (alpha 0
        // → invisible in the preview, so this never alters it).
        assert_eq!(scratch.output_rgba[11], 0, "alpha untouched");
        assert_ne!(
            &scratch.output_rgba[8..11],
            &[240, 230, 210],
            "transparent collar must take a visible neighbour colour"
        );
    }

    #[test]
    fn bleed_leaves_fully_opaque_image_untouched() {
        let w = 2u32;
        let h = 2u32;
        let mut scratch = fresh_scratch(w, h);
        let original: Vec<u8> = (0..16).map(|i| if i % 4 == 3 { 255 } else { i }).collect();
        scratch.output_rgba[..16].copy_from_slice(&original);
        bleed_edges(w, h, &mut scratch);
        assert_eq!(&scratch.output_rgba[..16], &original[..]);
    }

    #[test]
    fn rgb_channels_passthrough_in_both_paths() {
        let w = 1u32;
        let h = 1u32;
        let mut scratch = fresh_scratch(w, h);
        let rgba = solid_rgba(w, h, [123, 45, 67]);
        scratch.mask[0] = 255;
        scratch.alpha_f32[0] = 0.5;
        let mut params = BgRemovalParams {
            grow_px: 0.0, // isolate compose path from the default erode
            ..BgRemovalParams::default()
        };
        // Disable despill so this test isolates RGB passthrough (despill
        // intentionally rewrites fractional-alpha RGB; covered elsewhere).
        params.chroma.despill = false;
        let segment = SegmentResult::Chroma { bg_oklab: [0.0; 3] };
        // Path 1 (refined).
        write_output(
            &rgba,
            w,
            h,
            &params,
            &segment,
            true,
            None,
            None,
            &mut scratch,
        );
        assert_eq!(&scratch.output_rgba[0..3], &[123, 45, 67]);
        // Path 2 (soft band).
        write_output(
            &rgba,
            w,
            h,
            &params,
            &segment,
            false,
            None,
            None,
            &mut scratch,
        );
        assert_eq!(&scratch.output_rgba[0..3], &[123, 45, 67]);
    }
}
