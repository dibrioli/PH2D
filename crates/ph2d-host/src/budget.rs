//! Device memory + perf budgets (ADR-0049 §2.9/§2.10) — the inputs to the
//! fluid-pass graceful-degrade gate (`ph2d_painter_fluid::fluid_pass_eligible`).
//!
//! These live in `ph2d-host` because only the shell knows the device's VRAM
//! headroom + the measured per-frame painter time; the core reads the resulting
//! booleans. Pure value types — the shell populates them from the wgpu adapter
//! limits + its frame-time EWMA, the painter composes them with the brush's
//! `fluid_enabled` to decide whether to run the GPU solver this frame.

/// Coarse device class (ADR-0049 §2.9). `Ord` so `>= Mid` is a tier floor.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryTier {
    /// Old mobile GPUs / Intel UHD / Web baseline — fluid degrades to wet-mix.
    Low,
    /// A14+ iPad, Adreno 650+, Iris Xe, RDNA1 mobile — fluid eligible.
    Mid,
    /// M1+/RDNA1+ desktop — fluid eligible with headroom.
    High,
}

/// Free VRAM (after baseline allocations) + the device tier. The shell builds it
/// from the wgpu adapter; the painter reads [`MemoryBudget::fluid_capable`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MemoryBudget {
    /// Free VRAM in MB after atlas + history + snapshots baseline.
    pub vram_free_mb: u32,
    pub tier: MemoryTier,
}

/// Minimum free VRAM for the fluid pass (the 1/4-res textures + scratch, §2.10).
pub const FLUID_VRAM_FLOOR_MB: u32 = 32;

impl MemoryBudget {
    /// `true` if the platform has ≥ 32 MB free VRAM AND is at least mid-tier
    /// (ADR-0049 §2.9). The hard prerequisite for the live GPU fluid pass.
    #[must_use]
    pub fn fluid_capable(&self) -> bool {
        self.vram_free_mb >= FLUID_VRAM_FLOOR_MB && self.tier >= MemoryTier::Mid
    }
}

/// The painter per-frame sub-budget + the last measured painter time, so the
/// caller can see how much headroom remains for the fluid pass (ADR-0049 §2.10).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PerfBudget {
    /// Painter sub-budget per frame (08 §8.1.1: ~4.5 ms).
    pub painter_budget_ms: f32,
    /// Measured painter time last frame (the shell's EWMA-fed value).
    pub last_frame_painter_ms: f32,
}

impl PerfBudget {
    /// Remaining painter-budget headroom this frame (≥ 0). The fluid pass costs
    /// ~2.5–3.5 ms, so the caller requires `> 3.0` (ADR-0049 §2.8/§2.10).
    #[must_use]
    pub fn fluid_headroom_ms(&self) -> f32 {
        (self.painter_budget_ms - self.last_frame_painter_ms).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fluid_capable_requires_vram_and_tier() {
        assert!(MemoryBudget { vram_free_mb: 64, tier: MemoryTier::High }.fluid_capable());
        assert!(MemoryBudget { vram_free_mb: 32, tier: MemoryTier::Mid }.fluid_capable());
        assert!(!MemoryBudget { vram_free_mb: 31, tier: MemoryTier::High }.fluid_capable(), "below VRAM floor");
        assert!(!MemoryBudget { vram_free_mb: 999, tier: MemoryTier::Low }.fluid_capable(), "below tier floor");
    }

    #[test]
    fn headroom_is_clamped_non_negative() {
        let b = PerfBudget { painter_budget_ms: 4.5, last_frame_painter_ms: 1.0 };
        assert!((b.fluid_headroom_ms() - 3.5).abs() < 1e-6);
        let over = PerfBudget { painter_budget_ms: 4.5, last_frame_painter_ms: 9.0 };
        assert_eq!(over.fluid_headroom_ms(), 0.0, "over-budget clamps to 0");
    }
}
