//! W15.3 — GPU composite ↔ CPU reference parity (the correctness gate).
//!
//! The GPU [`FluidCompositor`] runs the K–M subtractive glaze that shipped on the
//! CPU (`ph2d_painter_brush::wet_composite::composite_wet_field_cpu`). Over the same
//! pigment field + backdrop + brush, the two must agree closely — that's what makes
//! the per-frame GPU composite a faithful replacement for the CPU path (removing the
//! pigment readback stall, ADR-0049 §0/§4).
//!
//! Not bit-equality: GPU lowers `exp`/`pow`/`sqrt` differently per backend, so the
//! gate is a tight mean/worst |Δ| over the RGBA8 output (a correct shader agrees to
//! a fraction of an LSB on average; a wrong one diverges by many LSB everywhere).
//!
//! `#[ignore]` — needs a real device (like the solver gate):
//!   cargo test -p ph2d-painter-fluid --features fluid --test composite_parity -- --ignored --nocapture
#![cfg(feature = "fluid")]

use ph2d_gpu::GpuContext;
use ph2d_painter_brush::diffusion::{DiffusionGrid, DiffusionParams};
use ph2d_painter_brush::wet_composite::composite_wet_field_cpu;
use ph2d_painter_fluid::{FluidCompositor, FluidParams, FluidSolver, step_cpu_reference};

const SCALE: u32 = 2;
const COVERAGE_K: f32 = 1.06;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// A bloomed yellow wash on a low-res field — smooth gradients exercise the bicubic
/// upsample; a real `step` makes the field non-uniform (catches sampling bugs).
fn seeded_field(gw: u32, gh: u32) -> DiffusionGrid {
    let mut g = DiffusionGrid::new(gw, gh, SCALE as f32);
    // Wet pool + a yellow dab straddling the canvas mid-line (where the backdrop
    // flips opaque→transparent), so one composite exercises BOTH glaze paths.
    g.splat(
        gw as f32 * 0.5,
        gh as f32 * 0.5,
        gw as f32 * 0.4,
        0.7,
        [0.0, 0.0, 0.0],
        0.0 + 0.0 + 0.0,
    );
    g.splat(
        gw as f32 * 0.5,
        gh as f32 * 0.5,
        7.0,
        0.8,
        [0.55, 0.42, 0.02],
        0.55 + 0.42 + 0.02,
    );
    let p = DiffusionParams::default();
    for _ in 0..6 {
        g.step(&p);
    }
    g
}

/// Backdrop: left half opaque saturated blue (K–M glaze → green), right half fully
/// transparent (straight-alpha "over" → pigment colour, no black fringe).
fn split_backdrop(cw: u32, ch: u32) -> Vec<u8> {
    let mut b = vec![0u8; (cw * ch * 4) as usize];
    for y in 0..ch {
        for x in 0..cw {
            let i = ((y * cw + x) * 4) as usize;
            if x < cw / 2 {
                b[i..i + 4].copy_from_slice(&[20, 40, 200, 255]);
            } // else stays [0,0,0,0]
        }
    }
    b
}

#[test]
#[ignore = "needs a GPU device"]
fn gpu_composite_matches_cpu_reference() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (gw, gh) = (40u32, 32u32);
    let (cw, ch) = (gw * SCALE, gh * SCALE);
    let grid = seeded_field(gw, gh);
    let pig = grid.pigment();
    let region = (0u32, 0u32, gw - 1, gh - 1);
    let backdrop = split_backdrop(cw, ch);

    // CPU reference (the parity ground truth).
    let mut cpu_canvas = backdrop.clone();
    composite_wet_field_cpu(
        &mut cpu_canvas,
        &backdrop,
        pig,
        gw,
        gh,
        cw,
        ch,
        SCALE,
        COVERAGE_K,
        region,
    );

    // GPU: same inputs (the 28-channel pigment field reads its own colour, ADR-0080).
    let compositor = FluidCompositor::new(&gpu.device);
    let gpu_canvas = compositor.composite_to_rgba(
        &gpu.device,
        &gpu.queue,
        gw,
        gh,
        cw,
        ch,
        SCALE,
        COVERAGE_K,
        pig,
        &backdrop,
        region,
    );

    assert_eq!(cpu_canvas.len(), gpu_canvas.len());

    // Mean + worst |Δ| over every RGBA8 byte (normalised to [0,1]).
    let mut sum = 0.0f64;
    let mut worst = 0u8;
    let mut worst_at = 0usize;
    for (k, (a, b)) in cpu_canvas.iter().zip(gpu_canvas.iter()).enumerate() {
        let d = a.abs_diff(*b);
        sum += f64::from(d);
        if d > worst {
            worst = d;
            worst_at = k;
        }
    }
    let mean = (sum / cpu_canvas.len() as f64) / 255.0;
    let worst_n = f32::from(worst) / 255.0;

    // Sanity: the GPU actually composited (not "all backdrop") — else parity is
    // meaningless. Count pixels the GPU changed vs the backdrop.
    let changed = gpu_canvas
        .chunks_exact(4)
        .zip(backdrop.chunks_exact(4))
        .filter(|(g, b)| g != b)
        .count();
    eprintln!(
        "composite GPU↔CPU: mean |Δ| = {mean:.6}, worst = {worst_n:.6} ({} LSB) @byte {worst_at}; \
         GPU changed {changed} px ({cw}×{ch})",
        worst
    );
    assert!(
        changed > 200,
        "GPU composited too few pixels — shader likely dead"
    );
    assert!(
        mean < 2.0e-3,
        "GPU↔CPU mean |Δ| {mean} too high — the WGSL diverges from the composite reference"
    );
    assert!(
        worst_n < 1.5e-2,
        "GPU↔CPU worst |Δ| {worst_n} ({worst} LSB) too high"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn composite_rows_matches_full_band() {
    // The shell per-frame path reads back only the wet row band; it must equal the
    // corresponding rows of the full-canvas composite (guards the offset/slicing).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (gw, gh) = (40u32, 32u32);
    let (cw, ch) = (gw * SCALE, gh * SCALE);
    let grid = seeded_field(gw, gh);
    let pig = grid.pigment();
    let region = (0u32, 0u32, gw - 1, gh - 1);
    let backdrop = split_backdrop(cw, ch);
    // `composite_to_rgba` uploads `pig` itself; for the rows path stash the same
    // pigment in a solver buffer (no step) and bind it — both composite the SAME field.
    let solver = FluidSolver::new(&gpu.device, gw, gh);
    solver.upload(&gpu.queue, grid.water(), grid.paper(), pig);
    let compositor = FluidCompositor::new(&gpu.device);
    let full = compositor.composite_to_rgba(
        &gpu.device,
        &gpu.queue,
        gw,
        gh,
        cw,
        ch,
        SCALE,
        COVERAGE_K,
        pig,
        &backdrop,
        region,
    );
    let (band, (px_lo, py_lo, px_hi, py_hi)) = compositor.composite_buffer_rows(
        &gpu.device,
        &gpu.queue,
        gw,
        gh,
        cw,
        ch,
        SCALE,
        COVERAGE_K,
        solver.pigment_buffer(),
        &backdrop,
        region,
    );
    // The band is full-width, so it equals the full composite's row band; the rect's
    // columns are what the shell actually blits (the sub-rect that avoids erasure).
    let lo = (py_lo * cw * 4) as usize;
    let hi = (py_hi * cw * 4) as usize;
    assert_eq!(band.len(), hi - lo, "row band length");
    assert_eq!(
        band,
        full[lo..hi],
        "row band must equal the full composite's band"
    );
    assert!(
        px_hi > px_lo && px_hi <= cw,
        "rect cols in range: {px_lo}..{px_hi}"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn composite_frame_fast_path_matches_one_shot() {
    // The persistent-buffer hot path (begin_stroke + composite_frame) must produce
    // the SAME band + rect as the per-call one-shot (composite_buffer_rows) — proves
    // the perf rewrite didn't change pixels.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (gw, gh) = (40u32, 32u32);
    let (cw, ch) = (gw * SCALE, gh * SCALE);
    let grid = seeded_field(gw, gh);
    let pig = grid.pigment();
    let region = (0u32, 0u32, gw - 1, gh - 1);
    let backdrop = split_backdrop(cw, ch);
    let solver = FluidSolver::new(&gpu.device, gw, gh);
    solver.upload(&gpu.queue, grid.water(), grid.paper(), pig);
    let mut compositor = FluidCompositor::new(&gpu.device);

    // Fast path (ss=2 to match the one-shot's WET_COMPOSITE_SS).
    compositor.begin_stroke(
        &gpu.device,
        &gpu.queue,
        gw,
        gh,
        cw,
        ch,
        SCALE,
        COVERAGE_K,
        2,
        solver.pigment_buffer(),
        &backdrop,
    );
    let (band_fast, rect_fast) = compositor.composite_frame(&gpu.device, &gpu.queue, region);

    // One-shot (the tested path).
    let (band_one, rect_one) = compositor.composite_buffer_rows(
        &gpu.device,
        &gpu.queue,
        gw,
        gh,
        cw,
        ch,
        SCALE,
        COVERAGE_K,
        solver.pigment_buffer(),
        &backdrop,
        region,
    );
    assert_eq!(rect_fast, rect_one, "fast-path rect must match one-shot");
    assert_eq!(
        band_fast, band_one,
        "fast-path band must match one-shot (byte-exact)"
    );

    // ss=1 (the full-res hot path) must also composite correctly: a wet opaque-blue
    // pixel still goes K–M green-dominant (single-sample, no supersampling).
    compositor.begin_stroke(
        &gpu.device,
        &gpu.queue,
        gw,
        gh,
        cw,
        ch,
        SCALE,
        COVERAGE_K,
        1,
        solver.pigment_buffer(),
        &backdrop,
    );
    let (band_ss1, (px_lo, py_lo, px_hi, _)) =
        compositor.composite_frame(&gpu.device, &gpu.queue, region);
    assert!(!band_ss1.is_empty(), "ss=1 composite must produce output");
    // Probe a wet pixel in the opaque-blue (left) half, inside the band.
    let cyr = ch / 2;
    let cxr = (cw / 2).saturating_sub(3).max(px_lo + 1).min(px_hi - 1);
    let i = ((cyr - py_lo) * cw + cxr) as usize * 4;
    let (r, g, b) = (
        band_ss1[i] as i32,
        band_ss1[i + 1] as i32,
        band_ss1[i + 2] as i32,
    );
    assert!(
        g >= r && g >= b,
        "ss=1 K–M still green-dominant over blue: [{r},{g},{b}]"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn composite_frame_pipelined_matches_sync() {
    // ADR-0078 S2: the pipelined (async, 1-frame-late) composite must produce the SAME
    // pixels as the synchronous composite_frame — only the read timing differs (no
    // per-frame device.poll(wait) stall). Frame 1 returns empty (its band maps async);
    // frame 2 returns frame 1's band, which must equal the sync band for the same field.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (gw, gh) = (40u32, 32u32);
    let (cw, ch) = (gw * SCALE, gh * SCALE);
    let grid = seeded_field(gw, gh);
    let pig = grid.pigment();
    let region = (0u32, 0u32, gw - 1, gh - 1);
    let backdrop = split_backdrop(cw, ch);
    let solver = FluidSolver::new(&gpu.device, gw, gh);
    solver.upload(&gpu.queue, grid.water(), grid.paper(), pig);

    let begin = |c: &mut FluidCompositor| {
        c.begin_stroke(
            &gpu.device,
            &gpu.queue,
            gw,
            gh,
            cw,
            ch,
            SCALE,
            COVERAGE_K,
            1,
            solver.pigment_buffer(),
            &backdrop,
        );
    };

    // Sync reference.
    let mut sync = FluidCompositor::new(&gpu.device);
    begin(&mut sync);
    let (band_sync, rect_sync) = sync.composite_frame(&gpu.device, &gpu.queue, region);

    // Pipelined: same field, two frames; the 2nd call returns the 1st's band.
    let mut pipe = FluidCompositor::new(&gpu.device);
    begin(&mut pipe);
    let (band0, _) = pipe.composite_frame_pipelined(&gpu.device, &gpu.queue, region);
    assert!(
        band0.is_empty(),
        "first pipelined frame returns no band yet"
    );
    // Simulate the inter-frame gap: live, the GPU finishes the tiny copy within the
    // ~4 ms frame + the next frame's non-blocking poll collects it. Back-to-back in a
    // test there's no gap, so force completion here.
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let (band1, rect1) = pipe.composite_frame_pipelined(&gpu.device, &gpu.queue, region);

    assert_eq!(rect1, rect_sync, "pipelined rect (1-late) must match sync");
    assert_eq!(
        band1, band_sync,
        "pipelined band (1-late) must be byte-identical to the sync composite"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn gpu_step_then_composite_resident_matches_cpu() {
    // The END-TO-END stall-removing seam: step the field on the GPU, then composite
    // reading the RESIDENT `pig_a` buffer directly (no pigment readback between) —
    // must match the CPU `step_cpu_reference` + `composite_wet_field_cpu`. This is
    // the per-frame flow the shell will drive (ADR-0049 §0/§4).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (gw, gh) = (40u32, 32u32);
    let (cw, ch) = (gw * SCALE, gh * SCALE);
    let steps = 6u32;
    let params = FluidParams::default();
    let region = (0u32, 0u32, gw - 1, gh - 1);
    let backdrop = split_backdrop(cw, ch);

    // CPU reference: step the grid on the CPU, then composite.
    let mut cpu_grid = DiffusionGrid::new(gw, gh, SCALE as f32);
    cpu_grid.splat(
        gw as f32 * 0.5,
        gh as f32 * 0.5,
        gw as f32 * 0.4,
        0.7,
        [0.0, 0.0, 0.0],
        0.0 + 0.0 + 0.0,
    );
    cpu_grid.splat(
        gw as f32 * 0.5,
        gh as f32 * 0.5,
        7.0,
        0.8,
        [0.55, 0.42, 0.02],
        0.55 + 0.42 + 0.02,
    );
    step_cpu_reference(&mut cpu_grid, &params, steps);
    let mut cpu_canvas = backdrop.clone();
    composite_wet_field_cpu(
        &mut cpu_canvas,
        &backdrop,
        cpu_grid.pigment(),
        gw,
        gh,
        cw,
        ch,
        SCALE,
        COVERAGE_K,
        region,
    );

    // GPU: seed the SAME field, step on the GPU (pigment ends in pig_a), then
    // composite reading pig_a directly — NO pigment readback in between.
    let mut seed = DiffusionGrid::new(gw, gh, SCALE as f32);
    seed.splat(
        gw as f32 * 0.5,
        gh as f32 * 0.5,
        gw as f32 * 0.4,
        0.7,
        [0.0, 0.0, 0.0],
        0.0 + 0.0 + 0.0,
    );
    seed.splat(
        gw as f32 * 0.5,
        gh as f32 * 0.5,
        7.0,
        0.8,
        [0.55, 0.42, 0.02],
        0.55 + 0.42 + 0.02,
    );
    let solver = FluidSolver::new(&gpu.device, gw, gh);
    solver.set_params(&gpu.queue, &params);
    solver.upload(&gpu.queue, seed.water(), seed.paper(), seed.pigment());
    solver.step(&gpu.device, &gpu.queue, steps);

    // The 28-channel pigment field carries its own colour now (ADR-0080), so the
    // composite reads `pig_a` directly — no brush prep, no chromaticity readback.
    let compositor = FluidCompositor::new(&gpu.device);
    let gpu_canvas = compositor.composite_buffer(
        &gpu.device,
        &gpu.queue,
        gw,
        gh,
        cw,
        ch,
        SCALE,
        COVERAGE_K,
        solver.pigment_buffer(),
        &backdrop,
        region,
    );

    // Mean + worst |Δ| over the RGBA8 output (GPU step + GPU composite vs CPU+CPU).
    // Looser than the composite-only gate: GPU diffuse/advect lower differently per
    // backend, so the pigment field itself drifts ~1e-3 before the composite runs.
    let mut sum = 0.0f64;
    let mut worst = 0u8;
    for (a, b) in cpu_canvas.iter().zip(gpu_canvas.iter()) {
        let d = a.abs_diff(*b);
        sum += f64::from(d);
        worst = worst.max(d);
    }
    let mean = (sum / cpu_canvas.len() as f64) / 255.0;
    let worst_n = f32::from(worst) / 255.0;
    eprintln!("step+composite GPU↔CPU: mean |Δ| = {mean:.6}, worst = {worst_n:.6} ({worst} LSB)");
    assert!(mean < 4.0e-3, "end-to-end mean |Δ| {mean} too high");
    assert!(
        worst_n < 6.0e-2,
        "end-to-end worst |Δ| {worst_n} ({worst} LSB) too high"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn gpu_composite_km_signature_and_no_fringe() {
    // The two discriminant cases run ON THE GPU (proving the K–M + straight-alpha
    // paths are alive, not just a backdrop copy): yellow over opaque blue → green;
    // partial coverage over a transparent backdrop → warm pigment, no black fringe.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (gw, gh) = (40u32, 32u32);
    let (cw, ch) = (gw * SCALE, gh * SCALE);
    let grid = seeded_field(gw, gh);
    let pig = grid.pigment();
    let region = (0u32, 0u32, gw - 1, gh - 1);
    let backdrop = split_backdrop(cw, ch);
    let compositor = FluidCompositor::new(&gpu.device);
    let out = compositor.composite_to_rgba(
        &gpu.device,
        &gpu.queue,
        gw,
        gh,
        cw,
        ch,
        SCALE,
        COVERAGE_K,
        pig,
        &backdrop,
        region,
    );

    // K–M signature: the wettest opaque-blue pixel (canvas centre, left of mid) is
    // green-dominant — the yellow-over-blue glaze (a linear "over" never is).
    let cyx = (ch / 2 * cw + (cw / 2 - 3)) * 4;
    let i = cyx as usize;
    let (r, g, b) = (out[i] as i32, out[i + 1] as i32, out[i + 2] as i32);
    eprintln!("GPU yellow-over-blue = [{r},{g},{b}]");
    assert!(
        g > r && g > b,
        "GPU K–M glaze must be green-dominant over blue: [{r},{g},{b}]"
    );

    // No black fringe: every painted pixel in the transparent (right) half keeps a
    // warm hue (red ≥ blue) and is not a near-black partial-coverage ring.
    for y in 0..ch {
        for x in cw / 2..cw {
            let p = ((y * cw + x) * 4) as usize;
            if out[p + 3] > 8 {
                assert!(
                    out[p] >= out[p + 2],
                    "GPU coral keeps red≥blue (no fringe) @({x},{y}): {:?}",
                    &out[p..p + 4]
                );
                assert!(
                    out[p] as u32 + out[p + 1] as u32 + out[p + 2] as u32 > 24,
                    "GPU partial-coverage edge is not a black fringe @({x},{y}): {:?}",
                    &out[p..p + 4]
                );
            }
        }
    }
}

#[test]
#[ignore = "needs a GPU device"]
fn gpu_composite_multi_pigment_subtractive_mix_matches_cpu() {
    // ADR-0080: with the 28-channel wet field, overlapping pigments mix SUBTRACTIVELY in
    // the composite — a blue dab + an overlapping yellow dab composite to GREEN at the
    // overlap (an additive RGB average would be grey). The GPU composite reading the
    // multi-channel field must reproduce the CPU `composite_wet_field_cpu` bytes AND read
    // green-dominant at the overlap on a transparent backdrop (straight-alpha "over", so
    // the canvas colour IS the pigment colour — no backdrop tint to confound the check).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (gw, gh) = (40u32, 32u32);
    let (cw, ch) = (gw * SCALE, gh * SCALE);
    let region = (0u32, 0u32, gw - 1, gh - 1);
    // Fully transparent backdrop → the composited colour is the pigment colour itself.
    let backdrop = vec![0u8; (cw * ch * 4) as usize];

    // Overlapping blue + yellow dabs at the field centre, then a short diffusion run.
    let mut grid = DiffusionGrid::new(gw, gh, SCALE as f32);
    let blue = [0.05f32, 0.10, 0.85];
    let yellow = [0.85f32, 0.80, 0.05];
    let (ovx, ovy) = (gw as f32 * 0.5, gh as f32 * 0.5);
    grid.splat(ovx - 2.0, ovy, 7.0, 0.8, blue, blue[0] + blue[1] + blue[2]);
    grid.splat(
        ovx + 2.0,
        ovy,
        7.0,
        0.8,
        yellow,
        yellow[0] + yellow[1] + yellow[2],
    );
    let p = DiffusionParams::default();
    for _ in 0..6 {
        grid.step(&p);
    }
    let pig = grid.pigment();

    // CPU reference.
    let mut cpu_canvas = backdrop.clone();
    composite_wet_field_cpu(
        &mut cpu_canvas,
        &backdrop,
        pig,
        gw,
        gh,
        cw,
        ch,
        SCALE,
        COVERAGE_K,
        region,
    );

    // GPU.
    let compositor = FluidCompositor::new(&gpu.device);
    let gpu_canvas = compositor.composite_to_rgba(
        &gpu.device,
        &gpu.queue,
        gw,
        gh,
        cw,
        ch,
        SCALE,
        COVERAGE_K,
        pig,
        &backdrop,
        region,
    );

    assert_eq!(cpu_canvas.len(), gpu_canvas.len());
    let mut worst = 0u8;
    for (a, b) in cpu_canvas.iter().zip(gpu_canvas.iter()) {
        worst = worst.max(a.abs_diff(*b));
    }

    // The overlap canvas pixel must read green-dominant on the GPU output (subtractive mix).
    let oi = (((ovy * SCALE as f32) as u32) * cw + ((ovx * SCALE as f32) as u32)) as usize * 4;
    let (r, g, b) = (
        gpu_canvas[oi] as i32,
        gpu_canvas[oi + 1] as i32,
        gpu_canvas[oi + 2] as i32,
    );
    eprintln!(
        "GPU multi-pigment composite overlap = [{r},{g},{b}], worst |Δ| vs CPU = {} LSB",
        worst
    );
    assert!(
        g > r && g > b,
        "GPU blue⊗yellow composite overlap must be green-dominant: [{r},{g},{b}]"
    );
    assert!(
        worst <= 4,
        "GPU multi-pigment composite diverged from CPU reference: {worst} LSB"
    );
}
