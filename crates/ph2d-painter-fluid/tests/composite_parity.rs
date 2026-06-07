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
use ph2d_painter_brush::wet_composite::{composite_wet_field_cpu, prepare_wet_composite};
use ph2d_painter_fluid::FluidCompositor;

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
    g.splat(gw as f32 * 0.5, gh as f32 * 0.5, gw as f32 * 0.4, 0.7, [0.0, 0.0, 0.0]);
    g.splat(gw as f32 * 0.5, gh as f32 * 0.5, 7.0, 0.8, [0.55, 0.42, 0.02]);
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
    let stroke_linear = [0.8f32, 0.6, 0.02]; // yellow, as the tool derives from OKLab
    let region = (0u32, 0u32, gw - 1, gh - 1);
    let backdrop = split_backdrop(cw, ch);

    // CPU reference (the parity ground truth).
    let brush = prepare_wet_composite(pig, stroke_linear);
    let mut cpu_canvas = backdrop.clone();
    composite_wet_field_cpu(
        &mut cpu_canvas, &backdrop, pig, gw, gh, cw, ch, SCALE, COVERAGE_K, &brush, region,
    );

    // GPU: same inputs.
    let pig4: Vec<[f32; 4]> = pig.iter().map(|p| [p[0], p[1], p[2], 0.0]).collect();
    let compositor = FluidCompositor::new(&gpu.device);
    let gpu_canvas = compositor.composite_to_rgba(
        &gpu.device, &gpu.queue, gw, gh, cw, ch, SCALE, COVERAGE_K, &pig4, &backdrop, &brush, region,
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
    assert!(changed > 200, "GPU composited too few pixels — shader likely dead");
    assert!(
        mean < 2.0e-3,
        "GPU↔CPU mean |Δ| {mean} too high — the WGSL diverges from the composite reference"
    );
    assert!(worst_n < 1.5e-2, "GPU↔CPU worst |Δ| {worst_n} ({worst} LSB) too high");
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
    let stroke_linear = [0.8f32, 0.6, 0.02];
    let region = (0u32, 0u32, gw - 1, gh - 1);
    let backdrop = split_backdrop(cw, ch);
    let brush = prepare_wet_composite(pig, stroke_linear);
    let pig4: Vec<[f32; 4]> = pig.iter().map(|p| [p[0], p[1], p[2], 0.0]).collect();
    let compositor = FluidCompositor::new(&gpu.device);
    let out = compositor.composite_to_rgba(
        &gpu.device, &gpu.queue, gw, gh, cw, ch, SCALE, COVERAGE_K, &pig4, &backdrop, &brush, region,
    );

    // K–M signature: the wettest opaque-blue pixel (canvas centre, left of mid) is
    // green-dominant — the yellow-over-blue glaze (a linear "over" never is).
    let cyx = (ch / 2 * cw + (cw / 2 - 3)) * 4;
    let i = cyx as usize;
    let (r, g, b) = (out[i] as i32, out[i + 1] as i32, out[i + 2] as i32);
    eprintln!("GPU yellow-over-blue = [{r},{g},{b}]");
    assert!(g > r && g > b, "GPU K–M glaze must be green-dominant over blue: [{r},{g},{b}]");

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
