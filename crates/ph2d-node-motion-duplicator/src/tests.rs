//! Unit tests for `motion.duplicator`, split from `lib.rs` (`#[path]` sibling).
//! `super` is the crate root.

use super::transfer::Transfer;
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
    let out = duplicate(&s, &p, 3, Pick::Off, 0, 0.0, Transfer::ShapeWins);
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
    let out = duplicate(&s, &p, 2, Pick::Off, 0, 0.0, Transfer::ShapeWins);
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
    let out = duplicate(
        &shapes(&[7.0]),
        &points(&[[1.0, 1.0]]),
        1,
        Pick::Off,
        0,
        0.0,
        Transfer::ShapeWins,
    );
    assert!(out.get("rot").is_none());
}

#[test]
fn index_and_count_are_continuous_across_the_product() {
    // 2 × 3 → one uninterrupted Index 0..6 and Count 6 everywhere, so a ramp
    // spans the whole stamped set instead of restarting per shape.
    let out = duplicate(
        &shapes(&[7.0, 3.0]),
        &points(&[[0.0, 0.0]; 3]),
        3,
        Pick::Off,
        0,
        0.0,
        Transfer::ShapeWins,
    );
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
    let out = duplicate(
        &s,
        &Stream::new(0),
        0,
        Pick::Off,
        0,
        0.0,
        Transfer::ShapeWins,
    );
    assert_eq!(out.count(), 2);
    let Column::Scalar(ids) = out.get("texture_id").unwrap() else {
        panic!("texture_id")
    };
    assert_eq!(ids, &vec![7.0, 3.0]);
}

#[test]
fn points_within_budget_caps_the_product() {
    // 3 shapes, max 25 → at most 8 points (3·8 = 24 ≤ 25). No shapes → 0.
    assert_eq!(points_within_budget(Pick::Off, 3, 99, 25), 8);
    assert_eq!(points_within_budget(Pick::Off, 3, 5, 25), 5); // honoured when it fits
    assert_eq!(points_within_budget(Pick::Off, 0, 5, 25), 0); // nothing to stamp
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
        let _ = duplicate(&shape, &pts, n, Pick::Off, 0, 0.0, Transfer::ShapeWins);
        let mut best = f64::INFINITY;
        for _ in 0..5 {
            let t = Instant::now();
            let out = std::hint::black_box(duplicate(
                &shape,
                std::hint::black_box(&pts),
                n,
                Pick::Off,
                0,
                0.0,
                Transfer::ShapeWins,
            ));
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

// ─────────────────────────────────────────────────────────────────────────────
// **A ESCALA DO PONTO** (doc 89 folha 08 — o defeito, não o knob).
// ─────────────────────────────────────────────────────────────────────────────

/// Um stream de pontos com `size` por elemento (o que um `motion.scatter` produz).
fn points_sized(ps: &[[f32; 2]], sizes: &[f32]) -> Stream {
    let mut s = points(ps);
    s.set(
        "size",
        Column::Vec2(sizes.iter().map(|&v| [v, v]).collect()),
    );
    s
}

/// **`point_scale = 0` É O MUNDO DE SEMPRE, E A AUSÊNCIA DA COLUNA FAZ PARTE DELE.**
///
/// ⚠️ **A metade que quase escapou:** escrever `1.0` em toda a linha quando ninguém autorou
/// escala CRIA uma coluna `size` que não existia — e uma coluna a mais viaja, é serializada
/// e muda o que um nó a jusante vê. O mundo de sempre é a **ausência** dela.
#[test]
fn a_point_scale_of_zero_leaves_the_stamp_exactly_as_it_shipped() {
    let s = shapes(&[5.0]);
    let p = points_sized(&[[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]], &[0.5, 1.0, 2.0]);
    let off = duplicate(&s, &p, 3, Pick::Off, 0, 0.0, Transfer::ShapeWins);
    assert!(
        off.get("size").is_none(),
        "sem escala pedida, o carimbo não pode inventar uma coluna `size`"
    );
    // O CONTROLE de que a fixture contém o fenômeno: com o peso a 1 ela aparece.
    let on = duplicate(&s, &p, 3, Pick::Off, 0, 1.0, Transfer::ShapeWins);
    assert!(
        on.get("size").is_some(),
        "com o peso a 1 a escala do ponto TEM de chegar ao carimbo"
    );
}

/// **A ESCALA DO PONTO CHEGA, E O PESO INTERPOLA.**
///
/// Medido antes desta wave (`measure_stream_join_defects`): pontos com `size = [0, 4, 8, 12]`
/// davam uma saída **sem coluna `size` nenhuma**.
#[test]
fn the_points_scale_reaches_the_stamp_and_the_weight_interpolates() {
    let s = shapes(&[5.0]);
    let p = points_sized(&[[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]], &[0.5, 1.0, 3.0]);
    let read = |t: f32| match duplicate(&s, &p, 3, Pick::Off, 0, t, Transfer::ShapeWins).get("size")
    {
        Some(Column::Vec2(v)) => v.iter().map(|q| q[0]).collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    // `t = 1`: a escala do ponto INTEIRA (a forma não autorou `size`, logo multiplica 1).
    assert_eq!(read(1.0), vec![0.5, 1.0, 3.0]);
    // `t = 0.5`: `lerp(1, p, 0.5)` — meio caminho entre não escalar e escalar.
    let half = read(0.5);
    for (got, want) in half.iter().zip([0.75f32, 1.0, 2.0]) {
        assert!(
            (got - want).abs() < 1e-6,
            "meio peso é o meio caminho: {half:?}"
        );
    }
}

/// **A ESCALA DA FORMA E A DO PONTO MULTIPLICAM-SE** — a forma continua a ser o template, e
/// o ponto MODULA-a; ele não a substitui.
///
/// ⚠️ É a distinção que separa esta wave de um `Set`: substituir apagaria o `size` que a
/// forma autorou, e a referência (Houdini `pscale`, Blender `Scale`) compõe.
#[test]
fn the_shape_scale_and_the_point_scale_multiply() {
    let mut s = shapes(&[5.0]);
    s.set("size", Column::Vec2(vec![[2.0, 2.0]]));
    let p = points_sized(&[[0.0, 0.0], [1.0, 0.0]], &[0.5, 3.0]);
    let out = duplicate(&s, &p, 2, Pick::Off, 0, 1.0, Transfer::ShapeWins);
    let Some(Column::Vec2(v)) = out.get("size") else {
        panic!("size")
    };
    assert_eq!(
        v.iter().map(|q| q[0]).collect::<Vec<_>>(),
        vec![1.0, 6.0],
        "2 × 0,5 e 2 × 3"
    );
}

/// **PONTOS SEM ESCALA NÃO ACORDAM O CAMINHO** — pedir peso sobre pontos que não autoraram
/// `size` deixa o carimbo intacto, em vez de o escalar por uma identidade inventada.
#[test]
fn points_without_a_size_column_leave_the_stamp_alone() {
    let mut s = shapes(&[5.0]);
    s.set("size", Column::Vec2(vec![[2.0, 2.0]]));
    let p = points(&[[0.0, 0.0], [1.0, 0.0]]);
    let out = duplicate(&s, &p, 2, Pick::Off, 0, 1.0, Transfer::ShapeWins);
    let Some(Column::Vec2(v)) = out.get("size") else {
        panic!("size")
    };
    assert_eq!(v, &vec![[2.0, 2.0]; 2], "a escala da forma, replicada");
}
