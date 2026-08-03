//! Unit tests for `motion.duplicator`, split from `lib.rs` (`#[path]` sibling).
//! `super` is the crate root.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// A shape stream of `ns` elements at the origin, each carrying a distinct
/// `texture_id` (so a bug that reads the point's appearance, or collapses the
/// shapes, cannot pass — the ids would differ).
fn shapes(ids: &[f32]) -> Stream {
    let n = ids.len();
    Stream::new(n)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; n]))
        .with("texture_id", Column::Scalar(ids.to_vec()))
}

/// A points stream from a list of positions.
fn points(pos: &[[f32; 2]]) -> Stream {
    Stream::new(pos.len()).with("P", Column::Vec2(pos.to_vec()))
}

#[test]
fn the_duplicator_stamps_each_shape_at_each_point() {
    // 2 shapes (ids 7, 3) × 3 points → 6 instances, shape-major. Each carries
    // its OWN shape's texture_id (mutation: read the point's appearance → all 0,
    // RED) and sits at its point's position.
    let s = shapes(&[7.0, 3.0]);
    let p = points(&[[10.0, 0.0], [20.0, 0.0], [30.0, 0.0]]);
    let out = duplicate(&s, &p, 3);
    assert_eq!(out.count(), 6);
    let Column::Scalar(ids) = out.get("texture_id").unwrap() else {
        panic!("texture_id")
    };
    assert_eq!(ids, &vec![7.0, 7.0, 7.0, 3.0, 3.0, 3.0]);
    let Column::Vec2(pp) = out.get("P").unwrap() else {
        panic!("P")
    };
    // shape 0 (P origin) at the 3 points, then shape 1 at the same 3 points.
    assert_eq!(
        pp,
        &vec![
            [10.0, 0.0],
            [20.0, 0.0],
            [30.0, 0.0],
            [10.0, 0.0],
            [20.0, 0.0],
            [30.0, 0.0],
        ]
    );
}

#[test]
fn p_and_rot_sum_both_inputs() {
    // The shape's P/rot is a base the point ADDS to (Copy-to-Points): a shape
    // cluster offset from the origin, stamped, keeps its internal offset.
    let s = Stream::new(1)
        .with("P", Column::Vec2(vec![[1.0, 0.0]]))
        .with("rot", Column::Scalar(vec![10.0]));
    let p = Stream::new(2)
        .with("P", Column::Vec2(vec![[0.0, 5.0], [0.0, 9.0]]))
        .with("rot", Column::Scalar(vec![1.0, 2.0]));
    let out = duplicate(&s, &p, 2);
    let Column::Vec2(pp) = out.get("P").unwrap() else {
        panic!("P")
    };
    assert_eq!(pp, &vec![[1.0, 5.0], [1.0, 9.0]]);
    let Column::Scalar(rr) = out.get("rot").unwrap() else {
        panic!("rot")
    };
    assert_eq!(rr, &vec![11.0, 12.0]);
}

#[test]
fn no_rot_column_when_neither_input_has_one() {
    // A pure position stamp emits no `rot` column — the lowering's default 0 is
    // the answer, and an empty column would be noise a downstream node reads.
    let out = duplicate(&shapes(&[7.0]), &points(&[[1.0, 1.0]]), 1);
    assert!(out.get("rot").is_none());
}

#[test]
fn index_and_count_are_continuous_across_the_product() {
    // 2 × 3 → one uninterrupted Index 0..6 and Count 6 everywhere, so a ramp
    // spans the whole stamped set instead of restarting per shape.
    let out = duplicate(&shapes(&[7.0, 3.0]), &points(&[[0.0, 0.0]; 3]), 3);
    let Column::Scalar(idx) = out.get("Index").unwrap() else {
        panic!("Index")
    };
    assert_eq!(idx, &vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
    let Column::Scalar(cnt) = out.get("Count").unwrap() else {
        panic!("Count")
    };
    assert_eq!(cnt, &vec![6.0; 6]);
}

#[test]
fn no_points_passes_the_shapes_through() {
    // A duplicator with nowhere to stamp is a passthrough of its shapes.
    let s = shapes(&[7.0, 3.0]);
    let out = duplicate(&s, &Stream::new(0), 0);
    assert_eq!(out.count(), 2);
    let Column::Scalar(ids) = out.get("texture_id").unwrap() else {
        panic!("texture_id")
    };
    assert_eq!(ids, &vec![7.0, 3.0]);
}

#[test]
fn points_within_budget_caps_the_product() {
    // 3 shapes, max 25 → at most 8 points (3·8 = 24 ≤ 25). No shapes → 0.
    assert_eq!(points_within_budget(3, 99, 25), 8);
    assert_eq!(points_within_budget(3, 5, 25), 5); // honoured when it fits
    assert_eq!(points_within_budget(0, 5, 25), 0); // nothing to stamp
}

// ── End-to-end: prove `eval` wires input 0 = shape, input 1 = points ─────────

struct FixedSrc(&'static NodeManifest, fn() -> Stream);
impl NodeOp for FixedSrc {
    fn manifest(&self) -> &'static NodeManifest {
        self.0
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit((self.1)());
    }
}

static SHAPE_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.duplicator.test.shape"),
    name: "motion.duplicator.test.shape",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
static POINTS_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.duplicator.test.points"),
    name: "motion.duplicator.test.points",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        static SHAPE: FixedSrc = FixedSrc(&SHAPE_MAN, || {
            Stream::new(1)
                .with("P", Column::Vec2(vec![[0.0, 0.0]]))
                .with("texture_id", Column::Scalar(vec![9.0]))
        });
        static POINTS: FixedSrc = FixedSrc(&POINTS_MAN, || points(&[[1.0, 0.0], [2.0, 0.0]]));
        static DUP: MotionDuplicator = MotionDuplicator;
        match ty {
            t if t == SHAPE_MAN.id => Some(&SHAPE),
            t if t == POINTS_MAN.id => Some(&POINTS),
            t if t == MANIFEST.id => Some(&DUP),
            _ => None,
        }
    }
}

#[test]
fn eval_reads_shape_from_input_0_and_points_from_input_1() {
    let mut g = Graph::new();
    let shape = g.add_node("motion.duplicator.test.shape");
    let pts = g.add_node("motion.duplicator.test.points");
    let dup = g.add_node("motion.duplicator");
    g.connect(Edge {
        from: (shape, 0),
        to: (dup, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (pts, 0),
        to: (dup, 1),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, dup, 0.0).unwrap();
    let s = out[0].as_stream();
    // 1 shape × 2 points → 2, each with the SHAPE's texture_id 9 (proving
    // input 0 is the shape, not the points — the points carry no texture_id).
    assert_eq!(s.count(), 2);
    let Column::Scalar(ids) = s.get("texture_id").unwrap() else {
        panic!("texture_id")
    };
    assert_eq!(ids, &vec![9.0, 9.0]);
    let Column::Vec2(pp) = s.get("P").unwrap() else {
        panic!("P")
    };
    assert_eq!(pp, &vec![[1.0, 0.0], [2.0, 0.0]]);
}

// doc 86 §7: the measured number. `duplicate()` is the O(N) work A1 adds over a
// plain `grid → output`; this confirms the per-instance cost is FLAT in N (the
// stamp is a linear replicate, not a quadratic). `#[ignore]` — a timing probe,
// not a suite gate; run with `-- --ignored --nocapture`.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_duplicate_is_flat_in_n() {
    use std::time::Instant;
    let shape = Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("size", Column::Vec2(vec![[1.0, 1.0]]))
        .with("tint", Column::Vec4(vec![[1.0, 1.0, 1.0, 1.0]]))
        .with("uv_rect", Column::Vec4(vec![[0.0, 0.0, 1.0, 1.0]]))
        .with("texture_id", Column::Scalar(vec![5.0]));
    for &n in &[16usize, 256, 4096, 65536] {
        let pts = points(&vec![[0.0, 0.0]; n]);
        // Warm, then take the median of a few runs (shared machine, doc 28 §5.49).
        let _ = duplicate(&shape, &pts, n);
        let mut best = f64::INFINITY;
        for _ in 0..5 {
            let t = Instant::now();
            let out = std::hint::black_box(duplicate(&shape, std::hint::black_box(&pts), n));
            best = best.min(t.elapsed().as_secs_f64());
            assert_eq!(out.count(), n);
        }
        eprintln!(
            "duplicate  N={n:>6}  {:>8.3} ms  =>  {:>6.1} ns/instance",
            best * 1e3,
            best * 1e9 / n as f64
        );
    }
}
