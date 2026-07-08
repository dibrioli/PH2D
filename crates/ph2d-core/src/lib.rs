#![forbid(unsafe_code)]
//! ph2d-core — math, time, memory budget, panic hook (M2).
//!
//! Foundation crate that every other ph2d-* crate depends on. Kept
//! intentionally tiny: re-exports `glam` for math (no in-house
//! reinvention per §15 anti-patterns), defines fixed-step accumulator
//! per the "Fix Your Timestep" pattern (Glenn Fiedler), declares
//! per-platform memory budgets per HR-13 + SKILL §12.1, and provides
//! a centralized panic hook per SKILL §12.5.

pub mod budget;
pub mod math;
pub mod panic;
pub mod playhead;
pub mod time;

pub use budget::{MemoryBudget, OverBudget, Platform, check_budget};
pub use math::{ScreenPos, WorldPos};
pub use panic::install_panic_hook;
pub use playhead::Playhead;
pub use time::{FixedStep, FixedStepReport};

// Re-export glam types used across the codebase. Keeping the surface
// small — add as needed, do not export the entire glam prelude.
pub use glam::{Mat3, Mat4, Quat, Vec2, Vec3, Vec4};
