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
