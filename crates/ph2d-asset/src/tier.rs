//! W1.T4 (ADR-0055-v4) — `TierIndex` newtype host-agnostic.
//!
//! Shadow numérico (`u8`) de [`ph2d_host::DeviceTier`] (ADR-0053 = 5 variants
//! FROZEN), **enquanto** o enum em `ph2d-host` ainda for vapor (verificado em
//! 2026-05-27 noite via `grep -rn "pub enum DeviceTier" crates/` = ZERO matches;
//! E2 do plano vivo §Open Issues).
//!
//! **Mandato HR-1**: este newtype é host-agnostic — `ph2d-asset` NÃO ganha dep
//! em `ph2d-host` por causa disso. Quando `DeviceTier` materializar (slot
//! futuro ADR), este módulo vira `pub type TierIndex = ph2d_host::DeviceTier;`
//! alias OR re-export, sem mudança de API downstream.
//!
//! Audit Lente δ W1.T3: §Symbol Registry do plano vivo (`docs/plans/2026-05-
//! texture-compression-waves.md`) registra esta entry como "W1.T4 cria".

use serde::{Deserialize, Serialize};

/// Discriminator numérico u8 dos 5 platform tiers do PH2D, alinhado com
/// [ADR-0053](../../../docs/architecture/decisions/0053-cross-platform-tier.md)
/// `DeviceTier` (FROZEN 2026-05-26):
///
/// | Variant      | u8  | Hardware-class                                     |
/// |--------------|-----|----------------------------------------------------|
/// | `Desktop`    | `0` | Windows/Linux/macOS Intel + Apple Silicon          |
/// | `Mobile`     | `1` | iPad Apple7+/M1+, iPhone, Android Vulkan 1.0+      |
/// | `Web`        | `2` | Chrome 113+, Safari 19+, Firefox 145+ (WebGPU)     |
/// | `LowEnd`     | `3` | Android low-tier sem ASTC (Mali T-series antigo)   |
/// | `Constrained`| `4` | Fallback uncompressed (RGBA8 raw)                  |
///
/// Valores 5+ são INVÁLIDOS — `TierIndex::new` retorna `None` para qualquer
/// `u8 > 4`. Arch-gate em `crates/ph2d-asset/tests/architecture_texture_ktx2.rs`
/// trava essa invariante via test exhaustive.
///
/// **Migration warning ⚠️ (audit ε-H1 W1.T4):** quando `ph2d_host::DeviceTier`
/// materializar via slot futuro de ADR, o type alias `pub type TierIndex =
/// ph2d_host::DeviceTier` SÓ funciona se a ordem do enum `DeviceTier` bater
/// EXATAMENTE com os u8 hardcoded acima (Desktop→0, Mobile→1, Web→2, LowEnd→3,
/// Constrained→4). Implementador da migration DEVE adicionar arch-gate
/// `cross_crate_tier_alignment` em `ph2d-host/tests/` que valida
/// `assert_eq!(DeviceTier::Desktop as u8, 0)` etc., antes de tornar TierIndex
/// um alias. Caso contrário, postcard wire format de cooked-asset metadata
/// degrada silenciosamente cross-version (HR-6 invariant violation).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TierIndex(u8);

impl TierIndex {
    /// `0 — Desktop` (BC family default).
    pub const DESKTOP: Self = Self(0);
    /// `1 — Mobile` (ASTC family default; BC opcional via runtime feature query).
    pub const MOBILE: Self = Self(1);
    /// `2 — Web` (adapter-dependent ladder).
    pub const WEB: Self = Self(2);
    /// `3 — LowEnd` (ETC2 fallback).
    pub const LOW_END: Self = Self(3);
    /// `4 — Constrained` (RGBA8 uncompressed).
    pub const CONSTRAINED: Self = Self(4);

    /// Número de variants válidos. Trava invariante em arch-gate.
    pub const COUNT: usize = 5;

    /// Constrói um `TierIndex` validando `0..=4`. Retorna `None` para `u8 >= 5`.
    #[must_use]
    pub const fn new(value: u8) -> Option<Self> {
        if value < Self::COUNT as u8 {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Acesso `u8` raw para serialização / lookup em arrays.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self.0
    }

    /// Nome canônico curto (mesma grafia de `ph2d_host::DeviceTier` esperada
    /// no slot futuro). Útil em logs / debug overlays.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self.0 {
            0 => "Desktop",
            1 => "Mobile",
            2 => "Web",
            3 => "LowEnd",
            4 => "Constrained",
            // Unreachable per invariant; constructor rejects > 4.
            _ => "Invalid",
        }
    }
}

impl std::fmt::Display for TierIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_index_constants_match_adr_0053_ordering() {
        assert_eq!(TierIndex::DESKTOP.as_u8(), 0);
        assert_eq!(TierIndex::MOBILE.as_u8(), 1);
        assert_eq!(TierIndex::WEB.as_u8(), 2);
        assert_eq!(TierIndex::LOW_END.as_u8(), 3);
        assert_eq!(TierIndex::CONSTRAINED.as_u8(), 4);
    }

    #[test]
    fn tier_index_new_rejects_out_of_range() {
        for v in 0..5u8 {
            assert!(TierIndex::new(v).is_some(), "{v} should be valid");
        }
        for v in 5..=255u8 {
            assert!(TierIndex::new(v).is_none(), "{v} should be invalid");
        }
    }

    #[test]
    fn tier_index_name_canonical() {
        assert_eq!(TierIndex::DESKTOP.name(), "Desktop");
        assert_eq!(TierIndex::MOBILE.name(), "Mobile");
        assert_eq!(TierIndex::WEB.name(), "Web");
        assert_eq!(TierIndex::LOW_END.name(), "LowEnd");
        assert_eq!(TierIndex::CONSTRAINED.name(), "Constrained");
    }

    #[test]
    fn tier_index_round_trips_via_postcard() {
        // HR-6: serialized via postcard pra cooked-asset metadata.
        let original = TierIndex::MOBILE;
        let bytes = postcard::to_allocvec(&original).unwrap();
        let decoded: TierIndex = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(original, decoded);
    }
}
