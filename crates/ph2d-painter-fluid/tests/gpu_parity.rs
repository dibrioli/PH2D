//! W15.3 phase 2 — GPU solver ↔ CPU reference parity (the correctness gate).
//!
//! The GPU `FluidSolver` runs the same gated diffusion-advection as the shipped
//! CPU `ph2d_painter_brush::diffusion`. Seeded from the same water/paper/pigment
//! and stepped the same number of times, the two must agree closely — that's what
//! makes the CPU path a true HR-5 det fallback (ADR-0049 §2.11) and proves the
//! WGSL reproduces the reference algorithm (not just "runs").
//!
//! Not bit-equality: GPU lowers `smoothstep`/`max`/FMA differently per backend,
//! so the gate is a tight mean/worst |Δ| over the field (a correct shader agrees
//! to ~1e-4; a wrong one diverges by ≥1e-1).
//!
//! `#[ignore]` — needs a real device (like ph2d-render / vector-fill GPU gates):
//!   cargo test -p ph2d-painter-fluid --features fluid --test gpu_parity -- --ignored --nocapture
#![cfg(feature = "fluid")]

use ph2d_gpu::GpuContext;
use ph2d_painter_brush::diffusion::DiffusionGrid;
use ph2d_painter_fluid::{FluidParams, FluidSolver, step_cpu_reference};

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// A grid with a blue dab splatted into a wet pool — exercises diffuse (bloom),
/// advect (paper channeling + wet→dry edge transport), and evaporation together.
fn seeded_grid(w: u32, h: u32) -> DiffusionGrid {
    let mut g = DiffusionGrid::new(w, h, 2.0);
    // Wet the whole field a bit so the gate is open + gradients exist, then a
    // concentrated pigment dab off-centre (asymmetry catches advection-direction bugs).
    g.splat(w as f32 * 0.5, h as f32 * 0.5, w as f32 * 0.45, 0.5, [0.0, 0.0, 0.0]);
    g.splat(w as f32 * 0.42, h as f32 * 0.5, 6.0, 0.6, [0.1, 0.2, 0.7]);
    g
}

#[test]
#[ignore = "needs a GPU device"]
fn gpu_solver_matches_cpu_reference() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (48u32, 40u32);
    let steps = 12u32;
    let params = FluidParams::default();

    // CPU reference.
    let mut cpu = seeded_grid(w, h);
    step_cpu_reference(&mut cpu, &params, steps);
    let cpu_pig = cpu.pigment();

    // GPU: seed from the SAME initial field, step the same count.
    let init = seeded_grid(w, h);
    let pig4: Vec<[f32; 4]> = init
        .pigment()
        .iter()
        .map(|p| [p[0], p[1], p[2], 0.0])
        .collect();
    let solver = FluidSolver::new(&gpu.device, w, h);
    solver.set_params(&gpu.queue, &params);
    solver.upload(&gpu.queue, init.water(), init.paper(), &pig4);
    solver.step(&gpu.device, &gpu.queue, steps);
    let gpu_pig = solver.read_pigment(&gpu.device, &gpu.queue);

    // Mean + worst |Δ| over the pigment field (xyz).
    let n = (w * h) as usize;
    let mut sum = 0.0f64;
    let mut worst = 0.0f32;
    for i in 0..n {
        for k in 0..3 {
            let d = (gpu_pig[i][k] - cpu_pig[i][k]).abs();
            sum += f64::from(d);
            worst = worst.max(d);
        }
    }
    let mean = (sum / (n * 3) as f64) as f32;
    // Sanity: the field is non-trivial (a wrong "all zero" GPU would pass a Δ test
    // against a bug; assert there IS pigment so parity is meaningful).
    let total: f32 = gpu_pig.iter().map(|p| p[0] + p[1] + p[2]).sum();
    eprintln!("fluid GPU↔CPU: mean |Δ| = {mean:.6}, worst = {worst:.6}, total pigment = {total:.3} ({w}×{h}, {steps} steps)");
    assert!(total > 0.01, "GPU produced no pigment — solver is dead, parity meaningless");
    assert!(
        mean < 1.0e-3,
        "GPU↔CPU mean |Δ| {mean} too high — the WGSL diverges from the diffusion reference"
    );
    assert!(worst < 2.0e-2, "GPU↔CPU worst |Δ| {worst} too high");
}

#[test]
#[ignore = "needs a GPU device"]
fn gpu_solver_conserves_then_dries() {
    // Without evaporation the diffuse+advect passes conserve pigment mass (the CPU
    // invariant). With evaporation the field eventually stops evolving (dries).
    // Here: a no-evaporation run must conserve total mass GPU-side.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (32u32, 32u32);
    let params = FluidParams {
        evaporation: 0.0, // isolate conservation
        ..FluidParams::default()
    };
    let init = seeded_grid(w, h);
    let before: f32 = init.pigment().iter().map(|p| p[0] + p[1] + p[2]).sum();
    let pig4: Vec<[f32; 4]> = init
        .pigment()
        .iter()
        .map(|p| [p[0], p[1], p[2], 0.0])
        .collect();
    let solver = FluidSolver::new(&gpu.device, w, h);
    solver.set_params(&gpu.queue, &params);
    solver.upload(&gpu.queue, init.water(), init.paper(), &pig4);
    solver.step(&gpu.device, &gpu.queue, 16);
    let after: f32 = solver
        .read_pigment(&gpu.device, &gpu.queue)
        .iter()
        .map(|p| p[0] + p[1] + p[2])
        .sum();
    eprintln!("fluid GPU mass: before = {before:.4}, after = {after:.4}");
    assert!(
        (after - before).abs() < before * 0.02,
        "GPU diffuse+advect must conserve pigment mass (no evaporation): {before} → {after}"
    );
}
