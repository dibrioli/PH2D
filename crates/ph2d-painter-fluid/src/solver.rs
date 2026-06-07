//! The solver: CPU reference (REUSE of the shipped `diffusion`) + the GPU
//! compute shader source. The live wgpu pipeline lands in W15.3 phase 2.
//!
//! ## CPU reference = the parity truth + det fallback
//! [`step_cpu_reference`] runs the exact gated diffusion-advection that shipped in
//! W15.2 ([`ph2d_painter_brush::diffusion::DiffusionGrid`]). It is (a) the HR-5
//! det-mode path (ADR-0049 §2.11 — GPU is non-deterministic), and (b) the
//! bit-near reference the GPU pass is validated against (phase-2 parity gate).
//!
//! ## GPU pass (phase 2)
//! [`FLUID_WGSL`] is the compute shader mirroring the CPU passes. The
//! diffusion pass is a pure GATHER (each cell sums conductance·(neighbour−self)),
//! which maps directly to a compute kernel. The advection pass is reformulated
//! from the CPU's SCATTER (push to the downstream neighbour) into a GATHER (each
//! cell pulls the net flux from its 4 neighbours) so it is atomics-free and
//! order-independent — the adaptation that makes it correct on the GPU. The wgpu
//! pipeline (storage textures, bind groups, dispatch, bbox upload) is wired in
//! phase 2 against a headless `GpuContext`, with a CPU↔GPU parity test.

use crate::params::FluidParams;
use ph2d_painter_brush::diffusion::DiffusionGrid;

/// The GPU compute shader (mirror of the CPU diffusion-advection). Embedded so a
/// dev-test can validate it through naga before any GPU init, and phase 2 can
/// `create_shader_module` it directly.
pub const FLUID_WGSL: &str = include_str!("shader/fluid.wgsl");

/// Run the CPU reference solver `steps` times in place. The det-mode path AND the
/// parity reference for the GPU pass — both go through the one shipped solver.
pub fn step_cpu_reference(grid: &mut DiffusionGrid, params: &FluidParams, steps: u32) {
    let dp = params.to_diffusion();
    for _ in 0..steps {
        grid.step(&dp);
    }
}
