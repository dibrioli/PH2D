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
    BrushSpec, ColorStampMask, Dab, Falloff, FalloffCurve, FusedDab, ImageRgb, RampAlphaMode,
    StrokeMethod, TextureSettings, accumulate_color_stamps_fused_batch, blend_over,
    blit_color_stamp, render_color_stamp_mask, render_ramp_color_stamp,
};
use ph2d_painter_effects::{BlendMode, apply_blend};
use std::sync::Arc;

/// Per-stroke accumulation for the per-layer-colour **recomposite**: the pre-stroke `canvas_rgba`
/// snapshot (`pre`) + one accumulated coverage map per Shape layer (`cov`). The stroke is re-composited
/// each batch as `pre ⊕ layer0 ⊕ … ⊕ layerN` (bottom→top), so the top layer paints above ALL of the
/// lower layer's accumulated coverage across the WHOLE stroke — not per dab (which buries earlier
/// highlights). Reset once per stroke in `paint_begin` for the incremental methods, or per fill for the
/// one-batch preview methods. `pre.is_empty()` ⇒ not yet initialised this stroke.
#[derive(Default)]
pub(super) struct PerLayerStroke {
    pub(super) pre: Vec<u8>,
    pub(super) cov: Vec<Vec<u8>>,
}

impl PerLayerStroke {
    /// Mark the accumulation un-initialised (drop the snapshot + coverage maps).
    pub(super) fn reset(&mut self) {
        self.pre.clear();
        self.cov.clear();
    }

    /// Set the base (`pre`; empty for the fill methods, which use the canvas) + allocate `n` zeroed maps
    /// of `len` (1 B/px coverage in the cached path, 4 B/px premul RGBA in the dynamic path).
    pub(super) fn init(&mut self, pre: Vec<u8>, n: usize, len: usize) {
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

/// Composite the per-layer **premultiplied** preview stamps (all at `size`) bottom→top under their
/// per-layer blend modes into one premultiplied-RGBA buffer — the thumbnail equivalent of the on-canvas
/// recomposite's stage-1, so the preview shows the blends. The bottom layer's blend is inert (composited
/// onto the transparent backdrop), matching the "base layer has no blend" rule. HR-5: float-only + the
/// effects blend formulas.
fn composite_preview_blended(stamps: &[ColorStampMask], blends: &[u8], size: u32) -> Vec<u8> {
    let n = (size as usize) * (size as usize);
    let mut out = vec![0u8; n * 4];
    let enc = |x: f32| (x.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    for p in 0..n {
        let o = p * 4;
        let mut s = [0.0f32; 4]; // straight rgba — the composite so far
        for (li, stamp) in stamps.iter().enumerate() {
            let d = stamp.data();
            let pa = f32::from(d[o + 3]) / 255.0;
            if pa <= 0.0 {
                continue;
            }
            let inv = 1.0 / pa;
            let src = [
                f32::from(d[o]) / 255.0 * inv,
                f32::from(d[o + 1]) / 255.0 * inv,
                f32::from(d[o + 2]) / 255.0 * inv,
                pa,
            ];
            s = apply_blend(
                BlendMode::from_u8(blends.get(li).copied().unwrap_or(0)),
                s,
                src,
            );
        }
        out[o] = enc(s[0] * s[3]);
        out[o + 1] = enc(s[1] * s[3]);
        out[o + 2] = enc(s[2] * s[3]);
        out[o + 3] = enc(s[3]);
    }
    out
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
        // Incremental freehand methods (Space/Dots/Airbrush) paint straight onto the canvas, accumulating
        // across batches → snapshot the canvas ONCE per stroke as the recomposite base, and keep the maps.
        // The fill methods (Line/Curve/Ellipse/Polygon) restore the canvas to the pre-shape each move (so the
        // canvas IS the base — no snapshot) and re-stamp the WHOLE shape; the maps are reused + self-cleared
        // in the recomposite, NOT reset+cloned+re-allocated per move — that per-move full-canvas clone + N
        // allocations + full-bbox recomposite was the fill FPS drop (Enio 2026-06-27).
        let incremental = matches!(
            brush.stroke_method,
            StrokeMethod::Space | StrokeMethod::Dots | StrokeMethod::Airbrush
        );
        let len = (w as usize) * (h as usize);
        if incremental {
            if self.paint.per_layer_stroke.pre.is_empty() {
                let snap = (*self.canvas_rgba).clone();
                self.paint.per_layer_stroke.init(snap, n, len);
            }
        } else if self.paint.per_layer_stroke.cov.len() != n {
            // `paint_begin` cleared the maps; allocate N zeroed maps (no snapshot — the canvas is the base).
            self.paint.per_layer_stroke.init(Vec::new(), n, len);
        }
        // Accumulate every dab's per-layer coverage (cache taken out to disjoin the borrows).
        let Some((stamps, key)) = self.paint.color_stamp_cache.take() else {
            return;
        };
        // One fused BATCH pass accumulates all dabs × all layers (shared bilinear coords, alpha-only),
        // row-banded across the cores — byte-identical to the per-dab loop (each pixel keeps its serial
        // dab order; gate `batched_fused_accumulate_is_bit_identical_to_sequential`). This was the
        // painter's per-Move bottleneck: 96.5% of the move and SERIAL, which is why the Mac mini and the
        // 32-core desktop stalled identically (`HANDOFF_per_layer_color_perf` §1.R).
        let fused: Vec<FusedDab> = dabs
            .iter()
            .map(|d| FusedDab {
                center: d.center,
                radius: d.radius_px,
                coverage: (d.coverage * brush.flow * brush.strength).clamp(0.0, 1.0),
            })
            .collect();
        let bbox = accumulate_color_stamps_fused_batch(
            &mut self.paint.per_layer_stroke.cov,
            w,
            h,
            &fused,
            &stamps,
        )
        .map(|r| Region {
            x: r.x,
            y: r.y,
            w: r.w,
            h: r.h,
        });
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
        // Per-layer blend mode + opacity (a brush-only scale on each layer's tip contribution). Texture
        // Color isn't possible on this cached path (it routes to the dynamic path). `any_blend` gates the
        // per-layer-blend stage-1: with all Normal, keep the byte-identical premultiplied source-over.
        let blends: Vec<u8> = (0..n)
            .map(|i| self.paint.shape_layers.effective_blend(i))
            .collect();
        let opac = self.paint.shape_layers.opacities();
        let any_blend = blends.iter().any(|&b| b != 0);
        let blend = brush.blend;
        {
            let buf = Arc::make_mut(&mut self.canvas_rgba);
            let PerLayerStroke { pre, cov } = &mut self.paint.per_layer_stroke;
            let enc = |x: f32| (x.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            let wus = w as usize;
            // The per-pixel recomposite for one row band: `bufband`/`covband` hold the band's rows
            // (first row `band_y0`); `pre` is the full canvas (absolute index). Every pixel is
            // independent (reads its own coverage, writes its own texel + self-clear), so disjoint row
            // bands are bit-identical to the serial sweep.
            let recomposite_band = |bufband: &mut [u8], covband: &mut [&mut [u8]], band_y0: u32| {
                let rows = (bufband.len() / (wus * 4)) as u32;
                for py in band_y0..band_y0 + rows {
                    for px in bb.x..bb.x + bb.w {
                        let lrow = ((py - band_y0) as usize) * wus + px as usize;
                        let p4 = lrow * 4;
                        // Skip pixels no layer covers: a fill's dirty bbox is mostly empty (a diagonal
                        // line's bbox ≈ the whole canvas) and those already hold the base.
                        if !covband.iter().take(n).any(|m| m[lrow] != 0) {
                            continue;
                        }
                        // (1) Composite the Shape layers among themselves (z-order) into one STRAIGHT
                        // stroke colour+alpha. Default (all Normal): premultiplied source-over, then
                        // un-premultiply — byte-identical to the prior path. With a per-layer blend
                        // pick: each layer composites onto the accumulated lower layers under its own
                        // mode (the layer-system blend formulas).
                        let stroke = if any_blend {
                            let mut s = [0.0f32; 4]; // straight rgba
                            for (i, c) in colors.iter().enumerate() {
                                let a = f32::from(covband[i][lrow]) / 255.0 * opac[i];
                                if a > 0.0 {
                                    s = apply_blend(
                                        BlendMode::from_u8(blends[i]),
                                        s,
                                        [c[0], c[1], c[2], a],
                                    );
                                }
                            }
                            s
                        } else {
                            let mut s = [0.0f32; 4]; // premultiplied rgba
                            for (i, c) in colors.iter().enumerate() {
                                let a = f32::from(covband[i][lrow]) / 255.0 * opac[i];
                                if a > 0.0 {
                                    let inv = 1.0 - a;
                                    s[0] = c[0] * a + s[0] * inv;
                                    s[1] = c[1] * a + s[1] * inv;
                                    s[2] = c[2] * a + s[2] * inv;
                                    s[3] = a + s[3] * inv;
                                }
                            }
                            if s[3] > 0.0 {
                                let inv = 1.0 / s[3];
                                [s[0] * inv, s[1] * inv, s[2] * inv, s[3]]
                            } else {
                                [0.0; 4]
                            }
                        };
                        // Base: the pre-stroke snapshot (incremental; absolute index) or the canvas
                        // itself (fills — already restored to the pre-shape this move). Copy 4 bytes
                        // so the read ends before the write.
                        let a4 = ((py as usize) * wus + px as usize) * 4;
                        let base4 = if incremental {
                            [pre[a4], pre[a4 + 1], pre[a4 + 2], pre[a4 + 3]]
                        } else {
                            [
                                bufband[p4],
                                bufband[p4 + 1],
                                bufband[p4 + 2],
                                bufband[p4 + 3],
                            ]
                        };
                        let mut acc = base4.map(|b| f32::from(b) / 255.0);
                        // (2) Blend the stroke onto the base once via the Brush blend mode.
                        if stroke[3] > 0.0 {
                            let pre_alpha = acc[3];
                            acc = blend_over(
                                blend,
                                acc,
                                [stroke[0], stroke[1], stroke[2]],
                                stroke[3],
                            );
                            if alpha_locked {
                                acc[3] = pre_alpha; // alpha-lock: colour only, not the layer's alpha
                            }
                        }
                        bufband[p4] = enc(acc[0]);
                        bufband[p4 + 1] = enc(acc[1]);
                        bufband[p4 + 2] = enc(acc[2]);
                        bufband[p4 + 3] = enc(acc[3]);
                        // Fills: clear this pixel's maps so they're clean for the next move (each fill
                        // move is a fresh full shape) — no per-move full clear / re-allocation.
                        if !incremental {
                            for m in covband.iter_mut().take(n) {
                                m[lrow] = 0;
                            }
                        }
                    }
                }
            };
            let rows = bb.h as usize;
            /// Below this bbox area (px) the serial sweep beats spawning scoped threads.
            const PARALLEL_MIN_AREA: usize = 1 << 17;
            let threads = std::thread::available_parallelism()
                .map(std::num::NonZero::get)
                .unwrap_or(1)
                .clamp(1, rows.max(1));
            let (by0, by1) = (bb.y as usize, (bb.y + bb.h) as usize);
            if (rows * bb.w as usize) < PARALLEL_MIN_AREA || threads == 1 || rows <= 1 {
                let mut covfull: Vec<&mut [u8]> = cov
                    .iter_mut()
                    .map(|m| &mut m[by0 * wus..by1 * wus])
                    .collect();
                recomposite_band(&mut buf[by0 * wus * 4..by1 * wus * 4], &mut covfull, bb.y);
            } else {
                // Split the canvas AND every layer map into the same disjoint row bands.
                let rpb = rows.div_ceil(threads);
                let nbands = rows.div_ceil(rpb);
                let mut bufbands: Vec<&mut [u8]> = Vec::with_capacity(nbands);
                let mut rest = &mut buf[by0 * wus * 4..by1 * wus * 4];
                for _ in 0..nbands {
                    let take = (rpb * wus * 4).min(rest.len());
                    let (head, tail) = rest.split_at_mut(take);
                    bufbands.push(head);
                    rest = tail;
                }
                let mut covbands: Vec<Vec<&mut [u8]>> =
                    (0..nbands).map(|_| Vec::with_capacity(n)).collect();
                for m in cov.iter_mut() {
                    let mut rest = &mut m[by0 * wus..by1 * wus];
                    for band in covbands.iter_mut() {
                        let take = (rpb * wus).min(rest.len());
                        let (head, tail) = rest.split_at_mut(take);
                        band.push(head);
                        rest = tail;
                    }
                }
                let recomposite_band = &recomposite_band;
                std::thread::scope(|s| {
                    let handles: Vec<_> = bufbands
                        .into_iter()
                        .zip(covbands)
                        .enumerate()
                        .map(|(bi, (bband, mut cband))| {
                            let band_y0 = bb.y + (bi * rpb) as u32;
                            s.spawn(move || recomposite_band(bband, &mut cband, band_y0))
                        })
                        .collect();
                    for h in handles {
                        h.join().expect("per-layer recomposite band panicked");
                    }
                });
            }
        }
        self.mark_dirty(bb);
    }

    /// Re-bake the per-layer coloured stamps (one per layer, bottom→top) when the appearance / mask size
    /// changed; a no-op on a hit. Each stamp is a single layer tinted by its colour (× Grain, × the Grain
    /// Colour Ramp if active — so the ramp tints the multi-colour tip).
    pub(super) fn ensure_color_stamp_cache(&mut self, brush: &BrushSpec, size: u32) {
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
            (0..masks.len())
                .map(|i| {
                    // Each layer bakes with its own colour source: the captured per-pixel RGB when the
                    // layer is in Texture Color (the default — served by the cached RGBA route), else
                    // the flat resolved tint (alpha-only cached route ignores the baked RGB). Opacity
                    // applies at recomposite (so `&[1.0]`).
                    let rgb = self.paint.shape_layers.rgb_image(i);
                    render_color_stamp_mask(
                        brush,
                        std::slice::from_ref(&masks[i]),
                        std::slice::from_ref(&colors[i]),
                        std::slice::from_ref(&rgb),
                        &[1.0],
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
    pub(super) fn ensure_per_layer_grain_ramp(&mut self, brush: &BrushSpec) -> bool {
        let active = brush.texture.is_active() && self.paint.texture_ramp_enabled;
        if active {
            self.ensure_ramp_lut(RampLutOwner::Grain);
        }
        active
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
            let layer_rgb: Vec<Option<ImageRgb>> = (0..masks.len())
                .map(|i| self.paint.shape_layers.rgb_image(i))
                .collect();
            let opac = self.paint.shape_layers.opacities();
            let blends: Vec<u8> = (0..masks.len())
                .map(|i| self.paint.shape_layers.effective_blend(i))
                .collect();
            let grain = self.paint.texture_image.as_ref().map(|i| i.as_mask());
            let grain_ramp = grain_ramp_active.then_some(self.paint.texture_ramp_lut.as_slice());
            // Bake EACH layer's premultiplied stamp on its own, then composite them bottom→top under their
            // per-layer blend modes (`apply_blend`) — so the thumbnail shows the blends, matching the
            // on-canvas recomposite (the all-layers `render_color_stamp_mask` only does Normal-over).
            let per_layer: Vec<_> = (0..masks.len())
                .map(|i| {
                    render_color_stamp_mask(
                        &brush,
                        std::slice::from_ref(&masks[i]),
                        std::slice::from_ref(&colors[i]),
                        std::slice::from_ref(&layer_rgb[i]),
                        std::slice::from_ref(&opac[i]),
                        grain.as_ref(),
                        grain_ramp,
                        PREVIEW_SIZE,
                    )
                })
                .collect();
            (
                Arc::new(composite_preview_blended(&per_layer, &blends, PREVIEW_SIZE)),
                key,
            )
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
