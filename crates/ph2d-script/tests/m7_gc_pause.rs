//! M7 GC budget gate: `Lua::gc_step` p99 ≤ 0.01 ms (10 µs).
//!
//! Methodology:
//! - Build up GC pressure with a synthetic workload (1000 entities
//!   worth of `ph2d.set` invocations + a Luau-side allocation loop).
//! - Run `gc_step` 1000 times back-to-back; record each duration.
//! - p99 = sample at index 990 of the sorted vector.
//!
//! Threshold rationale (from M7 plan): 0.01 ms p99 keeps GC
//! invisible at 60 Hz (16.7 ms budget) and 120 Hz (8.3 ms). If this
//! test ever flakes on slow CI runners, mark it `#[ignore]` and
//! schedule via the bench workflow instead — we deliberately did NOT
//! gate it on a CI hardware floor.

use ph2d_script::ScriptHost;
use std::time::{Duration, Instant};

const GC_STEPS: usize = 1000;
const P99_BUDGET: Duration = Duration::from_micros(10);

// Perf-gated test: Windows GitHub-hosted runner is too noisy
// (shared-tenant VM + Defender; observed p99=18 µs vs 10 µs budget
// twice already, on PR #15 M10 and PR #19 M12). Run locally with
// --ignored. When self-hosted Mac mini M2 lands (HR-4 infra),
// flip back to a real CI gate.
#[test]
#[ignore = "perf-gated; CI runners are too noisy. Run locally with --ignored"]
fn gc_step_p99_under_ten_microseconds() {
    let mut host = ScriptHost::new().unwrap();

    // Warm the VM with allocation-heavy work so gc_step has actual
    // work to do — measuring "step on an empty heap" is uninteresting.
    host.load_script(
        r#"
        local t = {}
        for i = 1, 1000 do
            t[i] = { x = i, y = i * 2, name = "entity_" .. tostring(i) }
            ph2d.set(i, "x", t[i].x)
            ph2d.set(i, "y", t[i].y)
        end
        "#,
    )
    .unwrap();
    let _ = host.drain_writes();

    let mut samples: Vec<Duration> = Vec::with_capacity(GC_STEPS);
    for _ in 0..GC_STEPS {
        let t0 = Instant::now();
        host.gc_step().unwrap();
        samples.push(t0.elapsed());
    }

    samples.sort_unstable();
    let p99 = samples[(GC_STEPS as f64 * 0.99) as usize];
    let p50 = samples[GC_STEPS / 2];
    let max = *samples.last().unwrap();

    println!("gc_step p50 = {p50:?}, p99 = {p99:?}, max = {max:?}");
    assert!(
        p99 <= P99_BUDGET,
        "gc_step p99 {p99:?} > {P99_BUDGET:?} budget"
    );
}
