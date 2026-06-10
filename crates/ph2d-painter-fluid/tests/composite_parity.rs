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
        0.0,
    );
    g.splat(
        gw as f32 * 0.5,
        gh as f32 * 0.5,
        7.0,
        0.8,
        [0.55, 0.42, 0.02],
        0.55 + 0.42 + 0.02,
        0.0,
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
        &backdrop, // ADR-0084 paper-reveal: paper == backdrop ⇒ exact no-op
        None,      // ADR-0084: dormant backdrop-lift (no lift buffer)
        None,      // wet-sheen water: dormant (no live water buffer)
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
        &backdrop, // ADR-0084 paper-reveal: paper == backdrop ⇒ exact no-op
        None,      // ADR-0084: dormant backdrop-lift (no lift buffer)
        None,      // wet-sheen water: dormant (no live water buffer)
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
            &backdrop, // ADR-0084 paper-reveal: paper == backdrop ⇒ exact no-op
            None,      // ADR-0084: dormant backdrop-lift (no lift buffer)
            None,      // wet-sheen water: dormant (no live water buffer)
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
        0.0,
    );
    cpu_grid.splat(
        gw as f32 * 0.5,
        gh as f32 * 0.5,
        7.0,
        0.8,
        [0.55, 0.42, 0.02],
        0.55 + 0.42 + 0.02,
        0.0,
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
        0.0,
    );
    seed.splat(
        gw as f32 * 0.5,
        gh as f32 * 0.5,
        7.0,
        0.8,
        [0.55, 0.42, 0.02],
        0.55 + 0.42 + 0.02,
        0.0,
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
    grid.splat(
        ovx - 2.0,
        ovy,
        7.0,
        0.8,
        blue,
        blue[0] + blue[1] + blue[2],
        0.0,
    );
    grid.splat(
        ovx + 2.0,
        ovy,
        7.0,
        0.8,
        yellow,
        yellow[0] + yellow[1] + yellow[2],
        0.0,
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

#[test]
#[ignore = "needs a GPU device"]
fn composite_lift_reveals_paper_not_transparency() {
    // ADR-0084 paper-reveal: lifting returns the pixel toward the session's ORIGINAL paper
    // content (Curtis desorption / Rebelle) — NEVER toward transparency. The old model dropped
    // the backdrop ALPHA (`eff_back_a = back_a·(1−lf)`), which over an OPAQUE canvas punched
    // holes revealing the dark editor background behind the sprite (the dark-blur smoke).
    // Setup: opaque beige backdrop = paper everywhere, a red square PAINTED into the backdrop
    // (backdrop ≠ paper only there), `lifted_frac = 1` over the square, NO wet pigment.
    // The square must come back ~beige with alpha STAYING 255; outside the square the output
    // must be byte-identical to the backdrop (`lf = 0` ⇒ the byte-exact bare-paper copy).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (gw, gh) = (32u32, 32u32);
    let scale = 1u32; // grid cell == canvas pixel, so the lf mask maps 1:1
    let (cw, ch) = (gw, gh);
    let beige = [228u8, 214, 184, 255];
    let red = [200u8, 30, 30, 255];
    let in_square = |x: u32, y: u32| (8..24).contains(&x) && (8..24).contains(&y);

    // Paper = solid opaque beige; backdrop = paper + the painted red square.
    let mut paper = vec![0u8; (cw * ch * 4) as usize];
    for px in paper.chunks_exact_mut(4) {
        px.copy_from_slice(&beige);
    }
    let mut backdrop = paper.clone();
    for y in 0..ch {
        for x in 0..cw {
            if in_square(x, y) {
                let i = ((y * cw + x) * 4) as usize;
                backdrop[i..i + 4].copy_from_slice(&red);
            }
        }
    }

    // lifted_frac: 1.0 over the square cells, 0 elsewhere.
    let mut lf = vec![0.0f32; (gw * gh) as usize];
    for gy in 0..gh {
        for gx in 0..gw {
            if in_square(gx, gy) {
                lf[(gy * gw + gx) as usize] = 1.0;
            }
        }
    }
    let lf_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test lifted_frac (lf=1 over square)"),
        size: (lf.len() * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    gpu.queue
        .write_buffer(&lf_buf, 0, bytemuck::cast_slice(&lf));

    // NO wet pigment: a fresh (zero) field buffer — every glaze sample is dry.
    use ph2d_painter_brush::diffusion::PIG_CH;
    let pig_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test zero pigment field"),
        size: ((gw * gh) as usize * PIG_CH * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    let mut compositor = FluidCompositor::new(&gpu.device);
    compositor.begin_stroke(
        &gpu.device,
        &gpu.queue,
        gw,
        gh,
        cw,
        ch,
        scale,
        COVERAGE_K,
        1,
        &pig_buf,
        &backdrop,
        &paper,
        Some(&lf_buf),
        None, // wet-sheen water: dormant
    );
    let region = (0u32, 0u32, gw - 1, gh - 1);
    let (band, (_, py_lo, _, py_hi)) = compositor.composite_frame(&gpu.device, &gpu.queue, region);
    assert_eq!(py_lo, 0, "full-canvas region starts at row 0");
    assert_eq!(py_hi, ch, "full-canvas region covers every row");
    assert_eq!(band.len(), (cw * ch * 4) as usize, "full-canvas band");

    let mut worst_rgb = 0u8; // worst lifted-pixel |Δ| vs the paper, RGB only
    for y in 0..ch {
        for x in 0..cw {
            let i = ((y * cw + x) * 4) as usize;
            let px = &band[i..i + 4];
            if in_square(x, y) {
                // (a) Lifted: the red paint came back ~beige (sRGB→linear→sRGB roundtrip ⇒ a
                // few-LSB tolerance) and the ALPHA STAYED 255 — the key regression: lifting an
                // opaque canvas must NOT punch an alpha hole.
                for c in 0..3 {
                    worst_rgb = worst_rgb.max(px[c].abs_diff(beige[c]));
                    assert!(
                        px[c].abs_diff(beige[c]) <= 2,
                        "lifted pixel must return to the paper @({x},{y}) ch{c}: {px:?} vs {beige:?}"
                    );
                }
                assert_eq!(
                    px[3], 255,
                    "lifted pixel alpha must STAY opaque @({x},{y}): {px:?}"
                );
            } else {
                // (b) lf = 0 ⇒ the byte-exact backdrop copy (non-destructive invariant).
                assert_eq!(
                    px,
                    &backdrop[i..i + 4],
                    "untouched pixel must be byte-identical to the backdrop @({x},{y})"
                );
            }
        }
    }
    eprintln!(
        "paper-reveal: lifted square back to paper, worst RGB |Δ| = {worst_rgb} LSB, alpha = 255 \
         everywhere; outside byte-identical"
    );
}

// ─── E4: premultiplied preview-texture output ────────────────────────────────

/// LOCAL mirror of the CPU premultiply the shell applies before uploading the live
/// preview (`shells/desktop/src/render_loop/painter_bridge.rs` calls
/// `ph2d_render::premultiply_rgba8`): `rgb' = (rgb·a + 127) / 255` — integer
/// round-to-nearest on the sRGB-ENCODED bytes (no linearisation), alpha unchanged.
/// MUST stay byte-identical to that fn; the gates below prove the GPU
/// `cs_premul_tex`/`cs_premul_init` match it byte-for-byte, which is what makes the
/// E4 texture path a drop-in replacement for readback + CPU premultiply + re-upload.
fn premultiply_rgba8_local(rgba: &mut [u8]) {
    for px in rgba.chunks_exact_mut(4) {
        let a = px[3] as u32;
        for c in &mut px[..3] {
            *c = ((*c as u32 * a + 127) / 255) as u8;
        }
    }
}

/// Read the canvas-res rgba8 preview texture back to tightly-packed bytes.
/// `copy_texture_to_buffer` requires `bytes_per_row` aligned to 256 — copy padded,
/// then strip the padding per row.
fn read_texture_rgba8(gpu: &GpuContext, tex: &wgpu::Texture, cw: u32, ch: u32) -> Vec<u8> {
    let padded_bpr =
        (cw * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test preview-tex readback"),
        size: u64::from(padded_bpr) * u64::from(ch),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("test preview-tex copy"),
        });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(ch),
            },
        },
        wgpu::Extent3d {
            width: cw,
            height: ch,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([enc.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    rx.recv().expect("map channel").expect("mapped");
    let mapped = staging.slice(..).get_mapped_range();
    let mut out = Vec::with_capacity((cw * ch * 4) as usize);
    for y in 0..ch {
        let off = (y * padded_bpr) as usize;
        out.extend_from_slice(&mapped[off..off + (cw * 4) as usize]);
    }
    drop(mapped);
    staging.unmap();
    out
}

#[test]
#[ignore = "needs a GPU device"]
fn gpu_preview_texture_matches_cpu_premultiply() {
    // E4 parity gate: the GPU texture path (cs_composite → cs_premul_tex → storage
    // texture, NO readback) must equal "readback the straight composite + CPU
    // premultiply" byte-for-byte — the texture the renderer samples is byte-identical
    // to the bytes the old round-trip uploaded. Pixels outside the composited rect
    // must hold the premultiplied BACKDROP (the begin_stroke init).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (gw, gh) = (40u32, 32u32);
    let (cw, ch) = (gw * SCALE, gh * SCALE);
    let grid = seeded_field(gw, gh);
    let pig = grid.pigment();
    let region = (0u32, 0u32, gw - 1, gh - 1);
    // split_backdrop covers a=255 and a=0; the composited band adds the full range of
    // fractional alphas (wet coverage), so the premul rounding is exercised broadly.
    let backdrop = split_backdrop(cw, ch);
    let solver = FluidSolver::new(&gpu.device, gw, gh);
    solver.upload(&gpu.queue, grid.water(), grid.paper(), pig);
    let mut compositor = FluidCompositor::new(&gpu.device);
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
        &backdrop, // ADR-0084 paper-reveal: paper == backdrop ⇒ exact no-op
        None,      // ADR-0084: dormant backdrop-lift (no lift buffer)
        None,      // wet-sheen water: dormant (no live water buffer)
    );

    // Ground truth: the existing sync readback (straight sRGB8 band) + the LOCAL CPU
    // premultiply, blitted over the premultiplied backdrop exactly like the shell did.
    let (band, rect) = compositor.composite_frame(&gpu.device, &gpu.queue, region);
    let (px_lo, py_lo, px_hi, py_hi) = rect;
    assert!(!band.is_empty(), "sync composite produced a band");
    let mut expected = backdrop.clone();
    premultiply_rgba8_local(&mut expected);
    for y in py_lo..py_hi {
        for x in px_lo..px_hi {
            let bi = (((y - py_lo) * cw + x) * 4) as usize;
            let mut px = [band[bi], band[bi + 1], band[bi + 2], band[bi + 3]];
            premultiply_rgba8_local(&mut px);
            expected[((y * cw + x) * 4) as usize..][..4].copy_from_slice(&px);
        }
    }

    // Texture path: same field, same region — then read the texture back.
    let rect_tex = compositor
        .composite_frame_to_texture(&gpu.device, &gpu.queue, region)
        .expect("non-empty region composites");
    assert_eq!(rect_tex, rect, "texture path rect must match the sync rect");
    let tex = compositor.preview_texture().expect("stroke state live");
    let got = read_texture_rgba8(&gpu, tex, cw, ch);

    assert_eq!(expected.len(), got.len());
    let mut worst = 0u8;
    let mut worst_at = 0usize;
    for (k, (e, g)) in expected.iter().zip(got.iter()).enumerate() {
        let d = e.abs_diff(*g);
        if d > worst {
            worst = d;
            worst_at = k;
        }
    }
    eprintln!(
        "preview texture ↔ CPU premultiply: worst |Δ| = {worst} LSB @byte {worst_at} \
         ({cw}×{ch}, rect {rect:?})"
    );
    // Integer premul on both sides (u32 on CPU, u32 in WGSL + an exact-integer unorm
    // store) ⇒ byte-exact, not just rounding-bound.
    assert_eq!(
        worst, 0,
        "GPU premul texel diverged from the CPU premultiply at byte {worst_at} \
         (expected {} got {})",
        expected[worst_at], got[worst_at]
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn preview_texture_initialized_to_backdrop() {
    // E4: after begin_stroke ONLY (no composite frames), the texture must equal the
    // PREMULTIPLIED backdrop everywhere — the per-frame regions only cover the wet
    // band, so never-composited pixels must already show the backdrop. Use a backdrop
    // sweeping rgb AND fractional alphas so the integer premul rounding is exercised
    // across the range (split_backdrop only has a ∈ {0, 255}).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (gw, gh) = (40u32, 32u32);
    let (cw, ch) = (gw * SCALE, gh * SCALE);
    let mut backdrop = vec![0u8; (cw * ch * 4) as usize];
    for y in 0..ch {
        for x in 0..cw {
            let i = ((y * cw + x) * 4) as usize;
            backdrop[i] = (x * 7 + y) as u8;
            backdrop[i + 1] = (y * 5 + 3 * x) as u8;
            backdrop[i + 2] = (x ^ y) as u8;
            backdrop[i + 3] = (x * 13 + y * 29) as u8; // fractional alphas incl. 0
        }
    }
    // Zero pigment field — the stroke hasn't composited anything yet.
    use ph2d_painter_brush::diffusion::PIG_CH;
    let pig_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test zero pigment field (E4 init)"),
        size: ((gw * gh) as usize * PIG_CH * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let mut compositor = FluidCompositor::new(&gpu.device);
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
        &pig_buf,
        &backdrop,
        &backdrop, // ADR-0084 paper-reveal: paper == backdrop ⇒ exact no-op
        None,      // ADR-0084: dormant backdrop-lift (no lift buffer)
        None,      // wet-sheen water: dormant (no live water buffer)
    );
    let tex = compositor.preview_texture().expect("stroke state live");
    let got = read_texture_rgba8(&gpu, tex, cw, ch);
    let mut expected = backdrop;
    premultiply_rgba8_local(&mut expected);
    let mut worst = 0u8;
    for (e, g) in expected.iter().zip(got.iter()) {
        worst = worst.max(e.abs_diff(*g));
    }
    eprintln!("preview texture init ↔ CPU premultiplied backdrop: worst |Δ| = {worst} LSB");
    assert_eq!(
        expected, got,
        "begin_stroke must initialize the texture to the premultiplied backdrop"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn gpu_straight_texture_matches_out_buf_bytes() {
    // E5 parity gate: the straight-texture path (cs_composite → cs_straight_tex,
    // NO premultiply) must hold EXACTLY the straight-sRGB8 `out_buf` bytes over
    // the composited rect, and the STRAIGHT backdrop everywhere else (the lazy
    // `cs_straight_init` seed) — this texture is what the shell injects into the
    // GPU layer compositor's slice cache (its slices are straight; premul happens
    // at the end of the layer chain). `unpack4x8unorm` → rgba8unorm store is an
    // exact byte round-trip, so the gate is byte-exact, not rounding-bound.
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
        &backdrop, // ADR-0084 paper-reveal: paper == backdrop ⇒ exact no-op
        None,      // ADR-0084: dormant backdrop-lift (no lift buffer)
        None,      // wet-sheen water: dormant (no live water buffer)
    );
    assert!(
        compositor.straight_texture().is_none(),
        "the straight texture is LAZY — absent until the first straight frame"
    );

    // Ground truth: the sync readback (straight sRGB8 band) blitted over the
    // STRAIGHT backdrop — no premultiply anywhere.
    let (band, rect) = compositor.composite_frame(&gpu.device, &gpu.queue, region);
    let (px_lo, py_lo, px_hi, py_hi) = rect;
    assert!(!band.is_empty(), "sync composite produced a band");
    let mut expected = backdrop.clone();
    for y in py_lo..py_hi {
        for x in px_lo..px_hi {
            let bi = (((y - py_lo) * cw + x) * 4) as usize;
            expected[((y * cw + x) * 4) as usize..][..4].copy_from_slice(&band[bi..bi + 4]);
        }
    }

    // Straight-texture path: same field, same region — then read the texture back.
    let rect_tex = compositor
        .composite_frame_to_straight_texture(&gpu.device, &gpu.queue, region)
        .expect("non-empty region composites");
    assert_eq!(
        rect_tex, rect,
        "straight path rect must match the sync rect"
    );
    let tex = compositor
        .straight_texture()
        .expect("created on the first straight frame");
    let got = read_texture_rgba8(&gpu, tex, cw, ch);

    assert_eq!(expected.len(), got.len());
    let mut worst = 0u8;
    let mut worst_at = 0usize;
    for (k, (e, g)) in expected.iter().zip(got.iter()).enumerate() {
        let d = e.abs_diff(*g);
        if d > worst {
            worst = d;
            worst_at = k;
        }
    }
    eprintln!(
        "straight texture ↔ out_buf bytes: worst |Δ| = {worst} LSB @byte {worst_at} \
         ({cw}×{ch}, rect {rect:?})"
    );
    assert_eq!(
        worst, 0,
        "GPU straight texel diverged from the out_buf byte at {worst_at} \
         (expected {} got {})",
        expected[worst_at], got[worst_at]
    );

    // A new stroke drops the straight texture (its seed backdrop changed).
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
        &backdrop,
        None,
        None,
    );
    assert!(
        compositor.straight_texture().is_none(),
        "begin_stroke must drop the straight texture"
    );
}

// ─── Wet-paper sheen (view-only preview-texture effect) ──────────────────────

/// f32 twins of the WGSL sRGB transfer fns (`srgb_to_linear`/`linear_to_srgb`).
fn srgb_to_linear_f(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}
fn linear_to_srgb_f(linear: f32) -> f32 {
    let v = linear.clamp(0.0, 1.0);
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}
fn smoothstep_f(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// CPU mirror of the WGSL `water_bilinear` (centre-coord mapping + clamped bilinear).
fn water_bilinear_cpu(water: &[f32], gw: u32, gh: u32, inv: f32, cx: u32, cy: u32) -> f32 {
    let fx = ((cx as f32 + 0.5) * inv - 0.5).clamp(0.0, gw as f32 - 1.0);
    let fy = ((cy as f32 + 0.5) * inv - 0.5).clamp(0.0, gh as f32 - 1.0);
    let x0 = fx.floor() as u32;
    let y0 = fy.floor() as u32;
    let x1 = (x0 + 1).min(gw - 1);
    let y1 = (y0 + 1).min(gh - 1);
    let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
    let at = |x: u32, y: u32| water[(y * gw + x) as usize];
    let top = at(x0, y0) + (at(x1, y0) - at(x0, y0)) * tx;
    let bot = at(x0, y1) + (at(x1, y1) - at(x0, y1)) * tx;
    top + (bot - top) * ty
}

/// CPU mirror of the WGSL `sheen_word` applied to one straight-sRGB8 pixel in place
/// (alpha untouched). `wet <= 0` ⇒ untouched (the shader's early-out).
fn sheen_px_cpu(px: &mut [u8], wet: f32) {
    if wet <= 0.0 {
        return;
    }
    let band = 4.0 * wet * (1.0 - wet);
    for c in &mut px[..3] {
        let lin = srgb_to_linear_f(f32::from(*c) / 255.0);
        let lin = (lin * (1.0 - 0.07 * wet) + 0.05 * band).clamp(0.0, 1.0);
        *c = (linear_to_srgb_f(lin) * 255.0).round() as u8;
    }
}

#[test]
#[ignore = "needs a GPU device"]
fn wet_sheen_off_is_byte_identical() {
    // With the wet-sheen flag OFF (the default) the preview texture must be
    // byte-identical to the CPU-premultiply reference EVEN WITH a live (non-zero)
    // water buffer bound — the flag, not the binding, gates the effect. This pins
    // the non-destructive default: enabling the plumbing changes zero pixels.
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
    compositor.set_wet_sheen(false); // explicit (also the default)
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
        &backdrop,
        None,
        Some(solver.water_buffer()), // LIVE water bound — flag off must still be a no-op
    );
    let (band, rect) = compositor.composite_frame(&gpu.device, &gpu.queue, region);
    let (px_lo, py_lo, px_hi, py_hi) = rect;
    assert!(!band.is_empty(), "sync composite produced a band");
    let mut expected = backdrop.clone();
    premultiply_rgba8_local(&mut expected);
    for y in py_lo..py_hi {
        for x in px_lo..px_hi {
            let bi = (((y - py_lo) * cw + x) * 4) as usize;
            let mut px = [band[bi], band[bi + 1], band[bi + 2], band[bi + 3]];
            premultiply_rgba8_local(&mut px);
            expected[((y * cw + x) * 4) as usize..][..4].copy_from_slice(&px);
        }
    }
    let rect_tex = compositor
        .composite_frame_to_texture(&gpu.device, &gpu.queue, region)
        .expect("non-empty region composites");
    assert_eq!(rect_tex, rect);
    let tex = compositor.preview_texture().expect("stroke state live");
    let got = read_texture_rgba8(&gpu, tex, cw, ch);
    assert_eq!(
        expected, got,
        "wet_sheen OFF must be byte-identical to the CPU premultiply reference"
    );
    eprintln!("wet_sheen off: byte-identical over {cw}×{ch} (live water bound)");
}

#[test]
#[ignore = "needs a GPU device"]
fn wet_sheen_matches_cpu_reference() {
    // Flag ON + a known water field: the GPU premul texture must equal the CPU
    // reference (sync composite bytes → sRGB decode → sheen formula → encode →
    // premultiply) within ≤ 1 LSB (GPU pow lowers differently per backend). Also
    // proves the sheen NEVER touches out_buf: the sync band re-read after the
    // texture pass must be unchanged (the bake stays sheen-free ⇒ dries lighter).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (gw, gh) = (40u32, 32u32);
    let (cw, ch) = (gw * SCALE, gh * SCALE);
    let grid = seeded_field(gw, gh);
    let pig = grid.pigment();
    let water = grid.water();
    let region = (0u32, 0u32, gw - 1, gh - 1);
    let backdrop = split_backdrop(cw, ch);
    let solver = FluidSolver::new(&gpu.device, gw, gh);
    solver.upload(&gpu.queue, water, grid.paper(), pig);
    let mut compositor = FluidCompositor::new(&gpu.device);
    compositor.set_wet_sheen(true);
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
        &backdrop,
        None,
        Some(solver.water_buffer()),
    );
    // Ground truth straight bytes (out_buf is sheen-free by design).
    let (band, rect) = compositor.composite_frame(&gpu.device, &gpu.queue, region);
    let (px_lo, py_lo, px_hi, py_hi) = rect;
    assert!(!band.is_empty(), "sync composite produced a band");
    let inv = 1.0 / SCALE as f32;
    let mut expected = backdrop.clone();
    premultiply_rgba8_local(&mut expected); // out-of-rect: premul backdrop, NO sheen (init)
    let mut sheened_px = 0usize;
    for y in py_lo..py_hi {
        for x in px_lo..px_hi {
            let bi = (((y - py_lo) * cw + x) * 4) as usize;
            let mut px = [band[bi], band[bi + 1], band[bi + 2], band[bi + 3]];
            let wet = smoothstep_f(0.05, 0.45, water_bilinear_cpu(water, gw, gh, inv, x, y));
            if wet > 0.0 {
                sheened_px += 1;
            }
            sheen_px_cpu(&mut px, wet);
            premultiply_rgba8_local(&mut px);
            expected[((y * cw + x) * 4) as usize..][..4].copy_from_slice(&px);
        }
    }
    assert!(
        sheened_px > 200,
        "test field too dry ({sheened_px} wet px) — the sheen path isn't exercised"
    );
    let rect_tex = compositor
        .composite_frame_to_texture(&gpu.device, &gpu.queue, region)
        .expect("non-empty region composites");
    assert_eq!(rect_tex, rect);
    let tex = compositor.preview_texture().expect("stroke state live");
    let got = read_texture_rgba8(&gpu, tex, cw, ch);
    let mut worst = 0u8;
    let mut worst_at = 0usize;
    for (k, (e, g)) in expected.iter().zip(got.iter()).enumerate() {
        let d = e.abs_diff(*g);
        if d > worst {
            worst = d;
            worst_at = k;
        }
    }
    eprintln!(
        "wet sheen ↔ CPU reference: worst |Δ| = {worst} LSB @byte {worst_at}; \
         {sheened_px} sheened px ({cw}×{ch}, rect {rect:?})"
    );
    assert!(
        worst <= 1,
        "GPU sheen diverged from the CPU reference: {worst} LSB @byte {worst_at} \
         (expected {} got {})",
        expected[worst_at],
        got[worst_at]
    );
    // The sheen is VIEW-ONLY: out_buf (the bake source) must be unchanged after
    // the sheened texture pass — re-run the sync readback and compare.
    let (band2, rect2) = compositor.composite_frame(&gpu.device, &gpu.queue, region);
    assert_eq!(rect2, rect);
    assert_eq!(
        band2, band,
        "out_buf changed after the sheened texture pass — the sheen leaked into the bake"
    );
}
