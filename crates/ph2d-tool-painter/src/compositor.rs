//! CPU layer compositor (W3.T3.2) — the correctness reference for the
//! layer stack. `docs/Painter_projeto/02_layers.md` §2.11.
//!
//! Composites a [`LayerStack`] **top-down recursively** in linear-sRGB:
//! each visible layer is decoded sRGB→linear, blended over the
//! accumulator via [`ph2d_painter_brush::apply_blend`] (opacity folded
//! into the source alpha), groups composite their children into a
//! sub-buffer first, then the final accumulator is encoded linear→sRGB.
//!
//! Scope (T3.2): blend mode + opacity + visibility + group recursion.
//! Masks (T3.5) and clipping (T3.6) extend this in later tasks; mask
//! layers are skipped here (they composite via their parent).
//!
//! This is the **reference** path — clear over fast. The real-time
//! zero-alloc GPU compositor (the `layers_composite_50_4k_under_5ms` /
//! `layers_no_alloc_hot_compose` perf gates) is the Coordinator's
//! `ph2d-render` sibling; this CPU path backs the dirty-rect / golden
//! correctness gates and is what tests assert against.

use crate::layers::{LayerId, LayerKind, LayerStack, MAX_GROUP_DEPTH};
use ph2d_color::srgb::{linear_to_srgb_byte, srgb_to_linear_byte};
use ph2d_painter_brush::{BlendMode, apply_blend};
use std::collections::BTreeMap;

/// RGBA8 (straight, sRGB-encoded) pixels for one layer — canvas-sized.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayerImage {
    pub width: u32,
    pub height: u32,
    /// `width * height * 4` bytes, row-major RGBA8.
    pub rgba8: Vec<u8>,
}

impl LayerImage {
    /// A transparent canvas-sized image.
    #[must_use]
    pub fn transparent(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            rgba8: vec![0u8; (width as usize) * (height as usize) * 4],
        }
    }
}

/// Resolves a layer's pixels for the compositor: the canvas-sized straight
/// sRGB8 RGBA bytes (`canvas_w * canvas_h * 4`) for a raster/mask layer.
/// Returns a borrowed slice (mirror of the GPU `LayerPixels { rgba8: &[u8] }`)
/// so a host can hand the active layer's working buffer (e.g. the tool's
/// `Arc<Vec<u8>>` canvas) without cloning. `None` for unknown/group layers.
pub trait LayerPixelSource {
    fn layer_rgba(&self, id: LayerId) -> Option<&[u8]>;
}

/// Trivial [`LayerPixelSource`] over a `BTreeMap` — tests + simple hosts.
/// `BTreeMap` (not `HashMap`) per HR-5.
#[derive(Clone, Debug, Default)]
pub struct MapPixelSource {
    pub images: BTreeMap<LayerId, LayerImage>,
}

impl MapPixelSource {
    pub fn insert(&mut self, id: LayerId, image: LayerImage) {
        self.images.insert(id, image);
    }
}

impl LayerPixelSource for MapPixelSource {
    fn layer_rgba(&self, id: LayerId) -> Option<&[u8]> {
        self.images.get(&id).map(|img| img.rgba8.as_slice())
    }
}

/// Decode one straight sRGB8 texel to straight linear RGBA `[f32; 4]`.
/// Alpha is linear coverage (no transfer function), per the stamp pipeline.
#[inline]
fn decode(rgba8: &[u8], idx: usize) -> [f32; 4] {
    let b = idx * 4;
    [
        srgb_to_linear_byte(rgba8[b]),
        srgb_to_linear_byte(rgba8[b + 1]),
        srgb_to_linear_byte(rgba8[b + 2]),
        rgba8[b + 3] as f32 / 255.0,
    ]
}

/// Straight grayscale value `[0, 1]` of a mask texel — Rec.601 luma of the
/// straight sRGB bytes (`R = G = B` for grayscale mask paint; the formula also
/// degrades gracefully for a non-grayscale mask). White (255) = fully visible,
/// black = hidden. The mask multiplies the parent's alpha — a coverage op, so
/// computed in straight space (no transfer function), per §2.7.
#[inline]
pub(crate) fn mask_value(rgba8: &[u8], idx: usize) -> f32 {
    let b = idx * 4;
    (0.299 * rgba8[b] as f32 + 0.587 * rgba8[b + 1] as f32 + 0.114 * rgba8[b + 2] as f32) / 255.0
}

/// A rectangular sub-region of the canvas (dirty rect), clamped to bounds
/// by the compositor.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// Composite the whole stack → canvas-sized straight sRGB8 RGBA.
#[must_use]
pub fn composite(
    stack: &LayerStack,
    src: &impl LayerPixelSource,
    width: u32,
    height: u32,
) -> Vec<u8> {
    // COLOR-RAW-OK: straight sRGB8 canvas pixels — GPU-uploadable blob (mirrors ph2d-render LayerPixels.rgba8), not a typed color value
    let region = Region {
        x: 0,
        y: 0,
        w: width,
        h: height,
    };
    let acc = composite_region_linear(stack, src, width, height, region);
    encode(&acc)
}

/// Composite only `region` → straight sRGB8 RGBA of size `region.w *
/// region.h`. Per-pixel compositing is spatially independent, so this is
/// bit-identical to cropping [`composite`] to the same rect — the basis of
/// dirty-rect recompose (`layers_dirty_rect_correctness`).
#[must_use]
pub fn composite_region(
    stack: &LayerStack,
    src: &impl LayerPixelSource,
    width: u32,
    height: u32,
    region: Region,
) -> Vec<u8> {
    let acc = composite_region_linear(stack, src, width, height, region);
    encode(&acc)
}

fn encode(acc: &[[f32; 4]]) -> Vec<u8> {
    let mut out = vec![0u8; acc.len() * 4];
    for (px, lin) in acc.iter().enumerate() {
        out[px * 4] = linear_to_srgb_byte(lin[0]);
        out[px * 4 + 1] = linear_to_srgb_byte(lin[1]);
        out[px * 4 + 2] = linear_to_srgb_byte(lin[2]);
        out[px * 4 + 3] = (lin[3].clamp(0.0, 1.0) * 255.0).round() as u8;
    }
    out
}

/// Composite `region` into a freshly-allocated linear accumulator
/// (`region.w * region.h` entries, row-major). Clamps the region to the
/// canvas bounds.
fn composite_region_linear(
    stack: &LayerStack,
    src: &impl LayerPixelSource,
    width: u32,
    height: u32,
    region: Region,
) -> Vec<[f32; 4]> {
    let rx = region.x.min(width);
    let ry = region.y.min(height);
    let rw = region.w.min(width - rx);
    let rh = region.h.min(height - ry);
    let mut acc = vec![[0.0f32; 4]; (rw as usize) * (rh as usize)];
    composite_into(&mut acc, stack.root(), stack, src, width, rx, ry, rw, rh, 0);
    acc
}

/// Blend the layers in `ids` (top-to-bottom) over `acc`, restricted to the
/// `rw * rh` window anchored at canvas `(rx, ry)`. `canvas_w` is the full
/// canvas stride (layer images are canvas-sized).
#[allow(clippy::too_many_arguments)]
fn composite_into(
    acc: &mut [[f32; 4]],
    ids: &[LayerId],
    stack: &LayerStack,
    src: &impl LayerPixelSource,
    canvas_w: u32,
    rx: u32,
    ry: u32,
    rw: u32,
    rh: u32,
    depth: usize,
) {
    // Defense-in-depth (audit W3): never recurse past the group-nesting cap,
    // even if a (future deserialized / forged) stack smuggles a cycle or an
    // over-deep tree past the data-model guards — bounded work, no stack
    // overflow. The runtime API already caps construction at MAX_GROUP_DEPTH.
    if depth > MAX_GROUP_DEPTH {
        return;
    }
    // Bottom-to-top (§2.11): the panel order is top-first, so iterate rev.
    // T3.6: a clipping layer clips to the nearest NON-clipping raster below it
    // (§2.8); consecutive clipping layers chain to the same base. As we walk up,
    // `clip_base` holds that base raster's straight pixels (alpha = the clip).
    let mut clip_base: Option<&[u8]> = None;
    for &id in ids.iter().rev() {
        let Some(layer) = stack.get(id) else { continue };
        if !layer.visible || layer.opacity <= 0.0 {
            continue;
        }
        let opacity = layer.opacity.clamp(0.0, 1.0);
        let mode = layer.blend_mode;
        match &layer.kind {
            LayerKind::Raster(_) => {
                let Some(rgba) = src.layer_rgba(id) else {
                    continue;
                };
                // Bounds guard: the highest texel index this window reads is
                // the bottom-right corner. Skip a too-short buffer rather than
                // panic-index (defense vs a mismatched/forged provider).
                let max_idx = if rw == 0 || rh == 0 {
                    0
                } else {
                    ((ry + rh - 1) * canvas_w + (rx + rw - 1)) as usize
                };
                if rgba.len() < (max_idx + 1) * 4 {
                    continue;
                }
                // T3.5: an attached grayscale mask multiplies this layer's alpha
                // (white = visible, black = hidden; `1 - value` when inverted).
                // Mask pixels are canvas-sized RGBA8 in the same source; a
                // missing/short mask buffer is treated as "no mask" (no panic).
                let mask = layer.mask.and_then(|mid| match &stack.get(mid)?.kind {
                    LayerKind::Mask(m) => {
                        let mrgba = src.layer_rgba(mid)?;
                        (mrgba.len() >= (max_idx + 1) * 4).then_some((mrgba, m.inverted))
                    }
                    _ => None,
                });
                // T3.6: a clipping layer paints only where its clip base is
                // opaque — multiply its alpha by the base's straight alpha.
                let clip = if layer.clipping { clip_base } else { None };
                blend_window(acc, rx, ry, rw, rh, mode, opacity, |gx, gy| {
                    let idx = (gy * canvas_w + gx) as usize;
                    let mut s = decode(rgba, idx);
                    if let Some((mrgba, inverted)) = mask {
                        let v = mask_value(mrgba, idx);
                        s[3] *= if inverted { 1.0 - v } else { v };
                    }
                    if let Some(base) = clip {
                        s[3] *= base[idx * 4 + 3] as f32 / 255.0;
                    }
                    s
                });
                // A NON-clipping raster becomes the clip base for the layers
                // above it; a clipping raster chains to the same base.
                if !layer.clipping {
                    clip_base = Some(rgba);
                }
            }
            LayerKind::Group(g) => {
                // Composite the children into their own sub-window, then
                // blend that as a single layer (group blend/opacity).
                let mut sub = vec![[0.0f32; 4]; (rw as usize) * (rh as usize)];
                composite_into(
                    &mut sub,
                    &g.children,
                    stack,
                    src,
                    canvas_w,
                    rx,
                    ry,
                    rw,
                    rh,
                    depth + 1,
                );
                blend_window(acc, rx, ry, rw, rh, mode, opacity, |gx, gy| {
                    let lx = gx - rx;
                    let ly = gy - ry;
                    sub[(ly * rw + lx) as usize]
                });
                // A group is not a raster clip base — it breaks the clip chain.
                clip_base = None;
            }
            // W4 (ADR-0045 §2.7, CPU-first): a non-destructive adjustment
            // transforms the layers BELOW it — already in `acc` (we walk
            // bottom-up). Copy the window, run the kind's compute, then blend
            // the result back over `acc` by the adjustment's OWN opacity × mask
            // in its blend mode (inner fields authoritative, amendment-1).
            // `apply_adjustment` works in straight linear f32 — no 8-bit round-
            // trip in the per-frame composite. Mask/opacity live HERE, not in
            // the compute hook (W4-triage Coord decision).
            LayerKind::Adjustment(adj) => {
                if !adj.visible || adj.opacity <= 0.0 {
                    continue;
                }
                let adj_opacity = adj.opacity.clamp(0.0, 1.0);
                let adj_mode = adj.blend_mode;
                let mut adjusted = acc.to_vec();
                ph2d_painter_brush::adjustments::apply_adjustment(
                    &adj.kind,
                    &adj.params,
                    &mut adjusted,
                );
                // Optional mask — raw layer-id (amendment-1): white = full
                // effect. Missing/short buffer = no mask (full effect).
                let mask = adj.mask.and_then(|mid| {
                    let m = match &stack.get(LayerId(mid))?.kind {
                        LayerKind::Mask(m) => m.inverted,
                        _ => return None,
                    };
                    Some((src.layer_rgba(LayerId(mid))?, m))
                });
                for ly in 0..rh {
                    for lx in 0..rw {
                        let i = (ly * rw + lx) as usize;
                        let gidx = ((ry + ly) * canvas_w + (rx + lx)) as usize;
                        let mut t = adj_opacity;
                        if let Some((mrgba, inverted)) = mask
                            && mrgba.len() >= (gidx + 1) * 4
                        {
                            let v = mask_value(mrgba, gidx);
                            t *= if inverted { 1.0 - v } else { v };
                        }
                        if t <= 0.0 {
                            continue;
                        }
                        let base = acc[i];
                        // Blend the adjusted color (carrying the base's coverage)
                        // over the base in the adjustment's mode, then lerp by t
                        // so opacity/mask scale the effect; coverage is kept.
                        let src_px = [adjusted[i][0], adjusted[i][1], adjusted[i][2], base[3]];
                        let blended = apply_blend(adj_mode, base, src_px);
                        acc[i] = [
                            base[0] + (blended[0] - base[0]) * t,
                            base[1] + (blended[1] - base[1]) * t,
                            base[2] + (blended[2] - base[2]) * t,
                            base[3],
                        ];
                    }
                }
                // An adjustment is not a raster clip base — it breaks the chain.
                clip_base = None;
            }
            // Mask layers composite via their parent (T3.5); skip standalone.
            LayerKind::Mask(_) => continue,
        }
    }
}

/// W4 compositor recomposition cache (ADR-0045 §2.7) — **skeleton**. Each
/// adjustment layer is a "cut point": the accumulator BELOW it is cached, so a
/// change at layer N only invalidates cut points ≥ N (above stay valid). This
/// is the perf lever for the soft `adjustment_layer_recomposition_perf_4k` gate
/// (≤1 ms slider-drag @ 4K). v1 composites inline (no cache wired into the hot
/// path yet); this type + the dirty-rect field land the contract (HR-5
/// `BTreeMap`, not `HashMap`) for the impl/T4.x to wire.
#[derive(Default)]
pub struct CompositorCache {
    /// Cached straight-linear accumulator just below each adjustment layer,
    /// keyed by the adjustment's [`LayerId`]. Stable key + deterministic
    /// iteration (HR-5 — no `std::HashMap` in the compositor, ADR-0022).
    cuts: std::collections::BTreeMap<LayerId, Vec<[f32; 4]>>,
    /// Recompose only this sub-rect (the dab/transform bbox); `None` = full.
    dirty_rect: Option<Region>,
}

impl CompositorCache {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Invalidate every cut point at or above the changed layer (skeleton:
    /// clears all until the dirty-tracking walk lands). Cuts strictly below
    /// `changed` stay valid.
    pub fn invalidate_from(&mut self, _changed: LayerId, _stack: &LayerStack) {
        self.cuts.clear();
    }

    /// Mark the sub-rect to recompose next drain.
    pub fn mark_dirty(&mut self, region: Region) {
        self.dirty_rect = Some(region);
    }

    #[must_use]
    pub fn dirty_rect(&self) -> Option<Region> {
        self.dirty_rect
    }
}

/// Blend one source layer (sampled by `sample(global_x, global_y)`) over
/// the window in `acc`, folding `opacity` into the source alpha.
#[allow(clippy::too_many_arguments)]
fn blend_window(
    acc: &mut [[f32; 4]],
    rx: u32,
    ry: u32,
    rw: u32,
    rh: u32,
    mode: BlendMode,
    opacity: f32,
    sample: impl Fn(u32, u32) -> [f32; 4],
) {
    for ly in 0..rh {
        for lx in 0..rw {
            let mut s = sample(rx + lx, ry + ly);
            s[3] *= opacity;
            let i = (ly * rw + lx) as usize;
            acc[i] = apply_blend(mode, acc[i], s);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, rgba: [u8; 4]) -> LayerImage {
        LayerImage {
            width: w,
            height: h,
            rgba8: rgba.repeat((w * h) as usize),
        }
    }

    #[test]
    fn two_rasters_normal_top_over_bottom() {
        // SMOKE-INTRA Day 4: top layer (opaque) fully covers bottom.
        let (w, h) = (2, 2);
        let mut s = LayerStack::new();
        let bottom = s.add_raster("bottom", w, h).unwrap();
        let top = s.add_raster("top", w, h).unwrap();
        let mut src = MapPixelSource::default();
        src.insert(bottom, solid(w, h, [200, 50, 50, 255]));
        src.insert(top, solid(w, h, [50, 50, 200, 255]));
        let out = composite(&s, &src, w, h);
        // Every pixel == top color (opaque over).
        for px in out.chunks_exact(4) {
            assert_eq!(&px[0..3], &[50, 50, 200], "expected top color");
            assert_eq!(px[3], 255);
        }
    }

    #[test]
    fn transparent_top_reveals_bottom() {
        let (w, h) = (1, 1);
        let mut s = LayerStack::new();
        let bottom = s.add_raster("bottom", w, h).unwrap();
        let top = s.add_raster("top", w, h).unwrap();
        let mut src = MapPixelSource::default();
        src.insert(bottom, solid(w, h, [200, 50, 50, 255]));
        src.insert(top, solid(w, h, [50, 50, 200, 0])); // alpha 0
        let out = composite(&s, &src, w, h);
        assert_eq!(&out[0..3], &[200, 50, 50]);
        assert_eq!(out[3], 255);
    }

    #[test]
    fn invisible_layer_is_skipped() {
        let (w, h) = (1, 1);
        let mut s = LayerStack::new();
        let bottom = s.add_raster("bottom", w, h).unwrap();
        let top = s.add_raster("top", w, h).unwrap();
        s.set_visible(top, false);
        let mut src = MapPixelSource::default();
        src.insert(bottom, solid(w, h, [10, 20, 30, 255]));
        src.insert(top, solid(w, h, [200, 200, 200, 255]));
        let out = composite(&s, &src, w, h);
        assert_eq!(&out[0..3], &[10, 20, 30]);
    }

    #[test]
    fn opacity_half_blends_toward_bottom() {
        // 50% white over black → mid gray (in *linear*, then sRGB-encoded
        // → ~188, NOT 128). Verifies opacity folds into alpha + linear blend.
        let (w, h) = (1, 1);
        let mut s = LayerStack::new();
        let bottom = s.add_raster("bottom", w, h).unwrap();
        let top = s.add_raster("top", w, h).unwrap();
        s.set_opacity(top, 0.5);
        let mut src = MapPixelSource::default();
        src.insert(bottom, solid(w, h, [0, 0, 0, 255]));
        src.insert(top, solid(w, h, [255, 255, 255, 255]));
        let out = composite(&s, &src, w, h);
        // linear 0.5 → sRGB ≈ 188.
        assert!(
            (out[0] as i32 - 188).abs() <= 1,
            "expected ~188 (linear-space half), got {}",
            out[0]
        );
        assert_eq!(out[3], 255);
    }

    #[test]
    fn multiply_blend_darkens() {
        let (w, h) = (1, 1);
        let mut s = LayerStack::new();
        let bottom = s.add_raster("bottom", w, h).unwrap();
        let top = s.add_raster("top", w, h).unwrap();
        s.set_blend_mode(top, BlendMode::Multiply);
        let mut src = MapPixelSource::default();
        src.insert(bottom, solid(w, h, [180, 180, 180, 255]));
        src.insert(top, solid(w, h, [180, 180, 180, 255]));
        let out = composite(&s, &src, w, h);
        // Multiply of equal mid-grays is strictly darker than either input.
        assert!(out[0] < 180, "multiply should darken, got {}", out[0]);
    }

    #[test]
    fn group_composites_children_then_blends() {
        // A group holding one opaque raster, group opacity 1.0 → same as
        // the raster composited directly.
        let (w, h) = (1, 1);
        let mut s = LayerStack::new();
        let bottom = s.add_raster("bottom", w, h).unwrap();
        let child = s.add_raster("child", w, h).unwrap();
        let g = s.add_group("group").unwrap();
        s.move_into_group(child, g);
        let mut src = MapPixelSource::default();
        src.insert(bottom, solid(w, h, [0, 0, 0, 255]));
        src.insert(child, solid(w, h, [120, 60, 200, 255]));
        let out = composite(&s, &src, w, h);
        assert_eq!(&out[0..3], &[120, 60, 200]);
    }

    #[test]
    fn group_opacity_attenuates_the_stack() {
        let (w, h) = (1, 1);
        let mut s = LayerStack::new();
        let bottom = s.add_raster("bottom", w, h).unwrap();
        let child = s.add_raster("child", w, h).unwrap();
        let g = s.add_group("group").unwrap();
        s.move_into_group(child, g);
        s.set_opacity(g, 0.5);
        let mut src = MapPixelSource::default();
        src.insert(bottom, solid(w, h, [0, 0, 0, 255]));
        src.insert(child, solid(w, h, [255, 255, 255, 255]));
        let out = composite(&s, &src, w, h);
        assert!(
            (out[0] as i32 - 188).abs() <= 1,
            "group 50% → ~188, got {}",
            out[0]
        );
    }

    #[test]
    fn mask_black_hides_parent_reveals_below() {
        // T3.5: a black mask on the top raster fully hides it → the layer below
        // shows through (mask multiplies the parent's alpha to 0).
        let (w, h) = (1, 1);
        let mut s = LayerStack::new();
        let bottom = s.add_raster("bottom", w, h).unwrap();
        let top = s.add_raster("top", w, h).unwrap();
        let mask = s.add_mask(top).unwrap();
        let mut src = MapPixelSource::default();
        src.insert(bottom, solid(w, h, [0, 200, 0, 255])); // green below
        src.insert(top, solid(w, h, [200, 0, 0, 255])); // red on top
        src.insert(mask, solid(w, h, [0, 0, 0, 255])); // black → hide top
        let out = composite(&s, &src, w, h);
        assert_eq!(
            &out[0..3],
            &[0, 200, 0],
            "black mask hides top → green shows"
        );
    }

    #[test]
    fn mask_white_keeps_parent_fully_visible() {
        let (w, h) = (1, 1);
        let mut s = LayerStack::new();
        let bottom = s.add_raster("bottom", w, h).unwrap();
        let top = s.add_raster("top", w, h).unwrap();
        let mask = s.add_mask(top).unwrap();
        let mut src = MapPixelSource::default();
        src.insert(bottom, solid(w, h, [0, 200, 0, 255]));
        src.insert(top, solid(w, h, [200, 0, 0, 255]));
        src.insert(mask, solid(w, h, [255, 255, 255, 255])); // white → full visible
        let out = composite(&s, &src, w, h);
        assert_eq!(&out[0..3], &[200, 0, 0], "white mask keeps top visible");
    }

    #[test]
    fn mask_inverted_flips_black_to_visible() {
        let (w, h) = (1, 1);
        let mut s = LayerStack::new();
        let bottom = s.add_raster("bottom", w, h).unwrap();
        let top = s.add_raster("top", w, h).unwrap();
        let mask = s.add_mask(top).unwrap();
        s.set_mask_inverted(mask, true);
        let mut src = MapPixelSource::default();
        src.insert(bottom, solid(w, h, [0, 200, 0, 255]));
        src.insert(top, solid(w, h, [200, 0, 0, 255]));
        src.insert(mask, solid(w, h, [0, 0, 0, 255])); // black, but inverted → visible
        let out = composite(&s, &src, w, h);
        assert_eq!(
            &out[0..3],
            &[200, 0, 0],
            "inverted black mask → top visible"
        );
    }

    #[test]
    fn mask_gray_is_partial_visibility() {
        // A mid-gray mask (~50%) partially reveals: result is between top and
        // bottom. Use Normal blend; assert the red channel lands strictly between
        // the fully-hidden (green's 0 red) and fully-shown (top's 200) extremes.
        let (w, h) = (1, 1);
        let mut s = LayerStack::new();
        let bottom = s.add_raster("bottom", w, h).unwrap();
        let top = s.add_raster("top", w, h).unwrap();
        let mask = s.add_mask(top).unwrap();
        let mut src = MapPixelSource::default();
        src.insert(bottom, solid(w, h, [0, 0, 0, 255])); // black below
        src.insert(top, solid(w, h, [255, 0, 0, 255])); // red on top
        src.insert(mask, solid(w, h, [128, 128, 128, 255])); // ~50% visible
        let out = composite(&s, &src, w, h);
        assert!(
            out[0] > 0 && out[0] < 255,
            "gray mask → partial red, got {}",
            out[0]
        );
    }

    #[test]
    fn clipping_layer_paints_only_where_base_is_opaque() {
        // T3.6: a clipping raster is masked by the base's straight alpha —
        // visible only where the base below it is opaque.
        let (w, h) = (2, 1);
        let mut s = LayerStack::new();
        let base = s.add_raster("base", w, h).unwrap();
        let clip = s.add_raster("clip", w, h).unwrap();
        s.get_mut(clip).unwrap().clipping = true;
        let mut src = MapPixelSource::default();
        let mut bimg = LayerImage::transparent(w, h);
        bimg.rgba8[0..4].copy_from_slice(&[0, 200, 0, 255]); // left opaque green
        bimg.rgba8[4..8].copy_from_slice(&[0, 0, 0, 0]); // right transparent
        src.insert(base, bimg);
        src.insert(clip, solid(w, h, [200, 0, 0, 255])); // opaque red everywhere
        let out = composite(&s, &src, w, h);
        assert_eq!(&out[0..4], &[200, 0, 0, 255], "clip shows over opaque base");
        assert_eq!(out[7], 0, "clip hidden where base is transparent");
    }

    #[test]
    fn consecutive_clipping_layers_share_one_base() {
        // T3.6: two clipping layers chain to the SAME base (the first
        // non-clipping raster below), not to each other.
        let (w, h) = (2, 1);
        let mut s = LayerStack::new();
        let base = s.add_raster("base", w, h).unwrap();
        let c1 = s.add_raster("c1", w, h).unwrap();
        let c2 = s.add_raster("c2", w, h).unwrap();
        s.get_mut(c1).unwrap().clipping = true;
        s.get_mut(c2).unwrap().clipping = true;
        let mut src = MapPixelSource::default();
        let mut bimg = LayerImage::transparent(w, h);
        bimg.rgba8[0..4].copy_from_slice(&[10, 10, 10, 255]); // left opaque
        bimg.rgba8[4..8].copy_from_slice(&[0, 0, 0, 0]); // right transparent
        src.insert(base, bimg);
        src.insert(c1, solid(w, h, [0, 0, 0, 0])); // c1 transparent (no cover)
        src.insert(c2, solid(w, h, [200, 0, 0, 255])); // c2 opaque red
        let out = composite(&s, &src, w, h);
        // c2 visible on the left proves it clips to the BASE, not to (transparent) c1.
        assert_eq!(
            &out[0..3],
            &[200, 0, 0],
            "c2 clips to the base → red on left"
        );
        assert_eq!(out[7], 0, "c2 hidden where the base is transparent");
    }

    #[test]
    fn dirty_rect_matches_full_recompose() {
        // layers_dirty_rect_correctness: recompositing a sub-rect equals
        // the same rect cropped from the full composite.
        let (w, h) = (4, 4);
        let mut s = LayerStack::new();
        let bottom = s.add_raster("bottom", w, h).unwrap();
        let top = s.add_raster("top", w, h).unwrap();
        s.set_blend_mode(top, BlendMode::Screen);
        s.set_opacity(top, 0.7);
        let mut src = MapPixelSource::default();
        // Vary pixels so a region crop is a real test (not uniform).
        let mut b = LayerImage::transparent(w, h);
        let mut t = LayerImage::transparent(w, h);
        for i in 0..(w * h) as usize {
            b.rgba8[i * 4] = (i * 11 % 256) as u8;
            b.rgba8[i * 4 + 1] = (i * 7 % 256) as u8;
            b.rgba8[i * 4 + 2] = (i * 5 % 256) as u8;
            b.rgba8[i * 4 + 3] = 255;
            t.rgba8[i * 4] = (i * 13 % 256) as u8;
            t.rgba8[i * 4 + 1] = (i * 3 % 256) as u8;
            t.rgba8[i * 4 + 2] = (i * 17 % 256) as u8;
            t.rgba8[i * 4 + 3] = 200;
        }
        src.insert(bottom, b);
        src.insert(top, t);

        let full = composite(&s, &src, w, h);
        let region = Region {
            x: 1,
            y: 1,
            w: 2,
            h: 2,
        };
        let part = composite_region(&s, &src, w, h, region);
        for ly in 0..region.h {
            for lx in 0..region.w {
                let gx = region.x + lx;
                let gy = region.y + ly;
                let fi = ((gy * w + gx) * 4) as usize;
                let pi = ((ly * region.w + lx) * 4) as usize;
                assert_eq!(
                    &full[fi..fi + 4],
                    &part[pi..pi + 4],
                    "dirty-rect pixel ({gx},{gy}) diverged from full recompose"
                );
            }
        }
    }

    #[test]
    fn dirty_rect_matches_full_with_group_and_blend() {
        // audit W3 F2: exercise the GROUP sub-window recursion in the dirty-rect
        // path (the flat-raster test above doesn't). A nested group with a
        // blended child must crop bit-for-bit too.
        let (w, h) = (4, 4);
        let mut s = LayerStack::new();
        let base = s.add_raster("base", w, h).unwrap();
        let group = s.add_group("group").unwrap();
        let child = s.add_raster("child", w, h).unwrap();
        s.move_into_group(child, group);
        s.set_blend_mode(child, BlendMode::Multiply);
        s.set_opacity(child, 0.6);
        s.set_blend_mode(group, BlendMode::Screen);
        s.set_opacity(group, 0.8);
        let mut src = MapPixelSource::default();
        let mut bimg = LayerImage::transparent(w, h);
        let mut cimg = LayerImage::transparent(w, h);
        for i in 0..(w * h) as usize {
            bimg.rgba8[i * 4] = (i * 11 % 256) as u8;
            bimg.rgba8[i * 4 + 1] = (i * 7 % 256) as u8;
            bimg.rgba8[i * 4 + 2] = (i * 5 % 256) as u8;
            bimg.rgba8[i * 4 + 3] = 255;
            cimg.rgba8[i * 4] = (i * 13 % 256) as u8;
            cimg.rgba8[i * 4 + 1] = (i * 3 % 256) as u8;
            cimg.rgba8[i * 4 + 2] = (i * 17 % 256) as u8;
            cimg.rgba8[i * 4 + 3] = 200;
        }
        src.insert(base, bimg);
        src.insert(child, cimg);

        let full = composite(&s, &src, w, h);
        let region = Region {
            x: 1,
            y: 0,
            w: 2,
            h: 3,
        };
        let part = composite_region(&s, &src, w, h, region);
        for ly in 0..region.h {
            for lx in 0..region.w {
                let gx = region.x + lx;
                let gy = region.y + ly;
                let fi = ((gy * w + gx) * 4) as usize;
                let pi = ((ly * region.w + lx) * 4) as usize;
                assert_eq!(
                    &full[fi..fi + 4],
                    &part[pi..pi + 4],
                    "group dirty-rect pixel ({gx},{gy}) diverged"
                );
            }
        }
    }

    #[test]
    fn adjustment_layer_noop_stub_is_identity() {
        // W4 T4.2: an adjustment layer composites end-to-end (the no-op
        // `apply_adjustment` stub leaves the layers below unchanged) — verifies
        // the LayerKind::Adjustment arm runs + the path is wired. A real arm
        // goes live the next frame once the implementer fills the compute.
        use ph2d_painter_brush::adjustments::AdjustmentKind;
        let (w, h) = (2, 2);
        let mut s = LayerStack::new();
        let base = s.add_raster("base", w, h).unwrap();
        let _adj = s
            .add_adjustment(AdjustmentKind::HueSaturationBrightness)
            .unwrap();
        let mut src = MapPixelSource::default();
        src.insert(base, solid(w, h, [120, 60, 200, 255]));
        let out = composite(&s, &src, w, h);
        for px in out.chunks_exact(4) {
            assert_eq!(
                &px[0..3],
                &[120, 60, 200],
                "no-op adjustment changed the pixel"
            );
            assert_eq!(px[3], 255);
        }
    }

    #[test]
    fn compositor_cache_skeleton_round_trips() {
        let mut c = CompositorCache::new();
        assert!(c.dirty_rect().is_none());
        c.mark_dirty(Region {
            x: 0,
            y: 0,
            w: 4,
            h: 4,
        });
        assert_eq!(c.dirty_rect().map(|r| r.w), Some(4));
        c.invalidate_from(LayerId(1), &LayerStack::new()); // skeleton: clears
    }

    #[test]
    #[ignore = "W4 soft perf gate (ADR-0045 §2.11): inline full-recompose is bandwidth-bound; W5 wires CompositorCache cut-points into composite + un-ignores (hard ≤1ms @4K)"]
    fn adjustment_layer_recomposition_perf_4k() {
        // Budget: slider-drag recompose @ 4K, 10 adjustment layers ≤ 1 ms.
        // Fleshed when the CompositorCache cut-point lands in the hot path.
    }
}
