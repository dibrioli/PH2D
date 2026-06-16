//! `RenderingMode` enum — modos de blending intrínsecos do brush (axis
//! ortogonal ao layer blend mode). FROZEN em 6 variants por ADR-0044 §2.4.
//!
//! Acoplado a `MAX_RENDERING_MODES = 6` const-driven: o `StampScheduler` W5+
//! pre-groupa stamps em `[Vec<Stamp>; MAX_RENDERING_MODES]` — bump silencioso
//! do enum sem bump do const = out-of-bounds. Mudar exige amendment ADR.

use serde::{Deserialize, Serialize};

/// Modo de composição do stamp na layer (ADR-0044 §2.4 + spec §1.5.2).
/// 6 variants FROZEN. WGSL shader lê como `u32` direto.
///
/// **Default = `LightGlaze`** (discriminant 0 — audit T1.5 round 4
/// R4-LH-1 revert). Round 1 (A-M1) moved the default to `UniformGlaze`
/// to align with the shader's `default:` arm — but that broke alignment
/// with the **ABI hot path**: `Stamp::zeroed()` (the canonical
/// `bytemuck::Zeroable` init) produces `rendering_mode = 0` →
/// `from_u32(0)` → `LightGlaze`. And `RenderingParams::default` also
/// uses `LightGlaze`. Three "defaults" must agree: enum default, Stamp
/// ABI zero-decode, and brush params default — all now `LightGlaze`.
/// The shader's `default:` fallback to `UniformGlaze` (used ONLY for
/// out-of-range discriminants from corrupted GPU data) is the
/// safer-degrade choice, intentionally distinct from the canonical
/// default.
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

    /// Decode a `u32` (typically `Stamp.rendering_mode`) into a
    /// `RenderingMode`. Unknown discriminants fall back to
    /// `UniformGlaze` — same default branch as the shader's
    /// `apply_rendering_mode` switch (`stamp.wgsl::default → uniform_glaze`).
    #[must_use]
    pub const fn from_u32(v: u32) -> Self {
        match v {
            0 => Self::LightGlaze,
            1 => Self::UniformGlaze,
            2 => Self::IntenseGlaze,
            3 => Self::HeavyGlaze,
            4 => Self::UniformBlending,
            5 => Self::IntenseBlending,
            _ => Self::UniformGlaze,
        }
    }

    /// The two **Blending** modes engage Wet Mix (mixer-brush) — Procreate gates
    /// the Wet Mix sliders behind exactly these (W7, ADR-0097). The four Glaze
    /// modes do not pick up / smear canvas colour.
    #[must_use]
    pub const fn is_blending(self) -> bool {
        matches!(self, Self::UniformBlending | Self::IntenseBlending)
    }
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
    fn default_aligns_with_stamp_zeroed_abi() {
        // Audit T1.5 round 4 R4-LH-1: the canonical default MUST align
        // with the byte-0 ABI decode (Stamp::zeroed produces
        // rendering_mode=0 → LightGlaze) AND with RenderingParams::default.
        // Round 1 A-M1's UniformGlaze default broke this triangle; round 4
        // reverts. Shader's `default:` fallback to UniformGlaze is the
        // safer-degrade for OUT-OF-RANGE discriminants (e.g., 99), NOT
        // the canonical default.
        assert_eq!(RenderingMode::default(), RenderingMode::LightGlaze);
        assert_eq!(RenderingMode::from_u32(0), RenderingMode::LightGlaze);
        // Out-of-range still safe-degrades to UniformGlaze (shader parity).
        assert_eq!(RenderingMode::from_u32(99), RenderingMode::UniformGlaze);
    }
}
