//! The **dynamic** per-layer-colour paint path — the per-pixel sibling of the cached
//! [`super::stamp_color_cache`] path, used (via the route) when a per-dab feature is on that the
//! constant-orientation cached stamps can't express: Shape **Rake / Random** rotation, **Randomize
//! Color**, or Grain **Rake / Random / Jitter Rotate**. Split from `stamp_color_cache` for the
//! workspace LOC cap; it shares [`super::stamp_color_cache::PerLayerStroke`] + the Grain-ramp helper.

use super::Region;
use crate::tool::PainterTool;
use ph2d_painter_brush::{
    BrushSpec, Dab, DynDab, FusedDab, ImageRgb, StrokeMethod, accumulate_color_stamps_rgba_batch,
    accumulate_shape_layers_rgba_batch, blend_over, shift_colors_like,
};
use ph2d_painter_effects::{BlendMode, apply_blend};
use std::sync::Arc;

impl PainterTool {
    /// Each dab resolves its own Shape basis (rotation) and colour (`d.color` carries Randomize Color),
    /// then each layer's silhouette × Grain (× Grain ramp) is resampled per pixel and over-accumulated —
    /// tinted — into that layer's per-stroke **premultiplied-RGBA** map. The layers stay separate, so the
    /// dirty region recomposites in z-order (higher above ALL lower across the stroke) and the whole tip
    /// blends onto the canvas ONCE via `brush.blend` — same z-order + blend + opacity as the cached path.
    ///
    /// Like the cached path, the **fill** methods (Line/Curve/Ellipse/Polygon) use the canvas itself as the
    /// recomposite base (no per-move snapshot) + self-clear the maps in the recomposite, so they don't pay
    /// a full-canvas clone + N-map re-allocation every pointer move (the fill FPS drop, Enio 2026-06-27).
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
        // Incremental methods snapshot the canvas once as the base + keep the maps; the fill methods use
        // the (restored) canvas as the base + reuse the maps (self-cleared below) — see the cached path.
        let incremental = matches!(
            brush.stroke_method,
            StrokeMethod::Space | StrokeMethod::Dots | StrokeMethod::Airbrush
        );
        let len = (w as usize) * (h as usize) * 4;
        self.ensure_per_layer_stroke(incremental, n, len);
        let grain_active = brush.texture.is_active();
        let grain_ramp_active = self.ensure_per_layer_grain_ramp(brush);
        let mut acc = std::mem::take(&mut self.paint.per_layer_stroke.cov);
        let grain_image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let grain_ramp = grain_ramp_active.then_some(self.paint.texture_ramp_lut.as_slice());
        let mut tex_rng = self.paint.tex_rng;
        let bbox: Option<Region>;
        {
            let masks = self.paint.shape_layers.masks();
            // The layers' resolved base colours (custom pick, else the brush base) — fixed across the
            // batch; Randomize Color then re-applies each dab's HSV shift to ALL of them below.
            let base_colors = self.paint.shape_layers.resolved_colors(brush.color);
            // Texture-Color layers sample their OWN per-pixel RGB (instead of the flat colour); `None`
            // for a tint layer. Fixed across the batch.
            let layer_rgb: Vec<Option<ImageRgb>> = (0..masks.len())
                .map(|i| self.paint.shape_layers.rgb_image(i))
                .collect();
            // Resolve the per-dab state (bases from `tex_rng` IN DAB ORDER — the RNG stream is exactly
            // the serial loop's — plus the Randomize-Color shift), then run ONE batched, row-banded,
            // layer-fused accumulate. This was the live "FPS 60→10" wall: the texture-colour default
            // routes here, and the per-dab × per-layer serial kernel measured 354 ms/move (N3) to
            // 1.87 s/move (N16) at 2048²·r100 (`per_layer_perf_live`). Byte-identical — gate
            // `batched_dynamic_accumulate_is_bit_identical_to_sequential` in `ph2d-painter-brush`.
            let dyn_dabs: Vec<DynDab> = dabs
                .iter()
                .map(|d| {
                    let spec = BrushSpec {
                        radius_px: d.radius_px,
                        ..*brush
                    };
                    // Jitter Rotate spins the whole footprint (and the Shape / View-Grain it drives); the
                    // per-slot Random Angle stays the basis `extra_rot = [1, 0]` so the pattern isn't
                    // double-rotated. Shape draws its Random from `tex_rng` before the Grain
                    // (byte-identical when off); Rake follows `d.dir`.
                    let fp = spec.footprint_deform().rotated_by(d.rotation);
                    let dims = [w as f32, h as f32];
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
                            [1.0, 0.0],
                            fp,
                        )
                    });
                    // Randomize Color: apply the dab's HSV shift (reconstructed from `brush.color →
                    // d.color`) to EVERY layer colour — custom picks included — so the whole tip jitters
                    // coherently per dab. Identity when Randomize Color is off (`d.color == brush.color`).
                    let mut colors = base_colors.clone();
                    shift_colors_like(brush.color, d.color, &mut colors);
                    DynDab {
                        center: d.center,
                        radius: d.radius_px,
                        spec,
                        shape_basis,
                        grain_basis,
                        colors,
                        coverage: (d.coverage * brush.flow * brush.strength).clamp(0.0, 1.0),
                    }
                })
                .collect();
            bbox = accumulate_shape_layers_rgba_batch(
                &mut acc,
                w,
                h,
                &dyn_dabs,
                &masks,
                &layer_rgb,
                grain_image.as_ref(),
                grain_ramp,
            )
            .map(|r| Region {
                x: r.x,
                y: r.y,
                w: r.w,
                h: r.h,
            });
        }
        self.paint.per_layer_stroke.cov = acc;
        self.paint.tex_rng = tex_rng;
        let Some(bb) = bbox else {
            return;
        };
        self.recomposite_per_layer_rgba(bb, brush, alpha_locked, incremental, w);
    }

    /// The cached **texture-colour** path (constant orientation): every layer's stamp is pre-baked WITH
    /// its captured per-pixel colour (premultiplied RGBA), so a Move costs one batched, row-banded,
    /// 4-channel scale-blit per dab across shared coordinates — instead of the dynamic kernel's per-layer
    /// silhouette + RGB resample per pixel (the live "FPS 60→10": 354 ms/move N3 · 1.87 s/move N16 at
    /// 2048²·r100, `per_layer_perf_live`). Routes here when Texture Color is on but NO per-dab dynamic is
    /// (Rake/Random/Jitter/Randomize-Color/canvas-fixed Grain — those still need the dynamic path). The
    /// maps are premul RGBA (4 B/px) and recomposite via [`Self::recomposite_per_layer_rgba`], exactly
    /// like the dynamic path — same z-order / blend / opacity semantics.
    pub(super) fn stamp_dabs_cached_color_rgba(
        &mut self,
        dabs: &[Dab],
        brush: &BrushSpec,
        alpha_locked: bool,
        w: u32,
        h: u32,
    ) {
        let max_r = dabs.iter().map(|d| d.radius_px).fold(0.0_f32, f32::max);
        self.ensure_color_stamp_cache(brush, super::stamp_cache::mask_size_for(max_r));
        let n = self
            .paint
            .color_stamp_cache
            .as_ref()
            .map_or(0, |(s, _)| s.len());
        if n == 0 {
            return;
        }
        let incremental = matches!(
            brush.stroke_method,
            StrokeMethod::Space | StrokeMethod::Dots | StrokeMethod::Airbrush
        );
        let len = (w as usize) * (h as usize) * 4;
        self.ensure_per_layer_stroke(incremental, n, len);
        let Some((stamps, key)) = self.paint.color_stamp_cache.take() else {
            return;
        };
        let fused: Vec<FusedDab> = dabs
            .iter()
            .map(|d| FusedDab {
                center: d.center,
                radius: d.radius_px,
                coverage: (d.coverage * brush.flow * brush.strength).clamp(0.0, 1.0),
            })
            .collect();
        let bbox = accumulate_color_stamps_rgba_batch(
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
        self.recomposite_per_layer_rgba(bb, brush, alpha_locked, incremental, w);
    }

    /// Recomposite the per-layer **premultiplied-RGBA** maps over `bb` onto the canvas — z-order among
    /// the layers (per-layer blend + opacity), then ONE `brush.blend` onto the base — and self-clear the
    /// fill maps. Shared by the DYNAMIC path and the cached **texture-colour** path (both keep 4 B/px
    /// maps). Row-banded across the cores (every pixel independent ⇒ bit-identical to serial).
    pub(super) fn recomposite_per_layer_rgba(
        &mut self,
        bb: Region,
        brush: &BrushSpec,
        alpha_locked: bool,
        incremental: bool,
        w: u32,
    ) {
        let n = self.paint.shape_layers.len();
        // Each layer's blend mode (the "B" dropdown) + opacity (a brush-only scale on its tip
        // contribution). `any_blend` gates per-layer blending — all Normal keeps the premultiplied over.
        let blends: Vec<u8> = (0..n)
            .map(|i| self.paint.shape_layers.effective_blend(i))
            .collect();
        let opac = self.paint.shape_layers.opacities();
        let any_blend = blends.iter().any(|&b| b != 0);
        let blend = brush.blend;
        {
            let buf = Arc::make_mut(&mut self.canvas_rgba);
            let super::stamp_color_cache::PerLayerStroke { pre, cov } =
                &mut self.paint.per_layer_stroke;
            let enc = |x: f32| (x.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            let wus = w as usize;
            // Row-banded like the cached recomposite: every pixel is independent (reads its own maps,
            // writes its own texel + self-clear), so disjoint bands are bit-identical to the serial sweep.
            let recomposite_band = |bufband: &mut [u8], covband: &mut [&mut [u8]], band_y0: u32| {
                let rows = (bufband.len() / (wus * 4)) as u32;
                for py in band_y0..band_y0 + rows {
                    for px in bb.x..bb.x + bb.w {
                        let p4 = (((py - band_y0) as usize) * wus + px as usize) * 4;
                        // Skip pixels no layer covers: a fill's dirty bbox is mostly empty + already holds
                        // the base (self-cleared below, so it stays clean for the next move).
                        if !covband.iter().take(n).any(|m| m[p4 + 3] != 0) {
                            continue;
                        }
                        // (1) Composite the per-layer premultiplied maps (z-order) into one STRAIGHT stroke.
                        // Default (all Normal): premultiplied source-over then un-premultiply — byte-identical
                        // to the prior path. With a per-layer blend pick: un-premultiply each layer and compose
                        // it onto the accumulated lower layers under its own mode (the layer-system formulas).
                        let stroke = if any_blend {
                            let mut s = [0.0f32; 4]; // straight rgba
                            for (i, map) in covband.iter().take(n).enumerate() {
                                let la = f32::from(map[p4 + 3]) / 255.0;
                                if la > 0.0 {
                                    let inv = 1.0 / la;
                                    // Un-premultiply to straight, then scale the layer's alpha by its opacity.
                                    let c = [
                                        f32::from(map[p4]) / 255.0 * inv,
                                        f32::from(map[p4 + 1]) / 255.0 * inv,
                                        f32::from(map[p4 + 2]) / 255.0 * inv,
                                        la * opac[i],
                                    ];
                                    s = apply_blend(BlendMode::from_u8(blends[i]), s, c);
                                }
                            }
                            s
                        } else {
                            let mut s = [0.0f32; 4]; // premultiplied rgba
                            for (i, map) in covband.iter().take(n).enumerate() {
                                // Scale the premultiplied colour AND its alpha by the layer's opacity (linear
                                // in premul space) — a brush-only opacity on that layer's tip contribution.
                                let o = opac[i];
                                let la = f32::from(map[p4 + 3]) / 255.0 * o;
                                if la > 0.0 {
                                    let inv = 1.0 - la;
                                    s[0] = f32::from(map[p4]) / 255.0 * o + s[0] * inv;
                                    s[1] = f32::from(map[p4 + 1]) / 255.0 * o + s[1] * inv;
                                    s[2] = f32::from(map[p4 + 2]) / 255.0 * o + s[2] * inv;
                                    s[3] = la + s[3] * inv;
                                }
                            }
                            if s[3] > 0.0 {
                                let inv = 1.0 / s[3];
                                [s[0] * inv, s[1] * inv, s[2] * inv, s[3]]
                            } else {
                                [0.0; 4]
                            }
                        };
                        // Base: the snapshot (incremental; absolute index) or the canvas itself (fills —
                        // already restored to the pre-shape this move).
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
                                acc[3] = pre_alpha;
                            }
                        }
                        bufband[p4] = enc(acc[0]);
                        bufband[p4 + 1] = enc(acc[1]);
                        bufband[p4 + 2] = enc(acc[2]);
                        bufband[p4 + 3] = enc(acc[3]);
                        // Fills: clear this pixel's maps so they're clean for the next move (each fill move
                        // is a fresh full shape) — avoids any per-move full clear / re-allocation.
                        if !incremental {
                            for m in covband.iter_mut().take(n) {
                                m[p4] = 0;
                                m[p4 + 1] = 0;
                                m[p4 + 2] = 0;
                                m[p4 + 3] = 0;
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
                    .map(|m| &mut m[by0 * wus * 4..by1 * wus * 4])
                    .collect();
                recomposite_band(&mut buf[by0 * wus * 4..by1 * wus * 4], &mut covfull, bb.y);
            } else {
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
                    let mut rest = &mut m[by0 * wus * 4..by1 * wus * 4];
                    for band in covbands.iter_mut() {
                        let take = (rpb * wus * 4).min(rest.len());
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
                        h.join().expect("dynamic recomposite band panicked");
                    }
                });
            }
        }
        self.mark_dirty(bb);
    }
}
