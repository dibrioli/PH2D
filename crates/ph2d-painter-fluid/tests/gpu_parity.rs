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
use ph2d_painter_fluid::{DabGpu, FluidParams, FluidSolver, step_cpu_reference};

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
    g.splat(
        w as f32 * 0.5,
        h as f32 * 0.5,
        w as f32 * 0.45,
        0.5,
        [0.0, 0.0, 0.0],
    );
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
    eprintln!(
        "fluid GPU↔CPU: mean |Δ| = {mean:.6}, worst = {worst:.6}, total pigment = {total:.3} ({w}×{h}, {steps} steps)"
    );
    assert!(
        total > 0.01,
        "GPU produced no pigment — solver is dead, parity meaningless"
    );
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
    assert!(
        worst_p < 2.0e-2,
        "step_grid pigment diverged from CPU: {worst_p}"
    );
    assert!(
        worst_w < 2.0e-2,
        "step_grid water diverged from CPU: {worst_w}"
    );
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
    let params = FluidParams {
        evaporation: 0.0,
        ..FluidParams::default()
    };
    let seed = seeded_grid(w, h);
    let pig4: Vec<[f32; 4]> = seed
        .pigment()
        .iter()
        .map(|p| [p[0], p[1], p[2], 0.0])
        .collect();

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
    assert!(
        worst < 1.0e-6,
        "resident deposit+step must match classic step: {worst}"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn cs_splat_matches_cpu_splat_bit_exact() {
    // Resident-sim input pass (4K real-time arch §4): `cs_splat` must reproduce the
    // CPU `DiffusionGrid::splat` — SAME covered cells, SAME falloff. It loops the dab
    // list per cell in the SAME order with the same per-dab water clamp, so the only
    // divergence from the CPU is FMA contraction (Metal fuses `a*b+c`), worth ~1e-7 —
    // far below the diffuse/advect gather passes' ~1e-4 and invisible after the
    // composite's u8 quantization. The tight bound proves the SHAPE is exact (a wrong
    // radius / falloff / coverage-set would diverge by ≥1e-2, changing the stroke).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (53u32, 47u32); // non-multiple-of-8 → exercises the dispatch tail
    // A spread of dabs: overlapping (accumulation + per-dab water clamp), near the
    // borders (bbox clamp), a sub-pixel radius (the `r.max(0.5)` floor), and one off
    // the edge (centre outside the grid, partial coverage).
    let raw = [
        (26.0f32, 23.0, 9.0, 0.5, [0.10f32, 0.20, 0.70]),
        (24.0, 24.0, 7.0, 0.6, [0.30, 0.05, 0.05]),
        (30.0, 20.0, 5.0, 0.7, [0.00, 0.40, 0.10]),
        (2.0, 3.0, 6.0, 0.8, [0.20, 0.20, 0.20]),
        (50.0, 44.0, 8.0, 0.9, [0.50, 0.10, 0.30]),
        (10.0, 40.0, 0.3, 1.0, [0.90, 0.90, 0.10]), // sub-pixel → r.max(0.5)
        (-3.0, 12.0, 7.0, 0.4, [0.10, 0.60, 0.60]), // centre off-grid
    ];

    // CPU reference: the same `splat` calls the tool makes into the live grid.
    let mut cpu = DiffusionGrid::new(w, h, 1.0);
    for &(cx, cy, r, wa, rgb) in &raw {
        cpu.splat(cx, cy, r, wa, rgb);
    }

    // GPU: the same dabs through `cs_splat` onto a zeroed resident field.
    let dabs: Vec<DabGpu> = raw
        .iter()
        .filter_map(|&(cx, cy, r, wa, rgb)| DabGpu::new(cx, cy, r, wa, rgb))
        .collect();
    let solver = FluidSolver::new(&gpu.device, w, h);
    solver.clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_water_gpu(&gpu.device, &gpu.queue);
    solver.splat_dabs(&gpu.device, &gpu.queue, &dabs);
    let gpu_pig = solver.read_pigment(&gpu.device, &gpu.queue);
    let gpu_water = solver.read_water(&gpu.device, &gpu.queue);

    let n = (w * h) as usize;
    let mut worst_p = 0.0f32;
    let mut worst_w = 0.0f32;
    let cpu_pig = cpu.pigment();
    let cpu_water = cpu.water();
    for i in 0..n {
        for k in 0..3 {
            worst_p = worst_p.max((gpu_pig[i][k] - cpu_pig[i][k]).abs());
        }
        worst_w = worst_w.max((gpu_water[i] - cpu_water[i]).abs());
    }
    let total: f32 = gpu_pig.iter().map(|p| p[0] + p[1] + p[2]).sum();
    eprintln!(
        "cs_splat vs CPU splat: worst pigment |Δ| = {worst_p:.9}, worst water |Δ| = {worst_w:.9}, total pigment = {total:.3}"
    );
    assert!(
        total > 0.01,
        "cs_splat deposited no pigment — the dab path is dead, parity meaningless"
    );
    // Tight bound: the only legitimate divergence is FMA rounding (~1e-7). A coverage
    // / radius / falloff bug diverges by ≥1e-2.
    assert!(
        worst_p < 1.0e-6,
        "cs_splat pigment diverged from the CPU splat (worst |Δ| = {worst_p}) — shape mismatch, not FMA noise"
    );
    assert!(
        worst_w < 1.0e-6,
        "cs_splat water diverged from the CPU splat (worst |Δ| = {worst_w}) — shape mismatch, not FMA noise"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn step_resident_splat_matches_cpu_splat_then_step() {
    // The fully GPU-resident hot loop (`splat_dabs` + diffuse/advect/evaporate, no
    // upload, no readback) must equal the CPU reference: the same dabs splatted into
    // a `DiffusionGrid`, then `step×substeps`. This is what makes switching the live
    // drive over to the resident path a no-op on the look (within the diffuse/advect
    // FMA tolerance the other GPU gates use).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (44u32, 38u32);
    let substeps = 12u32;
    let params = FluidParams::default();
    let raw = [
        (22.0f32, 19.0, 8.0, 0.6, [0.10f32, 0.20, 0.70]),
        (18.0, 22.0, 6.0, 0.7, [0.30, 0.05, 0.05]),
        (28.0, 16.0, 5.0, 0.8, [0.00, 0.40, 0.10]),
    ];

    // CPU reference: splat the dabs into a fresh grid, then step.
    let mut cpu = DiffusionGrid::new(w, h, 1.0);
    for &(cx, cy, r, wa, rgb) in &raw {
        cpu.splat(cx, cy, r, wa, rgb);
    }
    step_cpu_reference(&mut cpu, &params, substeps);

    // GPU resident: clear, set params, splat the dabs + step on the GPU.
    let dabs: Vec<DabGpu> = raw
        .iter()
        .filter_map(|&(cx, cy, r, wa, rgb)| DabGpu::new(cx, cy, r, wa, rgb))
        .collect();
    let solver = FluidSolver::new(&gpu.device, w, h);
    solver.set_params(&gpu.queue, &params);
    solver.clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_water_gpu(&gpu.device, &gpu.queue);
    // Paper must match the CPU grid's paper for the gate/flow to agree.
    solver.upload_paper(&gpu.queue, cpu.paper());
    // Full-grid region → the un-scoped pass (matches the CPU full-grid step).
    solver.step_resident_splat(&gpu.device, &gpu.queue, &dabs, substeps, (0, 0, w - 1, h - 1));
    let gpu_pig = solver.read_pigment(&gpu.device, &gpu.queue);
    let gpu_water = solver.read_water(&gpu.device, &gpu.queue);

    let n = (w * h) as usize;
    let mut worst_p = 0.0f32;
    let mut worst_w = 0.0f32;
    let (cpu_pig, cpu_water) = (cpu.pigment(), cpu.water());
    for i in 0..n {
        for k in 0..3 {
            worst_p = worst_p.max((gpu_pig[i][k] - cpu_pig[i][k]).abs());
        }
        worst_w = worst_w.max((gpu_water[i] - cpu_water[i]).abs());
    }
    let total: f32 = gpu_pig.iter().map(|p| p[0] + p[1] + p[2]).sum();
    eprintln!(
        "step_resident_splat vs CPU: worst pigment |Δ| = {worst_p:.6}, worst water |Δ| = {worst_w:.6}, total = {total:.3}"
    );
    assert!(total > 0.01, "resident splat+step produced no pigment");
    assert!(
        worst_p < 2.0e-2,
        "resident splat+step pigment diverged from CPU: {worst_p}"
    );
    assert!(
        worst_w < 2.0e-2,
        "resident splat+step water diverged from CPU: {worst_w}"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn gpu_transfer_matches_cpu_deposition() {
    // ADR-0078 S3b: the GPU `cs_transfer` pass must reproduce the CPU
    // `DiffusionGrid::transfer_pigment` — BOTH the flowing pigment left behind AND the
    // deposited layer it builds (edge-darkening + granulation). Same splat + step
    // count + deposition params on each side; agree to the diffuse/advect FMA band.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (44u32, 38u32);
    let substeps = 14u32;
    let (dep, dep_dry, gran) = (0.03f32, 0.06f32, 1.5f32);
    let base = FluidParams::default();
    let raw = [
        (22.0f32, 19.0, 9.0, 0.85, [0.10f32, 0.20, 0.70]),
        (26.0, 22.0, 6.0, 0.7, [0.30, 0.05, 0.05]),
        (16.0, 17.0, 5.0, 0.6, [0.00, 0.40, 0.10]),
    ];

    // CPU reference: same splats, deposition ON, then step.
    let mut cpu = DiffusionGrid::new(w, h, 1.0);
    for &(cx, cy, r, wa, rgb) in &raw {
        cpu.splat(cx, cy, r, wa, rgb);
    }
    let mut dp = base.to_diffusion();
    dp.deposition = dep;
    dp.deposition_dry = dep_dry;
    dp.granulation = gran;
    for _ in 0..substeps {
        cpu.step(&dp);
    }

    // GPU: same field, deposition enabled via set_deposition (full-grid region).
    let dabs: Vec<DabGpu> = raw
        .iter()
        .filter_map(|&(cx, cy, r, wa, rgb)| DabGpu::new(cx, cy, r, wa, rgb))
        .collect();
    let solver = FluidSolver::new(&gpu.device, w, h);
    solver.set_params(&gpu.queue, &base);
    solver.set_deposition(&gpu.queue, dep, dep_dry, gran);
    solver.clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_water_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_deposited_gpu(&gpu.device, &gpu.queue);
    solver.upload_paper(&gpu.queue, cpu.paper());
    solver.step_resident_splat(&gpu.device, &gpu.queue, &dabs, substeps, (0, 0, w - 1, h - 1));
    let gpu_flow = solver.read_pigment(&gpu.device, &gpu.queue);
    let gpu_dep = solver.read_deposited(&gpu.device, &gpu.queue);

    let n = (w * h) as usize;
    let (cpu_flow, cpu_dep) = (cpu.pigment(), cpu.deposited());
    let mut worst_flow = 0.0f32;
    let mut worst_dep = 0.0f32;
    let mut total_dep = 0.0f32;
    for i in 0..n {
        for k in 0..3 {
            worst_flow = worst_flow.max((gpu_flow[i][k] - cpu_flow[i][k]).abs());
            worst_dep = worst_dep.max((gpu_dep[i][k] - cpu_dep[i][k]).abs());
        }
        total_dep += gpu_dep[i][0] + gpu_dep[i][1] + gpu_dep[i][2];
    }
    eprintln!(
        "cs_transfer vs CPU: worst flowing |Δ| = {worst_flow:.6}, worst deposited |Δ| = {worst_dep:.6}, GPU deposited total = {total_dep:.3}"
    );
    assert!(
        total_dep > 0.05,
        "GPU deposited nothing — cs_transfer is dead, parity meaningless"
    );
    assert!(
        worst_flow < 2.0e-2,
        "cs_transfer flowing diverged from CPU: {worst_flow}"
    );
    assert!(
        worst_dep < 2.0e-2,
        "cs_transfer deposited diverged from CPU: {worst_dep}"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn gpu_combine_equals_flowing_plus_deposited() {
    // ADR-0078 S3c: cs_combine writes `total = flowing + deposited` (the buffer the
    // compositor reads, so deposited pigment is visible). After a deposition step the
    // total must equal flowing + deposited cell-by-cell, inside the stepped region.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (40u32, 36u32);
    let base = FluidParams::default();
    let dabs: Vec<DabGpu> = [
        (20.0f32, 18.0, 8.0, 0.8, [0.2f32, 0.3, 0.6]),
        (24.0, 20.0, 5.0, 0.6, [0.4, 0.1, 0.1]),
    ]
    .iter()
    .filter_map(|&(cx, cy, r, wa, rgb)| DabGpu::new(cx, cy, r, wa, rgb))
    .collect();
    let solver = FluidSolver::new(&gpu.device, w, h);
    solver.set_params(&gpu.queue, &base);
    solver.set_deposition(&gpu.queue, 0.03, 0.06, 1.5);
    solver.clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_water_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_deposited_gpu(&gpu.device, &gpu.queue);
    solver.step_resident_splat(&gpu.device, &gpu.queue, &dabs, 12, (0, 0, w - 1, h - 1));
    let flowing = solver.read_pigment(&gpu.device, &gpu.queue);
    let deposited = solver.read_deposited(&gpu.device, &gpu.queue);
    let total = solver.read_total(&gpu.device, &gpu.queue);

    let mut worst = 0.0f32;
    let mut total_sum = 0.0f32;
    for i in 0..(w * h) as usize {
        for k in 0..3 {
            worst = worst.max((total[i][k] - (flowing[i][k] + deposited[i][k])).abs());
            total_sum += total[i][k];
        }
    }
    eprintln!("cs_combine: worst |total − (flowing+deposited)| = {worst:.9}, total mass = {total_sum:.3}");
    assert!(total_sum > 0.05, "combine produced an empty total — pass is dead");
    assert!(
        worst < 1.0e-6,
        "cs_combine must equal flowing + deposited (worst |Δ| = {worst})"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn region_scoped_step_matches_full_grid_inside_region() {
    // ADR-0078 S1: scoping the diffuse/advect/evaporate dispatch to the wet envelope
    // must NOT change the result inside the region. A cell's value depends only on its
    // N-substep dependency cone; cells well inside the padded region have their whole
    // cone updated identically to the full-grid pass → BIT-EXACT. (Cells near the
    // region edge differ because their cone reaches stale outside-region neighbours —
    // but those are never composited, hence the SOLVER_REGION_PAD invariant.)
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (64u32, 48u32);
    let substeps = 8u32;
    let params = FluidParams::default();
    // One localized dab at the centre → pigment stays well inside the grid.
    let raw = (32.0f32, 24.0, 6.0, 0.7, [0.2f32, 0.3, 0.6]);
    let dabs: Vec<DabGpu> = DabGpu::new(raw.0, raw.1, raw.2, raw.3, raw.4)
        .into_iter()
        .collect();
    let paper = DiffusionGrid::new(w, h, 1.0).paper().to_vec();

    // Full-grid reference.
    let full = FluidSolver::new(&gpu.device, w, h);
    full.set_params(&gpu.queue, &params);
    full.clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
    full.clear_resident_water_gpu(&gpu.device, &gpu.queue);
    full.upload_paper(&gpu.queue, &paper);
    full.step_resident_splat(&gpu.device, &gpu.queue, &dabs, substeps, (0, 0, w - 1, h - 1));
    let full_pig = full.read_pigment(&gpu.device, &gpu.queue);

    // Scoped: a region around the dab; the solver pads it by SOLVER_REGION_PAD.
    let scoped_region = (20u32, 12u32, 44u32, 36u32);
    let scoped = FluidSolver::new(&gpu.device, w, h);
    scoped.set_params(&gpu.queue, &params);
    scoped.clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
    scoped.clear_resident_water_gpu(&gpu.device, &gpu.queue);
    scoped.upload_paper(&gpu.queue, &paper);
    scoped.step_resident_splat(&gpu.device, &gpu.queue, &dabs, substeps, scoped_region);
    let scoped_pig = scoped.read_pigment(&gpu.device, &gpu.queue);

    // Core box (well inside the padded region, > substeps from its boundary) must be
    // bit-exact; the dab pigment lives here.
    let core = (26u32, 18u32, 38u32, 30u32);
    let mut worst_core = 0.0f32;
    let mut core_total = 0.0f32;
    for y in core.1..=core.3 {
        for x in core.0..=core.2 {
            let i = (y * w + x) as usize;
            for k in 0..3 {
                worst_core = worst_core.max((full_pig[i][k] - scoped_pig[i][k]).abs());
            }
            core_total += scoped_pig[i].iter().take(3).sum::<f32>();
        }
    }
    eprintln!("region-scoped vs full inside core: worst |Δ| = {worst_core:.9}, core pigment = {core_total:.3}");
    assert!(core_total > 0.01, "no pigment in the core — test is vacuous");
    assert!(
        worst_core < 1.0e-6,
        "region-scoped step diverged from full-grid INSIDE the region ({worst_core}) — scoping changed the visible field"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn read_field_stats_matches_cpu_max_water_and_bbox() {
    // The GPU reduction (max-water + wet bbox) must agree with the CPU
    // `max_water` / `water_bbox` it replaces (4K real-time arch §4) — otherwise the
    // dry-check fires at the wrong time or the composite envelope clips.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (50u32, 40u32);
    let threshold = 1.0e-3f32;
    let raw = [
        (25.0f32, 20.0, 7.0, 0.6, [0.2f32, 0.2, 0.2]),
        (12.0, 30.0, 5.0, 0.9, [0.1, 0.1, 0.1]),
        (40.0, 10.0, 4.0, 0.5, [0.3, 0.0, 0.0]),
    ];
    let mut cpu = DiffusionGrid::new(w, h, 1.0);
    for &(cx, cy, r, wa, rgb) in &raw {
        cpu.splat(cx, cy, r, wa, rgb);
    }
    let cpu_max = cpu.max_water();
    let cpu_bbox = cpu.water_bbox(threshold);

    let dabs: Vec<DabGpu> = raw
        .iter()
        .filter_map(|&(cx, cy, r, wa, rgb)| DabGpu::new(cx, cy, r, wa, rgb))
        .collect();
    let solver = FluidSolver::new(&gpu.device, w, h);
    solver.clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_water_gpu(&gpu.device, &gpu.queue);
    solver.splat_dabs(&gpu.device, &gpu.queue, &dabs);
    let stats = solver.read_field_stats(&gpu.device, &gpu.queue, threshold);

    eprintln!(
        "field stats: GPU max={:.6} bbox={:?} | CPU max={:.6} bbox={:?}",
        stats.max_water, stats.bbox, cpu_max, cpu_bbox
    );
    // max-water is just an atomicMax over the same f32 values → bit-identical.
    assert_eq!(
        stats.max_water, cpu_max,
        "GPU max-water must equal CPU max_water"
    );
    // The wet-cell set is identical (water came from the bit-exact-shape splat), so
    // the inclusive bbox must match exactly.
    assert_eq!(
        stats.bbox, cpu_bbox,
        "GPU wet bbox must equal CPU water_bbox"
    );
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
