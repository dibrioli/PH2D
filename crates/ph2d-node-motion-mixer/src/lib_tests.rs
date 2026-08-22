//! Gates do `motion.mixer` — a redução, os modos e a geometria por lane.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o
//! corte é o que a crate já usa nos dois irmãos ao lado (`weights_tests.rs` e
//! `blend_field_tests.rs`): a LEI no `lib.rs`, as PROVAS num arquivo por assunto.

use super::*;

/// Avg mode (the production default arm), named here for the tests' readability.
const MODE_AVG: i64 = 0;

fn snap_p(p: Vec<[f32; 2]>) -> Snap {
    Snap {
        count: p.len(),
        cols: vec![("P".to_string(), Column::Vec2(p))],
    }
}

fn p_of(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P").unwrap() {
        Column::Vec2(v) => v.clone(),
        _ => panic!("P"),
    }
}

/// Avg is the midpoint of the inputs: two points averaged land halfway between.
#[test]
fn avg_is_the_midpoint() {
    let a = snap_p(vec![[0.0, 0.0], [2.0, 0.0]]);
    let b = snap_p(vec![[4.0, 0.0], [2.0, 4.0]]);
    let out = mix(MODE_AVG, &[&a, &b], &[0.5], &[1.0, 1.0], None);
    assert_eq!(p_of(&out), vec![[2.0, 0.0], [2.0, 2.0]]);
}

/// Add sums the inputs component-wise.
#[test]
fn add_sums_the_inputs() {
    let a = snap_p(vec![[1.0, 1.0]]);
    let b = snap_p(vec![[2.0, 3.0]]);
    let out = mix(MODE_ADD, &[&a, &b], &[0.5], &[1.0, 1.0], None);
    assert_eq!(p_of(&out), vec![[3.0, 4.0]]);
}

/// Blend lerps in0→in1: weight 0 is in0, 1 is in1, 0.25 is a quarter across.
/// FALSIFIED by an averaging that ignores the weight.
#[test]
fn blend_lerps_in0_to_in1() {
    let a = snap_p(vec![[0.0, 0.0]]);
    let b = snap_p(vec![[4.0, 8.0]]);
    assert_eq!(
        p_of(&mix(MODE_BLEND, &[&a, &b], &[0.0], &[1.0, 1.0], None)),
        vec![[0.0, 0.0]]
    );
    assert_eq!(
        p_of(&mix(MODE_BLEND, &[&a, &b], &[1.0], &[1.0, 1.0], None)),
        vec![[4.0, 8.0]]
    );
    assert_eq!(
        p_of(&mix(MODE_BLEND, &[&a, &b], &[0.25], &[1.0, 1.0], None)),
        vec![[1.0, 2.0]]
    );
}

/// Mismatched counts blend the common prefix (the minimum count).
#[test]
fn count_is_the_minimum() {
    let a = snap_p(vec![[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]]);
    let b = snap_p(vec![[2.0, 2.0]]);
    let out = mix(MODE_AVG, &[&a, &b], &[0.5], &[1.0, 1.0], None);
    assert_eq!(out.count(), 1, "truncated to the shorter input");
}

/// Deterministic + cooks through the registry: two sources blend to their midpoint at
/// blend 0.5.
#[test]
fn registers_and_mixes_through_the_cook() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    const fn src(id: &'static str) -> NodeManifest {
        NodeManifest {
            id: NodeTypeId::of(id),
            name: id,
            inputs: &[],
            outputs: &[PortSpec {
                name: "out",
                ty: INST_VEC2,
            }],
            effect: Effect::Pure,
            clock: Clock::Frame,
            params: &[] as &[ParamSpec],
            lowerings: &[LoweringKind::Cpu],
        }
    }
    static SA: NodeManifest = src("motion.mixer.test.a");
    static SB: NodeManifest = src("motion.mixer.test.b");
    struct A;
    impl NodeOp for A {
        fn manifest(&self) -> &'static NodeManifest {
            &SA
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0]])));
        }
    }
    struct B;
    impl NodeOp for B {
        fn manifest(&self) -> &'static NodeManifest {
            &SB
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[4.0, 0.0], [4.0, 0.0]])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SA.id => Some(&A),
                t if t == SB.id => Some(&B),
                t if t == MANIFEST.id => Some(&MotionMixer),
                _ => None,
            }
        }
    }
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());

    let mut g = Graph::new();
    let a = g.add_node("motion.mixer.test.a");
    let b = g.add_node("motion.mixer.test.b");
    let m = g.add_node("motion.mixer");
    g.connect(Edge {
        from: (a, 0),
        to: (m, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (b, 0),
        to: (m, 1),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, m, 0.0).unwrap();
    match out[0].as_stream().get("P").unwrap() {
        Column::Vec2(v) => assert_eq!(v[0], [2.0, 0.0], "midpoint of the two sources"),
        _ => panic!("P"),
    }
}
