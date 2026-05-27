//! `Brush` — agregado top-level dos 12 sub-structs Brush Studio (§1.3.x).
//!
//! Estrutura canônica congelada por ADR-0044 §2.2 (cascata W0 ratificada
//! 2026-05-26). **Top-level cap ≤ 14 fields** (v1 = 13: 12 sub-structs +
//! `version`).
//!
//! Sub-caps tabela ADR-0044 §2.2.1 — auditado por
//! `ph2d-painter-contracts/tests/architecture_painter_contract_surface.rs`.

use crate::about::AboutParams;
use crate::brush_handle::BrushParamsHash;
use crate::color_dynamics::ColorDynamicsParams;
use crate::dynamics::DynamicsParams;
use crate::grain::GrainParams;
use crate::pencil::PencilParams;
use crate::properties::PropertiesParams;
use crate::rendering::RenderingParams;
use crate::shape::ShapeParams;
use crate::stabilization::StabilizationParams;
use crate::stroke_path::StrokePathParams;
use crate::taper::TaperParams;
use crate::wet_mix::WetMixParams;
use serde::{Deserialize, Serialize};

/// Agregado top-level dos 12 sub-structs Brush Studio (§1.3.x). HR-14 version=1.
///
/// **Top-level cap ≤ 14 fields** ADR-0044 §2.2 (v1 = 13). Adicionar campo
/// novo top-level exige amendment ADR.
///
/// **No sub-sub-structs.** Sub-structs são folha (primitivos + enums +
/// `Option<T>` + `Vec<T>` onde T é primitivo). Gate
/// `brush_no_sub_sub_structs` em painter-contracts.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Brush {
    pub stroke_path: StrokePathParams,       // §1.3.1
    pub stabilization: StabilizationParams,  // §1.3.2
    pub taper: TaperParams,                  // §1.3.3
    pub shape: ShapeParams,                  // §1.3.4
    pub grain: GrainParams,                  // §1.3.5
    pub rendering: RenderingParams, // §1.3.6 (inclui pigment_mode + fluid_enabled ADR-0049)
    pub wet_mix: WetMixParams,      // §1.3.7
    pub color_dynamics: ColorDynamicsParams, // §1.3.8
    pub dynamics: DynamicsParams,   // §1.3.9
    pub pencil: PencilParams,       // §1.3.10
    pub properties: PropertiesParams, // §1.3.11
    pub about: AboutParams,         // §1.3.12
    pub version: u32,               // HR-14 — v1 = 1
                                    // === 1 slot top-level de headroom (cap ≤ 14) ===
}

impl Default for Brush {
    /// Default = Round Hard sensato. Library Brushes overridem campos
    /// específicos (vide `library.rs`).
    fn default() -> Self {
        Self {
            stroke_path: StrokePathParams::default(),
            stabilization: StabilizationParams::default(),
            taper: TaperParams::default(),
            shape: ShapeParams::default(),
            grain: GrainParams::default(),
            rendering: RenderingParams::default(),
            wet_mix: WetMixParams::default(),
            color_dynamics: ColorDynamicsParams::default(),
            dynamics: DynamicsParams::default(),
            pencil: PencilParams::default(),
            properties: PropertiesParams::default(),
            about: AboutParams::default(),
            version: 1,
        }
    }
}

impl Brush {
    /// Compute blake3 hash dos params do brush. Usado por
    /// `StrokeRecord.brush_params_hash` (ADR-0046 §2.2) para dedup em stroke
    /// history. **NÃO** inclui `about.reset_points` (cache mutável que NÃO
    /// afeta a identidade do brush).
    ///
    /// T1.3 stub: hash via postcard serialize + blake3. Implementação real
    /// fine-tuned em T1.4+ (excluir reset_points para estabilidade).
    pub fn params_blake3(&self) -> BrushParamsHash {
        // T1.3 stub: serialize whole Brush via postcard, hash via blake3.
        // T1.4+ pode otimizar: serialize só os campos params-relevant (sem
        // about.reset_points / about.created_at_ms / signature_uri).
        //
        // **Panic vs unwrap_or_default (audit 2026-05-26 B-C1):** se postcard
        // serialize falhar (caso impossível para Brush bem-formed, mas type
        // system não prova), `unwrap_or_default` retornaria `Vec::new()` cujo
        // blake3 é constante (`af1349b9...`). TODOS brushes mal-formed
        // colapsariam no mesmo hash, violando contrato de identidade do
        // `StrokeRecord.brush_params_hash` (ADR-0046 §2.2 dedup). Escolhemos
        // panic explícito sobre corrupção silenciosa. Audit T1.6 R7 J1-2:
        // see [`Self::try_params_blake3`] for the fallible variant used
        // by import paths that want to skip a malformed brush and
        // continue importing the rest of a batch.
        self.try_params_blake3()
            .expect("Brush must serialize via postcard")
    }

    /// Fallible variant of [`Self::params_blake3`]. Audit T1.6 R7 J1-2.
    ///
    /// Returns `Err` instead of panicking on serialize failure so
    /// import paths and other recovery-friendly call sites can degrade
    /// gracefully (skip the malformed brush, surface a toast) instead
    /// of aborting the process. The infallibility rationale on the
    /// panicking variant still holds for ordinary in-memory `Brush`
    /// values constructed by the library — this exists for paths that
    /// can't statically guarantee well-formedness (e.g., a brush
    /// rehydrated from an external file).
    pub fn try_params_blake3(&self) -> Result<BrushParamsHash, postcard::Error> {
        let bytes = postcard::to_allocvec(self)?;
        Ok(*blake3::hash(&bytes).as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brush_default_constructs_with_v1_version() {
        let b = Brush::default();
        assert_eq!(b.version, 1);
    }

    #[test]
    fn brush_serializes_via_postcard() {
        let b = Brush::default();
        let bytes = postcard::to_allocvec(&b).expect("serialize must succeed");
        assert!(!bytes.is_empty());
        let b2: Brush = postcard::from_bytes(&bytes).expect("deserialize must succeed");
        assert_eq!(b, b2);
    }

    #[test]
    fn params_blake3_is_deterministic() {
        let b1 = Brush::default();
        let b2 = Brush::default();
        assert_eq!(b1.params_blake3(), b2.params_blake3());
    }

    #[test]
    fn params_blake3_changes_with_field_change() {
        let mut b1 = Brush::default();
        let b2 = Brush::default();
        b1.stroke_path.spacing = 0.20;
        assert_ne!(b1.params_blake3(), b2.params_blake3());
    }
}
