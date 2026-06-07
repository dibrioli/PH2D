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
fn step_grid_matches_cpu_step_in_place() {
    // The drop-in accelerator: `FluidSolver::step_grid` (upload → GPU step → write
    // pigment + water back into the grid) must leave the grid in the same state as
    // `step_cpu_reference`. This is what the shell calls to GPU-accelerate the
    // tool's CPU diffusion while the grid stays the composite source of truth.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (40u32, 36u32);
    let params = FluidParams::default();
    let mut cpu = seeded_grid(w, h);
    let mut gpu_grid = seeded_grid(w, h);
    step_cpu_reference(&mut cpu, &params, 10);
    let solver = FluidSolver::new(&gpu.device, w, h);
    solver.step_grid(&gpu.device, &gpu.queue, &mut gpu_grid, &params, 10);
    let mut worst_p = 0.0f32;
    for (a, b) in gpu_grid.pigment().iter().zip(cpu.pigment().iter()) {
        for k in 0..3 {
            worst_p = worst_p.max((a[k] - b[k]).abs());
        }
    }
    let worst_w = gpu_grid
        .water()
        .iter()
        .zip(cpu.water().iter())
        .fold(0.0f32, |m, (a, b)| m.max((a - b).abs()));
    eprintln!("step_grid vs CPU: worst pigment |Δ| = {worst_p:.6}, worst water |Δ| = {worst_w:.6}");
    assert!(worst_p < 2.0e-2, "step_grid pigment diverged from CPU: {worst_p}");
    assert!(worst_w < 2.0e-2, "step_grid water diverged from CPU: {worst_w}");
}

#[test]
#[ignore = "needs a GPU device"]
fn step_resident_matches_classic_step_with_deposit() {
    // W15.3 resident path: depositing the seed pigment into a zeroed `pig_a` then
    // running diffuse+advect must equal the classic `step` (uploaded pigment +
    // diffuse+advect). Evaporation OFF isolates the deposit/residency equivalence
    // (the resident path moves evaporation to the CPU water mirror). `pig_a += 0`
    // for a fresh buffer is exact, so this should be bit-identical.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (40u32, 36u32);
    let steps = 10u32;
    let params = FluidParams { evaporation: 0.0, ..FluidParams::default() };
    let seed = seeded_grid(w, h);
    let pig4: Vec<[f32; 4]> = seed.pigment().iter().map(|p| [p[0], p[1], p[2], 0.0]).collect();

    // Path A — classic step (uploaded pigment).
    let a = FluidSolver::new(&gpu.device, w, h);
    a.set_params(&gpu.queue, &params);
    a.upload(&gpu.queue, seed.water(), seed.paper(), &pig4);
    a.step(&gpu.device, &gpu.queue, steps);
    let pa = a.read_pigment(&gpu.device, &gpu.queue);

    // Path B — resident: deposit the seed into a zeroed pig_a, same water, no evap.
    let b = FluidSolver::new(&gpu.device, w, h);
    b.set_params(&gpu.queue, &params);
    b.upload_paper(&gpu.queue, seed.paper());
    b.clear_resident_pigment(&gpu.queue);
    b.step_resident(&gpu.device, &gpu.queue, seed.water(), &pig4, steps);
    let pb = b.read_pigment(&gpu.device, &gpu.queue);

    let worst = pa
        .iter()
        .zip(&pb)
        .flat_map(|(x, y)| (0..3).map(move |k| (x[k] - y[k]).abs()))
        .fold(0.0f32, f32::max);
    eprintln!("resident vs classic (evap off): worst pigment |Δ| = {worst:.8}");
    assert!(worst < 1.0e-6, "resident deposit+step must match classic step: {worst}");
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
