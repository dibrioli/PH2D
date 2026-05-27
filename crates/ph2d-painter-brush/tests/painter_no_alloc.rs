//! `painter_no_alloc_hot_path` — HR-3 executable gate (audit T1.6 Z-1).
//!
//! Spec [`docs/Painter_projeto/01_brush_engine.md`](
//! ../../../docs/Painter_projeto/01_brush_engine.md) §1.11 +
//! [`08_performance_memory.md`](
//! ../../../docs/Painter_projeto/08_performance_memory.md) §8.5 / §8.13
//! mandate this gate: the `StampScheduler::advance` hot path MUST emit
//! zero heap allocations per frame, even under worst-case multi-stamp +
//! color-jitter load (T1.6 expansion). dhat-rs hooks the global allocator
//! in this integration-test binary and counts allocation blocks across
//! the warm hot-path window.
//!
//! **Scope:** scheduler-only (CPU stamp emission). The CPU `apply_stamps`
//! per-pixel rasterizer is OUT of this gate's scope — its budget is
//! covered by `painter_frame_budget_60hz` criterion bench (W2+) and is
//! known to NOT fit HR-3 for `size_px > 256` (documented in
//! `tool.rs` R4-LG-6).
//!
//! **Test structure:** dhat allows only ONE `Profiler` per binary
//! (re-entry panics). To exercise both the worst-case (T1.6 expansion)
//! AND the baseline (round_hard) paths, both scenarios run within a
//! single test function under one profiler. The baseline scenario also
//! serves as a regression sentinel — if T1.6 broke the base path,
//! THAT delta fires first.

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use ph2d_painter_brush::{
    PointerSample, StampScheduler,
    library::{round_hard, square_hard},
};

/// Run the scheduler over `frames` × 10 advances per frame using the
/// given brush + diameter. Returns the number of stamps emitted (sanity
/// — prevents the optimizer from eliding the entire body if a future
/// refactor de-publicizes the pool).
fn run_scheduler_workload(
    scheduler: &mut StampScheduler,
    brush: &ph2d_painter_brush::Brush,
    diameter: f32,
    frames: u64,
) -> usize {
    let mut total = 0;
    for frame in 0..frames {
        scheduler.begin_stroke(frame);
        for i in 0..10 {
            let pos = [(frame * 10 + i) as f32 * 5.0, (frame * 10 + i) as f32 * 5.0];
            let stamps = scheduler.advance(
                brush,
                PointerSample {
                    position: pos,
                    pressure: 0.5 + (i as f32) * 0.05,
                    tilt: 0.0,
                },
                diameter,
                [0.6, 0.1, 0.1, 1.0],
            );
            total += stamps.len();
        }
        scheduler.end_stroke();
    }
    total
}

/// HR-3 zero-alloc gate (audit T1.6 Z-1). Combined baseline + worst-case
/// in a single dhat profiler scope (re-entry panics).
///
/// Step 1 — baseline (round_hard): proves the BASE hot path is alloc-
/// free. If THIS fails, T1.6 broke `push_one_stamp` or the inner loop.
///
/// Step 2 — worst-case (square_hard + shape_count=16 + all 4 jitters +
/// scatter + rotation_follow + randomized + count_jitter): proves the
/// T1.6 expansion (multi-stamp + color jitter + rotation pipeline +
/// footprint enlargement) is alloc-free.
#[test]
fn painter_no_alloc_hot_path() {
    let _profiler = dhat::Profiler::builder().testing().build();

    // ─── Setup: brushes + scheduler, ALL outside measurement ──────────
    let baseline_brush = round_hard();
    let mut worst_brush = square_hard();
    worst_brush.shape.shape_count = 16;
    worst_brush.shape.shape_count_jitter = 1.0;
    worst_brush.shape.shape_scatter = 180.0;
    worst_brush.shape.shape_rotation_follow = true;
    worst_brush.shape.shape_randomized = true;
    worst_brush.color_dynamics.stamp_lightness_jitter = 0.5;
    worst_brush.color_dynamics.stamp_darkness_jitter = 0.3;
    worst_brush.color_dynamics.stamp_hue_jitter = 0.5;
    worst_brush.color_dynamics.stamp_saturation_jitter = 0.5;

    let mut scheduler = StampScheduler::new();

    // Warm-up: drive BOTH brushes through the scheduler. This forces
    // any LATE allocations (capacity growth, lazy-init, dhat's own
    // internal buffers) to happen BEFORE the measurement window.
    let _ = run_scheduler_workload(&mut scheduler, &baseline_brush, 32.0, 5);
    let _ = run_scheduler_workload(&mut scheduler, &worst_brush, 50.0, 5);

    // Also exercise `HeapStats::get` once in warm-up — dhat may lazily
    // allocate the snapshot buffer on first call (audit-driven defense
    // against false-positive in the measurement loop).
    let _warm_snap = dhat::HeapStats::get();

    // ─── Measurement window opens ─────────────────────────────────────
    let stats_before = dhat::HeapStats::get();

    // Step 1 — baseline.
    let baseline_total = run_scheduler_workload(&mut scheduler, &baseline_brush, 32.0, 100);

    let stats_mid = dhat::HeapStats::get();
    let baseline_blocks = stats_mid.total_blocks - stats_before.total_blocks;

    // Step 2 — worst-case.
    let worst_total = run_scheduler_workload(&mut scheduler, &worst_brush, 50.0, 100);

    let stats_after = dhat::HeapStats::get();
    let worst_blocks = stats_after.total_blocks - stats_mid.total_blocks;
    let total_blocks = stats_after.total_blocks - stats_before.total_blocks;
    // ─── Measurement window closes ────────────────────────────────────

    // Sanity (prevents optimizer eliding the body).
    assert!(baseline_total > 0, "baseline must emit stamps");
    assert!(worst_total > 0, "worst-case must emit stamps");

    assert_eq!(
        baseline_blocks, 0,
        "HR-3 violation (baseline / audit T1.6 Z-1): round_hard scheduler \
         hot path allocated {baseline_blocks} new heap blocks over 100 \
         frames × 10 advances. T1.6 base path regressed."
    );

    assert_eq!(
        worst_blocks, 0,
        "HR-3 violation (worst-case / audit T1.6 Z-1): square_hard + \
         shape_count=16 + all 4 jitters + scatter + rotation_follow + \
         randomized + count_jitter scheduler hot path allocated \
         {worst_blocks} new heap blocks over 100 frames × 10 advances. \
         T1.6 expansion path is NOT zero-alloc."
    );

    // Final paranoid check — total delta is 0 (no allocation in either
    // step OR in the `stats_mid` snapshot path itself).
    assert_eq!(
        total_blocks, 0,
        "HR-3 violation (combined): full window allocated {total_blocks} \
         blocks. Includes baseline ({baseline_blocks}) + worst-case \
         ({worst_blocks}) + dhat::HeapStats::get() overhead."
    );
}
