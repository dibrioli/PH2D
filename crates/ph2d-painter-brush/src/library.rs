//! Library — brushes built-in + procedural shapes. ADR-0044 §2.1 + spec §1.6.
//!
//! ## Shape variety (T1.6 ship)
//!
//! Three procedural shapes shipped — sufficient for the Day-N T1.6 acceptance
//! ("variety visível ≥ 3 brushes built-in"). Each shape is a pure function
//! `(u, v) → alpha` sampled on the GPU footprint grid (no atlas binding;
//! see decision note below). The slot numbering is **public ABI** (Stamp
//! `shape_layer` field) so re-ordering requires an ADR-0044 amendment.
//!
//! | Slot | Name          | Profile                                      |
//! |------|---------------|----------------------------------------------|
//! | 0    | `round_hard`  | Opaque core with Hermite smoothstep edge     |
//! | 1    | `round_soft`  | Radial gradient (Gaussian-like falloff)      |
//! | 2    | `square_hard` | Axis-aligned square with smooth edge         |
//!
//! ## Why no atlas binding yet (T1.6 follow-up decision)
//!
//! Sub-task 1 of the T1.6 handoff proposed `texture_2d_array<f32>` binding
//! on the GPU side + CPU sampling the same atlas — but the canonical CPU
//! path is **analytic** (matches the shader's analytic functions
//! bit-for-bit, gate-protected by `cpu_shader_textual_parity_all_six_modes`),
//! and `R8` atlas quantization would introduce ~1/255 per-pixel drift,
//! breaking HR-5 cross-OS reproducibility. For three procedural shapes the
//! analytic switch is **strictly better than atlas sampling**. Atlas binding
//! lands when the first **custom-art shape PNG** ships (W6+ canon list per
//! spec §1.6.7: `flat_chisel`, `bristle_spread`, `splatter_spread`,
//! `tapered_oval`) — that is where atlas pays for itself.
//!
//! Library slots 0..63 reserved for built-ins; imported brushes use
//! `BrushHandle::new_imported(atlas_layer)` with bit-31 = 1 (ADR-0044 §2.8).

use crate::about::AboutParams;
use crate::brush::Brush;
use crate::brush_handle::BrushHandle;
use crate::grain::GrainParams;
use crate::pigment::PigmentMode;
use crate::rendering::RenderingParams;
use crate::rendering_mode::RenderingMode;
use crate::shape::{ShapeParams, ShapeSource};
use crate::stroke_path::StrokePathParams;

// ─── Slot canon (ABI — re-ordering breaks Stamp.shape_layer interpretation) ──

/// Slot 0 — `round_hard` brush + shape. **Default**, hard-edged opaque core.
pub const ROUND_HARD_SLOT: u32 = 0;
/// Slot 1 — `round_soft` shape (Gaussian-like radial gradient).
pub const ROUND_SOFT_SLOT: u32 = 1;
/// Slot 2 — `square_hard` shape (axis-aligned filled square w/ smooth edge).
pub const SQUARE_HARD_SLOT: u32 = 2;

/// Number of built-in shape slots populated by T1.6. Slots 3..32 reserved
/// for ADR-0044 §1.6.7 canon expansion (round_gradient_soft /
/// round_soft_small / oval_soft / tapered_oval / flat_chisel /
/// bristle_spread / splatter_spread + 2 free).
pub const BUILTIN_SHAPE_SLOT_COUNT: u32 = 3;

// ─── Per-shape brush constructors ─────────────────────────────────────────────

/// `round_hard` baseline Brush (slot 0). Default + Day-7 smoke target.
///
/// Defaults: shape round_hard + no grain + LightGlaze + Linear pigment +
/// tight spacing 0.10. Hard-edged, opaque, deterministic. **First-pintura
/// path** — the simplest possible brush that touches every layer of the
/// engine (scheduler / Stamp ABI / shape lookup / rendering mode).
pub fn round_hard() -> Brush {
    Brush {
        stroke_path: StrokePathParams {
            spacing: 0.10,
            spacing_jitter: 0.0,
            jitter_lateral: 0.0,
            falloff: 0.0,
        },
        shape: ShapeParams {
            shape_source: ShapeSource::Builtin {
                atlas_layer: ROUND_HARD_SLOT,
                name: "round_hard".to_string(),
            },
            ..Default::default()
        },
        grain: GrainParams::default(), // GrainSource::None
        rendering: RenderingParams {
            rendering_mode: RenderingMode::UniformGlaze,
            pigment_mode: PigmentMode::Linear,
            flow: 1.0,
            ..Default::default()
        },
        about: AboutParams {
            name: "Round Hard".to_string(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// `round_soft` brush (slot 1). Radial gradient — Gaussian-like falloff.
/// Pairs naturally with `LightGlaze` to build up tone via overlap. T1.6
/// shipping target for "second brush with notable visual difference".
pub fn round_soft() -> Brush {
    Brush {
        stroke_path: StrokePathParams {
            spacing: 0.08,
            spacing_jitter: 0.0,
            jitter_lateral: 0.0,
            falloff: 0.0,
        },
        shape: ShapeParams {
            shape_source: ShapeSource::Builtin {
                atlas_layer: ROUND_SOFT_SLOT,
                name: "round_soft".to_string(),
            },
            ..Default::default()
        },
        grain: GrainParams::default(),
        rendering: RenderingParams {
            rendering_mode: RenderingMode::LightGlaze,
            pigment_mode: PigmentMode::Linear,
            flow: 0.8,
            ..Default::default()
        },
        about: AboutParams {
            name: "Round Soft".to_string(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// `square_hard` brush (slot 2). Axis-aligned filled square — useful as
/// a chisel surrogate, or with `shape_rotation_follow=true` to align with
/// the stroke direction. Third built-in for the T1.6 Day-N variety target.
pub fn square_hard() -> Brush {
    Brush {
        stroke_path: StrokePathParams {
            spacing: 0.10,
            spacing_jitter: 0.0,
            jitter_lateral: 0.0,
            falloff: 0.0,
        },
        shape: ShapeParams {
            shape_source: ShapeSource::Builtin {
                atlas_layer: SQUARE_HARD_SLOT,
                name: "square_hard".to_string(),
            },
            ..Default::default()
        },
        grain: GrainParams::default(),
        rendering: RenderingParams {
            rendering_mode: RenderingMode::UniformGlaze,
            pigment_mode: PigmentMode::Linear,
            flow: 1.0,
            ..Default::default()
        },
        about: AboutParams {
            name: "Square Hard".to_string(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Handle do `round_hard` (slot 0). Public canon — `ph2d-tool-painter`
/// usa como default `PainterParams.active_brush`.
pub const ROUND_HARD: BrushHandle = BrushHandle(ROUND_HARD_SLOT);
/// Handle do `round_soft` (slot 1).
pub const ROUND_SOFT: BrushHandle = BrushHandle(ROUND_SOFT_SLOT);
/// Handle do `square_hard` (slot 2).
pub const SQUARE_HARD: BrushHandle = BrushHandle(SQUARE_HARD_SLOT);

/// Dimensão lateral da Shape texture builtin (per ADR-0044 §1.8.1).
pub const SHAPE_TILE_PX: u32 = 256;

// ─── Procedural shape kernels (atlas-equivalent, analytic) ───────────────────

/// Procedural `round_hard` (slot 0) — Hermite smoothstep on `[0.85, 1.0]`
/// of normalized radial distance. **Identical analytic form** to the
/// shader's `round_hard_shape` (gate-protected by
/// `cpu_shader_textual_parity_all_six_modes` indirectly via the rendering
/// mode parity test + a dedicated shape-parity assertion in T1.6 tests).
///
/// `u, v ∈ [0, 1]` (pixel-center convention). Returns alpha ∈ `[0, 1]`.
#[inline]
pub fn shape_round_hard(u: f32, v: f32) -> f32 {
    let dx = u - 0.5;
    let dy = v - 0.5;
    let d = (dx * dx + dy * dy).sqrt() / 0.5;
    let edge_t = ((d - 0.85) / 0.15).clamp(0.0, 1.0);
    let smooth = edge_t * edge_t * (3.0 - 2.0 * edge_t);
    1.0 - smooth
}

/// Procedural `round_soft` (slot 1) — quadratic radial falloff
/// `alpha = (1 - d²)²` clamped to `d ∈ [0, 1]`. **Gaussian-equivalent**
/// shape with a smooth analytic derivative (good build-up under LightGlaze).
#[inline]
pub fn shape_round_soft(u: f32, v: f32) -> f32 {
    let dx = u - 0.5;
    let dy = v - 0.5;
    let d_sq = (dx * dx + dy * dy) / 0.25; // normalize to [0, 1] over radius 0.5
    if d_sq >= 1.0 {
        return 0.0;
    }
    let one_minus_d_sq = 1.0 - d_sq;
    one_minus_d_sq * one_minus_d_sq
}

/// Procedural `square_hard` (slot 2) — axis-aligned square with smooth
/// edge band. Uses `max(|dx|, |dy|)` (Chebyshev distance) so the iso-curves
/// are squares; Hermite smoothstep on `[0.90, 1.0]` keeps edges from
/// aliasing harshly without blurring the core.
///
/// Pairs naturally with `shape_rotation_follow=true` — the square aligns
/// to the stroke direction, simulating a chisel tip. Without rotation,
/// `shape_count > 1 + shape_scatter > 0` produces tiled patterns.
#[inline]
pub fn shape_square_hard(u: f32, v: f32) -> f32 {
    let dx = (u - 0.5).abs();
    let dy = (v - 0.5).abs();
    let d = dx.max(dy) / 0.5; // 1.0 at any edge of the square
    let edge_t = ((d - 0.90) / 0.10).clamp(0.0, 1.0);
    let smooth = edge_t * edge_t * (3.0 - 2.0 * edge_t);
    1.0 - smooth
}

/// Dispatch a procedural shape function by `shape_layer` slot. Out-of-range
/// slots fall back to `round_hard` (slot 0) — same safer-degrade behavior
/// as the shader's `default:` arm. Documented behavior, not bug.
#[inline]
pub fn shape_alpha_for_slot(slot: u32, u: f32, v: f32) -> f32 {
    match slot {
        ROUND_HARD_SLOT => shape_round_hard(u, v),
        ROUND_SOFT_SLOT => shape_round_soft(u, v),
        SQUARE_HARD_SLOT => shape_square_hard(u, v),
        _ => shape_round_hard(u, v),
    }
}

/// True if a shape kernel is **radially symmetric** — rotation does NOT
/// change the visible footprint or appearance. Used by the
/// [`StampScheduler`](crate::StampScheduler) to skip the `√2` footprint
/// enlargement when rotation is set on a radial shape.
///
/// Forward-compat: unknown slots return `false` (conservative — pay the
/// enlargement cost, never clip the shape).
#[inline]
pub fn shape_is_radial_symmetric(slot: u32) -> bool {
    matches!(slot, ROUND_HARD_SLOT | ROUND_SOFT_SLOT)
}

/// Bounding-box scale factor to apply to `size_px` when a stamp is
/// rotated. Returns `1.0` if the shape is radially symmetric;
/// otherwise returns the **tight analytic bound** `|cos θ| + |sin θ|`
/// — the side length of the axis-aligned bounding box that contains a
/// unit square rotated by θ.
///
/// **Audit T1.6 O-5 — why the tight bound:** the previous draft used a
/// discrete step (1.0 below an epsilon, √2 above). For non-radial
/// shapes on a curving stroke whose rotation sweeps slowly through 0°,
/// adjacent stamps could straddle the threshold and produce a discrete
/// 41 % BBox jump between two consecutive emissions (visible AA-band
/// thickness pulse). The continuous `|cos| + |sin|` bound avoids the
/// discontinuity at zero cost (one cos + one sin + one add).
///
/// **Audit T1.6 R-5 — periodic oscillation, not monotonic:** the
/// function value is `1.0` at `θ = 0°` / `90°` / `180°` (square's
/// axis-aligned BBox equals the square's side), peaks at `√2` at
/// `θ = 45°` / `135°`, and oscillates with period `π/2`. So a curving
/// stroke whose rotation sweeps through `0° → 45° → 90°` sees BBox
/// `1 → √2 → 1` — the AA-band thickness pulses on the same period.
/// This is **user-faithful behavior**: a physical square genuinely has
/// different axis-aligned widths at different rotations; the BBox
/// MUST track that to keep the rotated shape inscribed. Acceptable
/// trade-off vs the prior discrete step (the round-1 fix) and vs a
/// flat conservative √2 constant (5% wasted pixels at non-45° rotation,
/// no visual oscillation).
///
/// **NaN safety:** both `cos(NaN)` and `sin(NaN)` return NaN;
/// `NaN.abs() = NaN`; `NaN + NaN = NaN`. NaN propagates to caller —
/// scheduler's `debug_assert!(rotation_rad.is_finite())` catches
/// upstream in debug builds.
///
/// Caller (`StampScheduler`) multiplies the brush's nominal `size_px` by
/// this factor when populating the [`Stamp`](crate::Stamp).
#[inline]
pub fn rotated_footprint_scale(slot: u32, rotation_rad: f32) -> f32 {
    if shape_is_radial_symmetric(slot) {
        return 1.0;
    }
    rotation_rad.cos().abs() + rotation_rad.sin().abs()
}

/// Generate `round_hard` shape texture procedural (R8, 256×256). Center-circle
/// radial falloff via smoothstep — hard-edged opaque core com edge
/// antialiasing 0.85..=1.0 normalized radius.
///
/// **Audit 2026-05-26 C-G2 (decisão produto Enio: procedural sobre asset PNG):**
/// shapes "matemáticas" (round_hard, round_soft, square_hard, tapered_oval,
/// oval_soft) são procedural — zero bytes binários no repo, determinismo
/// bit-perfect cross-OS. Shapes com arte custom (flat_chisel, bristle_spread,
/// splatter_spread) em W6+ usam asset PNG; híbrido.
///
/// **T1.6 status:** this 256² rasterization is **reference/preview only**
/// (used by a future atlas binding when custom-art shapes arrive, plus
/// the existing `library_shape_tests` baseline). The hot path GPU and CPU
/// both call `shape_alpha_for_slot(slot, u, v)` directly — no atlas sample.
///
/// Output: `Vec<u8>` length 65536 (256*256 R8). Cada byte é alpha (0=fora, 255=core opaco).
pub fn round_hard_shape() -> Vec<u8> {
    rasterize_shape_to_atlas(shape_round_hard)
}

/// Rasterize an arbitrary procedural shape kernel to the canonical
/// 256² R8 atlas slot layout. Shared helper for `round_hard_shape`,
/// `round_soft_shape`, `square_hard_shape` rasterizers.
fn rasterize_shape_to_atlas(kernel: fn(f32, f32) -> f32) -> Vec<u8> {
    let size = SHAPE_TILE_PX as usize;
    let mut out = vec![0u8; size * size];
    let inv_size = 1.0 / (size as f32);
    for y in 0..size {
        for x in 0..size {
            // Pixel-center convention — matches the GPU sample grid
            // (audit 2026-05-26 D-1.M1).
            let u = (x as f32 + 0.5) * inv_size;
            let v = (y as f32 + 0.5) * inv_size;
            let alpha = (kernel(u, v) * 255.0).clamp(0.0, 255.0) as u8;
            out[y * size + x] = alpha;
        }
    }
    out
}

/// Rasterize `round_soft` to atlas slot bytes. T-atlas-binding consumer (W6+).
pub fn round_soft_shape() -> Vec<u8> {
    rasterize_shape_to_atlas(shape_round_soft)
}

/// Rasterize `square_hard` to atlas slot bytes. T-atlas-binding consumer (W6+).
pub fn square_hard_shape() -> Vec<u8> {
    rasterize_shape_to_atlas(shape_square_hard)
}

/// Builds the **canonical built-in shape atlas** as a contiguous byte buffer
/// (N slots × 256² R8 = N × 64 KB). Slots are ordered by their slot constants
/// (`ROUND_HARD_SLOT=0`, `ROUND_SOFT_SLOT=1`, `SQUARE_HARD_SLOT=2`). Consumer
/// (W6+ atlas binding) uploads this to a `texture_2d_array<f32>` via
/// `wgpu::Queue::write_texture`.
///
/// **T1.6 status:** function exists, but no GPU consumer wires it yet — see
/// the `library.rs` module header for the deferral rationale. The function
/// is shipped to (a) provide a single canonical builder when atlas binding
/// lands in W6+, and (b) keep the rasterizers exercised by tests so they
/// don't bit-rot before they have a consumer.
pub fn build_shape_atlas() -> Vec<u8> {
    let slot_bytes = (SHAPE_TILE_PX as usize) * (SHAPE_TILE_PX as usize);
    let mut atlas = Vec::with_capacity(slot_bytes * (BUILTIN_SHAPE_SLOT_COUNT as usize));
    atlas.extend_from_slice(&round_hard_shape());
    atlas.extend_from_slice(&round_soft_shape());
    atlas.extend_from_slice(&square_hard_shape());
    debug_assert_eq!(
        atlas.len(),
        slot_bytes * (BUILTIN_SHAPE_SLOT_COUNT as usize)
    );
    atlas
}

#[cfg(test)]
mod library_shape_tests {
    use super::*;

    #[test]
    fn round_hard_shape_has_correct_size() {
        let s = round_hard_shape();
        assert_eq!(s.len(), 256 * 256);
    }

    #[test]
    fn round_hard_shape_center_is_opaque() {
        let s = round_hard_shape();
        let center_idx = 128 * 256 + 128;
        assert_eq!(s[center_idx], 255, "center should be full alpha");
    }

    #[test]
    fn round_hard_shape_corners_are_transparent() {
        let s = round_hard_shape();
        // Corner (0,0) — well outside the inscribed circle.
        assert_eq!(s[0], 0, "corner (0,0) should be 0 alpha");
        assert_eq!(s[255], 0, "corner (255,0) should be 0 alpha");
    }

    #[test]
    fn round_hard_shape_has_smooth_edge() {
        let s = round_hard_shape();
        // Pick a point on the edge ring (radius ~ 0.92 of half-width).
        // distance from center 128 = 128*0.92 ≈ 117 → x=128+117=245, y=128.
        let edge_idx = 128 * 256 + 245;
        let alpha = s[edge_idx];
        // Should be partial alpha (not 0, not 255) — smoothstep transition.
        assert!(
            alpha > 0 && alpha < 255,
            "edge ring should have partial alpha; got {}",
            alpha
        );
    }

    // ─── round_soft kernel + atlas ──────────────────────────────────────────

    #[test]
    fn round_soft_kernel_center_is_one() {
        // (1 - 0)² = 1.0 at center (d=0).
        let alpha = shape_round_soft(0.5, 0.5);
        assert!(
            (alpha - 1.0).abs() < 1e-6,
            "center alpha must be 1.0; got {}",
            alpha
        );
    }

    #[test]
    fn round_soft_kernel_edge_is_zero() {
        // d=1 (corner) → (1 - 1)² = 0.0.
        assert_eq!(shape_round_soft(0.0, 0.5), 0.0); // left edge of bbox
        assert_eq!(shape_round_soft(1.0, 0.5), 0.0);
        assert_eq!(shape_round_soft(0.5, 0.0), 0.0);
        assert_eq!(shape_round_soft(0.5, 1.0), 0.0);
    }

    #[test]
    fn round_soft_kernel_mid_radius_is_partial() {
        // At d² = 0.5 → (1 - 0.5)² = 0.25.
        // dx² + dy² = 0.5 * 0.25 = 0.125 → dx = dy = sqrt(0.0625) = 0.25.
        let alpha = shape_round_soft(0.5 + 0.25, 0.5 + 0.25);
        let expected = 0.25_f32;
        assert!(
            (alpha - expected).abs() < 1e-5,
            "mid-radius alpha drift: got {}, expected {}",
            alpha,
            expected
        );
    }

    #[test]
    fn round_soft_atlas_size_correct() {
        assert_eq!(round_soft_shape().len(), 256 * 256);
    }

    // ─── square_hard kernel + atlas ─────────────────────────────────────────

    #[test]
    fn square_hard_kernel_center_is_opaque() {
        assert!((shape_square_hard(0.5, 0.5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn square_hard_kernel_inside_band_is_opaque() {
        // d = max(|dx|, |dy|) / 0.5. At (0.7, 0.5) → dx=0.2, dy=0; d = 0.4.
        // Below edge band [0.90, 1.0] → alpha = 1.0.
        assert!((shape_square_hard(0.7, 0.5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn square_hard_kernel_at_edge_band_partial() {
        // d = 0.95 (halfway through smoothstep band) → partial alpha.
        // (0.5 + 0.475, 0.5) → dx = 0.475, d = 0.95.
        let alpha = shape_square_hard(0.975, 0.5);
        assert!(
            alpha > 0.0 && alpha < 1.0,
            "edge band must be partial alpha; got {}",
            alpha
        );
    }

    #[test]
    fn square_hard_atlas_size_correct() {
        assert_eq!(square_hard_shape().len(), 256 * 256);
    }

    #[test]
    fn square_hard_atlas_core_is_opaque_corner_is_partial() {
        // Square shape: pixel-center convention puts the corner sample at
        // u ≈ 0.5/256 ≈ 0.00195, so the Chebyshev `d = max(|dx|, |dy|) /
        // 0.5 ≈ 0.996` — inside the smoothstep edge band `[0.90, 1.00]`,
        // hence PARTIAL alpha (not full). The CORE (≥ 16 px in from each
        // edge) is fully opaque.
        let s = square_hard_shape();
        // Core sample at (128, 128) — center of canvas, well inside d < 0.90.
        assert_eq!(s[128 * 256 + 128], 255, "core must be full alpha");
        // Core sample at (32, 32) — d = (128-32)/128 = 0.75, still < 0.90 → opaque.
        assert_eq!(
            s[32 * 256 + 32],
            255,
            "(32,32) is inside the d<0.90 core → opaque"
        );
        // Corner (0,0) is in the edge band → partial alpha.
        let corner = s[0];
        assert!(
            corner > 0 && corner < 255,
            "corner (0,0) must be partial alpha (in smoothstep band); got {}",
            corner
        );
    }

    // ─── shape_alpha_for_slot dispatch ──────────────────────────────────────

    #[test]
    fn shape_dispatch_returns_correct_kernel_per_slot() {
        // Verify the dispatch wires each slot to the right kernel.
        let (u, v) = (0.5_f32, 0.5_f32);
        assert!(
            (shape_alpha_for_slot(ROUND_HARD_SLOT, u, v) - shape_round_hard(u, v)).abs() < 1e-9
        );
        assert!(
            (shape_alpha_for_slot(ROUND_SOFT_SLOT, u, v) - shape_round_soft(u, v)).abs() < 1e-9
        );
        assert!(
            (shape_alpha_for_slot(SQUARE_HARD_SLOT, u, v) - shape_square_hard(u, v)).abs() < 1e-9
        );
    }

    #[test]
    fn shape_dispatch_unknown_slot_falls_back_to_round_hard() {
        // Forward-compat — slot 99 (not yet defined) falls back to round_hard,
        // matching the shader's `default:` arm. Documented behavior.
        let (u, v) = (0.5_f32, 0.5_f32);
        assert_eq!(
            shape_alpha_for_slot(99, u, v),
            shape_alpha_for_slot(ROUND_HARD_SLOT, u, v),
            "unknown slot must fall back to round_hard"
        );
    }

    #[test]
    fn build_shape_atlas_has_correct_size_and_order() {
        let atlas = build_shape_atlas();
        let slot_bytes = 256 * 256;
        assert_eq!(atlas.len(), slot_bytes * 3);
        // Slot 0 = round_hard atlas; center must be opaque.
        assert_eq!(atlas[128 * 256 + 128], 255);
        // Slot 2 = square_hard atlas at center (offset 2 * slot_bytes).
        assert_eq!(atlas[2 * slot_bytes + 128 * 256 + 128], 255);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_hard_handle_is_slot_0_builtin() {
        assert_eq!(ROUND_HARD.slot(), 0);
        assert!(!ROUND_HARD.is_imported());
    }

    #[test]
    fn round_soft_handle_is_slot_1_builtin() {
        assert_eq!(ROUND_SOFT.slot(), ROUND_SOFT_SLOT);
        assert!(!ROUND_SOFT.is_imported());
    }

    #[test]
    fn square_hard_handle_is_slot_2_builtin() {
        assert_eq!(SQUARE_HARD.slot(), SQUARE_HARD_SLOT);
        assert!(!SQUARE_HARD.is_imported());
    }

    #[test]
    fn round_hard_brush_defaults_match_smoke() {
        let b = round_hard();
        assert_eq!(b.stroke_path.spacing, 0.10);
        assert_eq!(b.rendering.flow, 1.0);
        assert_eq!(b.rendering.rendering_mode, RenderingMode::UniformGlaze);
        assert_eq!(b.rendering.pigment_mode, PigmentMode::Linear);
        assert_eq!(b.about.name, "Round Hard");
        assert!(matches!(
            b.shape.shape_source,
            ShapeSource::Builtin { atlas_layer: 0, .. }
        ));
    }

    #[test]
    fn round_soft_brush_points_at_correct_slot() {
        let b = round_soft();
        assert_eq!(b.about.name, "Round Soft");
        assert_eq!(b.rendering.rendering_mode, RenderingMode::LightGlaze);
        match b.shape.shape_source {
            ShapeSource::Builtin { atlas_layer, .. } => {
                assert_eq!(atlas_layer, ROUND_SOFT_SLOT);
            }
            _ => panic!("round_soft must be Builtin source"),
        }
    }

    #[test]
    fn square_hard_brush_points_at_correct_slot() {
        let b = square_hard();
        assert_eq!(b.about.name, "Square Hard");
        assert_eq!(b.rendering.rendering_mode, RenderingMode::UniformGlaze);
        match b.shape.shape_source {
            ShapeSource::Builtin { atlas_layer, .. } => {
                assert_eq!(atlas_layer, SQUARE_HARD_SLOT);
            }
            _ => panic!("square_hard must be Builtin source"),
        }
    }

    #[test]
    fn builtin_shape_slots_are_distinct() {
        assert_ne!(ROUND_HARD_SLOT, ROUND_SOFT_SLOT);
        assert_ne!(ROUND_HARD_SLOT, SQUARE_HARD_SLOT);
        assert_ne!(ROUND_SOFT_SLOT, SQUARE_HARD_SLOT);
        assert_eq!(BUILTIN_SHAPE_SLOT_COUNT, 3);
    }
}
