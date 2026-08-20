//! Os gates do campo em si — a lei de `field`/`curve`, a rampa e o `invert`.
//!
//! ⚠️ **Estavam INLINE e saíram por HR-18**: o `lib.rs` chegou a 711 linhas ao ganhar o
//! canal da folha 05, e o teto de `crates/` é 700. A cura de um teto é um split, e o corte
//! natural é o que os dois irmãos já usavam (`rotation_tests.rs`, `channel_tests.rs`) —
//! **um arquivo por assunto**, não *"os testes"* numa pilha.

use super::*;
use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

// Source: 3 instances on a line at x = 0, 5, 10 (y = 0).
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.falloff.test.src"),
    name: "motion.falloff.test.src",
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
struct Src;
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0], [5.0, 0.0], [10.0, 0.0]])));
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&Src),
            t if t == MANIFEST.id => Some(&MotionFalloff),
            _ => None,
        }
    }
}

fn falloff_of(g: &Graph, ops: &Ops, target: NodeId) -> Vec<f32> {
    let mut cook = Cook::new();
    let out = cook.cook(g, ops, target, 0.0).unwrap();
    match out[0].as_stream().get("falloff").unwrap() {
        Column::Scalar(v) => v.clone(),
        _ => panic!("falloff must be a Scalar column"),
    }
}

#[test]
fn default_circle_is_one_at_center_zero_at_edge() {
    let mut g = Graph::new();
    let src = g.add_node("motion.falloff.test.src");
    let foc = g.add_node("motion.falloff");
    g.connect(Edge {
        from: (src, 0),
        to: (foc, 0),
        delayed: false,
    })
    .unwrap();
    // Defaults: Circle + Smooth, radius 5, centre (0,0): x=0 → 1, x=5 → 0, x=10 → 0.
    assert_eq!(falloff_of(&g, &Ops, foc), vec![1.0, 0.0, 0.0]);
}

/// Fields COMPOSE multiplicatively (audit 2026-07-10: the promise at the
/// `base * field` site was untested): an upstream `falloff` column is
/// multiplied by this field, never overwritten — two stacked focus nodes
/// intersect their regions.
#[test]
fn a_prior_falloff_column_is_multiplied_not_overwritten() {
    static FSRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.falloff.test.fsrc"),
        name: "motion.falloff.test.fsrc",
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
    struct FSrc;
    impl NodeOp for FSrc {
        fn manifest(&self) -> &'static NodeManifest {
            &FSRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(
                Stream::new(2)
                    .with("P", Column::Vec2(vec![[0.0, 0.0], [10.0, 0.0]]))
                    .with("falloff", Column::Scalar(vec![0.5, 0.8])),
            );
        }
    }
    struct FOps;
    impl OpResolver for FOps {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == FSRC_MAN.id => Some(&FSrc),
                t if t == MANIFEST.id => Some(&MotionFalloff),
                _ => None,
            }
        }
    }
    let mut g = Graph::new();
    let src = g.add_node("motion.falloff.test.fsrc");
    let foc = g.add_node("motion.falloff");
    g.connect(Edge {
        from: (src, 0),
        to: (foc, 0),
        delayed: false,
    })
    .unwrap();
    // Defaults (Circle, radius 5, centre origin): field = 1 at x=0, 0 at
    // x=10. Composed with the carried [0.5, 0.8]: 0.5·1 and 0.8·0.
    let mut cook = Cook::new();
    let out = cook.cook(&g, &FOps, foc, 0.0).unwrap();
    match out[0].as_stream().get("falloff").unwrap() {
        Column::Scalar(v) => assert_eq!(v, &vec![0.5, 0.0]),
        _ => panic!("falloff"),
    }
}

#[test]
fn invert_flips_the_mask() {
    let mut g = Graph::new();
    let src = g.add_node("motion.falloff.test.src");
    let foc = g.add_node("motion.falloff");
    g.connect(Edge {
        from: (src, 0),
        to: (foc, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(foc, "invert", 1.0);
    // 1 - mask: x=0 → 0, x=5 → 1, x=10 → 1.
    assert_eq!(falloff_of(&g, &Ops, foc), vec![0.0, 1.0, 1.0]);
}

#[test]
fn linear_shape_ramps_across_x() {
    let mut g = Graph::new();
    let src = g.add_node("motion.falloff.test.src");
    let foc = g.add_node("motion.falloff");
    g.connect(Edge {
        from: (src, 0),
        to: (foc, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(foc, "shape", 2.0); // Linear
    g.set_param(foc, "curve", 0.0); // Linear curve → a pure ramp
    g.set_param(foc, "radius", 10.0);
    // s = x/10·0.5+0.5: x=0 → 0.5, x=5 → 0.75, x=10 → 1.0 (a left→right wipe).
    assert_eq!(falloff_of(&g, &Ops, foc), vec![0.5, 0.75, 1.0]);
}

#[test]
fn curves_are_smooth_and_endpoint_exact() {
    // Linear / Quad / Smooth / Smoother all map 0→0 and 1→1; the midpoint
    // differs (Linear .5, Quad .25, Smooth/Smoother symmetric .5).
    for k in 0..=3 {
        assert_eq!(curve(k, 0.0), 0.0, "curve {k} at 0");
        assert_eq!(curve(k, 1.0), 1.0, "curve {k} at 1");
    }
    assert_eq!(curve(0, 0.5), 0.5); // Linear
    assert_eq!(curve(1, 0.5), 0.25); // Quad
    assert_eq!(curve(2, 0.5), 0.5); // Smoothstep symmetric
    assert!((curve(3, 0.5) - 0.5).abs() < 1e-6); // Smootherstep symmetric
}

#[test]
fn rect_reaches_the_diagonal_corner_further_than_the_circle() {
    // At (3,3) with radius 5, curve Linear: Rect uses Chebyshev (max axis) →
    // s = 3/5 = .6 → 1−.6 = .4; Circle uses Euclidean → s = √18/5 ≈ .8485 →
    // 1−.8485 ≈ .1515. The box keeps more field into the corner.
    let rect = field(1, 3.0, 3.0, 5.0, 0, false);
    let circle = field(0, 3.0, 3.0, 5.0, 0, false);
    assert!((rect - 0.4).abs() < 1e-6);
    assert!((circle - 0.151_471_86).abs() < 1e-4);
    assert!(rect > circle);
}

#[test]
fn degenerate_radius_is_empty() {
    assert_eq!(field(0, 0.0, 0.0, 0.0, 2, false), 0.0);
    assert_eq!(field(2, 0.0, 0.0, 0.0, 2, false), 0.0);
}
