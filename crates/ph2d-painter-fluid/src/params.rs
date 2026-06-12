//! [`FluidParams`] — solver tuning (ADR-0049 §2.5, cap ≤ 12 fields).
//!
//! **Mirrors [`ph2d_painter_brush::diffusion::DiffusionParams`]** rather than the
//! Shallow-Water fields ADR-0049 §2.5 originally listed (viscosity / gravity /
//! Jacobi iterations), because the solver is the gated diffusion-advection model
//! (ADR-0049-amendment-1). The field set stays 1:1 with [`DiffusionParams`] so
//! [`Self::to_diffusion`] is the one conversion point that keeps the serialized
//! brush tuning and the GPU solver's `DiffusionParams` a single source of truth
//! (ADR-0085: the GPU solver is now the only live path).

use ph2d_painter_brush::diffusion::DiffusionParams;
use serde::{Deserialize, Serialize};

/// Per-brush / per-canvas tuning of the gated diffusion-advection solver.
/// Serializable like every other brush param so MCP/replay can drive it.
/// Cap ≤ 12 fields (ADR-0049 §2.13) — 9 used, 3 headroom.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct FluidParams {
    /// `D·dt` per step (≤ 0.24 explicit-Euler CFL). Higher = faster bloom.
    pub diffusivity: f32,
    /// Water lost per step; as wetness falls below `w_lo` the gate closes (drying).
    pub evaporation: f32,
    /// `β` — paper-slope advection: pigment drifts downhill into the tooth valleys.
    pub downhill: f32,
    /// `λ` — wet-gradient advection (Curtis FlowOutward): wet→dry transport that
    /// strands pigment at the receding boundary (edge darkening + backruns).
    pub flow_outward: f32,
    /// Wet-gate smoothstep band: the gate ramps 0→1 over wetness `[w_lo, w_hi]`.
    pub w_lo: f32,
    pub w_hi: f32,
    /// Paper permeability at a tooth valley (`h=0`) vs a crest (`h=1`).
    pub perm_valley: f32,
    pub perm_crest: f32,
    /// HR-14 schema version (v1 = 1).
    pub version: u32,
    // === 3 slots of headroom (cap ≤ 12) ===
}

/// v1 schema version for [`FluidParams`] / [`crate::FluidSim`] (HR-14).
pub const FLUID_PARAMS_VERSION: u32 = 1;

impl Default for FluidParams {
    fn default() -> Self {
        // Mirror DiffusionParams::default() so GPU + CPU share one tuning truth.
        let d = DiffusionParams::default();
        Self {
            diffusivity: d.diffusivity,
            evaporation: d.evaporation,
            downhill: d.downhill,
            flow_outward: d.flow_outward,
            w_lo: d.w_lo,
            w_hi: d.w_hi,
            perm_valley: d.perm_valley,
            perm_crest: d.perm_crest,
            version: FLUID_PARAMS_VERSION,
        }
    }
}

impl FluidParams {
    /// Project to the [`DiffusionParams`] the CPU reference solver consumes — the
    /// single conversion point that keeps GPU + CPU tuning identical.
    #[must_use]
    pub fn to_diffusion(&self) -> DiffusionParams {
        DiffusionParams {
            diffusivity: self.diffusivity,
            evaporation: self.evaporation,
            downhill: self.downhill,
            flow_outward: self.flow_outward,
            w_lo: self.w_lo,
            w_hi: self.w_hi,
            perm_valley: self.perm_valley,
            perm_crest: self.perm_crest,
            // The deposition / shallow-water / capillary / lift / branching layers are NOT
            // carried by `FluidParams` — the live GPU solver drives each per-brush via
            // `set_deposition` / `set_shallow_water` / `set_from_diffusion` (ADR-0078 S3-S5,
            // ADR-0081/0082). `to_diffusion` therefore zeroes them so the serialized brush
            // tuning maps cleanly onto the base diffusion-advection field; the live values
            // are layered on top by the solver, not round-tripped through this struct.
            deposition: 0.0,
            deposition_dry: 0.0,
            granulation: 0.0,
            velocity: 0.0,
            viscosity: 0.0,
            drag: 0.0,
            pressure: 0.0,
            capillary: 0.0,
            capillary_mobility: ph2d_painter_brush::diffusion::CAPILLARY_PIGMENT_MOBILITY,
            // First-order advection on this mapped path; the live solver raises sharpness
            // (MacCormack) per-brush via `set_from_diffusion`.
            sharpness: 0.0,
            lift: 0.0,
            capillary_branching: 0.0,
            // Not carried by the serialized base `FluidParams`; the live solver layers the brush's
            // value via `set_from_diffusion`. 0.0 = no front-absorption on this mapped path (the
            // raw unbounded wick); the live painting brush turns Bleed Limit on per-stroke.
            bleed_limit: 0.0,
        }
    }
}
