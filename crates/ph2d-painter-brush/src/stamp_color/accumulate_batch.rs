//! The **batched, row-banded** accumulate kernels — one call per pointer-Move instead of one per dab,
//! so the dab loop splits into disjoint row bands across the cores (bit-identical: each canvas pixel
//! belongs to exactly ONE band and every band walks the dabs in the serial order, so the per-pixel
//! `over`-accumulate sequence is exactly the serial kernels\' — the parity gates in `tests.rs` prove it).
//! Sibling of [`super::accumulate`] (the single-dab kernels these fall back to below the parallel
//! threshold); split out for the workspace file-LOC cap.

use super::accumulate::{accumulate_shape_layer_rgba, pinta_alfa};
use super::bandas::{Caixa, Lote, bandas_com_toque, caixas_do_carimbo, union_rect};
use super::{ColorStampMask, grain_modulation};
use crate::dab::{DirtyRect, encode};
use crate::spec::BrushSpec;
use crate::texture::{ImageMask, ImageRgb, sample_shape_rgb_unit};

/// One dab of a fused **batch** accumulate: the dab's centre/radius plus its already-folded
/// Flow × Strength × pressure coverage.
#[derive(Copy, Clone, Debug)]
pub struct FusedDab {
    pub center: [f32; 2],
    pub radius: f32,
    pub coverage: f32,
}

/// **Batched + row-banded** fused accumulate — one call per Move instead of one per dab, so the dab
/// loop can be split into disjoint row bands across the cores. Byte-for-byte identical to calling
/// [`accumulate_color_stamps_fused`] once per dab in order: each canvas pixel belongs to exactly ONE
/// band, and every band walks the dabs in the same order, so the per-pixel `over`-accumulate sequence
/// is exactly the serial one (HR-5; gate `batched_fused_accumulate_is_bit_identical_to_sequential`).
/// Returns the union of the touched per-dab rects (same union the per-dab loop produced). Small
/// batches stay serial via the per-dab kernel, so the thread spawn never costs more than it saves.
#[must_use]
pub fn accumulate_color_stamps_fused_batch(
    covs: &mut [Vec<u8>],
    width: u32,
    height: u32,
    dabs: &[FusedDab],
    stamps: &[ColorStampMask],
) -> Option<DirtyRect> {
    let n = stamps.len();
    if width == 0 || height == 0 || n == 0 || dabs.is_empty() {
        return None;
    }
    // Contract guard (sweep 2026-07-12): a mis-shaped buffer bails instead of indexing past the end, in the
    // artist's build too (`mis_shaped_maps_bail_instead_of_indexing_past_the_end`).
    let len = (width as usize) * (height as usize);
    if covs.len() != n || covs.iter().any(|c| c.len() != len) {
        return None;
    }
    let boxes = caixas_do_carimbo(dabs, stamps, width, height);
    let lote = Lote::de(&boxes)?;
    let threads = lote.faixas();
    let wus = width as usize;
    // Small batches: the per-dab serial kernel's body (identical bytes by construction).
    if threads <= 1 {
        let mut union: Option<DirtyRect> = None;
        for b in &boxes {
            if pinta_alfa(covs, wus, 0, b, b.y0, b.y1) {
                let r = DirtyRect {
                    x: b.x0 as u32,
                    y: b.y0 as u32,
                    w: (b.x1 - b.x0) as u32,
                    h: (b.y1 - b.y0) as u32,
                };
                union = Some(union.map_or(r, |u| union_rect(u, r)));
            }
        }
        return union;
    }
    bandas_com_toque(
        covs,
        width,
        1,
        &boxes,
        dabs.len(),
        &lote,
        threads,
        "fused accumulate band panicked",
        |maps, band_y0, band_y1, touched| {
            for b in &boxes {
                if pinta_alfa(maps, wus, band_y0, b, b.y0.max(band_y0), b.y1.min(band_y1)) {
                    touched[b.di] = true;
                }
            }
        },
    )
}

/// One dab of the DYNAMIC per-layer batch ([`accumulate_shape_layers_rgba_batch`]) — everything the
/// per-dab serial loop resolved before its pixel work: the dab-local Shape / Grain bases (drawn from the
/// tex RNG **in dab order** by the caller, so the stream is byte-identical to the serial loop), the
/// per-layer colours with the dab's Randomize-Color shift folded in, and the folded coverage.
pub struct DynDab {
    pub center: [f32; 2],
    pub radius: f32,
    pub spec: BrushSpec,
    pub shape_basis: crate::texture::TexDabBasis,
    pub grain_basis: Option<crate::texture::TexDabBasis>,
    /// Per layer: the FINAL flat colour of a layer without texture colour (its Randomize offset already
    /// applied by the caller), or the TINT of a texture-colour layer (× its texel, then `color_shift`).
    pub colors: Vec<[f32; 3]>,
    /// The dab's drawn Randomize-Color offset ([`crate::Dab::color_shift`]), applied per pixel to the
    /// texture-colour layers only.
    pub color_shift: [f32; 3],
    pub coverage: f32,
}

/// **Batched + row-banded + layer-fused** dynamic accumulate — the per-layer-colour TEXTURE-COLOUR path
/// (the capture default), which was the live "FPS 60→10" cliff: `accumulate_shape_layer_rgba` ran
/// serially per dab × per layer (measured 354 ms/move N3 · 1.87 s/move N16 @2048² r100). One call per
/// Move: the dab loop splits into disjoint row bands across the cores, and within a pixel the layers
/// share the dab's (u,v) + the Grain sample + the Stencil gate (identical for every layer — computed
/// once, lazily, instead of ×N). Byte-identical to the serial per-dab loop: each pixel belongs to ONE
/// band, bands walk dabs (then layers) in the serial order, and every per-(layer,pixel) operation is the
/// same arithmetic (gate `batched_dynamic_accumulate_is_bit_identical_to_sequential`). Returns the union
/// of the touched per-dab rects.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn accumulate_shape_layers_rgba_batch(
    accs: &mut [Vec<u8>],
    width: u32,
    height: u32,
    dabs: &[DynDab],
    layers: &[ImageMask<'_>],
    layer_rgb: &[Option<ImageRgb<'_>>],
    grain_image: Option<&ImageMask<'_>>,
    grain_ramp: Option<&[[f32; 4]]>,
) -> Option<DirtyRect> {
    let n = layers.len();
    if width == 0 || height == 0 || n == 0 || dabs.is_empty() {
        return None;
    }
    let len4 = (width as usize) * (height as usize) * 4;
    if accs.len() != n || accs.iter().any(|a| a.len() != len4) {
        return None;
    }
    struct Box_ {
        di: usize,
        x0: i64,
        y0: i64,
        x1: i64,
        y1: i64,
        cx: f32,
        cy: f32,
        inv_radius: f32,
        coverage: f32,
    }
    impl Caixa for Box_ {
        fn di(&self) -> usize {
            self.di
        }
        fn caixa(&self) -> [i64; 4] {
            [self.x0, self.y0, self.x1, self.y1]
        }
    }
    let mut boxes: Vec<Box_> = Vec::with_capacity(dabs.len());
    for (di, d) in dabs.iter().enumerate() {
        let coverage = d.coverage.clamp(0.0, 1.0);
        if coverage <= 0.0 || d.radius <= 0.0 {
            continue;
        }
        let (cx, cy) = (d.center[0], d.center[1]);
        // A caixa cobre a ponta de imagem inteira ([`crate::tip::dab_extent`], Bug #52).
        let [ex, ey] = crate::tip::dab_extent([1.0, 1.0], &d.spec.shape, Some(&d.shape_basis));
        let (rx, ry) = (d.radius * ex, d.radius * ey);
        let x0 = (cx - rx).floor().max(0.0) as i64;
        let y0 = (cy - ry).floor().max(0.0) as i64;
        let x1 = ((cx + rx).ceil() as i64 + 1).min(width as i64);
        let y1 = ((cy + ry).ceil() as i64 + 1).min(height as i64);
        if x0 >= x1 || y0 >= y1 {
            continue;
        }
        boxes.push(Box_ {
            di,
            x0,
            y0,
            x1,
            y1,
            cx,
            cy,
            inv_radius: 1.0 / d.radius,
            coverage,
        });
    }
    // Small batches: the per-dab serial kernel — the byte-identity baseline the parity gate compares
    // the banded path against; spawning threads there would cost more than it saves.
    let lote = Lote::de(&boxes)?;
    if lote.faixas() <= 1 {
        let mut union: Option<DirtyRect> = None;
        for b in &boxes {
            let d = &dabs[b.di];
            for (i, layer) in layers.iter().enumerate() {
                let grain = d.grain_basis.as_ref().map(|gb| (gb, grain_image));
                if let Some(r) = accumulate_shape_layer_rgba(
                    &mut accs[i],
                    width,
                    height,
                    d.center,
                    d.radius,
                    &d.spec,
                    &d.shape_basis,
                    layer,
                    grain,
                    grain_ramp,
                    d.colors[i],
                    layer_rgb.get(i).and_then(|o| o.as_ref()),
                    d.color_shift,
                    d.coverage,
                ) {
                    union = Some(union.map_or(r, |u| union_rect(u, r)));
                }
            }
        }
        return union;
    }
    let threads = lote.faixas();
    // A memória do grão Tiled ([`crate::texture::grao_cache`], B29.A3), pedida em ordem de dab.
    let graos: Vec<_> = boxes
        .iter()
        .map(|b| {
            let d = &dabs[b.di];
            d.grain_basis.as_ref().and_then(|gb| {
                crate::texture::grao_cache::para_o_dab(&d.spec.texture, gb, width, height)
            })
        })
        .collect();
    // E a do `t` do dente do papel (B26.A2).
    let dentes: Vec<_> = boxes
        .iter()
        .map(|b| {
            dabs[b.di].spec.dente.as_ref().and_then(|dt| {
                crate::texture::grao_cache::para_o_dente(dt, width, height)
            })
        })
        .collect();
    let wus = width as usize;
    bandas_com_toque(
        accs,
        width,
        4,
        &boxes,
        dabs.len(),
        &lote,
        threads,
        "dynamic accumulate band panicked",
        |maps, band_y0, band_y1, touched| {
            for ((b, grao), dente) in boxes.iter().zip(&graos).zip(&dentes) {
                let d = &dabs[b.di];
                let depth = d.spec.grain_depth();
                let y0 = b.y0.max(band_y0);
                let y1 = b.y1.min(band_y1);
                for py in y0..y1 {
                    let v = ((py as f32 + 0.5) - b.cy) * b.inv_radius;
                    let row = ((py - band_y0) as usize) * wus;
                    for px in b.x0..b.x1 {
                        let u = ((px as f32 + 0.5) - b.cx) * b.inv_radius;
                        // The Grain modulation + Stencil gate are IDENTICAL for every layer at
                        // this pixel (basis + coord only) — computed once, lazily (the serial
                        // loop skipped them whenever the layer silhouette was 0).
                        let mut grain_mg: Option<([f32; 4], f32)> = None;
                        for (i, layer) in layers.iter().enumerate() {
                            let mut a = crate::texture::sample_shape_silhouette_unit_px(
                                &d.spec.shape,
                                &d.shape_basis,
                                u,
                                v,
                                b.inv_radius,
                                Some(layer),
                            );
                            if a <= 0.0 {
                                continue;
                            }
                            let base = match layer_rgb.get(i).and_then(|o| o.as_ref()) {
                                Some(img) => super::texture_times_tint(
                                    sample_shape_rgb_unit(&d.spec.shape, &d.shape_basis, u, v, img),
                                    d.colors[i],
                                    d.color_shift,
                                ),
                                None => d.colors[i],
                            };
                            let mut tint = base;
                            if let Some(gb) = d.grain_basis.as_ref() {
                                let (m, gate) = *grain_mg.get_or_insert_with(|| {
                                    let g = if d.spec.texture.mapping.is_canvas_fixed() {
                                        let amostra = || {
                                            #[cfg(test)]
                                            super::bandas::sonda::conta(width);
                                            crate::texture::sample(
                                                &d.spec.texture,
                                                gb,
                                                px,
                                                py,
                                                d.center,
                                                d.radius,
                                                grain_image,
                                            )
                                        };
                                        match grao.as_deref() {
                                            Some(c) => c.em(px, py, amostra),
                                            None => amostra(),
                                        }
                                    } else {
                                        crate::texture::sample_unit_px(
                                            &d.spec.texture,
                                            gb,
                                            u,
                                            v,
                                            b.inv_radius,
                                            grain_image,
                                        )
                                    };
                                    (
                                        grain_modulation(
                                            g,
                                            depth,
                                            d.spec.effective_granulation(),
                                            grain_ramp,
                                            &d.spec.texture.lei,
                                        ),
                                        crate::texture::stencil_gate(&d.spec.texture, gb, px, py),
                                    )
                                });
                                tint = [base[0] * m[0], base[1] * m[1], base[2] * m[2]];
                                a *= m[3];
                                a *= gate;
                            }
                            a *= crate::dente::dente_com(&d.spec, dente.as_deref(), a, px, py);
                            a *= b.coverage;
                            if a <= 0.0 {
                                continue;
                            }
                            let acc = &mut maps[i];
                            let o = (row + px as usize) * 4;
                            let inv = 1.0 - a;
                            acc[o] = encode(tint[0] * a + f32::from(acc[o]) / 255.0 * inv);
                            acc[o + 1] = encode(tint[1] * a + f32::from(acc[o + 1]) / 255.0 * inv);
                            acc[o + 2] = encode(tint[2] * a + f32::from(acc[o + 2]) / 255.0 * inv);
                            acc[o + 3] = encode(a + f32::from(acc[o + 3]) / 255.0 * inv);
                            touched[b.di] = true;
                        }
                    }
                }
            }
        },
    )
}
