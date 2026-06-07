#![forbid(unsafe_code)]
//! ph2d-painter-fluid — Fluid Brushes Extension (ADR-0049, opt-in W15).
//!
//! Live watercolor wet-on-wet on the GPU: the dabs splat pigment + water into a
//! low-res field that diffuses + advects every frame (the wash keeps blooming +
//! drying after pen-up). This crate is the GPU port of the CPU live diffusion
//! that shipped in W15.2 (`ph2d_tool_painter` + `ph2d_painter_brush::diffusion`).
//!
//! ## Why a separate crate (ADR-0049 §4.4)
//! The GPU shallow-water/diffusion solver is the highest technical risk of the
//! painter; isolating it means a failed device-matrix pass is "don't compile the
//! feature", not a broken brush engine. It is a LEAF crate — only an optional,
//! feature-gated `ph2d-tool-painter` consumes it.
//!
//! ## Solver model (ADR-0049-amendment-1)
//! The solver MIRRORS [`ph2d_painter_brush::diffusion`] — gated diffusion-
//! advection (Curtis SIGGRAPH 1997), the model that shipped + was visually
//! ratified (ADR-0077 D11/D12), NOT the Shallow-Water sketch of ADR-0049 §2.7.
//! That keeps the GPU output matching the validated CPU look and makes the CPU
//! solver a true parity reference + the HR-5 det fallback (§2.11).
//!
//! ## Feature flag (ADR-0049 §2.1)
//! `--features fluid` compiles the real solver. Without it, only [`stub`] (a
//! no-op surface) exists, so builds that don't want fluid pay zero binary +
//! zero VRAM.

#[cfg(feature = "fluid")]
mod budget;
#[cfg(feature = "fluid")]
mod composite;
#[cfg(feature = "fluid")]
mod params;
#[cfg(feature = "fluid")]
mod sim;
#[cfg(feature = "fluid")]
mod solver;

#[cfg(feature = "fluid")]
pub use budget::fluid_pass_eligible;
#[cfg(feature = "fluid")]
pub use composite::{COMPOSITE_WGSL, FluidCompositor};
#[cfg(feature = "fluid")]
pub use params::FluidParams;
#[cfg(feature = "fluid")]
pub use sim::{FluidSim, GravitySource};
#[cfg(feature = "fluid")]
pub use solver::{FLUID_WGSL, FluidSolver, step_cpu_reference};

/// No-op surface compiled when the `fluid` feature is OFF (ADR-0049 §2.1). A
/// consumer can name these symbols unconditionally; they do nothing and the
/// solver never runs (the brush falls back to the math wet-mix, §2.8).
#[cfg(not(feature = "fluid"))]
pub mod stub {
    /// Always `false` — fluid is not compiled in, so no pass is ever eligible.
    #[must_use]
    pub fn fluid_pass_eligible(_fluid_enabled: bool, _capable: bool, _headroom_ms: f32) -> bool {
        false
    }
}
