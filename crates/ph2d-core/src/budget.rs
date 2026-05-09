//! Memory budget per HR-13 + SKILL §12.1.
//!
//! Each subsystem returns a [`MemoryBudget`] from its `Plugin::init`.
//! At boot, ph2d-core sums them and checks against the platform max
//! ([`Platform::max_total_mb`]). If sum > max, refuses boot with a
//! clear error — better than OOM jetsam in production.

/// Per-subsystem budget declaration.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct MemoryBudget {
    /// VRAM in MB (textures, depth buffers, intermediate render targets).
    pub vram_mb: u32,
    /// CPU-side RAM in MB (ECS world, asset cache, temp buffers).
    pub ram_mb: u32,
    /// Script heap (Luau VM + state_table) in MB.
    pub heap_script_mb: u32,
}

impl MemoryBudget {
    pub const fn new(vram_mb: u32, ram_mb: u32, heap_script_mb: u32) -> Self {
        Self {
            vram_mb,
            ram_mb,
            heap_script_mb,
        }
    }

    pub const fn total_mb(&self) -> u32 {
        self.vram_mb + self.ram_mb + self.heap_script_mb
    }
}

/// Target platforms with their app memory budget caps. Numbers from
/// SKILL §12.1 — kept in sync with the table there. If the constants
/// change in the SKILL, update both.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Platform {
    /// iPhone / iPad — strictest budget; iOS jetsam kicks at ~1 GB.
    IPad,
    /// macOS — generous, but VRAM is unified with RAM on Apple Silicon.
    Mac,
    /// Android — weaker hardware floor than iPad (mid-range targets).
    Android,
    /// Windows desktop — generous.
    Windows,
    /// Linux desktop — same as Windows.
    Linux,
    /// Web (browser) — heap caps and WASM linear memory limits.
    Web,
}

impl Platform {
    /// Total app memory budget in MB. Numbers per SKILL §12.1.
    pub const fn max_total_mb(self) -> u32 {
        match self {
            Self::IPad => 1000,
            Self::Mac => 3500,
            Self::Android => 700,
            Self::Windows => 3500,
            Self::Linux => 3500,
            Self::Web => 1000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OverBudget {
    pub platform: Platform,
    pub max_mb: u32,
    pub requested_mb: u32,
    pub overflow_mb: u32,
}

impl std::fmt::Display for OverBudget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "memory budget exceeded on {:?}: requested {} MB, max {} MB, over by {} MB",
            self.platform, self.requested_mb, self.max_mb, self.overflow_mb
        )
    }
}

impl std::error::Error for OverBudget {}

/// Sum the budgets of all subsystems and compare against the platform
/// cap. Called once at boot before any subsystem allocates.
pub fn check_budget(budgets: &[MemoryBudget], platform: Platform) -> Result<(), OverBudget> {
    let requested_mb: u32 = budgets.iter().map(|b| b.total_mb()).sum();
    let max_mb = platform.max_total_mb();
    if requested_mb > max_mb {
        Err(OverBudget {
            platform,
            max_mb,
            requested_mb,
            overflow_mb: requested_mb - max_mb,
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Subsystem budget table copied from SKILL §12.1 iPad column.
    /// If this fails, the SKILL table changed → update both places.
    fn ipad_subsystem_budgets() -> Vec<MemoryBudget> {
        vec![
            MemoryBudget::new(350, 0, 0), // Render textures + meshes
            MemoryBudget::new(0, 30, 0),  // Audio buffers
            MemoryBudget::new(0, 20, 0),  // Physics state
            MemoryBudget::new(80, 0, 0),  // Lighting (RC)
            MemoryBudget::new(0, 200, 0), // Asset DB cache
            MemoryBudget::new(0, 50, 0),  // ECS world
            MemoryBudget::new(0, 0, 64),  // Script heap (Luau)
            MemoryBudget::new(0, 64, 0),  // WASM linear memory
            MemoryBudget::new(80, 0, 0),  // Editor UI (iPad only)
            MemoryBudget::new(0, 60, 0),  // Working / temp
        ]
    }

    #[test]
    fn ipad_default_budget_fits() {
        let r = check_budget(&ipad_subsystem_budgets(), Platform::IPad);
        assert!(r.is_ok(), "default iPad budget should fit; got {r:?}");
    }

    #[test]
    fn over_budget_reports_overflow() {
        let huge = vec![MemoryBudget::new(2000, 0, 0)];
        let r = check_budget(&huge, Platform::IPad);
        let err = r.expect_err("expected over-budget");
        assert_eq!(err.platform, Platform::IPad);
        assert_eq!(err.max_mb, 1000);
        assert_eq!(err.requested_mb, 2000);
        assert_eq!(err.overflow_mb, 1000);
        // Display impl must mention the platform + numbers.
        let s = format!("{err}");
        assert!(s.contains("IPad"));
        assert!(s.contains("1000"));
    }

    #[test]
    fn empty_budget_is_ok() {
        let r = check_budget(&[], Platform::Web);
        assert!(r.is_ok());
    }

    #[test]
    fn total_mb_sums_three_categories() {
        let b = MemoryBudget::new(100, 50, 32);
        assert_eq!(b.total_mb(), 182);
    }

    #[test]
    fn each_platform_has_positive_max() {
        for &p in &[
            Platform::IPad,
            Platform::Mac,
            Platform::Android,
            Platform::Windows,
            Platform::Linux,
            Platform::Web,
        ] {
            assert!(p.max_total_mb() > 0, "{p:?} max_total_mb must be > 0");
        }
    }
}
