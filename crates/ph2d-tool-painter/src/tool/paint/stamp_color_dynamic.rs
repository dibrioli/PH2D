//! The **dynamic** per-layer-colour paint path — the per-pixel sibling of the cached
//! [`super::stamp_color_cache`] path, used (via the route) when a per-dab feature is on that the
//! constant-orientation cached stamps can't express: Shape **Rake / Random** rotation, **Randomize
//! Color**, or Grain **Rake / Random / Jitter Rotate**. Split from `stamp_color_cache` for the
//! workspace LOC cap; it shares [`super::stamp_color_cache::PerLayerStroke`] + the Grain-ramp helper.

use super::{Region, union_region};
use crate::tool::PainterTool;
use ph2d_painter_brush::{
    BrushSpec, Dab, ImageRgb, StrokeMethod, accumulate_shape_layer_rgba, blend_over,
    shift_colors_like,
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
    /// Like the cached path, the **fill** methods (Line/Curve/Circle/Polygon) use the canvas itself as the
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
        if incremental {
            if self.paint.per_layer_stroke.pre.is_empty() {
                let snap = (*self.canvas_rgba).clone();
                self.paint.per_layer_stroke.init(snap, n, len);
            }
        } else if self.paint.per_layer_stroke.cov.len() != n {
            self.paint.per_layer_stroke.init(Vec::new(), n, len);
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
            // Texture-Color layers sample their OWN per-pixel RGB (instead of the flat colour); `None`
            // for a tint layer. Fixed across the batch.
            let layer_rgb: Vec<Option<ImageRgb>> = (0..masks.len())
                .map(|i| self.paint.shape_layers.rgb_image(i))
                .collect();
            for d in dabs {
                let spec = BrushSpec {
                    radius_px: d.radius_px,
                    ..*brush
                };
                // Jitter Rotate spins the whole footprint (and the Shape / View-Grain it drives); the
                // per-slot Random Angle stays the basis `extra_rot = [1, 0]` so the pattern isn't
                // double-rotated. Shape draws its Random from `tex_rng` before the Grain (byte-identical
                // when off); Rake follows `d.dir`.
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
                // Randomize Color: apply the dab's HSV shift (reconstructed from `brush.color → d.color`)
                // to EVERY layer colour — custom picks included — so the whole tip jitters coherently per
                // dab. Identity when Randomize Color is off (`d.color == brush.color`).
                let mut colors = base_colors.clone();
                shift_colors_like(brush.color, d.color, &mut colors);
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
                        layer_rgb.get(i).and_then(|o| o.as_ref()),
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
            let pls = &mut self.paint.per_layer_stroke;
            let enc = |x: f32| (x.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            for py in bb.y..bb.y + bb.h {
                for px in bb.x..bb.x + bb.w {
                    let p4 = ((py * w + px) * 4) as usize;
                    // Skip pixels no layer covers: a fill's dirty bbox is mostly empty + already holds the
                    // base (self-cleared below, so it stays clean for the next move). The big win on fills.
                    if !pls.cov.iter().take(n).any(|m| m[p4 + 3] != 0) {
                        continue;
                    }
                    // (1) Composite the per-layer premultiplied maps (z-order) into one STRAIGHT stroke.
                    // Default (all Normal): premultiplied source-over then un-premultiply — byte-identical
                    // to the prior path. With a per-layer blend pick: un-premultiply each layer and compose
                    // it onto the accumulated lower layers under its own mode (the layer-system formulas).
                    let stroke = if any_blend {
                        let mut s = [0.0f32; 4]; // straight rgba
                        for (i, map) in pls.cov.iter().take(n).enumerate() {
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
                        for (i, map) in pls.cov.iter().take(n).enumerate() {
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
                    // Base: the snapshot (incremental) or the canvas itself (fills — already restored to
                    // the pre-shape this move). Copy 4 bytes so the read ends before the write.
                    let pre = {
                        let base: &[u8] = if incremental { &pls.pre } else { &*buf };
                        [base[p4], base[p4 + 1], base[p4 + 2], base[p4 + 3]]
                    };
                    let mut acc = pre.map(|b| f32::from(b) / 255.0);
                    // (2) Blend the stroke onto the base once via the Brush blend mode.
                    if stroke[3] > 0.0 {
                        let pre_alpha = acc[3];
                        acc = blend_over(blend, acc, [stroke[0], stroke[1], stroke[2]], stroke[3]);
                        if alpha_locked {
                            acc[3] = pre_alpha;
                        }
                    }
                    buf[p4] = enc(acc[0]);
                    buf[p4 + 1] = enc(acc[1]);
                    buf[p4 + 2] = enc(acc[2]);
                    buf[p4 + 3] = enc(acc[3]);
                    // Fills: clear this pixel's maps so they're clean for the next move (each fill move is a
                    // fresh full shape) — avoids any per-move full clear / re-allocation.
                    if !incremental {
                        for m in pls.cov.iter_mut().take(n) {
                            m[p4] = 0;
                            m[p4 + 1] = 0;
                            m[p4 + 2] = 0;
                            m[p4 + 3] = 0;
                        }
                    }
                }
            }
        }
        self.mark_dirty(bb);
    }
}
