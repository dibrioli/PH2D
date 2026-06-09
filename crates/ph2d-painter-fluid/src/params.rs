//! [`FluidParams`] — solver tuning (ADR-0049 §2.5, cap ≤ 12 fields).
//!
//! **Mirrors [`ph2d_painter_brush::diffusion::DiffusionParams`]** rather than the
//! Shallow-Water fields ADR-0049 §2.5 originally listed (viscosity / gravity /
//! Jacobi iterations), because the solver is the gated diffusion-advection model
//! (ADR-0049-amendment-1). Keeping the field set 1:1 with the CPU reference is
//! what makes [`crate::step_cpu_reference`] an exact parity fallback.

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
            // Deposition layer (ADR-0078 S3) stays OFF on the GPU-mirrored path until
            // the GPU `cs_transfer` + the FluidParams amendment land (S3b) — keeps the
            // CPU reference bit-parity with the current GPU solver.
            deposition: 0.0,
            deposition_dry: 0.0,
            granulation: 0.0,
            // Shallow-water velocity layer (ADR-0078 S3d) likewise OFF on this mapped path
            // until the GPU shallow-water passes + the FluidParams amendment land — the
            // live solver enables it via `FluidSolver::set_shallow_water` (S4d consts),
            // not through `FluidParams`. Keeps `to_diffusion(default) == default` (the
            // `cpu_reference_matches_diffusion_with_default_params` gate) bit-exact.
            velocity: 0.0,
            viscosity: 0.0,
            drag: 0.0,
            pressure: 0.0,
            // Capillary fringe (ADR-0078 S5) is likewise OFF on this `FluidParams`-mapped path
            // (the live solver drives it per-brush via `set_from_diffusion`, not `FluidParams`)
            // → keeps `to_diffusion(default) == default` bit-exact for the parity gate.
            capillary: 0.0,
            capillary_mobility: ph2d_painter_brush::diffusion::CAPILLARY_PIGMENT_MOBILITY,
            // First-order advection on this mapped path → `to_diffusion(default) == default`.
            sharpness: 0.0,
            // Lift (ADR-0081) is likewise OFF on this `FluidParams`-mapped path (the live solver
            // drives it per-brush via `set_from_diffusion`, not `FluidParams`) → keeps
            // `to_diffusion(default) == default` bit-exact for the parity gate.
            lift: 0.0,
            // Branched capillary (ADR-0082) is likewise OFF on this `FluidParams`-mapped path
            // (the live solver drives it per-brush via `set_from_diffusion`) → keeps
            // `to_diffusion(default) == default` bit-exact for the parity gate.
            capillary_branching: 0.0,
        }
    }
}
