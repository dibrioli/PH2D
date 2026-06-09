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
use ph2d_painter_brush::diffusion::{DiffusionGrid, DiffusionParams, PIG_MASS, PIG_STAIN, WetCell};
use ph2d_painter_fluid::{DabGpu, FluidParams, FluidSolver, step_cpu_reference};

/// Worst |Δ| over the REDUCED colour (linear-sRGB) + the mass channel of two wet
/// cells. ADR-0080: the raw K/S spectral bands span thousands for dark colours, so an
/// absolute per-channel bound is meaningless — compare the bounded, perceptual output
/// (`cell_color`) and the bounded mass instead. Both are O(1), so the gates' `2e-2`
/// tolerance stays correct.
fn cell_color_mass_delta(a: &WetCell, b: &WetCell) -> f32 {
    let ca = DiffusionGrid::cell_color(a);
    let cb = DiffusionGrid::cell_color(b);
    let mut worst = 0.0f32;
    for k in 0..3 {
        worst = worst.max((ca[k] - cb[k]).abs());
    }
    worst.max((a[PIG_MASS] - b[PIG_MASS]).abs())
}

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
        0.0 + 0.0 + 0.0,
        0.0,
    );
    g.splat(
        w as f32 * 0.42,
        h as f32 * 0.5,
        6.0,
        0.6,
        [0.1, 0.2, 0.7],
        0.1 + 0.2 + 0.7,
        0.0,
    );
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
    let solver = FluidSolver::new(&gpu.device, w, h);
    solver.set_params(&gpu.queue, &params);
    solver.upload(&gpu.queue, init.water(), init.paper(), init.pigment());
    solver.step(&gpu.device, &gpu.queue, steps);
    let gpu_pig = solver.read_pigment(&gpu.device, &gpu.queue);

    // Mean + worst |Δ| over the reduced colour + mass (ADR-0080: bounded channels).
    let n = (w * h) as usize;
    let mut sum = 0.0f64;
    let mut worst = 0.0f32;
    for i in 0..n {
        let gc = DiffusionGrid::cell_color(&gpu_pig[i]);
        let cc = DiffusionGrid::cell_color(&cpu_pig[i]);
        for k in 0..3 {
            let d = (gc[k] - cc[k]).abs();
            sum += f64::from(d);
            worst = worst.max(d);
        }
        let dm = (gpu_pig[i][PIG_MASS] - cpu_pig[i][PIG_MASS]).abs();
        sum += f64::from(dm);
        worst = worst.max(dm);
    }
    let mean = (sum / (n * 4) as f64) as f32;
    // Sanity: the field is non-trivial (a wrong "all zero" GPU would pass a Δ test
    // against a bug; assert there IS pigment so parity is meaningful).
    let total: f32 = gpu_pig.iter().map(|p| p[PIG_MASS]).sum();
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
        worst_p = worst_p.max(cell_color_mass_delta(a, b));
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

    // Path A — classic step (uploaded pigment).
    let a = FluidSolver::new(&gpu.device, w, h);
    a.set_params(&gpu.queue, &params);
    a.upload(&gpu.queue, seed.water(), seed.paper(), seed.pigment());
    a.step(&gpu.device, &gpu.queue, steps);
    let pa = a.read_pigment(&gpu.device, &gpu.queue);

    // Path B — resident: deposit the seed into a zeroed pig_a, same water, no evap.
    let b = FluidSolver::new(&gpu.device, w, h);
    b.set_params(&gpu.queue, &params);
    b.upload_paper(&gpu.queue, seed.paper());
    b.clear_resident_pigment(&gpu.queue);
    b.step_resident(&gpu.device, &gpu.queue, seed.water(), seed.pigment(), steps);
    let pb = b.read_pigment(&gpu.device, &gpu.queue);

    // This exact test just checks deposit+step == classic step; the robust low-dynamic-
    // range channel is the mass (ADR-0080), so compare it directly at the tight bound
    // (cell_color would add a spectral round-trip not under test here).
    let worst = pa
        .iter()
        .zip(&pb)
        .map(|(x, y)| (x[PIG_MASS] - y[PIG_MASS]).abs())
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
        cpu.splat(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0);
    }

    // GPU: the same dabs through `cs_splat` onto a zeroed resident field.
    let dabs: Vec<DabGpu> = raw
        .iter()
        .filter_map(|&(cx, cy, r, wa, rgb)| {
            DabGpu::new(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0)
        })
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
        // cs_splat builds the cell via the same `cell_from_color_mass`, so the robust
        // bit-exact channel is the deposited mass (ADR-0080); FMA contraction is the
        // only legitimate divergence.
        worst_p = worst_p.max((gpu_pig[i][PIG_MASS] - cpu_pig[i][PIG_MASS]).abs());
        worst_w = worst_w.max((gpu_water[i] - cpu_water[i]).abs());
    }
    let total: f32 = gpu_pig.iter().map(|p| p[PIG_MASS]).sum();
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
        cpu.splat(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0);
    }
    step_cpu_reference(&mut cpu, &params, substeps);

    // GPU resident: clear, set params, splat the dabs + step on the GPU.
    let dabs: Vec<DabGpu> = raw
        .iter()
        .filter_map(|&(cx, cy, r, wa, rgb)| {
            DabGpu::new(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0)
        })
        .collect();
    let solver = FluidSolver::new(&gpu.device, w, h);
    solver.set_params(&gpu.queue, &params);
    solver.clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_water_gpu(&gpu.device, &gpu.queue);
    // Paper must match the CPU grid's paper for the gate/flow to agree.
    solver.upload_paper(&gpu.queue, cpu.paper());
    // Full-grid region → the un-scoped pass (matches the CPU full-grid step).
    solver.step_resident_splat(
        &gpu.device,
        &gpu.queue,
        &dabs,
        substeps,
        (0, 0, w - 1, h - 1),
    );
    let gpu_pig = solver.read_pigment(&gpu.device, &gpu.queue);
    let gpu_water = solver.read_water(&gpu.device, &gpu.queue);

    let n = (w * h) as usize;
    let mut worst_p = 0.0f32;
    let mut worst_w = 0.0f32;
    let (cpu_pig, cpu_water) = (cpu.pigment(), cpu.water());
    for i in 0..n {
        worst_p = worst_p.max(cell_color_mass_delta(&gpu_pig[i], &cpu_pig[i]));
        worst_w = worst_w.max((gpu_water[i] - cpu_water[i]).abs());
    }
    let total: f32 = gpu_pig.iter().map(|p| p[PIG_MASS]).sum();
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
        cpu.splat(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0);
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
        .filter_map(|&(cx, cy, r, wa, rgb)| {
            DabGpu::new(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0)
        })
        .collect();
    let solver = FluidSolver::new(&gpu.device, w, h);
    solver.set_params(&gpu.queue, &base);
    solver.set_deposition(&gpu.queue, dep, dep_dry, gran);
    solver.clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_water_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_deposited_gpu(&gpu.device, &gpu.queue);
    solver.upload_paper(&gpu.queue, cpu.paper());
    solver.step_resident_splat(
        &gpu.device,
        &gpu.queue,
        &dabs,
        substeps,
        (0, 0, w - 1, h - 1),
    );
    let gpu_flow = solver.read_pigment(&gpu.device, &gpu.queue);
    let gpu_dep = solver.read_deposited(&gpu.device, &gpu.queue);

    let n = (w * h) as usize;
    let (cpu_flow, cpu_dep) = (cpu.pigment(), cpu.deposited());
    let mut worst_flow = 0.0f32;
    let mut worst_dep = 0.0f32;
    let mut total_dep = 0.0f32;
    for i in 0..n {
        worst_flow = worst_flow.max(cell_color_mass_delta(&gpu_flow[i], &cpu_flow[i]));
        worst_dep = worst_dep.max(cell_color_mass_delta(&gpu_dep[i], &cpu_dep[i]));
        total_dep += gpu_dep[i][PIG_MASS];
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
    .filter_map(|&(cx, cy, r, wa, rgb)| DabGpu::new(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0))
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

    // `total = flowing + deposited` is a pure per-channel addition (no dynamic-range
    // problem), so check the identity on EVERY raw channel of the wet cell (ADR-0080).
    let mut worst = 0.0f32;
    let mut total_sum = 0.0f32;
    for i in 0..(w * h) as usize {
        for k in 0..ph2d_painter_brush::diffusion::PIG_CH {
            worst = worst.max((total[i][k] - (flowing[i][k] + deposited[i][k])).abs());
        }
        total_sum += total[i][PIG_MASS];
    }
    eprintln!(
        "cs_combine: worst |total − (flowing+deposited)| = {worst:.9}, total mass = {total_sum:.3}"
    );
    assert!(
        total_sum > 0.05,
        "combine produced an empty total — pass is dead"
    );
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
    let dabs: Vec<DabGpu> = DabGpu::new(
        raw.0,
        raw.1,
        raw.2,
        raw.3,
        raw.4,
        raw.4[0] + raw.4[1] + raw.4[2],
        0.0,
    )
    .into_iter()
    .collect();
    let paper = DiffusionGrid::new(w, h, 1.0).paper().to_vec();

    // Full-grid reference.
    let full = FluidSolver::new(&gpu.device, w, h);
    full.set_params(&gpu.queue, &params);
    full.clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
    full.clear_resident_water_gpu(&gpu.device, &gpu.queue);
    full.upload_paper(&gpu.queue, &paper);
    full.step_resident_splat(
        &gpu.device,
        &gpu.queue,
        &dabs,
        substeps,
        (0, 0, w - 1, h - 1),
    );
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
            // GPU full vs GPU scoped run the identical resident path → every raw wet
            // channel must be bit-exact inside the padded core (ADR-0080).
            for k in 0..ph2d_painter_brush::diffusion::PIG_CH {
                worst_core = worst_core.max((full_pig[i][k] - scoped_pig[i][k]).abs());
            }
            core_total += scoped_pig[i][PIG_MASS];
        }
    }
    eprintln!(
        "region-scoped vs full inside core: worst |Δ| = {worst_core:.9}, core pigment = {core_total:.3}"
    );
    assert!(
        core_total > 0.01,
        "no pigment in the core — test is vacuous"
    );
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
        cpu.splat(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0);
    }
    let cpu_max = cpu.max_water();
    let cpu_bbox = cpu.water_bbox(threshold);

    let dabs: Vec<DabGpu> = raw
        .iter()
        .filter_map(|&(cx, cy, r, wa, rgb)| {
            DabGpu::new(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0)
        })
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
    let before: f32 = init.pigment().iter().map(|p| p[PIG_MASS]).sum();
    let solver = FluidSolver::new(&gpu.device, w, h);
    solver.set_params(&gpu.queue, &params);
    solver.upload(&gpu.queue, init.water(), init.paper(), init.pigment());
    solver.step(&gpu.device, &gpu.queue, 16);
    let after: f32 = solver
        .read_pigment(&gpu.device, &gpu.queue)
        .iter()
        .map(|p| p[PIG_MASS])
        .sum();
    eprintln!("fluid GPU mass: before = {before:.4}, after = {after:.4}");
    assert!(
        (after - before).abs() < before * 0.02,
        "GPU diffuse+advect must conserve pigment mass (no evaporation): {before} → {after}"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn gpu_shallow_water_matches_cpu_move_water() {
    // ADR-0078 S3d: the GPU shallow-water passes (`cs_add_forces` / `cs_divergence` /
    // `cs_clear_pressure` / `cs_jacobi` / `cs_project` / `cs_advect_velocity`) must
    // reproduce the CPU reference `DiffusionGrid::move_water` (+ velocity-mode advect) —
    // BOTH the evolved velocity field AND the pigment it transports. Same splats + step
    // count + shallow-water params on each side; agree to the diffuse/advect FMA band
    // (the result threads add_forces → 6 Jacobi sweeps → project → upwind advect, so the
    // tolerance is the looser end of the GPU gates, but a wrong port diverges by ≥1e-1).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (48u32, 40u32);
    let substeps = 16u32;
    let (vel, visc, drag, pressure) = (1.4f32, 0.1f32, 0.08f32, 0.4f32);
    let base = FluidParams::default();
    // Off-centre, overlapping dabs into a wet pool → an asymmetric flow + a ring.
    let raw = [
        (24.0f32, 20.0, 11.0, 0.9, [0.10f32, 0.20, 0.70]),
        (20.0, 22.0, 6.0, 0.7, [0.30, 0.05, 0.05]),
        (28.0, 18.0, 5.0, 0.6, [0.00, 0.40, 0.10]),
    ];

    // CPU reference: same splats, velocity layer ON, then step.
    let mut cpu = DiffusionGrid::new(w, h, 1.0);
    for &(cx, cy, r, wa, rgb) in &raw {
        cpu.splat(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0);
    }
    let mut dp = base.to_diffusion();
    dp.velocity = vel;
    dp.viscosity = visc;
    dp.drag = drag;
    dp.pressure = pressure;
    for _ in 0..substeps {
        cpu.step(&dp);
    }

    // GPU: same field, shallow-water enabled via set_shallow_water (full-grid region).
    let dabs: Vec<DabGpu> = raw
        .iter()
        .filter_map(|&(cx, cy, r, wa, rgb)| {
            DabGpu::new(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0)
        })
        .collect();
    let solver = FluidSolver::new(&gpu.device, w, h);
    solver.set_params(&gpu.queue, &base);
    solver.set_shallow_water(&gpu.queue, vel, visc, drag, pressure);
    solver.clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_water_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_velocity_gpu(&gpu.device, &gpu.queue);
    solver.upload_paper(&gpu.queue, cpu.paper());
    solver.step_resident_splat(
        &gpu.device,
        &gpu.queue,
        &dabs,
        substeps,
        (0, 0, w - 1, h - 1),
    );
    let gpu_pig = solver.read_pigment(&gpu.device, &gpu.queue);
    let gpu_vel = solver.read_velocity(&gpu.device, &gpu.queue);

    let n = (w * h) as usize;
    let cpu_pig = cpu.pigment();
    let (cpu_u, cpu_v) = cpu.velocity();
    let mut worst_pig = 0.0f32;
    let mut worst_vel = 0.0f32;
    let mut vel_mag = 0.0f32;
    for i in 0..n {
        worst_pig = worst_pig.max(cell_color_mass_delta(&gpu_pig[i], &cpu_pig[i]));
        worst_vel = worst_vel.max((gpu_vel[i][0] - cpu_u[i]).abs());
        worst_vel = worst_vel.max((gpu_vel[i][1] - cpu_v[i]).abs());
        vel_mag = vel_mag.max(gpu_vel[i][0].abs()).max(gpu_vel[i][1].abs());
    }
    let total: f32 = gpu_pig.iter().map(|p| p[PIG_MASS]).sum();
    eprintln!(
        "shallow-water GPU↔CPU: worst pigment |Δ| = {worst_pig:.6}, worst velocity |Δ| = {worst_vel:.6}, max |vel| = {vel_mag:.4}, total pigment = {total:.3}"
    );
    assert!(
        total > 0.01,
        "no pigment — the velocity advect is dead, parity meaningless"
    );
    assert!(
        vel_mag > 1.0e-3,
        "GPU velocity field is ~zero — move_water is dead, parity meaningless"
    );
    assert!(
        worst_vel < 2.0e-2,
        "GPU velocity diverged from the CPU move_water: {worst_vel}"
    );
    assert!(
        worst_pig < 2.0e-2,
        "GPU velocity-advected pigment diverged from CPU: {worst_pig}"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn gpu_capillary_matches_cpu_capillary() {
    // ADR-0078 S5: the GPU capillary passes (`cs_capillary` + `cs_copy_water`) must reproduce
    // the CPU reference `DiffusionGrid::capillary_flow` — BOTH the water wicked outward into
    // the dry-paper fringe AND the pigment that bleeds into it once the gate opens there. A
    // wet, pigmented blob on DRY paper (so there IS a dry fringe to wick into); capillary ON,
    // the velocity/deposition layers OFF (Default) to isolate the new pass. Same splats +
    // step count + params on each side; agree to the diffuse/advect FMA band (a wrong port —
    // wrong face order, missing copy-back, bad region — diverges by ≥1e-1).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (56u32, 48u32);
    let substeps = 20u32;
    let (cx, cy) = (28.0f32, 24.0);
    let raw = [
        (cx, cy, 9.0, 1.0, [0.10f32, 0.20, 0.70]),
        (24.0, 26.0, 5.0, 0.8, [0.30, 0.05, 0.05]),
    ];
    // Capillary on; no evaporation so the wick is clean; velocity/deposition off (Default).
    let dp = DiffusionParams {
        capillary: 0.2,
        evaporation: 0.0,
        ..Default::default()
    };

    // CPU reference: splat on dry paper, then step.
    let mut cpu = DiffusionGrid::new(w, h, 1.0);
    for &(sx, sy, r, wa, rgb) in &raw {
        cpu.splat(sx, sy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0);
    }
    for _ in 0..substeps {
        cpu.step(&dp);
    }

    // GPU resident: capillary enabled via the live `set_from_diffusion` entry.
    let dabs: Vec<DabGpu> = raw
        .iter()
        .filter_map(|&(sx, sy, r, wa, rgb)| {
            DabGpu::new(sx, sy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0)
        })
        .collect();
    let solver = FluidSolver::new(&gpu.device, w, h);
    solver.set_from_diffusion(&gpu.queue, &dp);
    solver.clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_water_gpu(&gpu.device, &gpu.queue);
    solver.upload_paper(&gpu.queue, cpu.paper());
    solver.step_resident_splat(
        &gpu.device,
        &gpu.queue,
        &dabs,
        substeps,
        (0, 0, w - 1, h - 1),
    );
    let gpu_pig = solver.read_pigment(&gpu.device, &gpu.queue);
    let gpu_water = solver.read_water(&gpu.device, &gpu.queue);

    let n = (w * h) as usize;
    let (cpu_pig, cpu_water) = (cpu.pigment(), cpu.water());
    let mut worst_p = 0.0f32;
    let mut worst_w = 0.0f32;
    for i in 0..n {
        worst_p = worst_p.max(cell_color_mass_delta(&gpu_pig[i], &cpu_pig[i]));
        worst_w = worst_w.max((gpu_water[i] - cpu_water[i]).abs());
    }
    // The fringe must actually have wicked OUT past the r=9 splat — else the pass is dead and
    // bit-parity is trivially satisfied by "both did nothing".
    let mut fringe = 0.0f32;
    let mut fringe_n = 0.0f32;
    for y in 0..h {
        for x in 0..w {
            let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
            // Just past the r=9 splat (which deposits ZERO water at d ≥ 9), so all water here
            // is wicked — the near fringe, where the diffusion fills first and strongest.
            if (9.5..11.5).contains(&d) {
                fringe += gpu_water[(y * w + x) as usize];
                fringe_n += 1.0;
            }
        }
    }
    let fringe = fringe / fringe_n.max(1.0);
    eprintln!(
        "capillary GPU↔CPU: worst pigment |Δ| = {worst_p:.6}, worst water |Δ| = {worst_w:.6}, gpu fringe water = {fringe:.4}"
    );
    assert!(
        fringe > 0.01,
        "GPU capillary didn't wick a fringe — the pass is dead, parity meaningless"
    );
    assert!(
        worst_w < 2.0e-2,
        "GPU capillary water diverged from CPU: {worst_w}"
    );
    assert!(
        worst_p < 2.0e-2,
        "GPU capillary-fringe pigment diverged from CPU: {worst_p}"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn gpu_maccormack_matches_cpu_sharpness() {
    // ADR-0078 S5c: the GPU MacCormack passes (`cs_advect_velocity_rev` + `cs_advect_correct`)
    // must reproduce the CPU reference `DiffusionGrid::advect_maccormack` — the sharpened,
    // error-compensated velocity advection (forward φ̂, reverse φ̄, correct `φ̂+s·½(φ−φ̄)` clamped
    // to local extrema). Same splats + velocity + sharpness on each side; agree to the advect
    // FMA band (a wrong port — wrong reverse flow, bad clamp, wrong φ̄ buffer — diverges by ≥1e-1).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (48u32, 40u32);
    let substeps = 16u32;
    let raw = [
        (24.0f32, 20.0, 11.0, 0.9, [0.10f32, 0.20, 0.70]),
        (20.0, 22.0, 6.0, 0.7, [0.30, 0.05, 0.05]),
        (28.0, 18.0, 5.0, 0.6, [0.00, 0.40, 0.10]),
    ];
    let mut dp = FluidParams::default().to_diffusion();
    dp.velocity = 1.4;
    dp.viscosity = 0.1;
    dp.drag = 0.08;
    dp.pressure = 0.4;
    dp.sharpness = 1.0; // full MacCormack

    let mut cpu = DiffusionGrid::new(w, h, 1.0);
    for &(cx, cy, r, wa, rgb) in &raw {
        cpu.splat(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0);
    }
    for _ in 0..substeps {
        cpu.step(&dp);
    }

    let dabs: Vec<DabGpu> = raw
        .iter()
        .filter_map(|&(cx, cy, r, wa, rgb)| {
            DabGpu::new(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0)
        })
        .collect();
    let solver = FluidSolver::new(&gpu.device, w, h);
    solver.set_from_diffusion(&gpu.queue, &dp);
    solver.clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_water_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_velocity_gpu(&gpu.device, &gpu.queue);
    solver.upload_paper(&gpu.queue, cpu.paper());
    solver.step_resident_splat(
        &gpu.device,
        &gpu.queue,
        &dabs,
        substeps,
        (0, 0, w - 1, h - 1),
    );
    let gpu_pig = solver.read_pigment(&gpu.device, &gpu.queue);

    let n = (w * h) as usize;
    let cpu_pig = cpu.pigment();
    let mut worst = 0.0f32;
    for i in 0..n {
        worst = worst.max(cell_color_mass_delta(&gpu_pig[i], &cpu_pig[i]));
    }
    let total: f32 = gpu_pig.iter().map(|p| p[PIG_MASS]).sum();
    eprintln!("maccormack GPU↔CPU: worst pigment |Δ| = {worst:.6}, total pigment = {total:.3}");
    assert!(
        total > 0.01,
        "no pigment — the sharpened advect is dead, parity meaningless"
    );
    assert!(
        worst < 2.0e-2,
        "GPU MacCormack diverged from CPU reference: {worst}"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn gpu_multi_pigment_subtractive_mix_matches_cpu() {
    // ADR-0080: the wet field carries 28-channel Kubelka–Munk pigment per cell, so two
    // overlapping dabs of DIFFERENT pigment mix SUBTRACTIVELY (not the old additive RGB
    // average). A blue dab + an overlapping yellow dab must read GREEN at the overlap —
    // and the GPU resident splat+step must reproduce the CPU `DiffusionGrid` field's
    // reduced colour there bit-for-bit (within the diffuse/advect FMA band). This is the
    // whole point of the multi-channel rewrite: subtractive mixing is bit-parity on GPU.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (48u32, 40u32);
    let substeps = 8u32;
    let params = FluidParams::default();
    // Two overlapping dabs at the centre: a saturated blue and a saturated yellow.
    let blue = [0.05f32, 0.10, 0.85];
    let yellow = [0.85f32, 0.80, 0.05];
    let (ovx, ovy) = (24.0f32, 20.0);
    let raw = [
        (ovx - 3.0, ovy, 8.0, 0.7, blue),
        (ovx + 3.0, ovy, 8.0, 0.7, yellow),
    ];

    // CPU reference: splat both dabs into a fresh grid, then step.
    let mut cpu = DiffusionGrid::new(w, h, 1.0);
    for &(cx, cy, r, wa, rgb) in &raw {
        cpu.splat(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0);
    }
    step_cpu_reference(&mut cpu, &params, substeps);

    // GPU resident: the SAME two dabs through the resident splat+step path.
    let dabs: Vec<DabGpu> = raw
        .iter()
        .filter_map(|&(cx, cy, r, wa, rgb)| {
            DabGpu::new(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0)
        })
        .collect();
    let solver = FluidSolver::new(&gpu.device, w, h);
    solver.set_params(&gpu.queue, &params);
    solver.clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_water_gpu(&gpu.device, &gpu.queue);
    solver.upload_paper(&gpu.queue, cpu.paper());
    solver.step_resident_splat(
        &gpu.device,
        &gpu.queue,
        &dabs,
        substeps,
        (0, 0, w - 1, h - 1),
    );
    let gpu_pig = solver.read_pigment(&gpu.device, &gpu.queue);

    // Parity over the reduced colour + mass everywhere (ADR-0080 bounded channels).
    let n = (w * h) as usize;
    let cpu_pig = cpu.pigment();
    let mut worst = 0.0f32;
    for i in 0..n {
        worst = worst.max(cell_color_mass_delta(&gpu_pig[i], &cpu_pig[i]));
    }

    // The overlap cell must read GREEN-dominant on BOTH sides — the subtractive blue⊗yellow
    // mix the multi-channel field exists to produce (an additive average would be grey).
    let oi = (ovy as u32 * w + ovx as u32) as usize;
    let gc = DiffusionGrid::cell_color(&gpu_pig[oi]);
    let cc = DiffusionGrid::cell_color(&cpu_pig[oi]);
    eprintln!(
        "multi-pigment overlap: GPU colour = {gc:?}, CPU colour = {cc:?}, worst field |Δ| = {worst:.6}"
    );
    assert!(
        gc[1] > gc[0] && gc[1] > gc[2],
        "GPU blue⊗yellow overlap must be green-dominant (subtractive mix): {gc:?}"
    );
    assert!(
        cc[1] > cc[0] && cc[1] > cc[2],
        "CPU blue⊗yellow overlap must be green-dominant (subtractive mix): {cc:?}"
    );
    assert!(
        worst < 2.0e-2,
        "GPU multi-pigment field diverged from CPU reduced colour: {worst}"
    );
}

/// Build the `DiffusionParams` for an ISOLATED lift step (ADR-0081): only the lift pass does
/// anything. `diffusivity = 0` makes `cs_diffuse` / CPU `diffuse` a no-op; `downhill =
/// flow_outward = 0` (and velocity off) make the advect flow `(0,0)` → no transport; `deposition
/// = deposition_dry = 0` keeps `cs_transfer` dormant; `evaporation = 0` freezes the water so the
/// wet gate is identical on both sides. The default wet band / permeability so the gate matches
/// the deposit-phase field. Only `lift` is live → the GPU `cs_lift` vs the CPU `lift_pigment`.
fn lift_only_params(lift: f32) -> DiffusionParams {
    DiffusionParams {
        diffusivity: 0.0,
        evaporation: 0.0,
        downhill: 0.0,
        flow_outward: 0.0,
        deposition: 0.0,
        deposition_dry: 0.0,
        granulation: 0.0,
        velocity: 0.0,
        viscosity: 0.0,
        drag: 0.0,
        pressure: 0.0,
        capillary: 0.0,
        sharpness: 0.0,
        lift,
        ..DiffusionParams::default()
    }
}

#[test]
#[ignore = "needs a GPU device"]
fn gpu_lift_matches_cpu_lift() {
    // ADR-0081: the GPU `cs_lift` pass must reproduce the CPU `DiffusionGrid::lift_pigment` —
    // the inverse of deposition. After dabbing a NON-staining pigment (staining = 0) into a wet
    // field and freezing it into the DEPOSITED layer (deposition ON, several steps), ONE lift
    // step must re-mobilize the same fraction of deposited pigment back into the FLOWING layer
    // on BOTH sides. We assert parity over the reduced colour + mass of BOTH layers (flowing AND
    // deposited), then add a staining-resist check (a staining = 1 deposit must NOT lift).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (44u32, 38u32);
    let deposit_steps = 12u32;
    let (dep, dep_dry, gran) = (0.05f32, 0.04f32, 1.2f32);
    let lift_rate = 0.6f32;
    let base = FluidParams::default();
    // Non-staining pigment dabs (staining = 0 → fully liftable) into a wet pool.
    let raw = [
        (22.0f32, 19.0, 9.0, 0.9, [0.10f32, 0.20, 0.70]),
        (26.0, 22.0, 6.0, 0.85, [0.30, 0.05, 0.05]),
        (16.0, 17.0, 5.0, 0.8, [0.00, 0.40, 0.10]),
    ];

    // ── CPU reference ───────────────────────────────────────────────────────────────────────
    // Deposit phase: splat (staining 0), deposition ON, step → builds the deposited layer.
    let mut cpu = DiffusionGrid::new(w, h, 1.0);
    for &(cx, cy, r, wa, rgb) in &raw {
        cpu.splat(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0);
    }
    let mut dep_dp = base.to_diffusion();
    dep_dp.deposition = dep;
    dep_dp.deposition_dry = dep_dry;
    dep_dp.granulation = gran;
    for _ in 0..deposit_steps {
        cpu.step(&dep_dp);
    }
    // One isolated lift step.
    cpu.step(&lift_only_params(lift_rate));

    // ── GPU ─────────────────────────────────────────────────────────────────────────────────
    let dabs: Vec<DabGpu> = raw
        .iter()
        .filter_map(|&(cx, cy, r, wa, rgb)| {
            DabGpu::new(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0)
        })
        .collect();
    let solver = FluidSolver::new(&gpu.device, w, h);
    solver.set_params(&gpu.queue, &base);
    solver.set_deposition(&gpu.queue, dep, dep_dry, gran);
    solver.clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_water_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_deposited_gpu(&gpu.device, &gpu.queue);
    solver.upload_paper(&gpu.queue, cpu.paper());
    // Deposit phase (same splats + step count + deposition params as the CPU).
    solver.step_resident_splat(
        &gpu.device,
        &gpu.queue,
        &dabs,
        deposit_steps,
        (0, 0, w - 1, h - 1),
    );
    // One isolated lift step: lift ON, everything else off (no new dabs). `set_from_diffusion`
    // pushes the full DiffusionParams (incl. `lift`); `cs_lift` runs before the no-op diffuse.
    solver.set_from_diffusion(&gpu.queue, &lift_only_params(lift_rate));
    solver.step_resident_splat(&gpu.device, &gpu.queue, &[], 1, (0, 0, w - 1, h - 1));
    let gpu_flow = solver.read_pigment(&gpu.device, &gpu.queue);
    let gpu_dep = solver.read_deposited(&gpu.device, &gpu.queue);

    // ── Parity over BOTH layers (reduced colour + mass) ───────────────────────────────────────
    let n = (w * h) as usize;
    let (cpu_flow, cpu_dep) = (cpu.pigment(), cpu.deposited());
    let mut worst_flow = 0.0f32;
    let mut worst_dep = 0.0f32;
    let mut total_lifted = 0.0f32; // GPU flowing mass after the lift (must be > 0)
    for i in 0..n {
        worst_flow = worst_flow.max(cell_color_mass_delta(&gpu_flow[i], &cpu_flow[i]));
        worst_dep = worst_dep.max(cell_color_mass_delta(&gpu_dep[i], &cpu_dep[i]));
        total_lifted += gpu_flow[i][PIG_MASS];
    }
    eprintln!(
        "cs_lift vs CPU: worst flowing |Δ| = {worst_flow:.6}, worst deposited |Δ| = {worst_dep:.6}, GPU flowing total after lift = {total_lifted:.3}"
    );
    assert!(
        total_lifted > 0.01,
        "GPU lifted nothing — cs_lift is dead, parity meaningless"
    );
    assert!(
        worst_flow < 2.0e-2,
        "cs_lift flowing diverged from CPU: {worst_flow}"
    );
    assert!(
        worst_dep < 2.0e-2,
        "cs_lift deposited diverged from CPU: {worst_dep}"
    );

    // ── Staining-resist check (ADR-0081): a STAINING pigment (staining = 1) must NOT lift ─────
    // Same field, but the dabs are fully staining → the per-cell `stain = stain_acc/mass → 1`,
    // so `rate = lift·wet·(1 − stain) = 0`. The deposited layer is unchanged by the lift step;
    // the GPU must reproduce that resist exactly (the WHOLE point of the staining accumulator).
    let mut cpu_s = DiffusionGrid::new(w, h, 1.0);
    for &(cx, cy, r, wa, rgb) in &raw {
        cpu_s.splat(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 1.0);
    }
    for _ in 0..deposit_steps {
        cpu_s.step(&dep_dp);
    }
    let dabs_s: Vec<DabGpu> = raw
        .iter()
        .filter_map(|&(cx, cy, r, wa, rgb)| {
            DabGpu::new(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 1.0)
        })
        .collect();
    let solver_s = FluidSolver::new(&gpu.device, w, h);
    solver_s.set_params(&gpu.queue, &base);
    solver_s.set_deposition(&gpu.queue, dep, dep_dry, gran);
    solver_s.clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
    solver_s.clear_resident_water_gpu(&gpu.device, &gpu.queue);
    solver_s.clear_resident_deposited_gpu(&gpu.device, &gpu.queue);
    solver_s.upload_paper(&gpu.queue, cpu_s.paper());
    solver_s.step_resident_splat(
        &gpu.device,
        &gpu.queue,
        &dabs_s,
        deposit_steps,
        (0, 0, w - 1, h - 1),
    );
    let dep_before: Vec<WetCell> = solver_s.read_deposited(&gpu.device, &gpu.queue);
    solver_s.set_from_diffusion(&gpu.queue, &lift_only_params(lift_rate));
    solver_s.step_resident_splat(&gpu.device, &gpu.queue, &[], 1, (0, 0, w - 1, h - 1));
    let dep_after: Vec<WetCell> = solver_s.read_deposited(&gpu.device, &gpu.queue);

    // The staining deposit must (a) carry stain ≈ mass (stain_acc/mass → 1) and (b) be
    // unchanged by the lift, on BOTH the GPU (before vs after) and vs the CPU.
    let cpu_s_dep = cpu_s.deposited();
    let mut worst_resist = 0.0f32; // GPU before vs after the lift (must stay put)
    let mut worst_s_parity = 0.0f32; // GPU after vs CPU after
    let mut max_stain_ratio = 0.0f32;
    for i in 0..n {
        worst_resist = worst_resist.max(cell_color_mass_delta(&dep_before[i], &dep_after[i]));
        worst_s_parity = worst_s_parity.max(cell_color_mass_delta(&dep_after[i], &cpu_s_dep[i]));
        let m = dep_after[i][PIG_MASS];
        if m > 1e-3 {
            max_stain_ratio = max_stain_ratio.max(dep_after[i][PIG_STAIN] / m);
        }
    }
    eprintln!(
        "cs_lift staining resist: GPU deposited before↔after |Δ| = {worst_resist:.6}, GPU↔CPU after |Δ| = {worst_s_parity:.6}, max stain ratio = {max_stain_ratio:.3}"
    );
    assert!(
        max_stain_ratio > 0.9,
        "staining deposit should carry stain ≈ 1 (got {max_stain_ratio}) — the accumulator is broken"
    );
    assert!(
        worst_resist < 2.0e-2,
        "staining pigment LIFTED on the GPU (should resist): {worst_resist}"
    );
    assert!(
        worst_s_parity < 2.0e-2,
        "GPU staining-resist deposited diverged from CPU: {worst_s_parity}"
    );
}
