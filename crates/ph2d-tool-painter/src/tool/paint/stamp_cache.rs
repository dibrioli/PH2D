//! The Blender-style cached brush stamp on the tool side: render the falloff × View-texture mask
//! once (per appearance / size), then scale-blit it per dab. This turns a large textured re-stamp
//! (Anchored re-drawn every pointer move) from a per-pixel falloff+texture recompute into a cheap
//! bilinear blit — the texture is sampled once per stroke, not per pixel per dab. Eligibility is
//! [`ph2d_painter_brush::TextureSettings::is_cacheable`] (checked by the caller in `paint.rs`).

use super::{Region, union_region};
use crate::tool::PainterTool;
use ph2d_painter_brush::{
    BrushSpec, Dab, Falloff, FalloffCurve, TextureSettings, blit_canvas_cached, blit_stamp,
    render_stamp_mask,
};
use std::sync::Arc;

/// Lazily-filled CANVAS-SPACE texture cache for the **Tiled / Stencil** mappings (canvas-fixed, so a
/// pixel's texture value is dab-independent): each canvas pixel's texture is computed once per stroke
/// and reused by every later dab — see [`ph2d_painter_brush::blit_canvas_cached`]. `tex` + `ready`
/// are `width*height`; persisted across the stroke (and strokes) until the key / size changes.
pub(super) struct CanvasTexCache {
    tex: Vec<u8>,
    ready: Vec<u8>,
    width: u32,
    height: u32,
    key: CanvasKey,
}

/// The cache holds the **texture** only (falloff is applied at blit), so it invalidates on the
/// texture settings or the imported image — NOT on falloff / hardness / colour / blend / radius.
#[derive(Clone, Copy, PartialEq)]
struct CanvasKey {
    texture: TextureSettings,
    image_version: u64,
}

/// Identifies the appearance the cached [`ph2d_painter_brush::StampMask`] was rendered for. The mask depends on the
/// falloff shape, the (View) texture, the imported image, and the mask resolution — but NOT on the
/// radius (we scale), colour or blend (applied at blit time), so an Anchored size-drag mostly reuses
/// one mask.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct StampKey {
    falloff: Falloff,
    hardness: f32,
    custom: FalloffCurve,
    texture: TextureSettings,
    image_version: u64,
    /// The Shape slot + its image version + Grain Depth all change the baked silhouette × grain mask.
    shape: TextureSettings,
    shape_image_version: u64,
    /// The Shape value ramp (B&W tonal remap) also changes the baked silhouette.
    shape_ramp_version: u64,
    grain_depth: f32,
    size: u32,
}

/// Mask resolution for a dab of `radius` px: enough texels to be 1:1 (`2·radius`), rounded up to a
/// power of two so a growing Anchored dab only re-renders a few times, clamped. Beyond the max the
/// texture is bilinearly upscaled (softer) — Blender caps its brush image likewise.
fn mask_size_for(radius: f32) -> u32 {
    const MIN: u32 = 32;
    const MAX: u32 = 1024;
    ((2.0 * radius.ceil()).max(1.0) as u32)
        .next_power_of_two()
        .clamp(MIN, MAX)
}

impl PainterTool {
    /// Scale-blit the cached stamp for each dab (the cacheable path of [`Self::stamp_dabs`]).
    pub(super) fn stamp_dabs_cached(
        &mut self,
        dabs: &[Dab],
        brush: &BrushSpec,
        alpha_locked: bool,
        w: u32,
        h: u32,
    ) {
        let max_r = dabs.iter().map(|d| d.radius_px).fold(0.0_f32, f32::max);
        self.ensure_stamp_cache(brush, mask_size_for(max_r));
        let Some((mask, _)) = self.paint.stamp_cache.as_ref() else {
            return;
        };
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let mut touched: Option<Region> = None;
        for d in dabs {
            let spec = BrushSpec {
                radius_px: d.radius_px,
                color: d.color,
                ..*brush
            };
            if let Some(r) = blit_stamp(
                buf,
                w,
                h,
                d.center,
                d.radius_px,
                mask,
                &spec,
                d.coverage,
                alpha_locked,
            ) {
                let rect = Region {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                };
                touched = Some(touched.map_or(rect, |acc| union_region(acc, rect)));
            }
        }
        if let Some(rect) = touched {
            self.mark_dirty(rect);
        }
    }

    /// Scale-blit each dab through the lazily-filled canvas texture cache — the Tiled / Stencil path
    /// of [`Self::stamp_dabs`]. The texture is computed once per canvas pixel per stroke; the falloff
    /// is still per-pixel (so it's appearance-identical to the per-pixel path).
    pub(super) fn stamp_dabs_canvas_cached(
        &mut self,
        dabs: &[Dab],
        brush: &BrushSpec,
        alpha_locked: bool,
        w: u32,
        h: u32,
    ) {
        self.ensure_canvas_cache(brush, w, h);
        self.ensure_shape_ramp_lut();
        // Disjoint borrows: the canvas (Arc), the cache (tex+ready), the imported images + the Shape
        // ramp LUT are all separate fields, so they can be held at once.
        let image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        // B&W tone ramp applies only WITH a Grain (no Grain ⇒ the Shape's ramp is the COLOUR ramp on
        // the ramped path) — suppress the tone remap here (Enio 2026-06-25).
        let shape_ramp_lut = (self.paint.shape_ramp_enabled && brush.texture.is_active())
            .then_some(self.paint.shape_ramp_lut.as_slice());
        let Some(cache) = self.paint.canvas_tex_cache.as_mut() else {
            return;
        };
        let (tex, ready) = (&mut cache.tex, &mut cache.ready);
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let mut touched: Option<Region> = None;
        for d in dabs {
            let spec = BrushSpec {
                radius_px: d.radius_px,
                color: d.color,
                ..*brush
            };
            if let Some(r) = blit_canvas_cached(
                buf,
                tex,
                ready,
                w,
                h,
                d.center,
                d.radius_px,
                &spec,
                image.as_ref(),
                shape_image.as_ref(),
                shape_ramp_lut,
                d.coverage,
                alpha_locked,
            ) {
                let rect = Region {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                };
                touched = Some(touched.map_or(rect, |acc| union_region(acc, rect)));
            }
        }
        if let Some(rect) = touched {
            self.mark_dirty(rect);
        }
    }

    /// (Re)allocate the canvas texture cache when the texture / image / canvas size changed; the
    /// fresh `ready` (all zero) means every pixel recomputes once on next touch. No-op on a hit.
    fn ensure_canvas_cache(&mut self, brush: &BrushSpec, w: u32, h: u32) {
        let key = CanvasKey {
            texture: brush.texture,
            image_version: self.paint.texture_image_version,
        };
        if matches!(&self.paint.canvas_tex_cache, Some(c) if c.width == w && c.height == h && c.key == key)
        {
            return;
        }
        let n = (w as usize) * (h as usize);
        self.paint.canvas_tex_cache = Some(CanvasTexCache {
            tex: vec![0u8; n],
            ready: vec![0u8; n],
            width: w,
            height: h,
            key,
        });
    }

    /// Stamp each dab through the **Color Ramp** colour path — the texture's scalar indexes the baked
    /// ramp LUT for the per-texel colour (the scalar-only caches can't carry colour, so this is
    /// per-pixel; the 256-entry LUT keeps the lookup cheap). Mirrors the per-pixel branch of
    /// [`Self::stamp_dabs`], calling [`ph2d_painter_brush::stamp_dab_ramped`].
    pub(super) fn stamp_dabs_ramped(
        &mut self,
        dabs: &[Dab],
        brush: &BrushSpec,
        alpha_locked: bool,
        w: u32,
        h: u32,
    ) {
        self.ensure_ramp_lut();
        self.ensure_shape_ramp_lut();
        let textured = brush.texture.is_active();
        // Disjoint borrows: the imported image + the ramp LUT (both `self.paint` sub-fields) and the
        // canvas (`self.canvas_rgba`) are held at once; the texture RNG is copied out + written back.
        let image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        let shape_active = brush.shape_silhouette_active(shape_image.is_some());
        // B&W tone ramp applies only WITH a Grain (no Grain ⇒ the Shape's ramp is the COLOUR ramp on
        // the ramped path) — suppress the tone remap here (Enio 2026-06-25).
        let shape_ramp_lut = (self.paint.shape_ramp_enabled && brush.texture.is_active())
            .then_some(self.paint.shape_ramp_lut.as_slice());
        let lut = &self.paint.texture_ramp_lut;
        let alpha_mode = self.paint.texture_ramp_alpha_mode;
        let rake = brush.texture.rake;
        let shape_rake = brush.shape.rake;
        let mut tex_rng = self.paint.tex_rng;
        let mut rake_dir = self.paint.rake_dir;
        let mut rake_accum = self.paint.rake_accum;
        let mut shape_rake_dir = self.paint.shape_rake_dir;
        let mut shape_rake_accum = self.paint.shape_rake_accum;
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let mut touched: Option<Region> = None;
        for (i, d) in dabs.iter().enumerate() {
            let spec = BrushSpec {
                radius_px: d.radius_px,
                color: d.color,
                ..*brush
            };
            // Resolve the Shape frame first (fixed slot order shape→grain, HR-5); its Rake follows the
            // stroke via the Shape's own heading. Random draws here, before the Grain.
            let shape_dir = advance_rake(
                shape_rake,
                &mut shape_rake_dir,
                &mut shape_rake_accum,
                super::brush_settings::dab_tangent(dabs, i),
                2.0 * d.radius_px,
            );
            let shape_basis = shape_active.then(|| {
                ph2d_painter_brush::texture::dab_basis(
                    &spec.shape,
                    shape_dir,
                    &mut tex_rng,
                    [w as f32, h as f32],
                    [1.0, 0.0],
                )
            });
            let shape_in = shape_basis
                .as_ref()
                .map(|sb| ph2d_painter_brush::ShapeInput {
                    basis: sb,
                    image: shape_image.as_ref(),
                    ramp_lut: shape_ramp_lut,
                });
            let dir = advance_rake(
                rake,
                &mut rake_dir,
                &mut rake_accum,
                super::brush_settings::dab_tangent(dabs, i),
                2.0 * d.radius_px,
            );
            let basis = textured.then(|| {
                ph2d_painter_brush::texture::dab_basis(
                    &spec.texture,
                    dir,
                    &mut tex_rng,
                    [w as f32, h as f32],
                    d.rotation,
                )
            });
            if let Some(r) = ph2d_painter_brush::stamp_dab_ramped(
                buf,
                w,
                h,
                d.center,
                &spec,
                d.coverage,
                alpha_locked,
                basis.as_ref(),
                image.as_ref(),
                shape_in,
                lut,
                alpha_mode,
            ) {
                let rect = Region {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                };
                touched = Some(touched.map_or(rect, |acc| union_region(acc, rect)));
            }
        }
        self.paint.tex_rng = tex_rng;
        self.paint.rake_dir = rake_dir;
        self.paint.rake_accum = rake_accum;
        self.paint.shape_rake_dir = shape_rake_dir;
        self.paint.shape_rake_accum = shape_rake_accum;
        if let Some(rect) = touched {
            self.mark_dirty(rect);
        }
    }

    /// Per-pixel stamp path — used when no cache applies (no texture, a canvas-relative / per-dab
    /// mapping, or per-dab Jitter Rotate). Resolves the per-dab texture frame (with `d.rotation`) and
    /// the per-dab Randomize-Color `d.color`, then stamps. The texture RNG is copied out (canvas
    /// borrow) + restored, exactly like [`Self::stamp_dabs_ramped`].
    pub(super) fn stamp_dabs_per_pixel(
        &mut self,
        dabs: &[Dab],
        brush: &BrushSpec,
        alpha_locked: bool,
        w: u32,
        h: u32,
    ) {
        self.ensure_shape_ramp_lut();
        let textured = brush.texture.is_active();
        let rake = brush.texture.rake;
        let shape_rake = brush.shape.rake;
        let image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        // B&W tone ramp applies only WITH a Grain (no Grain ⇒ the Shape's ramp is the COLOUR ramp on
        // the ramped path) — suppress the tone remap here (Enio 2026-06-25).
        let shape_ramp_lut = (self.paint.shape_ramp_enabled && brush.texture.is_active())
            .then_some(self.paint.shape_ramp_lut.as_slice());
        let shape_active = brush.shape_silhouette_active(shape_image.is_some());
        let mut tex_rng = self.paint.tex_rng;
        let mut rake_dir = self.paint.rake_dir;
        let mut rake_accum = self.paint.rake_accum;
        let mut shape_rake_dir = self.paint.shape_rake_dir;
        let mut shape_rake_accum = self.paint.shape_rake_accum;
        // Accumulate OFF (Strength < 1): hand the per-pixel blit the per-stroke coverage mask so it
        // caps each pixel at Strength. `paint_begin` cleared it on pointer-down; grow it to canvas size
        // (only the first dab of a stroke actually zero-fills — later dabs/frames keep the accumulation).
        let accumulate_cap = !brush.accumulate && brush.strength < 1.0;
        if accumulate_cap {
            self.paint
                .stroke_mask
                .resize((w as usize) * (h as usize), 0);
        }
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let mut mask: Option<&mut [u8]> =
            accumulate_cap.then_some(self.paint.stroke_mask.as_mut_slice());
        let mut touched: Option<Region> = None;
        for (i, d) in dabs.iter().enumerate() {
            // Per-dab Randomize Color rides on `d.color`; the radius is already jittered in `d`.
            let spec = BrushSpec {
                radius_px: d.radius_px,
                color: d.color,
                ..*brush
            };
            // Shape frame first (fixed slot order shape→grain, HR-5). Its Rake follows the stroke via
            // the Shape's own heading state; Random draws from `tex_rng` here, *before* the Grain — so a
            // brush with no Shape Random/Rake leaves the Grain stream byte-identical to before.
            let shape_dir = advance_rake(
                shape_rake,
                &mut shape_rake_dir,
                &mut shape_rake_accum,
                super::brush_settings::dab_tangent(dabs, i),
                2.0 * d.radius_px,
            );
            let shape_basis = shape_active.then(|| {
                ph2d_painter_brush::texture::dab_basis(
                    &spec.shape,
                    shape_dir,
                    &mut tex_rng,
                    [w as f32, h as f32],
                    [1.0, 0.0],
                )
            });
            let shape_in = shape_basis
                .as_ref()
                .map(|sb| ph2d_painter_brush::ShapeInput {
                    basis: sb,
                    image: shape_image.as_ref(),
                    ramp_lut: shape_ramp_lut,
                });
            let dir = advance_rake(
                rake,
                &mut rake_dir,
                &mut rake_accum,
                super::brush_settings::dab_tangent(dabs, i),
                2.0 * d.radius_px,
            );
            let basis = textured.then(|| {
                ph2d_painter_brush::texture::dab_basis(
                    &spec.texture,
                    dir,
                    &mut tex_rng,
                    [w as f32, h as f32],
                    d.rotation,
                )
            });
            if let Some(r) = ph2d_painter_brush::stamp_dab_textured_masked(
                buf,
                w,
                h,
                d.center,
                &spec,
                d.coverage,
                alpha_locked,
                basis.as_ref(),
                image.as_ref(),
                shape_in,
                mask.as_deref_mut(),
            ) {
                let rect = Region {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                };
                touched = Some(touched.map_or(rect, |acc| union_region(acc, rect)));
            }
        }
        self.paint.tex_rng = tex_rng;
        self.paint.rake_dir = rake_dir;
        self.paint.rake_accum = rake_accum;
        self.paint.shape_rake_dir = shape_rake_dir;
        self.paint.shape_rake_accum = shape_rake_accum;
        if let Some(rect) = touched {
            self.mark_dirty(rect);
        }
    }

    /// (Re)bake the ramp LUT when the ramp changed: `eval` is linear RGBA, but the dab blends in the
    /// layer's straight-sRGB space (matching `brush.color` = the picked colour / 255), so the RGB is
    /// converted linear → sRGB; alpha stays straight (no gamma).
    fn ensure_ramp_lut(&mut self) {
        if !self.paint.texture_ramp_dirty && self.paint.texture_ramp_lut.len() == 256 {
            return;
        }
        let mut lut = vec![[0.0f32; 4]; 256];
        self.paint.texture_ramp.bake_into(&mut lut);
        for c in &mut lut {
            c[0] = f32::from(ph2d_color::srgb::linear_to_srgb_byte(c[0])) / 255.0;
            c[1] = f32::from(ph2d_color::srgb::linear_to_srgb_byte(c[1])) / 255.0;
            c[2] = f32::from(ph2d_color::srgb::linear_to_srgb_byte(c[2])) / 255.0;
        }
        self.paint.texture_ramp_lut = lut;
        self.paint.texture_ramp_dirty = false;
    }

    /// Bake the **Shape value ramp** into its 256-entry grayscale LUT (scalar `[0,1]`, no gamma — it
    /// remaps the silhouette's coverage value). A no-op when clean.
    fn ensure_shape_ramp_lut(&mut self) {
        if !self.paint.shape_ramp_dirty && self.paint.shape_ramp_lut.len() == 256 {
            return;
        }
        let mut lut = vec![0.0f32; 256];
        self.paint.shape_ramp.bake_into(&mut lut);
        self.paint.shape_ramp_lut = lut;
        self.paint.shape_ramp_dirty = false;
    }

    /// The active Shape **tone** value-ramp LUT slice when enabled AND a Grain is active (no Grain ⇒
    /// the Shape's ramp is the colour ramp, not the tone ramp); caller `ensure_shape_ramp_lut`s first.
    fn shape_ramp_lut_slice(&self, grain_active: bool) -> Option<&[f32]> {
        (self.paint.shape_ramp_enabled && grain_active)
            .then_some(self.paint.shape_ramp_lut.as_slice())
    }

    /// Re-render the cached stamp mask when the appearance / mask size changed; a no-op on a hit.
    fn ensure_stamp_cache(&mut self, brush: &BrushSpec, size: u32) {
        self.ensure_shape_ramp_lut();
        let key = StampKey {
            falloff: brush.falloff,
            hardness: brush.hardness,
            custom: brush.custom_falloff,
            texture: brush.texture,
            image_version: self.paint.texture_image_version,
            shape: brush.shape,
            shape_image_version: self.paint.shape_image_version,
            shape_ramp_version: self.paint.shape_ramp_version,
            grain_depth: brush.grain_depth,
            size,
        };
        if self.paint.stamp_cache.as_ref().map(|(_, k)| *k) == Some(key) {
            return;
        }
        let mask = {
            let image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
            let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
            let shape_ramp_lut = self.shape_ramp_lut_slice(brush.texture.is_active());
            render_stamp_mask(
                brush,
                image.as_ref(),
                shape_image.as_ref(),
                shape_ramp_lut,
                size,
            )
        };
        self.paint.stamp_cache = Some((mask, key));
    }
}

/// Easing applied each time the **Rake** heading re-aims at a fresh long-baseline direction.
const RAKE_LERP: f32 = 0.5;

/// Minimum travel (px) before the **Rake** heading re-aims, regardless of brush size — keeps a small
/// brush from re-aiming on a single noisy chord.
const RAKE_BASELINE_MIN_PX: f32 = 10.0;

/// The texture-frame direction for a dab under **Rake**. The raw inter-dab chord is only ~3px (dabs ride
/// a stabilizer-smoothed spline), so its DIRECTION is noise — easing toward it still random-walks (the
/// "anarchy" Enio reported 2026-06-24). Instead ACCUMULATE the chords (`accum`) across dabs/segments and
/// re-aim the carried `dir` only once travel clears a real baseline (~½ diameter), easing toward that
/// long-baseline heading. `dir`/`accum` persist in `PaintState` across batches. Returns the unit
/// direction for `dab_basis` (`[0,0]` only before any travel → `dab_basis` falls back to the Angle);
/// with Rake off the raw tangent passes through (ignored downstream).
pub(super) fn advance_rake(
    rake: bool,
    dir: &mut [f32; 2],
    accum: &mut [f32; 2],
    tangent: [f32; 2],
    diameter_px: f32,
) -> [f32; 2] {
    if !rake {
        return tangent;
    }
    accum[0] += tangent[0];
    accum[1] += tangent[1];
    let baseline = (diameter_px * 0.5).max(RAKE_BASELINE_MIN_PX);
    let alen = (accum[0] * accum[0] + accum[1] * accum[1]).sqrt();
    if alen >= baseline {
        let nt = [accum[0] / alen, accum[1] / alen];
        *dir = if *dir == [0.0, 0.0] {
            nt // first re-aim → snap (no lerp from a zero heading)
        } else {
            let d = [
                dir[0] + (nt[0] - dir[0]) * RAKE_LERP,
                dir[1] + (nt[1] - dir[1]) * RAKE_LERP,
            ];
            let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
            if l < 1e-4 { nt } else { [d[0] / l, d[1] / l] } // near-180° reversal → snap
        };
        *accum = [0.0, 0.0];
        return *dir;
    }
    // Below baseline: hold the established heading. Before the first re-aim, use the partial NET
    // displacement (already summed, far cleaner than one chord) so the stroke START tracks; only fall
    // back to the Angle (`[0,0]`) when there is no travel yet.
    if *dir != [0.0, 0.0] {
        *dir
    } else if alen >= 1e-3 {
        [accum[0] / alen, accum[1] / alen]
    } else {
        [0.0, 0.0]
    }
}

#[cfg(test)]
mod rake_tests {
    use super::advance_rake;

    #[test]
    fn advance_rake_uses_long_baseline_and_persists() {
        // Rake OFF → the raw tangent passes through, no state kept.
        let (mut dir, mut acc) = ([0.0, 0.0], [0.0, 0.0]);
        assert_eq!(
            advance_rake(false, &mut dir, &mut acc, [3.0, 4.0], 50.0),
            [3.0, 4.0]
        );
        assert_eq!((dir, acc), ([0.0, 0.0], [0.0, 0.0]));

        // Below baseline (½·50 = 25): a single +x chord does NOT re-aim the heading, but the partial
        // net displacement already points +x — so a noisy first chord can't whip the texture around.
        let (mut dir, mut acc) = ([0.0, 0.0], [0.0, 0.0]);
        let r = advance_rake(true, &mut dir, &mut acc, [3.0, 0.0], 50.0);
        assert_eq!(dir, [0.0, 0.0], "not re-aimed below baseline");
        assert!(
            (r[0] - 1.0).abs() < 1e-5 && r[1].abs() < 1e-5,
            "partial tracks +x"
        );

        // Accumulate +x past the baseline → the heading re-aims to +x.
        for _ in 0..12 {
            advance_rake(true, &mut dir, &mut acc, [3.0, 0.0], 50.0);
        }
        assert!(
            (dir[0] - 1.0).abs() < 1e-5 && dir[1].abs() < 1e-5,
            "heading re-aimed to +x"
        );

        // Parked (≈0 tangent) keeps the established heading (no fallback to the Angle).
        let prev = dir;
        assert_eq!(
            advance_rake(true, &mut dir, &mut acc, [0.0, 0.0], 50.0),
            prev
        );
    }
}
