//! The cached **multi-layer coloured stamp** on the tool side (per-layer-colour mode): bake the
//! z-ordered layer composite — each layer tinted by its resolved colour — once into a
//! [`ph2d_painter_brush::ColorStampMask`], then scale-blit it per dab. Same performance profile as the
//! grayscale cached stamp ([`super::stamp_cache`]): the N-layer compositing runs only on an appearance
//! change, never per pixel per dab. Split from `stamp_cache` for the workspace LOC cap.
//!
//! The baked stamp folds in the Shape **Angle** + dab flatten (its View basis), like the grayscale
//! stamp; per-dab Shape Rake/Random rotation of the coloured stamp is a follow-up (it would re-bake or
//! rotate-at-blit). The Shape colour ramp is intentionally NOT consulted here — per-layer-colour mode
//! supersedes it (the panel hides the ramp section while it is on).

use super::ramp_lut::RampLutOwner;
use super::stamp_cache::mask_size_for;
use super::{Region, union_region};
use crate::tool::PainterTool;
use ph2d_painter_brush::{
    BrushSpec, Dab, Falloff, FalloffCurve, RampAlphaMode, StrokeMethod, TextureSettings,
    accumulate_color_stamp_coverage, accumulate_shape_layer_rgba, blend_over, blit_color_stamp,
    render_color_stamp_mask, render_ramp_color_stamp,
};
use std::sync::Arc;

/// Per-stroke accumulation for the per-layer-colour **recomposite**: the pre-stroke `canvas_rgba`
/// snapshot (`pre`) + one accumulated coverage map per Shape layer (`cov`). The stroke is re-composited
/// each batch as `pre ⊕ layer0 ⊕ … ⊕ layerN` (bottom→top), so the top layer paints above ALL of the
/// lower layer's accumulated coverage across the WHOLE stroke — not per dab (which buries earlier
/// highlights). Reset once per stroke in `paint_begin` for the incremental methods, or per fill for the
/// one-batch preview methods. `pre.is_empty()` ⇒ not yet initialised this stroke.
#[derive(Default)]
pub(super) struct PerLayerStroke {
    pre: Vec<u8>,
    cov: Vec<Vec<u8>>,
}

impl PerLayerStroke {
    /// Mark the accumulation un-initialised (drop the snapshot + coverage maps).
    pub(super) fn reset(&mut self) {
        self.pre.clear();
        self.cov.clear();
    }

    /// Snapshot the pre-stroke canvas + allocate `n` empty coverage maps of `len` pixels.
    fn init(&mut self, pre: Vec<u8>, n: usize, len: usize) {
        self.pre = pre;
        self.cov = (0..n).map(|_| vec![0u8; len]).collect();
    }
}

/// The panel's coloured-Shape **preview** resolution (texels per side) — the multi-layer composite is
/// baked once at this size and displayed scaled.
const PREVIEW_SIZE: u32 = 128;

/// Cached coloured Shape **preview** (the multi-layer composite as premultiplied RGBA), keyed by the same
/// appearance [`ColorStampKey`] at [`PREVIEW_SIZE`] — so the host re-renders it only when the Shape
/// appearance changes, NOT per frame (it was re-baking + re-allocating every frame). `None` ⇒ not in
/// Per-Layer Color mode → the panel falls back to the grayscale silhouette.
#[derive(Default)]
pub(super) struct ShapeColorPreview {
    cache: Option<(Arc<Vec<u8>>, ColorStampKey)>,
}

/// Identifies the appearance the cached [`ph2d_painter_brush::ColorStampMask`] was baked for. The bake
/// depends on the Shape frame (Angle / Size / Offset), the captured layers + their colours
/// (`layers_version` bumps on any of those), the brush base colour (an un-coloured layer paints it), the
/// Grain (image + depth), the dab flatten/rotate, and the mask resolution — but NOT the radius (scaled).
#[derive(Clone, Copy, PartialEq)]
pub(super) struct ColorStampKey {
    shape: TextureSettings,
    layers_version: u64,
    brush_color: [f32; 3],
    texture: TextureSettings,
    grain_image_version: u64,
    grain_depth: f32,
    /// The baked Grain Colour Ramp LUT version (`ramp_lut_version`), or `0` when no Grain ramp tints.
    grain_ramp_version: u64,
    dab_flatten: f32,
    dab_angle_deg: u16,
    size: u32,
}

impl PainterTool {
    /// Paint a dab batch in the per-layer-colour mode — the path of [`Self::stamp_dabs`].
    ///
    /// **Cross-stroke z-order (the 3-D-shaded stroke):** each layer accumulates its own per-stroke
    /// coverage map (`accumulate_color_stamp_coverage`), then the batch's dirty region is RE-COMPOSITED
    /// from the pre-stroke canvas: `pre ⊕ layer0 ⊕ … ⊕ layerN` (bottom→top). So the top layer is above
    /// ALL of the lower layer's accumulated painting across the WHOLE stroke — fixing the case where a
    /// real freehand stroke emits dabs in many batches and a later batch's lower layer buried the earlier
    /// batch's highlight (only the tip survived; worse the slower the stroke). A direct per-dab composite
    /// cannot do this in real time; the recomposite-from-pre-stroke can (Enio 2026-06-26).
    pub(super) fn stamp_dabs_cached_color(
        &mut self,
        dabs: &[Dab],
        brush: &BrushSpec,
        alpha_locked: bool,
        w: u32,
        h: u32,
    ) {
        let max_r = dabs.iter().map(|d| d.radius_px).fold(0.0_f32, f32::max);
        self.ensure_color_stamp_cache(brush, mask_size_for(max_r));
        let n = self
            .paint
            .color_stamp_cache
            .as_ref()
            .map_or(0, |(s, _)| s.len());
        if n == 0 {
            return;
        }
        // One-batch preview methods re-stamp the WHOLE shape each call → reset so each fill is fresh; the
        // incremental freehand methods accumulate across the whole stroke (reset once, in `paint_begin`).
        let incremental = matches!(
            brush.stroke_method,
            StrokeMethod::Space | StrokeMethod::Dots | StrokeMethod::Airbrush
        );
        if !incremental {
            self.paint.per_layer_stroke.reset();
        }
        if self.paint.per_layer_stroke.pre.is_empty() {
            let snap = (*self.canvas_rgba).clone();
            self.paint
                .per_layer_stroke
                .init(snap, n, (w as usize) * (h as usize));
        }
        // Accumulate every dab's per-layer coverage (cache taken out to disjoin the borrows).
        let Some((stamps, key)) = self.paint.color_stamp_cache.take() else {
            return;
        };
        let mut bbox: Option<Region> = None;
        for d in dabs {
            let cov = (d.coverage * brush.flow * brush.strength).clamp(0.0, 1.0);
            for (i, stamp) in stamps.iter().enumerate() {
                if let Some(r) = accumulate_color_stamp_coverage(
                    &mut self.paint.per_layer_stroke.cov[i],
                    w,
                    h,
                    d.center,
                    d.radius_px,
                    stamp,
                    cov,
                ) {
                    let rect = Region {
                        x: r.x,
                        y: r.y,
                        w: r.w,
                        h: r.h,
                    };
                    bbox = Some(bbox.map_or(rect, |acc| union_region(acc, rect)));
                }
            }
        }
        self.paint.color_stamp_cache = Some((stamps, key));
        let Some(bb) = bbox else {
            return;
        };
        // Recomposite the dirty region. Two stages so the **Brush blend mode** applies to the WHOLE
        // multi-layer tip, not to each layer in turn: (1) composite the Shape layers among themselves with
        // Normal source-over (bottom→top, premultiplied) into one stroke colour+alpha — the higher layer
        // above the lower; (2) blend that stroke onto the pre-stroke canvas ONCE via `brush.blend`. The
        // old per-layer `blend_over` double-applied Multiply/Screen/… across overlapping layers (and was a
        // no-op on a transparent canvas for the colour modes) → the blend mode looked ignored (Enio).
        let colors = self.paint.shape_layers.resolved_colors(brush.color);
        let blend = brush.blend;
        {
            let buf = Arc::make_mut(&mut self.canvas_rgba);
            let pls = &self.paint.per_layer_stroke;
            let enc = |x: f32| (x.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            for py in bb.y..bb.y + bb.h {
                for px in bb.x..bb.x + bb.w {
                    let p4 = ((py * w + px) * 4) as usize;
                    let idx = (py * w + px) as usize;
                    // (1) Premultiplied stroke = the layers composited over each other (z-order).
                    let mut s = [0.0f32; 4];
                    for (i, c) in colors.iter().enumerate() {
                        let a = f32::from(pls.cov[i][idx]) / 255.0;
                        if a > 0.0 {
                            let inv = 1.0 - a;
                            s[0] = c[0] * a + s[0] * inv;
                            s[1] = c[1] * a + s[1] * inv;
                            s[2] = c[2] * a + s[2] * inv;
                            s[3] = a + s[3] * inv;
                        }
                    }
                    let mut acc = [
                        f32::from(pls.pre[p4]) / 255.0,
                        f32::from(pls.pre[p4 + 1]) / 255.0,
                        f32::from(pls.pre[p4 + 2]) / 255.0,
                        f32::from(pls.pre[p4 + 3]) / 255.0,
                    ];
                    // (2) Blend the stroke onto the canvas once (un-premultiply for `blend_over`'s straight
                    // RGB + the stroke's own alpha as coverage).
                    if s[3] > 0.0 {
                        let pre_alpha = acc[3];
                        let inv = 1.0 / s[3];
                        acc = blend_over(blend, acc, [s[0] * inv, s[1] * inv, s[2] * inv], s[3]);
                        if alpha_locked {
                            acc[3] = pre_alpha; // alpha-lock: paint modifies colour, not the layer's alpha
                        }
                    }
                    buf[p4] = enc(acc[0]);
                    buf[p4 + 1] = enc(acc[1]);
                    buf[p4 + 2] = enc(acc[2]);
                    buf[p4 + 3] = enc(acc[3]);
                }
            }
        }
        self.mark_dirty(bb);
    }

    /// Re-bake the per-layer coloured stamps (one per layer, bottom→top) when the appearance / mask size
    /// changed; a no-op on a hit. Each stamp is a single layer tinted by its colour (× Grain, × the Grain
    /// Colour Ramp if active — so the ramp tints the multi-colour tip).
    fn ensure_color_stamp_cache(&mut self, brush: &BrushSpec, size: u32) {
        let grain_ramp_active = self.ensure_per_layer_grain_ramp(brush);
        let key = ColorStampKey {
            shape: brush.shape,
            layers_version: self.paint.shape_layers.version(),
            brush_color: brush.color,
            texture: brush.texture,
            grain_image_version: self.paint.texture_image_version,
            grain_depth: brush.grain_depth,
            grain_ramp_version: if grain_ramp_active {
                self.paint.ramp_lut_version
            } else {
                0
            },
            dab_flatten: brush.dab_flatten,
            dab_angle_deg: brush.dab_angle_deg,
            size,
        };
        if self.paint.color_stamp_cache.as_ref().map(|(_, k)| *k) == Some(key) {
            return;
        }
        let stamps = {
            let masks = self.paint.shape_layers.masks();
            let colors = self.paint.shape_layers.resolved_colors(brush.color);
            let grain_image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
            let grain_ramp = grain_ramp_active.then_some(self.paint.texture_ramp_lut.as_slice());
            masks
                .iter()
                .zip(colors.iter())
                .map(|(m, c)| {
                    render_color_stamp_mask(
                        brush,
                        std::slice::from_ref(m),
                        std::slice::from_ref(c),
                        grain_image.as_ref(),
                        grain_ramp,
                        size,
                    )
                })
                .collect()
        };
        self.paint.color_stamp_cache = Some((stamps, key));
    }

    /// Ensure the **Grain Colour Ramp** LUT is baked for the per-layer-colour paths (which suppress the
    /// Shape ramp, so the Grain is the colour owner) and report whether it's active: a Grain texture is
    /// active AND the Grain ramp is enabled. `false` ⇒ scalar Grain (no ramp tint).
    fn ensure_per_layer_grain_ramp(&mut self, brush: &BrushSpec) -> bool {
        let active = brush.texture.is_active() && self.paint.texture_ramp_enabled;
        if active {
            self.ensure_ramp_lut(RampLutOwner::Grain);
        }
        active
    }

    /// The **dynamic** per-layer-colour path — used (via the route) when a per-dab feature is on that the
    /// constant-orientation cached path can't express: Shape **Rake / Random** rotation, **Randomize
    /// Color**, or Grain **Jitter Rotate**. Each dab resolves its own Shape basis (rotation) and colour
    /// (`d.color` carries Randomize Color), then each layer's silhouette × Grain is resampled per pixel and
    /// over-accumulated — tinted — into that layer's per-stroke **premultiplied-RGBA** map. The layers stay
    /// separate, so the dirty region recomposites in z-order (the higher above ALL the lower across the
    /// stroke) and the whole tip blends onto the canvas ONCE via `brush.blend` — same z-order + blend +
    /// opacity semantics as the cached path, just with per-dab dynamics (Enio 2026-06-26).
    pub(super) fn stamp_dabs_per_layer_dynamic(
        &mut self,
        dabs: &[Dab],
        brush: &BrushSpec,
        alpha_locked: bool,
        w: u32,
        h: u32,
    ) {
        let n = self.paint.shape_layers.len();
        if n == 0 {
            return;
        }
        // The per-layer maps are premultiplied RGBA here (4 B/px), vs the cached path's 1 B/px coverage.
        let incremental = matches!(
            brush.stroke_method,
            StrokeMethod::Space | StrokeMethod::Dots | StrokeMethod::Airbrush
        );
        if !incremental {
            self.paint.per_layer_stroke.reset();
        }
        if self.paint.per_layer_stroke.pre.is_empty() {
            let snap = (*self.canvas_rgba).clone();
            self.paint
                .per_layer_stroke
                .init(snap, n, (w as usize) * (h as usize) * 4);
        }
        let grain_active = brush.texture.is_active();
        let grain_ramp_active = self.ensure_per_layer_grain_ramp(brush);
        let mut acc = std::mem::take(&mut self.paint.per_layer_stroke.cov);
        let grain_image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let grain_ramp = grain_ramp_active.then_some(self.paint.texture_ramp_lut.as_slice());
        let mut tex_rng = self.paint.tex_rng;
        let mut bbox: Option<Region> = None;
        {
            let masks = self.paint.shape_layers.masks();
            // The layers' resolved base colours (custom pick, else the brush base) — fixed across the
            // batch; Randomize Color then re-applies each dab's HSV shift to ALL of them below.
            let base_colors = self.paint.shape_layers.resolved_colors(brush.color);
            for d in dabs {
                let spec = BrushSpec {
                    radius_px: d.radius_px,
                    ..*brush
                };
                let fp = spec.footprint_deform();
                let dims = [w as f32, h as f32];
                // Shape draws its Random from `tex_rng` before the Grain (byte-identical stream when off);
                // Rake follows `d.dir`. Grain composes the per-dab Jitter Rotate (`d.rotation`).
                let shape_basis = ph2d_painter_brush::texture::dab_basis(
                    &spec.shape,
                    d.dir,
                    &mut tex_rng,
                    dims,
                    [1.0, 0.0],
                    fp,
                );
                let grain_basis = grain_active.then(|| {
                    ph2d_painter_brush::texture::dab_basis(
                        &spec.texture,
                        d.dir,
                        &mut tex_rng,
                        dims,
                        d.rotation,
                        fp,
                    )
                });
                // Randomize Color: apply the dab's HSV shift (reconstructed from `brush.color → d.color`)
                // to EVERY layer colour — custom picks included — so the whole tip jitters coherently per
                // dab. Identity when Randomize Color is off (`d.color == brush.color`).
                let mut colors = base_colors.clone();
                ph2d_painter_brush::shift_colors_like(brush.color, d.color, &mut colors);
                let cov = (d.coverage * brush.flow * brush.strength).clamp(0.0, 1.0);
                for (i, layer) in masks.iter().enumerate() {
                    let grain = grain_basis.as_ref().map(|gb| (gb, grain_image.as_ref()));
                    if let Some(r) = accumulate_shape_layer_rgba(
                        &mut acc[i],
                        w,
                        h,
                        d.center,
                        d.radius_px,
                        &spec,
                        &shape_basis,
                        layer,
                        grain,
                        grain_ramp,
                        colors[i],
                        cov,
                    ) {
                        let rect = Region {
                            x: r.x,
                            y: r.y,
                            w: r.w,
                            h: r.h,
                        };
                        bbox = Some(bbox.map_or(rect, |a| union_region(a, rect)));
                    }
                }
            }
        }
        self.paint.per_layer_stroke.cov = acc;
        self.paint.tex_rng = tex_rng;
        let Some(bb) = bbox else {
            return;
        };
        let blend = brush.blend;
        {
            let buf = Arc::make_mut(&mut self.canvas_rgba);
            let pls = &self.paint.per_layer_stroke;
            let enc = |x: f32| (x.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            for py in bb.y..bb.y + bb.h {
                for px in bb.x..bb.x + bb.w {
                    let p4 = ((py * w + px) * 4) as usize;
                    // (1) Composite the per-layer premultiplied maps over each other (z-order) → stroke.
                    let mut s = [0.0f32; 4];
                    for map in pls.cov.iter().take(n) {
                        let la = f32::from(map[p4 + 3]) / 255.0;
                        if la > 0.0 {
                            let inv = 1.0 - la;
                            s[0] = f32::from(map[p4]) / 255.0 + s[0] * inv;
                            s[1] = f32::from(map[p4 + 1]) / 255.0 + s[1] * inv;
                            s[2] = f32::from(map[p4 + 2]) / 255.0 + s[2] * inv;
                            s[3] = la + s[3] * inv;
                        }
                    }
                    let mut acc = [
                        f32::from(pls.pre[p4]) / 255.0,
                        f32::from(pls.pre[p4 + 1]) / 255.0,
                        f32::from(pls.pre[p4 + 2]) / 255.0,
                        f32::from(pls.pre[p4 + 3]) / 255.0,
                    ];
                    // (2) Blend the stroke onto the canvas once (un-premultiply for `blend_over`).
                    if s[3] > 0.0 {
                        let pre_alpha = acc[3];
                        let inv = 1.0 / s[3];
                        acc = blend_over(blend, acc, [s[0] * inv, s[1] * inv, s[2] * inv], s[3]);
                        if alpha_locked {
                            acc[3] = pre_alpha;
                        }
                    }
                    buf[p4] = enc(acc[0]);
                    buf[p4 + 1] = enc(acc[1]);
                    buf[p4 + 2] = enc(acc[2]);
                    buf[p4 + 3] = enc(acc[3]);
                }
            }
        }
        self.mark_dirty(bb);
    }

    /// Re-bake the coloured Shape **preview** (the multi-layer composite) IFF its appearance changed —
    /// a cheap key-compare on a hit, so the panel preview costs a render only on an edit, never per
    /// frame. Returns `true` when the cache changed (the host then re-publishes it to the panel). Keyed
    /// by the same [`ColorStampKey`] as the paint stamps, at the fixed [`PREVIEW_SIZE`].
    pub fn refresh_shape_color_preview(&mut self) -> bool {
        let color_mode = self.paint.shape_layers.is_color_mode();
        let brush = self.paint.brush;
        // Only touch the Grain ramp LUT in colour mode, so the per-frame preview never flips the ramp
        // owner away from the normal (non-per-layer) paint path.
        let grain_ramp_active = color_mode && self.ensure_per_layer_grain_ramp(&brush);
        let want = color_mode.then(|| ColorStampKey {
            shape: brush.shape,
            layers_version: self.paint.shape_layers.version(),
            brush_color: brush.color,
            texture: brush.texture,
            grain_image_version: self.paint.texture_image_version,
            grain_depth: brush.grain_depth,
            grain_ramp_version: if grain_ramp_active {
                self.paint.ramp_lut_version
            } else {
                0
            },
            dab_flatten: brush.dab_flatten,
            dab_angle_deg: brush.dab_angle_deg,
            size: PREVIEW_SIZE,
        });
        if want
            == self
                .paint
                .shape_color_preview
                .cache
                .as_ref()
                .map(|(_, k)| *k)
        {
            return false;
        }
        self.paint.shape_color_preview.cache = want.map(|key| {
            let masks = self.paint.shape_layers.masks();
            let colors = self.paint.shape_layers.resolved_colors(brush.color);
            let grain = self.paint.texture_image.as_ref().map(|i| i.as_mask());
            let grain_ramp = grain_ramp_active.then_some(self.paint.texture_ramp_lut.as_slice());
            let stamp = render_color_stamp_mask(
                &brush,
                &masks,
                &colors,
                grain.as_ref(),
                grain_ramp,
                PREVIEW_SIZE,
            );
            (Arc::new(stamp.data().to_vec()), key)
        });
        true
    }

    /// The cached coloured Shape preview `(premultiplied RGBA, w, h)` (a cheap `Arc` clone) for the panel;
    /// `None` ⇒ the grayscale silhouette. Call [`Self::refresh_shape_color_preview`] first to bake it.
    #[must_use]
    pub fn shape_color_preview(&self) -> Option<(Arc<Vec<u8>>, u32, u32)> {
        self.paint
            .shape_color_preview
            .cache
            .as_ref()
            .map(|(d, k)| (d.clone(), k.size, k.size))
    }
}

/// Identifies the appearance the cached **Grain+Ramp** coloured stamp was baked for: the silhouette
/// (falloff or Shape) inputs, the Grain (image + frame), the baked owner ramp LUT (`ramp_lut_version`),
/// whether the ramp alpha scales coverage, the dab flatten/rotate, and the resolution — NOT the radius.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct RampColorStampKey {
    falloff: Falloff,
    hardness: f32,
    custom: FalloffCurve,
    texture: TextureSettings,
    grain_image_version: u64,
    shape: TextureSettings,
    shape_image_version: u64,
    shape_ramp_version: u64,
    ramp_lut_version: u64,
    strength_alpha: bool,
    dab_flatten: f32,
    dab_angle_deg: u16,
    size: u32,
}

impl PainterTool {
    /// Scale-blit the cached **Grain+Ramp** coloured stamp per dab — the cacheable fast path for a
    /// colour ramp over an active (View) Grain (the Grain value indexes the ramp colour). Bakes the
    /// grain×ramp once, then one blit per dab, vs the per-pixel Grain+ramp recompute (the slow fills).
    pub(super) fn stamp_dabs_cached_ramp_color(
        &mut self,
        dabs: &[Dab],
        brush: &BrushSpec,
        alpha_locked: bool,
        w: u32,
        h: u32,
    ) {
        let owner = self.color_ramp_owner(brush.texture.is_active());
        self.ensure_ramp_lut(owner);
        self.ensure_shape_ramp_lut();
        let max_r = dabs.iter().map(|d| d.radius_px).fold(0.0_f32, f32::max);
        self.ensure_ramp_color_stamp_cache(brush, mask_size_for(max_r));
        let Some((stamp, _)) = self.paint.ramp_color_stamp_cache.as_ref() else {
            return;
        };
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let mut touched: Option<Region> = None;
        for d in dabs {
            let spec = BrushSpec {
                radius_px: d.radius_px,
                ..*brush
            };
            if let Some(r) = blit_color_stamp(
                buf,
                w,
                h,
                d.center,
                d.radius_px,
                stamp,
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

    /// Re-bake the Grain+Ramp coloured stamp when the appearance / mask size changed; a no-op on a hit.
    fn ensure_ramp_color_stamp_cache(&mut self, brush: &BrushSpec, size: u32) {
        let owner = self.color_ramp_owner(brush.texture.is_active());
        let strength_alpha = matches!(self.active_ramp_alpha_mode(owner), RampAlphaMode::Strength);
        let key = RampColorStampKey {
            falloff: brush.falloff,
            hardness: brush.hardness,
            custom: brush.custom_falloff,
            texture: brush.texture,
            grain_image_version: self.paint.texture_image_version,
            shape: brush.shape,
            shape_image_version: self.paint.shape_image_version,
            shape_ramp_version: self.paint.shape_ramp_version,
            ramp_lut_version: self.paint.ramp_lut_version,
            strength_alpha,
            dab_flatten: brush.dab_flatten,
            dab_angle_deg: brush.dab_angle_deg,
            size,
        };
        if self.paint.ramp_color_stamp_cache.as_ref().map(|(_, k)| *k) == Some(key) {
            return;
        }
        let stamp = {
            let grain_image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
            let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
            let shape_tone = self.shape_tone_lut_slice();
            let ramp = self.paint.texture_ramp_lut.as_slice();
            render_ramp_color_stamp(
                brush,
                grain_image.as_ref(),
                shape_image.as_ref(),
                shape_tone,
                ramp,
                strength_alpha,
                size,
            )
        };
        self.paint.ramp_color_stamp_cache = Some((stamp, key));
    }
}
