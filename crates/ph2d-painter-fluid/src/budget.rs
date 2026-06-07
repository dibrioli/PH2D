//! Fluid-pass eligibility (ADR-0049 §2.8) — the graceful-degrade gate.
//!
//! The pure decision is decoupled from the host budget types (`MemoryBudget` /
//! `PerfBudget` live in `ph2d-host`): the caller measures `fluid_capable` (≥ 32 MB
//! free VRAM AND tier ≥ Mid, §2.9) and `painter_headroom_ms` (§2.10) and passes
//! the values in. Keeping the logic param-free makes it trivially testable (the
//! §2.15 `fluid_pass_activates_only_when_eligible` truth table) and keeps this
//! leaf crate from depending on the host trait.

/// Minimum painter sub-budget headroom (ms) for the fluid pass — it costs
/// 2.5–3.5 ms, so we require > 3.0 (ADR-0049 §2.8/§2.10).
pub const FLUID_HEADROOM_FLOOR_MS: f32 = 3.0;

/// `true` iff the live fluid pass should run this frame: the brush opted in AND
/// the device can afford it (VRAM + tier) AND there is perf headroom. When
/// `false`, the caller falls back to the math wet-mix — the brush identity is
/// preserved, only the physical "wow" is dropped (ADR-0049 §2.8).
#[must_use]
pub fn fluid_pass_eligible(fluid_enabled: bool, fluid_capable: bool, painter_headroom_ms: f32) -> bool {
    fluid_enabled && fluid_capable && painter_headroom_ms > FLUID_HEADROOM_FLOOR_MS
}
