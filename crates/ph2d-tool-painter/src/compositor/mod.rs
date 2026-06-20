//! CPU layer compositor (W3.T3.2) — the correctness reference for the
//! layer stack. `docs/Painter_projeto/02_layers.md` §2.11.
//!
//! Composites a [`LayerStack`] **top-down recursively** in linear-sRGB:
//! each visible layer is decoded sRGB→linear, blended over the
//! accumulator via [`ph2d_painter_effects::apply_blend`] (opacity folded
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
use ph2d_color::srgb::srgb_to_linear_byte;
use ph2d_painter_effects::{BlendMode, apply_blend};
use std::collections::BTreeMap;
use std::sync::LazyLock;

// ─────────────────────── sRGB transfer LUTs (PERF) ───────────────────────
//
// The CPU compositor decodes every layer pixel sRGB→linear and encodes the
// composite linear→sRGB. Both `ph2d_color` byte transfers are `powf`, and a
// full-canvas adjustment recompose runs EVERY drag frame (no dirty-rect: a
// global adjustment changes the whole canvas). Measured 1024² = ~80 ms/frame,
// ~80 ns/px = 6 powf/px (3 decode + 3 encode) — the slider-drag FPS sink Enio
// hit. Both are 1-D transfers, so a per-process LUT removes the per-pixel powf:
//   - decode: input is `u8` (256 values) → a 256-entry table is BIT-IDENTICAL
//     (gate `decode_lut_is_bit_exact_with_srgb_to_linear_byte`).
//   - encode: 255-threshold table + `partition_point` returns the SAME byte as
//     `linear_to_srgb_byte`'s round-to-nearest (gate
//     `encode_via_threshold_matches_linear_to_srgb_byte`), no per-pixel powf.

/// sRGB→linear decode LUT, `u8` → linear f32. Each entry IS `srgb_to_linear_byte`
/// so the decode stays bit-exact.
static SRGB_DECODE_LUT: LazyLock<[f32; 256]> =
    LazyLock::new(|| core::array::from_fn(|i| srgb_to_linear_byte(i as u8)));

/// Linear thresholds of the 255 byte-rounding boundaries: `THRESH[b]` is the
/// smallest linear value whose `linear_to_srgb_byte` is `> b` (i.e. the b↔b+1
/// step). `partition_point(|t| t <= v)` then counts how many boundaries `v` has
/// crossed = the exact byte. The thresholds are binary-searched against
/// `linear_to_srgb_byte` ITSELF (not the analytic `srgb_to_linear` inverse) so
/// the result is BIT-for-BIT identical to the powf path — a `powf(2.4)`-derived
/// threshold rounds differently within ~1 ULP of a boundary (gate
/// `encode_via_threshold_matches_linear_to_srgb_byte`).
static SRGB_ENCODE_THRESH: LazyLock<[f32; 255]> = LazyLock::new(|| {
    use ph2d_color::srgb::linear_to_srgb_byte;
    core::array::from_fn(|b| {
        let target = b as u8 + 1; // first byte ABOVE this boundary
        // Bisect [0,1] for the step edge; 40 iters ≫ f32 precision over [0,1].
        let (mut lo, mut hi) = (0.0f32, 1.0f32);
        for _ in 0..40 {
            let mid = 0.5 * (lo + hi);
            if linear_to_srgb_byte(mid) >= target {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        hi
    })
});

/// Coarse linear→byte guess LUT (4096 cells), `ENCODE_COARSE[round(v*4095)]`.
/// The cell width (1/4095) is below the tightest byte-boundary spacing, so the
/// guess is within ±1 of the exact byte everywhere; [`encode_byte`] refines it
/// against [`SRGB_ENCODE_THRESH`]. This makes encode a branch-light index +
/// (usually zero) correction instead of an 8-step binary search per channel.
static SRGB_ENCODE_COARSE: LazyLock<[u8; 4096]> = LazyLock::new(|| {
    use ph2d_color::srgb::linear_to_srgb_byte;
    core::array::from_fn(|i| linear_to_srgb_byte(i as f32 / 4095.0))
});

/// Encode one straight-linear channel to an 8-bit sRGB byte — byte-exact with
/// `ph2d_color::srgb::linear_to_srgb_byte`, no per-pixel `powf` and no binary
/// search. Tables passed by ref so the caller forces each `LazyLock` once per
/// composite. `thresh[b]` is the b↔b+1 step edge; the coarse guess is corrected
/// ±1 (the `while`s run at most once in practice; the bound makes it robust).
#[inline]
fn encode_byte(thresh: &[f32; 255], coarse: &[u8; 4096], v: f32) -> u8 {
    let v = v.clamp(0.0, 1.0);
    let mut b = coarse[(v * 4095.0) as usize] as usize;
    while b < 255 && v >= thresh[b] {
        b += 1;
    }
    while b > 0 && v < thresh[b - 1] {
        b -= 1;
    }
    b as u8
}

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
    let lut = &*SRGB_DECODE_LUT; // hoist the LazyLock force out of the channels
    [
        lut[rgba8[b] as usize],
        lut[rgba8[b + 1] as usize],
        lut[rgba8[b + 2] as usize],
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

// ── Submodules (god-module split, 2026-06-04; pure mechanical move) ──
mod cache;
mod compose;
#[cfg(test)]
mod tests;
pub use cache::CompositorCache;
pub use compose::{composite, composite_region, composite_with_cache};
