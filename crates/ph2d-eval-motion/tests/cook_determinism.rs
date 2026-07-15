//! **The cook-level determinism gate that did not exist** (GPU/M5 Fase 0).
//!
//! Before Fase 0, the only golden hash in the repo was `ph2d-ecs`'s
//! `transform_determinism` over the ECS `GlobalTransform` matrices — it never
//! sees the node-graph cook's `Stream` columns. So a rayon reorder inside a node
//! (a parallel float reduction, a non-order-preserving collect) would change the
//! cooked particle positions and **pass CI silently**. This gate closes that
//! hole: it cooks a chain of the retrofitted (now-parallel) nodes at a count
//! **above `PAR_THRESHOLD`** — so the parallel path actually runs — and pins the
//! result.
//!
//! It asserts two things:
//! 1. **Reproducibility** — two independent cooks of the same graph are
//!    byte-identical. A parallel *reduction* of floats (forbidden — thread
//!    scheduling reorders IEEE addition) would make these two runs differ. This
//!    is the load-bearing, cross-OS-safe check.
//! 2. **A pinned FNV** of the cooked columns — a regression tripwire for any
//!    accidental change to a node's math or the parallel machinery. Captured on
//!    the dev machine; the arithmetic is HR-5 deterministic (parabolic sin,
//!    libm pinned), so it is expected cross-OS stable like the ECS golden. A
//!    legitimate drift = re-pin with a captured value + an explanation.

use ph2d_nodegraph::attr::{Column, Stream, PAR_THRESHOLD};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_node_registry::NodeRegistry;

fn registry() -> NodeRegistry {
    let mut r = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut r).unwrap();
    ph2d_node_motion_oscillator::register(&mut r).unwrap();
    ph2d_node_motion_move::register(&mut r).unwrap();
    ph2d_node_motion_transform::register(&mut r).unwrap();
    r
}

/// `grid → oscillator → move → transform` — a source generator plus three
/// per-instance maps, all retrofitted to `par_build`. 160×160 = 25 600 elements,
/// comfortably above `PAR_THRESHOLD` (8192), so every node runs its parallel
/// branch. Returns the graph and the sink (`transform`).
fn build_graph() -> (Graph, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let osc = g.add_node("motion.oscillator");
    let mv = g.add_node("motion.move");
    let xf = g.add_node("motion.transform");
    for (a, b) in [(grid, osc), (osc, mv), (mv, xf)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .unwrap();
    }
    // Non-trivial params so every arithmetic path carries a fractional value that
    // would ULP-drift under a reorder.
    g.set_param(grid, "rows", 160.0);
    g.set_param(grid, "cols", 160.0);
    g.set_param(grid, "gap_x", 0.37);
    g.set_param(grid, "gap_y", 0.41);
    g.set_param(osc, "amplitude", 1.3);
    g.set_param(osc, "frequency", 2.1);
    g.set_param(osc, "phase_stagger", 0.017);
    g.set_param(mv, "dx", 3.5);
    g.set_param(mv, "dy", -1.25);
    g.set_param(xf, "scale", 1.35);
    (g, xf)
}

/// FNV-1a over the stream's columns, in the stream's deterministic column order
/// (`BTreeMap`), hashing each `f32`'s IEEE bit pattern. Dependency-free and
/// stable — a pinnable fingerprint of the whole cooked stream.
fn fingerprint(s: &Stream) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    let eat = |bytes: &[u8], h: &mut u64| {
        for &b in bytes {
            *h ^= u64::from(b);
            *h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    eat(&(s.count() as u64).to_le_bytes(), &mut h);
    for (name, col) in s.columns() {
        eat(name.as_bytes(), &mut h);
        let words: Vec<u32> = match col {
            Column::Scalar(v) => v.iter().map(|x| x.to_bits()).collect(),
            Column::Vec2(v) => v.iter().flat_map(|p| p.iter().map(|x| x.to_bits())).collect(),
            Column::Vec3(v) => v.iter().flat_map(|p| p.iter().map(|x| x.to_bits())).collect(),
            Column::Vec4(v) => v.iter().flat_map(|p| p.iter().map(|x| x.to_bits())).collect(),
        };
        for w in words {
            eat(&w.to_le_bytes(), &mut h);
        }
    }
    h
}

fn cook_once(g: &Graph, reg: &NodeRegistry, sink: NodeId) -> Stream {
    let mut cook = Cook::new();
    cook.cook(g, reg, sink, 1.5).unwrap()[0].as_stream().clone()
}

/// Captured on the dev machine (Linux). Re-pin with an explanation if a
/// deliberate node-math change moves it. `0` is the placeholder before the first
/// capture — the test prints the observed value on mismatch.
const EXPECTED_FINGERPRINT: u64 = 0x1aa7_e05c_4bdb_713f;

/// Manual perf probe (not a gate — `#[ignore]`, meaningful only in `--release`).
/// Cooks a ~500k-instance chain and prints the wall time. Run it twice to read
/// the parallel speedup on THIS machine, since the same code path serializes
/// under one rayon worker:
///
/// ```text
/// cargo test -p ph2d-eval-motion --release --test cook_determinism -- --ignored --nocapture
/// RAYON_NUM_THREADS=1 cargo test -p ph2d-eval-motion --release --test cook_determinism -- --ignored --nocapture
/// ```
#[test]
#[ignore = "perf probe, --release + --nocapture"]
fn cook_500k_timing() {
    let reg = registry();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let osc = g.add_node("motion.oscillator");
    let mv = g.add_node("motion.move");
    let xf = g.add_node("motion.transform");
    for (a, b) in [(grid, osc), (osc, mv), (mv, xf)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .unwrap();
    }
    g.set_param(grid, "rows", 707.0);
    g.set_param(grid, "cols", 707.0); // 499_849 instances
    g.set_param(osc, "amplitude", 1.3);
    g.set_param(osc, "frequency", 2.1);
    g.set_param(mv, "dx", 3.5);
    g.set_param(xf, "scale", 1.35);

    let threads = rayon::current_num_threads();
    // A fresh cook each iteration (the memo would hit on repeats); advance the
    // playhead so nothing is reused.
    let iters = 20u32;
    let start = std::time::Instant::now();
    let mut n = 0usize;
    for k in 0..iters {
        let mut cook = Cook::new();
        let s = cook.cook(&g, &reg, xf, 1.0 + f64::from(k) * 0.013).unwrap()[0].as_stream();
        n = s.count();
        std::hint::black_box(s.get("P"));
    }
    let per = start.elapsed().as_secs_f64() * 1000.0 / f64::from(iters);
    println!("cook {n} instances: {per:.2} ms/cook over {threads} rayon threads");
}

#[test]
fn the_parallel_cook_is_reproducible_and_pinned_at_scale() {
    let reg = registry();
    let (g, sink) = build_graph();

    let a = cook_once(&g, &reg, sink);
    assert!(
        a.count() >= PAR_THRESHOLD,
        "the golden must exercise the PARALLEL path (count {} < threshold {PAR_THRESHOLD})",
        a.count()
    );

    // 1. Reproducibility: a second, independent cook must be byte-identical.
    //    A parallel float reduction would make these differ run-to-run.
    let b = cook_once(&g, &reg, sink);
    assert_eq!(
        a, b,
        "two cooks of the same chain at N > threshold diverged — a non-deterministic parallel reduction slipped into a node"
    );

    // 2. Regression pin.
    let got = fingerprint(&a);
    assert_eq!(
        got, EXPECTED_FINGERPRINT,
        "cook fingerprint drifted (got {got:#018x}). If this was a deliberate node-math change, re-pin EXPECTED_FINGERPRINT with an explanation."
    );
}
