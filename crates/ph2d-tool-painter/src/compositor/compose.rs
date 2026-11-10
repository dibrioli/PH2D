//! Compositing entry points + accumulator helpers, split out of the former `compositor.rs`. The
//! accumulator holds straight ENCODED channels (ADR-0177).

use super::*;

/// Composite the whole stack → canvas-sized straight sRGB8 RGBA.
#[must_use]
pub fn composite(
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
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
    let (acc, _) = composite_region_linear(stack, src, width, height, region, None);
    encode(&acc)
}

/// Composite only `region` → straight sRGB8 RGBA of size `region.w *
/// region.h`. Per-pixel compositing is spatially independent, so this is
/// bit-identical to cropping [`composite`] to the same rect — the basis of
/// dirty-rect recompose (`layers_dirty_rect_correctness`).
#[must_use]
pub fn composite_region(
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    width: u32,
    height: u32,
    region: Region,
) -> Vec<u8> {
    let (acc, _) = composite_region_linear(stack, src, width, height, region, None);
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
        None,
    );
    encode(&acc)
}

pub(super) fn encode(acc: &[[f32; 4]]) -> Vec<u8> {
    let mut out = vec![0u8; acc.len() * 4];
    // Pure per-pixel map → split across the cores for large buffers (bit-identical: disjoint chunks,
    // no cross-pixel state). Small buffers stay serial (thread spawn would cost more than it saves).
    let threads = parallel_threads(acc.len(), acc.len());
    let cpb = acc.len().div_ceil(threads);
    let encode_chunk = |out_chunk: &mut [u8], acc_chunk: &[[f32; 4]]| {
        for (px, chans) in acc_chunk.iter().enumerate() {
            for (k, &v) in chans.iter().enumerate() {
                out_chunk[px * 4 + k] = encode_byte(v);
            }
        }
    };
    if threads <= 1 {
        encode_chunk(&mut out, acc);
        return out;
    }
    std::thread::scope(|s| {
        let handles: Vec<_> = out
            .chunks_mut(cpb * 4)
            .zip(acc.chunks(cpb))
            .map(|(oc, lc)| s.spawn(move || encode_chunk(oc, lc)))
            .collect();
        for h in handles {
            h.join().expect("encode chunk panicked");
        }
    });
    out
}

/// How many worker threads a per-pixel pass of `area` px (with `rows` splittable units) should use:
/// `1` (serial) below the minimum area, else the core count clamped to the unit count.
pub(super) fn parallel_threads(area: usize, rows: usize) -> usize {
    /// Below this area the serial loop beats spawning scoped threads.
    const PARALLEL_MIN_AREA: usize = 1 << 17;
    if area < PARALLEL_MIN_AREA || rows <= 1 {
        return 1;
    }
    std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(1)
        .clamp(1, rows)
}

/// `true` when any VISIBLE adjustment in `ids` (recursing into groups) is a SPATIAL kind — its result
/// at a pixel depends on NEIGHBOURING pixels, so row-band compositing would seam at the band edges.
/// Per-pixel kinds (Curves / HSB / Levels / Noise / …) are band-safe.
pub(super) fn has_spatial_adjustment(stack: &LayerStack, ids: &[LayerId]) -> bool {
    ids.iter()
        .any(|&id| match stack.get(id).map(|l| (&l.kind, l.visible)) {
            Some((LayerKind::Group(g), true)) => has_spatial_adjustment(stack, &g.children),
            Some((LayerKind::Adjustment(a), true)) => a.visible && a.kind.reads_the_image_layout(),
            _ => false,
        })
}

/// Composite `region` into a freshly-allocated linear accumulator
/// (`region.w * region.h` entries, row-major). Clamps the region to the
/// canvas bounds.
/// Devolve o acumulador e, com `vidros`, a transparência por canal do papel (o fundo da pilha).
pub(super) fn composite_region_linear(
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    width: u32,
    height: u32,
    region: Region,
    vidros: Option<&super::vidro::Vidros>,
) -> (Vec<[f32; 4]>, Option<Vec<[f32; 3]>>) {
    let rx = region.x.min(width);
    let ry = region.y.min(height);
    let rw = region.w.min(width - rx);
    let rh = region.h.min(height - ry);
    let mut acc = vec![[0.0f32; 4]; (rw as usize) * (rh as usize)];
    // A transparência do papel nasce 1 (nada por cima): o enchimento em paralelo (`plane_copy`).
    let mut t = vidros.map(|_| {
        let mut t = Vec::new();
        crate::plane_copy::size_to(&mut t, acc.len(), [1.0f32; 3]);
        t
    });
    // Per-pixel compositing is spatially independent → split the region's rows into disjoint bands
    // across the cores (bit-identical to the serial walk; each band runs the same bottom→top layer
    // walk over its own rows). This was the painter's biggest per-frame cost on a non-trivial doc
    // stack: `take_preview_arc` recomposites the shape methods' whole-shape dirty bbox EVERY move
    // (~31 ms serial @2048², Enio's "FPS 60→10"). SPATIAL adjustments (blur/bloom/…) read
    // neighbouring pixels — banding would seam at band edges — so those stacks stay serial (they are
    // already clamp-to-edge approximate on a sub-window; the GPU pass-graph is their real-time path).
    let threads = parallel_threads(
        (rw as usize) * (rh as usize),
        if has_spatial_adjustment(stack, stack.root()) {
            1
        } else {
            rh as usize
        },
    );
    if threads <= 1 {
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
            vidros.zip(t.as_deref_mut()),
        );
        return (acc, t);
    }
    let rpb = (rh as usize).div_ceil(threads);
    let band_px = rpb * (rw as usize);
    let mut t_bands: Vec<Option<&mut [[f32; 3]]>> = match t.as_deref_mut() {
        Some(tt) => tt.chunks_mut(band_px).map(Some).collect(),
        None => (0..acc.len().div_ceil(band_px)).map(|_| None).collect(),
    };
    std::thread::scope(|s| {
        let handles: Vec<_> = acc
            .chunks_mut(band_px)
            .zip(t_bands.iter_mut())
            .enumerate()
            .map(|(bi, (band, tb))| {
                let band_ry = ry + (bi * rpb) as u32;
                let band_rh = (band.len() / (rw as usize)) as u32;
                let tb = tb.take();
                s.spawn(move || {
                    composite_into(
                        band,
                        stack.root(),
                        stack,
                        src,
                        width,
                        rx,
                        band_ry,
                        rw,
                        band_rh,
                        0,
                        None,
                        vidros.zip(tb),
                    );
                })
            })
            .collect();
        for h in handles {
            h.join().expect("composite band panicked");
        }
    });
    drop(t_bands);
    (acc, t)
}

/// Blend the layers in `ids` (top-to-bottom) over `acc`, restricted to the
/// `rw * rh` window anchored at canvas `(rx, ry)`. `canvas_w` is the full
/// canvas stride (layer images are canvas-sized).
#[allow(clippy::too_many_arguments)]
pub(super) fn composite_into(
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
    mut vidro: Option<(&super::vidro::Vidros, &mut [[f32; 3]])>,
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
    // B19.1: o elo do recorte é da ESTRUTURA, não da visibilidade. Base escondida ⇒ os recortados
    // somem (`clip_hidden`); base a opacidade 0 continua a ser a base; grupo/ajuste escondido quebra o elo.
    let mut clip_hidden = false;
    let need = if rw == 0 || rh == 0 {
        4
    } else {
        (((ry + rh - 1) * canvas_w + (rx + rw - 1)) as usize + 1) * 4
    };
    for &id in ids.iter().rev() {
        let Some(layer) = stack.get(id) else { continue };
        if !layer.visible || layer.opacity <= 0.0 {
            match &layer.kind {
                LayerKind::Raster(_) | LayerKind::Texture(_) if !layer.clipping => {
                    if !layer.visible {
                        clip_hidden = true;
                    } else if let Some(rgba) = src.layer_rgba(id).filter(|r| r.len() >= need) {
                        (clip_base, clip_hidden) = (Some(rgba), false);
                    }
                }
                LayerKind::Group(_) | LayerKind::Adjustment(_) => {
                    (clip_base, clip_hidden) = (None, false);
                }
                _ => {}
            }
            continue;
        }
        let opacity = layer.opacity.clamp(0.0, 1.0);
        let mode = layer.blend_mode;
        match &layer.kind {
            // A Texture layer is raster-backed (its pixels are pre-rendered into the source's
            // `images[id]`), so it blends through the exact same path as a painted raster — mask,
            // clipping, blend, opacity, and clip-base all apply identically.
            LayerKind::Raster(_) | LayerKind::Texture(_) => {
                if layer.clipping && clip_hidden {
                    continue;
                }
                let Some(rgba) = src.layer_rgba(id) else {
                    continue;
                };
                // Bounds guard: the highest texel index this window reads is
                // the bottom-right corner. Skip a too-short buffer rather than
                // panic-index (defense vs a mismatched/forged provider).
                if rgba.len() < need {
                    continue;
                }
                // T3.5: an attached grayscale mask multiplies this layer's alpha
                // (white = visible, black = hidden; `1 - value` when inverted).
                // Mask pixels are canvas-sized RGBA8 in the same source; a
                // missing/short mask buffer is treated as "no mask" (no panic).
                let mask = layer.mask.and_then(|mid| match &stack.get(mid)?.kind {
                    LayerKind::Mask(m) => {
                        let mrgba = src.layer_rgba(mid)?;
                        (mrgba.len() >= need).then_some((mrgba, m.inverted))
                    }
                    _ => None,
                });
                // T3.6: a clipping layer paints only where its clip base is
                // opaque — multiply its alpha by the base's straight alpha.
                let clip = if layer.clipping { clip_base } else { None };
                let sample = |gx: u32, gy: u32| {
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
                };
                // O MESMO factor (máscara × recorte), à parte: o vidro precisa dele num texel de alfa 0.
                let fator = |idx: usize| {
                    let m = mask.map_or(1.0, |(mrgba, inverted)| {
                        let v = mask_value(mrgba, idx);
                        if inverted { 1.0 - v } else { v }
                    });
                    m * clip.map_or(1.0, |base| base[idx * 4 + 3] as f32 / 255.0)
                };
                blend_window(acc, rx, ry, rw, rh, mode, opacity, sample);
                // O papel atravessa esta camada canal a canal (o vidro só no modo Normal: os outros
                // modos não são um filtro sobre o que está por baixo).
                if let Some((planos, t)) = vidro.as_mut() {
                    let plano = planos
                        .get(&id)
                        .filter(|p| mode == BlendMode::Normal && p.len() * 4 >= rgba.len());
                    for ly in 0..rh {
                        for lx in 0..rw {
                            let (gx, gy) = (rx + lx, ry + ly);
                            let idx = (gy * canvas_w + gx) as usize;
                            super::vidro::atravessa(
                                &mut t[(ly * rw + lx) as usize],
                                fator(idx) * opacity,
                                &rgba[idx * 4..idx * 4 + 4],
                                plano.map(|p| &p[idx]),
                            );
                        }
                    }
                }
                // A NON-clipping raster becomes the clip base for the layers
                // above it; a clipping raster chains to the same base.
                if !layer.clipping {
                    (clip_base, clip_hidden) = (Some(rgba), false);
                }
            }
            LayerKind::Group(g) => {
                // Composite the children into their own sub-window, then
                // blend that as a single layer (group blend/opacity).
                let mut sub = vec![[0.0f32; 4]; (rw as usize) * (rh as usize)];
                let mut sub_t = vidro.as_ref().map(|_| vec![[1.0f32; 3]; sub.len()]);
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
                    vidro.as_ref().map(|v| v.0).zip(sub_t.as_deref_mut()),
                );
                blend_window(acc, rx, ry, rw, rh, mode, opacity, |gx, gy| {
                    let lx = gx - rx;
                    let ly = gy - ry;
                    sub[(ly * rw + lx) as usize]
                });
                // O grupo inteiro é uma camada: no modo Normal cada canal tira o alfa do canal dele.
                if let (Some((_, t)), Some(st)) = (vidro.as_mut(), sub_t.as_ref()) {
                    for (i, tc) in t.iter_mut().enumerate() {
                        for c in 0..3 {
                            let a = if mode == BlendMode::Normal {
                                1.0 - st[i][c]
                            } else {
                                sub[i][3]
                            };
                            tc[c] *= 1.0 - a * opacity;
                        }
                    }
                }
                // A group is not a raster clip base — it breaks the clip chain.
                (clip_base, clip_hidden) = (None, false);
            }
            // W4 (ADR-0045 §2.7, CPU-first): a non-destructive adjustment
            // transforms the layers BELOW it — already in `acc` (we walk
            // bottom-up). Copy the window, run the kind's compute, then blend
            // the result back over `acc` by the adjustment's OWN opacity × mask
            // in its blend mode (inner fields authoritative, amendment-1).
            // Each kind receives the ENCODED accumulator and converts at its own
            // boundary if it is defined in light (ADR-0177); the blend-back runs
            // in the accumulator's space.
            // Mask/opacity live HERE, not in the compute hook (W4-triage).
            LayerKind::Adjustment(adj) => {
                if !adj.visible || adj.opacity <= 0.0 {
                    (clip_base, clip_hidden) = (None, false);
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
                let mut adjusted: Vec<[f32; 4]> = acc.to_vec();
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
                            // A cobertura espalhou-se: a ABSORÇÃO de cada canal segue-a na proporção do
                            // alfa (escalar a transparência passaria de 1 num canal que o vidro deixa
                            // passar — pontos escuros na borda desfocada).
                            if let Some((_, tp)) = vidro.as_mut() {
                                let (a0, a1) = (base[3], acc[i][3]);
                                for tc in tp[i].iter_mut() {
                                    *tc = if a0 > 1e-4 {
                                        (1.0 - (1.0 - *tc) * a1 / a0).clamp(0.0, 1.0)
                                    } else {
                                        tc.min(1.0 - a1)
                                    };
                                }
                            }
                        } else {
                            // Per-pixel colour adjustment: it changes the COLOUR the
                            // pixel has, never its coverage — so the mode mixes the two
                            // colours as if both were opaque (an `over` carrying the
                            // base's alpha applied only part of a 100 % adjustment on a
                            // translucent pixel: 79 steps off the GIMP oracle,
                            // `oraculo_ajustes_tests`), lerp by t, coverage KEPT.
                            let opaco = |p: [f32; 4]| [p[0], p[1], p[2], 1.0];
                            let blended = apply_blend(adj_mode, opaco(base), opaco(adjusted[i]));
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
                (clip_base, clip_hidden) = (None, false);
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
    let salta = mode == BlendMode::Normal && salto_ligado();
    for ly in 0..rh {
        for lx in 0..rw {
            let mut s = sample(rx + lx, ry + ly);
            s[3] *= opacity;
            let i = (ly * rw + lx) as usize;
            if salta && s[3] <= 0.0 && normal_transparente_e_identidade(acc[i], s) {
                continue;
            }
            acc[i] = apply_blend(mode, acc[i], s);
        }
    }
}

/// B19.10/B19.11: a `Normal` source with α ≤ 0 leaves `dst` **to the bit** over an opaque normalised
/// pixel (each channel in `[+0, 1]`: not −0, NaN nor out of range — the bit compare does all three)
/// with a finite source colour, or over `[+0; 4]`. Gate `o_salto_alfa_zero_e_o_apply_ao_bit`.
#[inline]
fn normal_transparente_e_identidade(d: [f32; 4], s: [f32; 4]) -> bool {
    let norm = |c: f32| c.to_bits() <= 1.0f32.to_bits();
    (d[3].to_bits() == 1.0f32.to_bits()
        && norm(d[0])
        && norm(d[1])
        && norm(d[2])
        && s[0].is_finite()
        && s[1].is_finite()
        && s[2].is_finite())
        || d.map(f32::to_bits) == [0; 4]
}

/// The α = 0 skip, off only for the gate's and the probe's same-process A/B.
#[cfg(not(test))]
#[inline]
fn salto_ligado() -> bool {
    true
}
#[cfg(test)]
pub(super) static SEM_SALTO: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
#[cfg(test)]
fn salto_ligado() -> bool {
    !SEM_SALTO.load(std::sync::atomic::Ordering::Relaxed)
}

#[cfg(test)]
mod salto_tests;
#[cfg(test)]
mod recorte_tests;
