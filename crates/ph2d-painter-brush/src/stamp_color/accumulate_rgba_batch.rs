//! The **batched + row-banded fused RGBA** accumulate (the texture-colour cached path) — sibling of
//! [`super::accumulate_batch`], split out for the workspace file-LOC cap (pure move).

use super::ColorStampMask;
use super::accumulate_batch::FusedDab;
use super::bandas::{CaixaCarimbo, Lote, bandas_com_toque, caixas_do_carimbo};
use crate::dab::{DirtyRect, encode};

/// **Batched + row-banded fused RGBA** accumulate — the constant-orientation **texture-colour** cached
/// path: each layer's stamp is pre-BAKED with its per-pixel colour (premultiplied RGBA,
/// `render_color_stamp_mask` with the layer's captured RGB), so the per-canvas-pixel work collapses to
/// one shared bilinear coordinate + N four-channel premul samples — instead of the dynamic kernel's
/// per-layer silhouette + RGB resampling (~7× per px). Same structure as
/// [`super::accumulate_color_stamps_fused_batch`]; the per-(layer,pixel) accumulate matches
/// [`super::accumulate_shape_layer_rgba`]'s `over` (`c·a_cov + prev·(1−a·cov)` in premul form). NOT
/// byte-identical to the dynamic path (the stamp is quantised at the mask resolution — the same
/// trade the flat cached path already makes); it IS bit-identical across band counts (each pixel
/// belongs to one band; dab order preserved). Layers of different sides each read their own level (B29.A2).
#[must_use]
pub fn accumulate_color_stamps_rgba_batch(
    accs: &mut [Vec<u8>],
    width: u32,
    height: u32,
    dabs: &[FusedDab],
    stamps: &[ColorStampMask],
) -> Option<DirtyRect> {
    let n = stamps.len();
    if width == 0 || height == 0 || n == 0 || dabs.is_empty() {
        return None;
    }
    let len4 = (width as usize) * (height as usize) * 4;
    if accs.len() != n || accs.iter().any(|a| a.len() != len4) {
        return None;
    }
    let boxes = caixas_do_carimbo(dabs, stamps, width, height);
    let lote = Lote::de(&boxes)?;
    // ⚠️ Este sítio NUNCA teve piso — ele dividia sempre, e por isso pagava 32 spawns para um lote de
    // um dab pequeno. A porta única lhe dá o piso e a contagem de uma vez.
    let threads = lote.faixas();
    let wus = width as usize;
    bandas_com_toque(
        accs,
        width,
        4,
        &boxes,
        dabs.len(),
        &lote,
        threads,
        |maps, band_y0, band_y1, touched| {
            for b in &boxes {
                if pinta_rgba(maps, wus, band_y0, b, b.y0.max(band_y0), b.y1.min(band_y1)) {
                    touched[b.di] = true;
                }
            }
        },
    )
}

/// O corpo por píxel do acumulado RGBA premultiplicado das camadas assadas sobre as linhas `[y0, y1)` da caixa
/// `b`: as coordenadas bilineares partilham-se por grupo de camadas do mesmo lado. Diz se tocou.
fn pinta_rgba(
    maps: &mut [&mut [u8]],
    wus: usize,
    band_y0: i64,
    b: &CaixaCarimbo<'_>,
    y0: i64,
    y1: i64,
) -> bool {
    let mut touched = false;
    for grupo in &b.grupos {
        let size = grupo[0].1.size;
        let sz = size as usize;
        for py in y0..y1 {
            let v = ((py as f32 + 0.5) - b.cy) * b.inv_radius;
            // Fora da ponta não há dab ([`crate::tip::axis_taps`]).
            let Some((ty0, ty1, ty)) = crate::tip::axis_taps(v, size) else {
                continue;
            };
            let (yr0, yr1) = (ty0 * sz, ty1 * sz);
            let row = ((py - band_y0) as usize) * wus;
            for px in b.x0..b.x1 {
                let u = ((px as f32 + 0.5) - b.cx) * b.inv_radius;
                let Some((xc0, xc1, tx)) = crate::tip::axis_taps(u, size) else {
                    continue;
                };
                // The four texel byte offsets, shared by every layer of the group.
                let o00 = (yr0 + xc0) * 4;
                let o10 = (yr0 + xc1) * 4;
                let o01 = (yr1 + xc0) * 4;
                let o11 = (yr1 + xc1) * 4;
                let o = (row + px as usize) * 4;
                for &(i, nivel) in grupo {
                    let d = &nivel.data;
                    // Premultiplied 4-channel bilinear (shared coords).
                    let ch = |c: usize| {
                        let f = |off: usize| f32::from(d[off + c]) / 255.0;
                        let top = f(o00) + (f(o10) - f(o00)) * tx;
                        let bot = f(o01) + (f(o11) - f(o01)) * tx;
                        top + (bot - top) * ty
                    };
                    let sa = ch(3);
                    let a = sa * b.coverage;
                    if a <= 0.0 {
                        continue;
                    }
                    let acc = &mut *maps[i];
                    let inv = 1.0 - a;
                    // Premul over-accumulate: `tint·a + prev·(1−a)` with the stamp's premul colour already
                    // = tint·sa, scaled by the dab coverage.
                    acc[o] = encode(ch(0) * b.coverage + f32::from(acc[o]) / 255.0 * inv);
                    acc[o + 1] = encode(ch(1) * b.coverage + f32::from(acc[o + 1]) / 255.0 * inv);
                    acc[o + 2] = encode(ch(2) * b.coverage + f32::from(acc[o + 2]) / 255.0 * inv);
                    acc[o + 3] = encode(a + f32::from(acc[o + 3]) / 255.0 * inv);
                    touched = true;
                }
            }
        }
    }
    touched
}
