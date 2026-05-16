//! HR-13 — Memory budget aggregation lint.
//!
//! For every platform in the §4 matrix of the SKILL, sum the
//! `memory_budget` of every registered tool manifest plus a synthetic
//! "core engine" baseline, and verify the total does not exceed
//! `Platform::max_total_mb`. This is the CI gate that prevents a tool
//! migrating with a hidden 500 MB VRAM appetite from being accepted
//! silently.
//!
//! PR 3 — Foundation. Registry is empty so the test trivially passes
//! against synthetic baselines; real value lands as tools migrate in
//! PRs 4/6/7. The structure is in place so the gate fires the moment
//! any tool overshoots.

use ph2d_core::{MemoryBudget, Platform};
use ph2d_tool_registry::Registry;
use ph2d_tool_registry_init::register_all;

/// Synthetic baseline representing core engine subsystems (render,
/// physics, audio, ECS, asset DB, script heap, working buffers) as a
/// single aggregate so the test isn't tied to real per-subsystem
/// numbers — those are owned by each subsystem's `Plugin::init`
/// (HR-13). The baseline is intentionally conservative (≤ 60% of the
/// strictest target, iPad). Anything left over after the baseline is
/// the headroom tools must collectively respect.
///
/// **Update this only when the SKILL §12.1 baseline shifts** —
/// otherwise the gate either becomes meaningless (too generous) or
/// blocks legitimate tools (too strict).
const SYNTHETIC_CORE_BASELINE: MemoryBudget = MemoryBudget::new(
    // VRAM: ~300 MB headroom on iPad after textures + lighting + shadow buffers.
    300, // vram_mb
    // RAM: ~200 MB for ECS world + asset cache + working buffers.
    200, // ram_mb
    // Script heap: 64 MB per ADR-0010 default.
    64, // heap_script_mb
);

fn sum_budgets(registry: &Registry) -> MemoryBudget {
    let mut total = MemoryBudget::new(0, 0, 0);
    for m in registry.manifests() {
        total = MemoryBudget::new(
            total.vram_mb + m.memory_budget.vram_mb,
            total.ram_mb + m.memory_budget.ram_mb,
            total.heap_script_mb + m.memory_budget.heap_script_mb,
        );
    }
    total
}

fn fits_under_platform(tool_total: MemoryBudget, platform: Platform) -> Result<(), String> {
    let combined = MemoryBudget::new(
        SYNTHETIC_CORE_BASELINE.vram_mb + tool_total.vram_mb,
        SYNTHETIC_CORE_BASELINE.ram_mb + tool_total.ram_mb,
        SYNTHETIC_CORE_BASELINE.heap_script_mb + tool_total.heap_script_mb,
    );
    let max = platform.max_total_mb();
    if combined.total_mb() > max {
        Err(format!(
            "HR-13 violated on {platform:?}: core baseline {core} MB + tool sum {tools} MB \
             = {combined} MB > {max} MB cap",
            core = SYNTHETIC_CORE_BASELINE.total_mb(),
            tools = tool_total.total_mb(),
            combined = combined.total_mb(),
        ))
    } else {
        Ok(())
    }
}

#[test]
fn registered_tools_fit_under_every_platform_cap() {
    let mut reg = Registry::default();
    register_all(&mut reg);
    reg.build().expect("registry should build");

    let tool_total = sum_budgets(&reg);

    let mut violations = Vec::new();
    for platform in [
        Platform::IPad,
        Platform::Mac,
        Platform::Android,
        Platform::Windows,
        Platform::Linux,
        Platform::Web,
    ] {
        if let Err(e) = fits_under_platform(tool_total, platform) {
            violations.push(e);
        }
    }
    assert!(
        violations.is_empty(),
        "HR-13 memory budget violations:\n{}",
        violations.join("\n")
    );
}

#[test]
fn synthetic_overshoot_fires_the_gate() {
    // Regression guard: simulate a tool that asks for 4 GB of VRAM.
    // Chosen to overshoot every platform's cap (max is Mac/Win/Linux
    // at 3500 MB) so the gate fires uniformly — proves the lint is
    // platform-aware, not just iPad-aware.
    let bloated = MemoryBudget::new(4096, 0, 0);
    for platform in [
        Platform::IPad,
        Platform::Mac,
        Platform::Android,
        Platform::Windows,
        Platform::Linux,
        Platform::Web,
    ] {
        assert!(
            fits_under_platform(bloated, platform).is_err(),
            "HR-13 gate must reject 4 GB VRAM tool on {platform:?}"
        );
    }
}

#[test]
fn ipad_strictest_platform_rejects_modest_overshoot() {
    // The strictest cap is iPad (1000 MB total). With a 564 MB
    // baseline, a tool reserving 500 MB of VRAM busts iPad but not
    // desktop platforms — proves the gate scales per-platform.
    let modest = MemoryBudget::new(500, 0, 0);
    assert!(
        fits_under_platform(modest, Platform::IPad).is_err(),
        "iPad cap must reject 500 MB VRAM tool"
    );
    assert!(
        fits_under_platform(modest, Platform::Mac).is_ok(),
        "Mac (3500 MB cap) must accept the same 500 MB tool"
    );
}

#[test]
fn empty_registry_fits_everywhere() {
    // Sanity: baseline alone fits under every platform cap. If this
    // fails, the SYNTHETIC_CORE_BASELINE above is too high and
    // tightens until tools have zero room — adjust there.
    let zero = MemoryBudget::new(0, 0, 0);
    for platform in [
        Platform::IPad,
        Platform::Mac,
        Platform::Android,
        Platform::Windows,
        Platform::Linux,
        Platform::Web,
    ] {
        fits_under_platform(zero, platform).unwrap_or_else(|e| {
            panic!(
                "core baseline alone violates {platform:?}: {e}. \
                 Synthetic baseline in this test file is too high; \
                 lower it or revisit SKILL §12.1."
            );
        });
    }
}
