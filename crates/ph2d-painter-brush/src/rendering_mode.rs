//! `RenderingMode` enum — modos de blending intrínsecos do brush (axis
//! ortogonal ao layer blend mode). FROZEN em 6 variants por ADR-0044 §2.4.
//!
//! Acoplado a `MAX_RENDERING_MODES = 6` const-driven: o `StampScheduler` W5+
//! pre-groupa stamps em `[Vec<Stamp>; MAX_RENDERING_MODES]` — bump silencioso
//! do enum sem bump do const = out-of-bounds. Mudar exige amendment ADR.

use serde::{Deserialize, Serialize};

/// Modo de composição do stamp na layer (ADR-0044 §2.4 + spec §1.5.2).
/// 6 variants FROZEN. WGSL shader lê como `u32` direto.
#[repr(u32)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum RenderingMode {
    /// `new = s * α_s + L * (1 - α_s * 0.6)` — aquarela leve.
    #[default]
    LightGlaze = 0,
    /// `new = s * α_s + L * (1 - α_s)` — Porter-Duff "over" puro (Photoshop Normal).
    UniformGlaze = 1,
    /// Alpha curve agressiva — cobre rápido, ainda permite sobreposição.
    IntenseGlaze = 2,
    /// Cor pura e sólida, traços marcantes.
    HeavyGlaze = 3,
    /// Mistura linear contínua + accumulator — mídia úmida (óleo/acrílico).
    UniformBlending = 4,
    /// Uniform Blending + smudge factor — Wet Mix forte; tinta puxa.
    IntenseBlending = 5,
    // === ZERO slots de headroom — frozen. ===
}

/// Const-driven sentinel acoplado ao tamanho do enum `RenderingMode`.
/// Usado por `StampScheduler` pre-grouping (`stamps_by_mode: [Vec<Stamp>; MAX_RENDERING_MODES]`).
/// Bump exige amendment ADR-0044 simultâneo do enum + const.
pub const MAX_RENDERING_MODES: usize = 6;

impl RenderingMode {
    /// Cycle through all 6 variants (útil pra UI dropdown e tests).
    pub const ALL: [Self; MAX_RENDERING_MODES] = [
        Self::LightGlaze,
        Self::UniformGlaze,
        Self::IntenseGlaze,
        Self::HeavyGlaze,
        Self::UniformBlending,
        Self::IntenseBlending,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_has_max_rendering_modes_entries() {
        assert_eq!(RenderingMode::ALL.len(), MAX_RENDERING_MODES);
    }

    #[test]
    fn discriminants_are_0_through_5() {
        for (i, m) in RenderingMode::ALL.iter().enumerate() {
            assert_eq!(*m as u32, i as u32, "discriminant must match index");
        }
    }

    #[test]
    fn default_is_light_glaze() {
        assert_eq!(RenderingMode::default(), RenderingMode::LightGlaze);
    }
}
