//! CPU layer compositor (W3.T3.2) — the correctness reference for the
//! layer stack. `docs/Painter_projeto/02_layers.md` §2.11.
//!
//! Composites a [`LayerStack`] **top-down recursively** in **tones of the screen** (ADR-0177): each
//! visible layer's straight sRGB8 bytes are read as `byte / 255` (no transfer curve), blended over
//! the accumulator via [`ph2d_painter_effects::apply_blend`] (opacity folded into the source alpha),
//! groups composite their children into a sub-buffer first, and the result goes back by `round`.
//! The brush mixes in the same space, so a stroke on a new layer is the stroke on the layer below.
//! Adjustments defined in light convert at their own boundary (`compose.rs`, the Adjustment arm).
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
use ph2d_painter_effects::{BlendMode, apply_blend};
use std::collections::BTreeMap;
use std::sync::Arc;

/// One encoded channel `[0, 1]` → byte (`round`), the exact inverse of [`decode_byte`] on every byte
/// (gate `the_byte_round_trip_is_the_identity`).
#[inline]
pub(crate) fn encode_byte(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// RGBA8 (straight, sRGB-encoded) pixels for one layer — canvas-sized.
///
/// Serializable because a painted document has to survive a restart: `LayerStack` (the structure) has
/// been serde all along, and these are the pixels it points at.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LayerImage {
    pub width: u32,
    pub height: u32,
    /// `width * height * 4` bytes, row-major RGBA8. Partilhado (B18.1): clonar a imagem — duplicar a camada,
    /// o instantâneo do desfazer, a coleta para o disco — é um refcount; quem escreve faz `Arc::make_mut`.
    /// No disco é o mesmo `Vec<u8>` de sempre (serde `rc`).
    pub rgba8: Arc<Vec<u8>>,
}

impl LayerImage {
    /// A transparent canvas-sized image.
    #[must_use]
    pub fn transparent(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            rgba8: Arc::new(vec![0u8; (width as usize) * (height as usize) * 4]),
        }
    }

    /// The pixels to write: copies them first only when someone else shares the plane.
    pub fn rgba8_mut(&mut self) -> &mut Vec<u8> {
        Arc::make_mut(&mut self.rgba8)
    }
}

/// Resolves a layer's pixels for the compositor: the canvas-sized straight
/// sRGB8 RGBA bytes (`canvas_w * canvas_h * 4`) for a raster/mask layer.
/// Returns a borrowed slice (mirror of the GPU `LayerPixels { rgba8: &[u8] }`)
/// so a host can hand the active layer's working buffer (e.g. the tool's
/// `Arc<Vec<u8>>` canvas) without cloning. `None` for unknown/group layers.
pub trait LayerPixelSource {
    fn layer_rgba(&self, id: LayerId) -> Option<&[u8]>;
    /// The shared plane BEHIND [`Self::layer_rgba`] (the same bytes), when there is one: B19.10 keys its
    /// per-layer empty-block summary on the plane's `Arc` allocation (a `Weak`), which every write moves.
    fn layer_plane(&self, _id: LayerId) -> Option<&Arc<Vec<u8>>> {
        None
    }
    /// B19.10: one byte per (canvas row, 64-px column block), `0` = every texel α = 0. Only the
    /// compositor's own wrapper answers it.
    fn layer_vazios(&self, _id: LayerId) -> Option<&[u8]> {
        None
    }
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
    fn layer_plane(&self, id: LayerId) -> Option<&Arc<Vec<u8>>> {
        self.images.get(&id).map(|img| &img.rgba8)
    }
}

/// Read one straight sRGB8 texel as straight encoded RGBA `[f32; 4]`: every channel is `byte / 255`
/// (ADR-0177 — colour stays in tones of the screen; alpha is coverage, as always).
#[inline]
fn decode(rgba8: &[u8], idx: usize) -> [f32; 4] {
    let b = idx * 4;
    [
        decode_byte(rgba8[b]),
        decode_byte(rgba8[b + 1]),
        decode_byte(rgba8[b + 2]),
        decode_byte(rgba8[b + 3]),
    ]
}

/// One byte channel → encoded `[0, 1]`, the door every texel and seeded ground goes through.
#[inline]
pub(crate) fn decode_byte(b: u8) -> f32 {
    b as f32 / 255.0
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

// ── Submodules (god-module split, 2026-06-04; pure mechanical move) ──
mod abaixo; // the watercolor backdrop (`composite_below`), split out of `compose` (B52.R3)
#[cfg(test)]
mod ajustes_na_fronteira_tests;
mod cache;
mod compose;
mod gpu_ops; // the stack → the GPU compositor's op-list (two consumers)
#[cfg(test)]
mod oraculo_ajustes_tests;
#[cfg(test)]
mod oraculo_gimp_tests;
#[cfg(test)]
mod oraculo_vizinhanca_tests;
#[cfg(test)]
mod tests;
pub mod vidro;
pub use abaixo::{Corte, composite_below, composite_below_cortavel};
#[cfg(test)]
pub(crate) use cache::SAIDAS_REUSADAS;
pub use cache::{CompositorCache, composite_with_cache, composite_with_cache_into};
#[cfg(test)]
pub(crate) use compose::FIOS_FORCADOS;
pub use compose::{composite, composite_region};
pub(crate) use compose::{em_faixas_de_linhas, parallel_threads};
pub use gpu_ops::flatten_for_gpu;
pub(crate) use vidro::composite_region_sobre_o_papel;
