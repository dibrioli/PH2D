//! Library — brushes built-in. ADR-0044 §2.1 + spec §1.6.
//!
//! T1.3 ship com **ROUND_HARD baseline** (slot 0). T1.6 expande para 12
//! brushes nominais (pencil_2b, ink_studio_pen, watercolor_wash, etc.) com
//! per-brush Mixbox defaults §1.6.9.
//!
//! Library slots 0..63 reservados para built-ins; imported brushes usam
//! `BrushHandle::new_imported(atlas_layer)` com bit-31 = 1 (ADR-0044 §2.8).

use crate::about::AboutParams;
use crate::brush::Brush;
use crate::brush_handle::BrushHandle;
use crate::grain::GrainParams;
use crate::pigment::PigmentMode;
use crate::rendering::RenderingParams;
use crate::rendering_mode::RenderingMode;
use crate::shape::{ShapeParams, ShapeSource};
use crate::stroke_path::StrokePathParams;

/// Slot 0 — `round_hard` brush. Default + smoke target T1.X ("primeira
/// pintura" Day 7).
///
/// Defaults: shape round_hard + no grain + UniformGlaze + Linear pigment +
/// flow 1.0 + tight spacing 0.10. Hard-edged, opaque, deterministic. O
/// brush mais simples possível — primeira pintura PH2D testa exatamente
/// este path.
pub const ROUND_HARD_SLOT: u32 = 0;

/// Construct the `round_hard` baseline Brush. **Não é `const`** porque
/// `AboutParams.name: String` exige heap alloc. T1.6 pode trocar para
/// `&'static str` se quiser const possível.
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
                atlas_layer: 0,
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

/// Handle do `round_hard` (slot 0 built-in). Public canon — `ph2d-tool-painter`
/// usa como default `PainterParams.active_brush`.
pub const ROUND_HARD: BrushHandle = BrushHandle(ROUND_HARD_SLOT);

/// Dimensão lateral da Shape texture builtin (per ADR-0044 §1.8.1).
pub const SHAPE_TILE_PX: u32 = 256;

/// Generate `round_hard` shape texture procedural (R8, 256×256). Center-circle
/// radial falloff via smoothstep — hard-edged opaque core com edge
/// antialiasing 0.85..=1.0 normalized radius.
///
/// **Audit 2026-05-26 C-G2 (decisão produto Enio: procedural sobre asset PNG):**
/// shapes "matemáticas" (round_hard, round_soft, oval_soft, square_hard,
/// tapered_oval) são procedural — zero bytes binários no repo, determinismo
/// bit-perfect cross-OS. Shapes com arte custom (flat_chisel, bristle_spread,
/// splatter_spread) em W6+ usam asset PNG; híbrido.
///
/// Output: `Vec<u8>` length 65536 (256*256 R8). Cada byte é alpha (0=fora, 255=core opaco).
pub fn round_hard_shape() -> Vec<u8> {
    let size = SHAPE_TILE_PX as usize;
    let mut out = vec![0u8; size * size];
    let center = (size as f32 - 1.0) * 0.5;
    let radius = center;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            // Normalize distance to [0, 1] where 1.0 = full radius edge.
            let d = (dx * dx + dy * dy).sqrt() / radius;
            // smoothstep(0.85, 1.0, d) inverted: 1.0 no core, 0.0 fora do edge.
            let edge_t = ((d - 0.85) / (1.0 - 0.85)).clamp(0.0, 1.0);
            let alpha = (1.0 - edge_t * edge_t * (3.0 - 2.0 * edge_t)) * 255.0;
            out[y * size + x] = alpha.clamp(0.0, 255.0) as u8;
        }
    }
    out
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
}
