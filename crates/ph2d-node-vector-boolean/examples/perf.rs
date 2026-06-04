//! T3.5 lens C — boolean perf, measured in `--release` (dev is ~7× slower and
//! lies — memory `project_painter_composite_perf_2026_06_03`).
//!
//! Run: `cargo run --release -p ph2d-node-vector-boolean --example perf`
//!
//! Context: the exact Linesweeper is the *reconcile* path (async / on-commit,
//! ADR-0059 §2.4) — NOT the per-frame hot path (that is the SDF GPU draft,
//! ≤ 0.5 ms, ADR-0065 §2.5). So the question is "does a realistic batch finish
//! fast enough for an interactive commit", not "does it fit one 8 ms frame".

use std::time::Instant;

use glam::Vec2;
use ph2d_node_vector_boolean::boolean;
use ph2d_vector_doc::{BooleanOp, VectorNetwork, primitives};

fn poly(cx: f32, cy: f32, r: f32, sides: u32) -> VectorNetwork {
    let mut n = primitives::polygon(Vec2::new(cx, cy), r, sides, 0.0);
    n.deterministic = true;
    n
}

fn main() {
    let sides = 24; // ~circle, 24 cubic segments per operand

    // ── 100 independent Union ops on overlapping 24-gon pairs ──────────────
    let pairs: Vec<(VectorNetwork, VectorNetwork)> = (0..100)
        .map(|i| {
            let x = (i % 10) as f32 * 30.0;
            let y = (i / 10) as f32 * 30.0;
            (
                poly(x, y, 20.0, sides),
                poly(x + 10.0, y + 10.0, 20.0, sides),
            )
        })
        .collect();
    let t = Instant::now();
    let mut regions = 0usize;
    for (a, b) in &pairs {
        regions += boolean(a, b, BooleanOp::Union).regions.len();
    }
    let dt = t.elapsed().as_secs_f64() * 1000.0;
    println!(
        "100 independent Union ops (24-gon pairs): {dt:.2} ms total, {:.3} ms/op  ({regions} result regions)",
        dt / 100.0
    );

    // ── 500-op stress (plan §stress: 500 sequential boolean ops) ───────────
    let ops = [
        BooleanOp::Union,
        BooleanOp::Subtract,
        BooleanOp::Intersect,
        BooleanOp::Exclude,
    ];
    let t = Instant::now();
    let mut sink = 0usize;
    for i in 0..500 {
        let a = poly((i % 17) as f32 * 11.0, (i % 13) as f32 * 9.0, 18.0, sides);
        let b = poly(
            (i % 17) as f32 * 11.0 + 8.0,
            (i % 13) as f32 * 9.0 + 6.0,
            18.0,
            sides,
        );
        sink += boolean(&a, &b, ops[i % ops.len()]).regions.len();
    }
    let dt = t.elapsed().as_secs_f64() * 1000.0;
    println!(
        "500 mixed ops (stress): {dt:.2} ms total, {:.3} ms/op  (no crash, {sink} regions)",
        dt / 500.0
    );

    // ── Accumulate-union of 50 overlapping polys into one growing blob ─────
    // Stresses growing-complexity inputs (the result feeds the next op).
    let t = Instant::now();
    let mut acc = poly(0.0, 0.0, 25.0, sides);
    for i in 1..50 {
        let c = poly(i as f32 * 12.0, ((i * 7) % 40) as f32, 25.0, sides);
        acc = boolean(&acc, &c, BooleanOp::Union);
    }
    let dt = t.elapsed().as_secs_f64() * 1000.0;
    println!(
        "accumulate-union of 50 polys (49 chained ops): {dt:.2} ms total  (final blob: {} regions, {} segments)",
        acc.regions.len(),
        acc.segments.len()
    );

    println!("\nframe-budget refs: 120 Hz ProMotion = 8.3 ms/frame, 60 Hz = 16.6 ms/frame");
}
