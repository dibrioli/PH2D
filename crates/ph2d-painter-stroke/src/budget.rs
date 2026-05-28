//! `PainterMemoryBudget` — buckets de memória do Painter (ADR-0046 §2.10).
//!
//! ## Estado STANDALONE T1.8 + pendência Coord-A
//!
//! ADR-0046 §2.10 prescreve que `ph2d-host::MemoryBudget` ganhe o field
//! `painter: PainterMemoryBudget`. Como `ph2d-host` é foundational (caminho
//! C, DIRETRIZ §3.C) e o gate `painter_memory_budget_field_count_is_capped`
//! checa `ph2d-host::PainterMemoryBudget` (não daqui), T1.8 ship com a struct
//! standalone aqui. Coord-A subsequente:
//!
//! 1. Move `pub struct PainterMemoryBudget` para `crates/ph2d-host/src/memory_budget.rs`
//!    sob `pub mod painter; pub use painter::PainterMemoryBudget;` (ou inline).
//! 2. Adiciona o field `pub painter: PainterMemoryBudget` em `MemoryBudget`.
//! 3. Adiciona dep `ph2d-host = { path = "..." }` aqui e troca a definição
//!    local por `pub use ph2d_host::PainterMemoryBudget;`.
//! 4. Gate ativa automaticamente (vacuous → real).
//!
//! Cap: ≤ 8 fields.

use serde::{Deserialize, Serialize};

/// Buckets Painter dentro do orçamento de memória global.
///
/// Defaults vivem em [`Self::DEFAULT_HIGH_TIER`] (≥ 16 GiB RAM desktop) e
/// [`Self::DEFAULT_LOW_TIER`] (iPad 2018 / Android entry). Tier detection
/// é Coord-A (`ph2d-host::PlatformHost::device_capability()`).
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PainterMemoryBudget {
    /// Stroke history (`Vec<StrokeRecord>`) — ~20k strokes em 150 MB.
    /// Eviction NÃO automática (é history, não cache). `< 150` força `Ring`.
    pub stroke_history_mb: u32,
    /// Snapshot LRU cap. Offload disk sob memory pressure; metadata fica RAM.
    pub snapshot_cap_mb: u32,
    /// Shape atlas (R8) — 4 MB típico, static após boot.
    pub atlas_shape_mb: u32,
    /// Grain atlas (R8) — 32 MB típico, static após boot (Procedural Grain
    /// reduziu de 64 MB; ADR-0044).
    pub atlas_grain_mb: u32,
    // === 4 slots de headroom (mixbox_lut_mb, fluid_solver_mb,
    //                         mcp_audit_log_mb, hdr_canvas_overhead_mb) ===
}

impl PainterMemoryBudget {
    /// Default high-tier: desktop 16 GiB+ / iPad Pro M1+ / Android top.
    pub const DEFAULT_HIGH_TIER: Self = Self {
        stroke_history_mb: 150,
        snapshot_cap_mb: 200,
        atlas_shape_mb: 4,
        atlas_grain_mb: 32,
    };

    /// Default low-tier: iPad 2018 (3 GiB) / Android entry / Web (200 MB total).
    /// Stroke history cai pra Ring (vide [`crate::history::StrokeHistory::for_budget`]).
    pub const DEFAULT_LOW_TIER: Self = Self {
        stroke_history_mb: 30,
        snapshot_cap_mb: 60,
        atlas_shape_mb: 4,
        atlas_grain_mb: 16,
    };

    /// Soma dos buckets em MiB — sanity check pro tier policy.
    pub const fn total_mb(&self) -> u32 {
        self.stroke_history_mb + self.snapshot_cap_mb + self.atlas_shape_mb + self.atlas_grain_mb
    }
}

impl Default for PainterMemoryBudget {
    fn default() -> Self {
        Self::DEFAULT_HIGH_TIER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_tier_totals_within_400mb() {
        let b = PainterMemoryBudget::DEFAULT_HIGH_TIER;
        // ADR-0046 §2.10: ~400 MB no extremo (com brush_snapshots ~10 MB
        // que vive fora desta struct). 150 + 200 + 4 + 32 = 386 MB ✓.
        assert!(b.total_mb() <= 400);
        assert!(b.total_mb() >= 350);
    }

    #[test]
    fn low_tier_totals_under_150mb() {
        let b = PainterMemoryBudget::DEFAULT_LOW_TIER;
        assert!(b.total_mb() <= 150);
    }

    #[test]
    fn high_tier_default_eligible_for_full_history() {
        // `stroke_history_mb ≥ 150` → `StrokeHistory::Full` (vide history.rs).
        use crate::history::{FULL_HISTORY_MIN_BUDGET_MB, StrokeHistory};
        let b = PainterMemoryBudget::DEFAULT_HIGH_TIER;
        assert!(b.stroke_history_mb >= FULL_HISTORY_MIN_BUDGET_MB);
        let h = StrokeHistory::for_budget(b.stroke_history_mb);
        assert!(matches!(h, StrokeHistory::Full(_)));
    }

    #[test]
    fn low_tier_default_falls_to_ring() {
        use crate::history::StrokeHistory;
        let b = PainterMemoryBudget::DEFAULT_LOW_TIER;
        let h = StrokeHistory::for_budget(b.stroke_history_mb);
        assert!(matches!(h, StrokeHistory::Ring { .. }));
    }

    #[test]
    fn budget_roundtrips_postcard() {
        let b = PainterMemoryBudget::DEFAULT_HIGH_TIER;
        let bytes = postcard::to_allocvec(&b).unwrap();
        let back: PainterMemoryBudget = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(b, back);
    }
}
