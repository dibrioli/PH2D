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
use ph2d_painter_fluid::{DabGpu, FluidSolver};

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
    let mut dp = DiffusionParams::default();
    dp.evaporation = 0.0;
    dp.deposition = 0.0;
    dp.deposition_dry = 0.0;
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
        for k in 0..PIG_CH {
            assert!(c[k].is_finite(), "pigment channel {k} has NaN/Inf");
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
    let mut dp = DiffusionParams::default();
    dp.evaporation = 0.0;
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
    let mut dp = DiffusionParams::default();
    dp.deposition = 0.03;
    dp.deposition_dry = 0.06;
    dp.granulation = 1.5;
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
