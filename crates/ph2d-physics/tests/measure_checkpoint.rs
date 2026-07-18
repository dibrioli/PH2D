//! **The W1.5 kill-check, measured.** The plan (00_plano_waves §W1.5) says:
//! *"Se o checkpoint estourar 20 MB com K razoável, o scrub-back não existe
//! nesta forma"* — so the number comes before the ring, not after it.
//!
//! ```text
//! cargo test -p ph2d-physics --test measure_checkpoint -- --nocapture
//! ```
//!
//! HR-13 gives physics a 20 MB budget, and HR-13's amendment (ADR-0117) is
//! that whoever declares a budget owns a gate that MEASURES it. This is that
//! gate: dhat reports the real heap cost of one checkpoint at three scene
//! sizes, and the ring's stride `K` is chosen from these numbers rather than
//! guessed.

use ph2d_physics::PhysicsWorld;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// The HR-13 physics budget. The whole ring must fit inside it, with room
/// left for the live world itself.
const BUDGET_BYTES: f64 = 20.0 * 1_048_576.0;

fn world_with(bodies: usize) -> PhysicsWorld {
    let mut w = PhysicsWorld::new();
    w.add_static_cuboid(0.0, 0.0, 500.0, 0.1);
    for i in 0..bodies {
        let row = (i / 20) as f32;
        let col = (i % 20) as f32;
        w.add_dynamic_circle(col * 0.6 - 6.0, 5.0 + row * 0.6, 0.25, 1.0);
    }
    // Settle a while so the broad/narrow phase carry real contact pairs —
    // a checkpoint of a scene that has never touched anything would be a
    // fixture that flatters the measurement.
    for _ in 0..120 {
        w.step();
    }
    w
}

/// Bytes actually on the heap for ONE checkpoint of an `n`-body world.
fn measure_one(n: usize) -> (f64, f64) {
    let w = world_with(n);
    let profiler = dhat::Profiler::builder().testing().build();
    let cp = w.checkpoint();
    let stats = dhat::HeapStats::get();
    drop(profiler);
    let reported = cp.approx_bytes() as f64;
    std::hint::black_box(&cp);
    (stats.max_bytes as f64, reported)
}

#[test]
fn a_checkpoint_of_a_real_scene_fits_the_physics_budget() {
    println!("\n=== W1.5 kill-check: what does one checkpoint cost? ===");
    println!(
        "HR-13 physics budget: {:.1} MB\n",
        BUDGET_BYTES / 1_048_576.0
    );

    let mut rows = Vec::new();
    for n in [10usize, 50, 200] {
        let (measured, reported) = measure_one(n);
        println!(
            "{n:>4} bodies: {:>9.1} KB measured   ({:>8.1} KB reported, {:>5.0} B/body)",
            measured / 1024.0,
            reported / 1024.0,
            measured / n as f64,
        );
        rows.push((n, measured));
    }

    // The decisive number: a dense ring (one checkpoint per tick) over the
    // 300-tick window `ph2d-eval-motion` uses, at a scene size a 2D game
    // actually ships.
    let (_, per_cp) = rows[1];
    let dense_300 = per_cp * 300.0;
    println!(
        "\nDense ring (K=1, 300 ticks) at 50 bodies: {:.1} MB — {} the budget",
        dense_300 / 1_048_576.0,
        if dense_300 <= BUDGET_BYTES {
            "inside"
        } else {
            "OVER"
        }
    );
    for k in [5u32, 10, 15, 30] {
        let n_cp = (300 / k) as f64;
        println!(
            "  K={k:>2} ({n_cp:>2.0} checkpoints over 300 ticks): {:>6.2} MB   worst-case replay {k} steps",
            per_cp * n_cp / 1_048_576.0
        );
    }

    // The gate itself: whatever stride we pick, ONE checkpoint must be a
    // small fraction of the budget, or the ring is not viable in this shape
    // and the wave falls back to sparse keyframes + re-sim.
    assert!(
        per_cp < BUDGET_BYTES / 20.0,
        "a single 50-body checkpoint costs {:.1} KB — more than 5% of the {:.0} MB physics \
         budget. A ring of any useful depth cannot fit; W1.5 must fall back to sparse \
         keyframes + re-sim (plan §W1.5 kill-check).",
        per_cp / 1024.0,
        BUDGET_BYTES / 1_048_576.0
    );

    // The reported estimate must track the measured truth — otherwise
    // `approx_bytes` is a comment that lies, and the ring's own memory gate
    // (which uses it) would be measuring nothing.
    let (measured, reported) = measure_one(50);
    let ratio = reported / measured;
    println!("\napprox_bytes / dhat = {ratio:.2}×");
    assert!(
        (0.25..=4.0).contains(&ratio),
        "approx_bytes reports {reported:.0} B but dhat measures {measured:.0} B ({ratio:.2}×) — \
         the estimate has drifted from the truth"
    );
}
