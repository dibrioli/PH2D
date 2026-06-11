//! Wet-field composite shared constants (ADR-0049 / ADR-0077 D12, ADR-0085).
//!
//! ADR-0085 makes the GPU the single source of truth for the live watercolor composite — the
//! CPU composite reference (`composite_wet_field_cpu`) and its bicubic/reduce helpers are gone.
//! What remains here is the small surface the GPU compositor (`ph2d-painter-fluid`) still shares
//! with the tool/shell so both cover the same pixels: the canvas-region bbox math and the two
//! parity literals the GPU `composite.wgsl` mirrors band-for-band.

/// Canvas-pixel bbox `(px_lo, py_lo, px_hi, py_hi)` (exclusive hi) covered by a
/// grid region at `scale`, padded **2 grid cells each side** and clamped to the
/// canvas. The dispatch/loop bounds for the composite — shared by the CPU loop and
/// the GPU dispatch so they cover the same pixels.
///
/// **Pad = 2 cells (not 1):** the Catmull-Rom bicubic reads ±1.5 cells, the 2×2
/// coverage supersample adds a ±0.25-px sub-position offset, and the gated diffusion
/// leaks pigment ~1 cell past the wet gate (`diffuse`'s face-conductance). A 1-cell
/// pad under-covered the soft falloff → the round dab was hard-cut to the rectangle
/// (Enio's "quinas retangulares"). Caller MUST also feed a region that already
/// contains all pigment (the all-time wet envelope, not the receding water bbox).
#[must_use]
pub fn composite_canvas_region(
    grid_region: (u32, u32, u32, u32),
    scale: u32,
    cw: u32,
    ch: u32,
) -> (u32, u32, u32, u32) {
    let (gx0, gy0, gx1, gy1) = grid_region;
    let px_lo = gx0.saturating_sub(2) * scale;
    let py_lo = gy0.saturating_sub(2) * scale;
    let px_hi = ((gx1 + 3) * scale).min(cw);
    let py_hi = ((gy1 + 3) * scale).min(ch);
    (px_lo, py_lo, px_hi, py_hi)
}

/// Composite supersampling factor — `N×N` coverage samples per canvas pixel,
/// premultiplied-averaged. The antialiasing that smooths an OPAQUE stroke's
/// silhouette: the pigment field is bicubic-smooth, but a steep coverage edge is
/// under-sampled at pixel centers → jaggies ("baixa resolução nas bordas"). The GPU
/// `composite.wgsl` mirrors this `N` exactly (parity). `N=1` ⇒ the original
/// single-sample composite (no AA), bit-identical to pre-W15.3.
pub const WET_COMPOSITE_SS: u32 = 2;

/// **Value-opacity floor (ADR-0079, re-tuned 2026-06-09).** The coverage divisor is
/// `color_sum = FLOOR + (1−FLOOR)·value`, so a deeper (lower-value) mixed pigment builds
/// coverage faster (the ADR-0079 intent). The original floor 0.3 gave dark pigments a 3.3×
/// mass-efficiency, which made their visible lum(mass) curve near-BINARY: the stroke rim's
/// per-cell texture (granulation, gate, deposition — continuous mass variation) rendered as
/// hard on/off pixel teeth ("borda pixelada", Enio 2026-06-09; light pigments render the SAME
/// texture as soft grading). Measured on a roughened-rim probe: intermediate-lum fraction for
/// dark went 0.04 (binary teeth) → 0.14 with floor 0.55 (≈ the 0.19 of a light pigment), while
/// keeping a 1.8× max dark-coverage boost. The GPU `composite.wgsl` mirrors the literal.
pub const VALUE_OPACITY_FLOOR: f32 = 0.55;
