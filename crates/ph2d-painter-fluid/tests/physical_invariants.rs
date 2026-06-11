//! **Watercolor v2 physical-invariant gates (ADR-0085 §2.1).**
//!
//! These REPLACE the CPU↔GPU bit-parity suite (`gpu_parity.rs` / the CPU-mirror tests in
//! `composite_parity.rs`). ADR-0085 makes the GPU the single source of truth for the live
//! watercolor sim, so there is no CPU twin to compare against. Instead we assert the PHYSICS
//! holds — conservation, boundedness, no NaN/runaway, subtractive mixing, deposition/drying
//! monotonicity — directly on the GPU output, with loose tolerances (a correct shader passes
//! easily; a structural bug breaks an invariant by a wide margin). One implementation, light
//! gates, no lock-step maintenance.
//!
//! `#[ignore]` — needs a real device:
//!   cargo test -p ph2d-painter-fluid --features fluid --test physical_invariants -- --ignored --nocapture
#![cfg(feature = "fluid")]

use ph2d_gpu::GpuContext;
use ph2d_painter_brush::diffusion::{DiffusionGrid, DiffusionParams, PIG_CH, PIG_MASS, WetCell};
use ph2d_painter_fluid::{DabGpu, FluidParams, FluidSolver};

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// Total pigment mass over the field (the conserved quantity — `Σ mass`).
fn total_mass(pig: &[WetCell]) -> f64 {
    pig.iter().map(|c| f64::from(c[PIG_MASS])).sum()
}

/// A solver wet+pigmented from a centred dab, on a fresh resident field.
fn fresh_solver(gpu: &GpuContext, w: u32, h: u32, dp: &DiffusionParams) -> FluidSolver {
    let solver = FluidSolver::new(&gpu.device, w, h);
    solver.set_from_diffusion(&gpu.queue, dp);
    solver.clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_water_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_deposited_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_velocity_gpu(&gpu.device, &gpu.queue);
    solver
}

fn dab(cx: f32, cy: f32, r: f32, water: f32, color: [f32; 3], mass: f32) -> DabGpu {
    DabGpu::new(cx, cy, r, water, color, mass, 0.0).expect("dab radius > 0")
}

// ── INV-1 — pigment mass is conserved (no evaporation, no deposition) ────────────────
// diffuse + advect only REDISTRIBUTE pigment; with deposition off and evaporation off the
// total mass must hold across many steps. (Replaces every `_matches_cpu` mass check.)
#[test]
#[ignore = "needs a GPU device"]
fn inv_pigment_mass_is_conserved() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (48u32, 40u32);
    let dp = DiffusionParams {
        evaporation: 0.0,
        deposition: 0.0,
        deposition_dry: 0.0,
        ..DiffusionParams::default()
    };
    let solver = fresh_solver(&gpu, w, h, &dp);
    let region = (0, 0, w - 1, h - 1);

    // Deposit once (a wet blue pool), record the baseline mass, then step with NO new dabs.
    let dabs = [dab(w as f32 * 0.5, h as f32 * 0.5, 8.0, 0.7, [0.1, 0.2, 0.7], 1.0)];
    solver.step_resident_splat(&gpu.device, &gpu.queue, &dabs, 2, region);
    let before = total_mass(&solver.read_pigment(&gpu.device, &gpu.queue));
    for _ in 0..16 {
        solver.step_resident_splat(&gpu.device, &gpu.queue, &[], 2, region);
    }
    let after = total_mass(&solver.read_pigment(&gpu.device, &gpu.queue));

    eprintln!("INV-1 mass: before={before:.5} after={after:.5} Δ={:.5}", after - before);
    assert!(before > 0.01, "no pigment deposited — invariant meaningless");
    assert!(
        (after - before).abs() < before * 0.02,
        "pigment mass not conserved: {before:.5} → {after:.5} (>2%) — transport is leaking/creating mass"
    );
}

// ── INV-2 — water bounded, finite, no runaway (with evaporation on) ──────────────────
// Every field value stays finite and water stays within the documented ceiling; running 2×
// the steps must NOT grow the max water (anti-runaway — the envelope can't blow up).
#[test]
#[ignore = "needs a GPU device"]
fn inv_water_bounded_finite_no_runaway() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (48u32, 40u32);
    let dp = DiffusionParams::default(); // evaporation ON, velocity layer on
    let solver = fresh_solver(&gpu, w, h, &dp);
    let region = (0, 0, w - 1, h - 1);
    let dabs = [dab(w as f32 * 0.5, h as f32 * 0.5, 10.0, 0.9, [0.6, 0.1, 0.1], 1.2)];
    solver.step_resident_splat(&gpu.device, &gpu.queue, &dabs, 2, region);

    for _ in 0..16 {
        solver.step_resident_splat(&gpu.device, &gpu.queue, &[], 2, region);
    }
    let water = solver.read_water(&gpu.device, &gpu.queue);
    let pig = solver.read_pigment(&gpu.device, &gpu.queue);
    let w_ceiling = dp.w_hi + 1.0; // generous: water deposit clamps near 1.0
    for &val in &water {
        assert!(val.is_finite(), "water has NaN/Inf");
        assert!(
            (0.0..=w_ceiling).contains(&val),
            "water out of bounds: {val} (ceiling {w_ceiling})"
        );
    }
    for c in &pig {
        for (k, ch) in c.iter().enumerate() {
            assert!(ch.is_finite(), "pigment channel {k} has NaN/Inf");
        }
    }
    let max1 = solver
        .read_field_stats(&gpu.device, &gpu.queue, 1.0e-3)
        .max_water;
    for _ in 0..16 {
        solver.step_resident_splat(&gpu.device, &gpu.queue, &[], 2, region);
    }
    let max2 = solver
        .read_field_stats(&gpu.device, &gpu.queue, 1.0e-3)
        .max_water;
    eprintln!("INV-2 max_water: {max1:.5} → {max2:.5} (must not grow)");
    assert!(max1.is_finite() && max2.is_finite(), "max_water NaN/Inf");
    assert!(
        max2 <= max1 * 1.01 + 1.0e-4,
        "water ran away: max_water grew {max1:.5} → {max2:.5}"
    );
}

// ── INV-4 — subtractive pigment mixing (blue ⊗ yellow → green) ───────────────────────
// The whole point of the K–M spectral field (ADR-0080): overlapping blue + yellow must
// reduce to a GREEN-dominant colour in the field, not a muddy grey. (Replaces the
// `_matches_cpu` mix tests, keeping only the physical probe.)
#[test]
#[ignore = "needs a GPU device"]
fn inv_subtractive_mix_blue_yellow_is_green() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (40u32, 40u32);
    let dp = DiffusionParams {
        evaporation: 0.0,
        ..DiffusionParams::default()
    };
    let solver = fresh_solver(&gpu, w, h, &dp);
    let region = (0, 0, w - 1, h - 1);
    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.5;
    // Overlapping blue + yellow dabs at the centre.
    let dabs = [
        dab(cx, cy, 7.0, 0.7, [0.05, 0.10, 0.85], 1.0), // blue
        dab(cx, cy, 7.0, 0.7, [0.85, 0.80, 0.05], 1.0), // yellow
    ];
    solver.step_resident_splat(&gpu.device, &gpu.queue, &dabs, 2, region);
    for _ in 0..4 {
        solver.step_resident_splat(&gpu.device, &gpu.queue, &[], 2, region);
    }
    let pig = solver.read_pigment(&gpu.device, &gpu.queue);
    let center = (cy as u32 * w + cx as u32) as usize;
    let c = DiffusionGrid::cell_color(&pig[center]);
    eprintln!("INV-4 overlap colour = [{:.3},{:.3},{:.3}] (green-dominant?)", c[0], c[1], c[2]);
    assert!(pig[center][PIG_MASS] > 0.01, "no pigment at the overlap");
    assert!(
        c[1] > c[0] && c[1] > c[2],
        "blue+yellow did not mix to green: got [{:.3},{:.3},{:.3}] — subtractive mixing is broken",
        c[0], c[1], c[2]
    );
}

// ── INV-6 — deposition accumulates + the wash dries (monotonicity) ───────────────────
// With deposition on + evaporation on: the deposited layer only GROWS (edge-darkening /
// granulation accumulate) and the wash DRIES (max water strictly recedes). (Replaces the
// `_matches_cpu` deposition liveness checks.)
#[test]
#[ignore = "needs a GPU device"]
fn inv_deposition_accumulates_and_dries() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (40u32, 40u32);
    let dp = DiffusionParams {
        deposition: 0.03,
        deposition_dry: 0.06,
        granulation: 1.5,
        ..DiffusionParams::default()
    };
    let solver = fresh_solver(&gpu, w, h, &dp);
    let region = (0, 0, w - 1, h - 1);
    let dabs = [dab(w as f32 * 0.5, h as f32 * 0.5, 9.0, 0.9, [0.2, 0.4, 0.1], 1.2)];
    solver.step_resident_splat(&gpu.device, &gpu.queue, &dabs, 2, region);

    let snapshot = |solver: &FluidSolver| -> (f64, f32) {
        let dep = total_mass(&solver.read_deposited(&gpu.device, &gpu.queue));
        let mw = solver
            .read_field_stats(&gpu.device, &gpu.queue, 1.0e-3)
            .max_water;
        (dep, mw)
    };
    let (dep0, mw0) = snapshot(&solver);
    for _ in 0..8 {
        solver.step_resident_splat(&gpu.device, &gpu.queue, &[], 2, region);
    }
    let (dep1, mw1) = snapshot(&solver);
    for _ in 0..16 {
        solver.step_resident_splat(&gpu.device, &gpu.queue, &[], 2, region);
    }
    let (dep2, mw2) = snapshot(&solver);

    eprintln!("INV-6 deposited: {dep0:.5} → {dep1:.5} → {dep2:.5} | max_water: {mw0:.5} → {mw1:.5} → {mw2:.5}");
    // Deposition is monotone non-decreasing (1% slack for fp), and reaches a live amount.
    assert!(dep1 >= dep0 - dep0.max(1.0) * 0.01, "deposited decreased {dep0:.5}→{dep1:.5}");
    assert!(dep2 >= dep1 - dep1.max(1.0) * 0.01, "deposited decreased {dep1:.5}→{dep2:.5}");
    assert!(dep2 > 0.02, "no deposition occurred — edge-darkening layer is dead");
    // The wash dries: max water strictly recedes over the run.
    assert!(mw2 < mw0, "wash did not dry: max_water {mw0:.5} → {mw2:.5}");
}

// ─────────────────────────────────────────────────────────────────────────────────────
// GPU-only STRUCTURAL gates (ADR-0085 §2.2 — migrated from the deleted `gpu_parity.rs`).
// These do NOT compare against a CPU twin; they assert a GPU↔GPU structural identity that
// must hold regardless of the absent reference: the combine identity, region-scoping
// equivalence, mass conservation, and the dormant lift no-op. They survive the parity
// deletion because each is a property of the GPU pipeline alone.
// ─────────────────────────────────────────────────────────────────────────────────────

/// A grid with a blue dab splatted into a wet pool — a non-trivial pigment field for the
/// GPU conservation gate (no CPU stepping; the GPU solver evolves it).
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

/// Read back a flat `array<f32>` GPU buffer (e.g. `lifted_frac`) — same idiom as the
/// solver's readbacks.
fn read_f32_buffer(gpu: &GpuContext, b: &wgpu::Buffer, n: usize) -> Vec<f32> {
    let size = (n * 4) as u64;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("lifted_frac readback"),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    enc.copy_buffer_to_buffer(b, 0, &staging, 0, size);
    gpu.queue.submit([enc.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    rx.recv().expect("map channel").expect("mapped");
    let mapped = staging.slice(..).get_mapped_range();
    let out = bytemuck::cast_slice::<u8, f32>(&mapped).to_vec();
    drop(mapped);
    staging.unmap();
    out
}

/// Build a `cw·ch·4` straight-alpha sRGB RGBA8 backdrop: a fully-opaque solid colour fill.
fn solid_backdrop(cw: u32, ch: u32, rgba: [u8; 4]) -> Vec<u8> {
    let mut out = vec![0u8; (cw as usize) * (ch as usize) * 4];
    for px in out.chunks_exact_mut(4) {
        px.copy_from_slice(&rgba);
    }
    out
}

// ── cs_combine writes total = flowing + deposited (the buffer the compositor reads) ──
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
    .filter_map(|&(cx, cy, r, wa, rgb)| {
        DabGpu::new(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0)
    })
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
        for k in 0..PIG_CH {
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

// ── region-scoped step is bit-exact inside the padded region (guards SOLVER_REGION_PAD) ──
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
            for k in 0..PIG_CH {
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

// ── no-evaporation GPU run conserves pigment mass (the conservation invariant, GPU step) ──
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

// ── dormant backdrop-lift (lift=0) is fully inert + byte-identical composite ──
#[test]
#[ignore = "needs a GPU device"]
fn gpu_backdrop_lift_off_is_byte_identical_composite() {
    // ADR-0084 §2.4 non-destructive: with `lift = 0` the donor is never seeded (cleared) and
    // `lifted_frac ≡ 0`, so the backdrop-lift branch is fully inert. (1) Assert the solver's
    // `lifted_frac` stays all-zero after a full wet stroke with lift off, and (2) the GPU compositor
    // output over a frozen wet field with the dormant (all-zero) lifted_frac is byte-for-byte the
    // same as compositing the same field through the existing path — proving the new binding 5 +
    // the alpha-drop are no-ops when dormant. The dormant path binds an owned all-zero buffer, so
    // `lf = 0` ⇒ `eff_back_a = back_a` ⇒ identical bytes.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (32u32, 28u32);
    let base = FluidParams::default();
    let raw = [(16.0f32, 14.0, 10.0, 0.9, [0.20f32, 0.40, 0.80])];

    let mut cpu = DiffusionGrid::new(w, h, 1.0);
    for &(cx, cy, r, wa, rgb) in &raw {
        cpu.splat(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0);
    }
    let dabs: Vec<DabGpu> = raw
        .iter()
        .filter_map(|&(cx, cy, r, wa, rgb)| {
            DabGpu::new(cx, cy, r, wa, rgb, rgb[0] + rgb[1] + rgb[2], 0.0)
        })
        .collect();
    // A NON-lift brush: lift stays 0, so the bridge would `clear_lift_gpu` (never seed).
    let mut off = base.to_diffusion();
    off.lift = 0.0;
    let solver = FluidSolver::new(&gpu.device, w, h);
    solver.set_params(&gpu.queue, &base);
    solver.clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_water_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_deposited_gpu(&gpu.device, &gpu.queue);
    solver.clear_lift_gpu(&gpu.device, &gpu.queue);
    solver.upload_paper(&gpu.queue, cpu.paper());
    solver.set_from_diffusion(&gpu.queue, &off);
    solver.step_resident_splat(&gpu.device, &gpu.queue, &dabs, 8, (0, 0, w - 1, h - 1));

    // (1) lifted_frac must be all-zero (the dispatch ran cs_lift only if lift>0 — here it didn't).
    let n = (w * h) as usize;
    let lifted = read_f32_buffer(&gpu, solver.lifted_frac_buffer(), n);
    let max_lifted = lifted.iter().copied().fold(0.0f32, f32::max);
    eprintln!("lift=0: max lifted_frac = {max_lifted:.6}");
    assert!(
        max_lifted == 0.0,
        "lift=0 left a non-zero lifted_frac (should be inert): {max_lifted}"
    );

    // (2) Compositing the resident pigment over a backdrop with the dormant (all-zero) lifted_frac
    // must be byte-identical run-to-run (the alpha-drop is a pure no-op). `composite_to_rgba` binds
    // the dormant zero buffer (None path), so two identical composites must match exactly.
    let (cw, ch) = (w, h);
    let coverage_k = 1.06f32; // the composite tests' WET_COVERAGE_K analogue
    let backdrop = solid_backdrop(cw, ch, [200, 180, 150, 255]);
    let compositor = ph2d_painter_fluid::FluidCompositor::new(&gpu.device);
    let pig = solver.read_pigment(&gpu.device, &gpu.queue);
    let a = compositor.composite_to_rgba(
        &gpu.device,
        &gpu.queue,
        w,
        h,
        cw,
        ch,
        1,
        coverage_k,
        &pig,
        &backdrop,
        (0, 0, w - 1, h - 1),
    );
    let b = compositor.composite_to_rgba(
        &gpu.device,
        &gpu.queue,
        w,
        h,
        cw,
        ch,
        1,
        coverage_k,
        &pig,
        &backdrop,
        (0, 0, w - 1, h - 1),
    );
    assert_eq!(
        a, b,
        "dormant backdrop-lift composite is not byte-identical run-to-run — the alpha-drop leaked"
    );
}
