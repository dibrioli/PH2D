//! W15.3 4K real-time arch — per-frame cost of the GPU-resident hot loop, MEASURED
//! on a real device, AFTER the resident-core rewrite (E1+E2+E3). The old §4 table
//! profiled the deposit path (full-grid upload + CPU O(grid) alloc/scan); this
//! re-measures the new path so the E4 decision (kill the per-frame composite
//! readback) is data-driven, not assumed.
//!
//! Reports, per canvas size, the wall-clock of:
//!   - `step` = splat + diffuse/advect/evaporate (GPU-resident, no readback). Timed
//!     with a trailing `read_field_stats` poll so the async GPU work is actually
//!     waited on (otherwise we'd time only the CPU submit).
//!   - `step+composite` = the real per-frame cost: `step_resident_splat` then
//!     `composite_frame` (whose `device.poll(wait)` readback is the E4 target). The
//!     gap above `step` ≈ the composite compute + the band readback.
//!   - run for a representative stroke BAND (≈20% canvas height) and a FULL-canvas
//!     wash (worst case) — the readback scales with the band, so the band-vs-full
//!     gap reveals how much the readback actually costs.
//!
//! `#[ignore]` + needs a device + RELEASE for meaningful numbers (dev = opt0):
//!   cargo test -p ph2d-painter-fluid --features fluid --release --test perf_resident -- --ignored --nocapture
#![cfg(feature = "fluid")]

use ph2d_gpu::GpuContext;
use ph2d_painter_brush::diffusion::PIG_CH;
use ph2d_painter_fluid::{DabGpu, FluidCompositor, FluidParams, FluidSolver};
use std::time::Instant;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// A handful of dabs sweeping across the band — what a brush emits per frame.
fn frame_dabs(gw: u32, band_lo: u32, band_hi: u32) -> Vec<DabGpu> {
    let cy = ((band_lo + band_hi) / 2) as f32;
    let n = 8u32;
    (0..n)
        .filter_map(|i| {
            let cx = (gw as f32) * (i as f32 + 0.5) / n as f32;
            DabGpu::new(cx, cy, 16.0, 0.55, [0.25, 0.12, 0.02], 0.25 + 0.12 + 0.02, 0.0)
        })
        .collect()
}

fn time_size(gpu: &GpuContext, cw: u32, ch: u32) {
    // Hires: grid = canvas (scale = 1), the resident path's full-res mode.
    let (gw, gh) = (cw, ch);
    let scale = 1u32;
    let coverage_k = 1.06f32;
    let params = FluidParams::default();

    let solver = FluidSolver::new(&gpu.device, gw, gh);
    solver.set_params(&gpu.queue, &params);
    solver.clear_resident_pigment_gpu(&gpu.device, &gpu.queue);
    solver.clear_resident_water_gpu(&gpu.device, &gpu.queue);
    let paper = vec![0.5f32; (gw * gh) as usize];
    solver.upload_paper(&gpu.queue, &paper);

    let backdrop = vec![0u8; (cw * ch * 4) as usize];
    let mut comp = FluidCompositor::new(&gpu.device);
    comp.begin_stroke(
        &gpu.device,
        &gpu.queue,
        gw,
        gh,
        cw,
        ch,
        scale,
        coverage_k,
        1, // ss = 1 at full-res (the resident-path default)
        solver.pigment_buffer(),
        &backdrop,
    );

    // Two regions: a representative stroke band (~20% height) + the full canvas.
    let band_lo = gh * 2 / 5;
    let band_hi = gh * 3 / 5;
    let band_region = (0u32, band_lo, gw - 1, band_hi);
    let full_region = (0u32, 0u32, gw - 1, gh - 1);
    let dabs = frame_dabs(gw, band_lo, band_hi);

    // Iteration counts shrink with size so the 4K case stays under a few seconds.
    let cells = (gw as u64) * (gh as u64);
    let iters: u32 = if cells > 4_000_000 {
        20
    } else if cells > 1_000_000 {
        40
    } else {
        80
    };
    let warm = 5u32;

    // ── step only (splat + diffuse/advect/evaporate), region-scoped to the band,
    //    polled via read_field_stats ──
    for _ in 0..warm {
        solver.step_resident_splat(&gpu.device, &gpu.queue, &dabs, 1, band_region);
        let _ = solver.read_field_stats(&gpu.device, &gpu.queue, 1.0e-3);
    }
    let t = Instant::now();
    for _ in 0..iters {
        solver.step_resident_splat(&gpu.device, &gpu.queue, &dabs, 1, band_region);
        let _ = solver.read_field_stats(&gpu.device, &gpu.queue, 1.0e-3);
    }
    let step_ms = t.elapsed().as_secs_f64() * 1000.0 / iters as f64;

    // ── step + composite (band) — the real per-frame hot loop (sim scoped to band) ──
    let band_ms = time_loop(gpu, &solver, &mut comp, &dabs, band_region, warm, iters);
    // ── step + composite (full canvas) — worst-case wash (sim full grid) ──
    let full_ms = time_loop(gpu, &solver, &mut comp, &dabs, full_region, warm, iters);

    let band_h = band_hi - band_lo;
    eprintln!(
        "{cw}x{ch} (grid {gw}x{gh}, {:.1}M cells): step={step_ms:.3}ms | step+composite band({band_h}px)={band_ms:.3}ms | full={full_ms:.3}ms | composite-readback portion ≈ band {:.3}ms / full {:.3}ms",
        cells as f64 / 1e6,
        band_ms - step_ms,
        full_ms - step_ms,
    );
}

fn time_loop(
    gpu: &GpuContext,
    solver: &FluidSolver,
    comp: &mut FluidCompositor,
    dabs: &[DabGpu],
    region: (u32, u32, u32, u32),
    warm: u32,
    iters: u32,
) -> f64 {
    for _ in 0..warm {
        solver.step_resident_splat(&gpu.device, &gpu.queue, dabs, 1, region);
        let _ = comp.composite_frame(&gpu.device, &gpu.queue, region);
    }
    let t = Instant::now();
    for _ in 0..iters {
        solver.step_resident_splat(&gpu.device, &gpu.queue, dabs, 1, region);
        let _ = comp.composite_frame(&gpu.device, &gpu.queue, region);
    }
    t.elapsed().as_secs_f64() * 1000.0 / iters as f64
}

#[test]
#[ignore = "needs a GPU device; run --release for meaningful numbers"]
fn per_frame_cost_by_canvas_size() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    eprintln!("--- GPU-resident fluid per-frame cost (post E1+E2+E3) ---");
    // The wet-field carries PIG_CH (=32) channels/cell (ADR-0080/0081), so a full-res grid
    // pigment buffer is `cw·ch·32·4` bytes (~1.06 GB at 4K). **ADR-0083** raised the device's
    // `max_storage_buffer_binding_size` to the adapter's max, so full-res 4K now allocates +
    // runs where the hardware has the VRAM (Apple Silicon, modern dGPUs). The PRODUCTION path
    // uses a LOW-RES grid (canvas/4) regardless; this bench forces grid = canvas to measure the
    // full-res worst case, and still skips any size beyond the device's (now adapter-max) cap on
    // smaller GPUs (no silent cap — it logs the skip).
    let max_buf = u64::from(gpu.device.limits().max_storage_buffer_binding_size);
    for &(cw, ch) in &[(64u32, 64u32), (1408, 768), (2048, 2048), (3840, 2160)] {
        let pig_bytes = u64::from(cw) * u64::from(ch) * PIG_CH as u64 * 4;
        if pig_bytes > max_buf {
            eprintln!(
                "{cw}x{ch}: SKIP — pigment buffer {:.0}MB > device max storage buffer {:.0}MB (PIG_CH={PIG_CH}; production uses a low-res grid)",
                pig_bytes as f64 / 1e6,
                max_buf as f64 / 1e6,
            );
            continue;
        }
        time_size(&gpu, cw, ch);
    }
}
