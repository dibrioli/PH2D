//! Compositing entry points + linear-accumulator helpers, split out of the
//! former `compositor.rs` (pure mechanical move).

use super::*;

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

/// Composite the FULL canvas with the cut-point cache (ADR-0045 §2.7) — the
/// slider-drag FPS lever. On a param-only change of a root adjustment (after
/// `cache.invalidate_above(adj, stack)`), this restarts from that adjustment's
/// cached accumulator-below instead of recomposing the layers underneath; on a
/// cold cache (or structural change → `invalidate_from`) it composes the whole
/// stack and (re)fills the cuts. **Bit-identical to [`composite`]** — the layers
/// below the restart point are unchanged, and an adjustment resets `clip_base`, so
/// the restart state matches the full walk (gate `cache_matches_full_recompose`).
#[must_use]
pub fn composite_with_cache(
    stack: &LayerStack,
    src: &impl LayerPixelSource,
    width: u32,
    height: u32,
    cache: &mut CompositorCache,
) -> Vec<u8> {
    let root = stack.root();
    // Highest root adjustment (smallest index = MOST below-layers cached) whose
    // cut is still valid. Its cut is the composite of everything below it.
    let start = root.iter().enumerate().find(|&(_, &id)| {
        matches!(
            stack.get(id).map(|l| &l.kind),
            Some(LayerKind::Adjustment(_))
        ) && cache.cuts.contains_key(&id)
    });
    let (mut acc, ids): (Vec<[f32; 4]>, &[LayerId]) = match start {
        // `composite_into` walks `ids` REVERSED (panel order is top-first;
        // index 0 = topmost). So the cut at panel index `i` = composite of the
        // layers BELOW it (`root[i+1..]`). Seed `acc` with that cut, then hand
        // `root[..=i]` (the adjustment + everything above): the reversed walk
        // processes the adjustment FIRST (on the correct below-acc), then the
        // layers above it. Restarting from the smallest valid index reuses the
        // largest cut (most below-layers cached) → fewest layers recomposed.
        Some((i, &id)) => (cache.cuts[&id].clone(), &root[..=i]),
        None => (
            vec![[0.0f32; 4]; (width as usize) * (height as usize)],
            root,
        ),
    };
    composite_into(
        &mut acc,
        ids,
        stack,
        src,
        width,
        0,
        0,
        width,
        height,
        0,
        Some(cache),
    );
    encode(&acc)
}

fn encode(acc: &[[f32; 4]]) -> Vec<u8> {
    // Force each LazyLock once per composite, not per pixel.
    let thresh = &*SRGB_ENCODE_THRESH;
    let coarse = &*SRGB_ENCODE_COARSE;
    let mut out = vec![0u8; acc.len() * 4];
    for (px, lin) in acc.iter().enumerate() {
        out[px * 4] = encode_byte(thresh, coarse, lin[0]);
        out[px * 4 + 1] = encode_byte(thresh, coarse, lin[1]);
        out[px * 4 + 2] = encode_byte(thresh, coarse, lin[2]);
        // Alpha is straight coverage — no transfer function (round, not LUT).
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
    composite_into(
        &mut acc,
        stack.root(),
        stack,
        src,
        width,
        rx,
        ry,
        rw,
        rh,
        0,
        None,
    );
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
    mut cache: Option<&mut CompositorCache>,
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
            // A Texture layer is raster-backed (its pixels are pre-rendered into the source's
            // `images[id]`), so it blends through the exact same path as a painted raster — mask,
            // clipping, blend, opacity, and clip-base all apply identically.
            LayerKind::Raster(_) | LayerKind::Texture(_) => {
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
                // Skip THIS layer's mask multiply while that mask is the active edit target — the layer
                // stays fully visible so the artist masks against the image (non-destructive quick-mask;
                // the on-canvas colour film shows the coverage). Normal compositing applies it otherwise.
                let mask = layer
                    .mask
                    .filter(|&mid| src.active_layer() != Some(mid))
                    .and_then(|mid| match &stack.get(mid)?.kind {
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
                    None,
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
                // W5 cut-point cache (ADR-0045 §2.7): at the ROOT, snapshot the
                // accumulator BELOW this adjustment (the composite of everything
                // below it) keyed by its id, so a later param-only change can
                // restart from here (`composite_with_cache`) instead of recomposing
                // the whole stack. Stored BEFORE applying the adjustment. Only
                // depth-0 adjustments cache; group-internal ones recompose in their
                // (smaller) sub-buffer. An adjustment resets `clip_base` below, so
                // restarting here is bit-identical to the full walk.
                if depth == 0
                    && let Some(c) = cache.as_deref_mut()
                {
                    c.cuts.insert(id, acc.to_vec());
                }
                let adj_opacity = adj.opacity.clamp(0.0, 1.0);
                let adj_mode = adj.blend_mode;
                let mut adjusted = acc.to_vec();
                // Window-aware dispatch: spatial blurs (Gaussian/Sharpen/Motion/
                // Chroma) need the 2-D layout; Noise/Halftone need the absolute
                // canvas coordinate (origin + local). Per-pixel kinds delegate to
                // the flat `apply_adjustment` inside. (Spatial blur on a dirty-rect
                // sub-window is clamp-to-edge approximate at the seam — the GPU
                // pass-graph with halo is the exact real-time path; a full-canvas
                // recompose here, the common param-drag case, is exact.)
                ph2d_painter_effects::adjustments::apply_adjustment_windowed(
                    &adj.kind,
                    &adj.params,
                    &mut adjusted,
                    ph2d_painter_effects::adjustments::AdjustWindow {
                        width: rw,
                        height: rh,
                        origin_x: rx,
                        origin_y: ry,
                    },
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
                // A COVERAGE-FEATHERING kind (the blur family + Bloom) produces a
                // NEW version of the below-composite with FEATHERED alpha (it ran
                // premultiplied, so the coverage spread outward). Its combine must
                // therefore adopt the adjusted alpha (a blur of a transparent layer
                // softens its silhouette; Bloom haloes outward), unlike a per-pixel
                // or tonal adjustment (incl. Shadows/Highlights), which keeps the
                // base coverage. Lock-step with the GPU pass-graph combine.
                let is_spatial = adj.kind.feathers_coverage();
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
                        if is_spatial {
                            // Replace (Normal) or blend the filtered RGBA over the
                            // base, then lerp ALL 4 channels by t — so opacity/mask
                            // scale the effect AND its new coverage. A neutral
                            // (radius 0) kernel left `adjusted == base`, so this is
                            // an exact identity.
                            let result = if adj_mode == BlendMode::Normal {
                                adjusted[i]
                            } else {
                                apply_blend(adj_mode, base, adjusted[i])
                            };
                            acc[i] = [
                                base[0] + (result[0] - base[0]) * t,
                                base[1] + (result[1] - base[1]) * t,
                                base[2] + (result[2] - base[2]) * t,
                                base[3] + (result[3] - base[3]) * t,
                            ];
                        } else {
                            // Per-pixel colour adjustment: blend the adjusted color
                            // (carrying the base's coverage) over the base, lerp by
                            // t; coverage is KEPT (adjustments don't change alpha).
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
                }
                // An adjustment is not a raster clip base — it breaks the chain.
                clip_base = None;
            }
            // Mask layers composite via their parent (T3.5); skip standalone.
            LayerKind::Mask(_) => continue,
        }
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
