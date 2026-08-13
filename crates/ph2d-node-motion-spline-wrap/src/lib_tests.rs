//! Os gates do EMBRULHO — a lei da cúbica, o falloff e a máscara.
//!
//! Saíram do `lib.rs` no teto de LOC, por assunto: o pai fica com **o que o nó É**
//! (o manifesto, a curva, o embrulho, o registro) e este irmão com **o que ele
//! promete**. Segue FILHO por `#[path]`, então `use super::*` alcança os privados.
use super::*;

/// A curva INTEIRA sem deslize -- o mapeamento que shipava.
const WHOLE: ArcMap = ArcMap {
    from: 0.0,
    to: 1.0,
    offset: 0.0,
};
const S_CURVE: [P2; 4] = [[-3.0, -1.5], [-1.0, 2.0], [1.0, -2.0], [3.0, 1.5]];
const LINE: [P2; 4] = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]];
// A symmetric arch (hump) — its arc-midpoint lifts clearly off the endpoint chord,
// unlike the antisymmetric S-curve whose midpoint sits *on* the chord.
const ARCH: [P2; 4] = [[-3.0, 0.0], [-1.0, 3.0], [1.0, 3.0], [3.0, 0.0]];

/// `amount` 0 is the identity — the layout is untouched.
#[test]
fn amount_zero_is_the_identity() {
    let p = vec![[-2.0, 0.5], [0.0, -0.3], [2.0, 0.1]];
    let out = wrap(&p, &Curve::cubic(&S_CURVE), 1.0, WHOLE, 0.0, &[]);
    for (o, q) in out.iter().zip(&p) {
        assert!(
            (o[0] - q[0]).abs() < 1e-5 && (o[1] - q[1]).abs() < 1e-5,
            "{o:?} vs {q:?}"
        );
    }
}

/// Wrapping onto a straight horizontal line keeps a straight input row straight (the
/// remap is affine there): three points at constant y stay collinear.
#[test]
fn a_row_on_a_straight_curve_stays_straight() {
    let p = vec![[-2.0, 0.4], [0.0, 0.4], [2.0, 0.4]];
    let out = wrap(&p, &Curve::cubic(&LINE), 1.0, WHOLE, 1.0, &[]);
    // Constant normal (+y) ⇒ all share the same y; collinear.
    assert!((out[0][1] - out[1][1]).abs() < 1e-3 && (out[1][1] - out[2][1]).abs() < 1e-3);
}

/// Wrapping onto a curved spline BENDS a straight input row: the midpoint leaves the
/// chord between the endpoints. FALSIFIED by a flat deformer (midpoint on the chord).
#[test]
fn a_row_on_a_curved_spline_bends() {
    let p = vec![[-3.0, 0.0], [0.0, 0.0], [3.0, 0.0]]; // a straight row along x
    let out = wrap(&p, &Curve::cubic(&ARCH), 1.0, WHOLE, 1.0, &[]);
    // Cross product of (mid−a) and (b−a): non-zero ⇒ the midpoint bent off the line.
    let (a, mid, b) = (out[0], out[1], out[2]);
    let cross = (mid[0] - a[0]) * (b[1] - a[1]) - (mid[1] - a[1]) * (b[0] - a[0]);
    assert!(cross.abs() > 0.5, "the row bent (cross {cross})");
}

/// Falloff masks the wrap per element: falloff 0 leaves an element where it was.
#[test]
fn falloff_masks_the_wrap() {
    let p = vec![[-3.0, 0.0], [0.0, 0.0], [3.0, 0.0]];
    let falloff = vec![1.0, 0.0, 1.0]; // middle element pinned
    let out = wrap(&p, &Curve::cubic(&S_CURVE), 1.0, WHOLE, 1.0, &falloff);
    assert_eq!(out[1], p[1], "falloff 0 -> unchanged");
}

/// Deterministic + cooks through the registry, copying columns and wrapping P.
#[test]
fn registers_and_wraps_through_the_cook() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.spline_wrap.test.src"),
        name: "motion.spline_wrap.test.src",
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
            &SRC
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(
                Stream::new(3)
                    .with("P", Column::Vec2(vec![[-3.0, 0.0], [0.0, 0.0], [3.0, 0.0]]))
                    .with("size", Column::Vec2(vec![[0.3, 0.3]; 3])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionSplineWrap),
                _ => None,
            }
        }
    }
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());

    let mut g = Graph::new();
    let src = g.add_node("motion.spline_wrap.test.src");
    let sw = g.add_node("motion.spline_wrap");
    g.connect(Edge {
        from: (src, 0),
        to: (sw, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, sw, 0.0).unwrap();
    let s = out[0].as_stream();
    assert!(s.get("size").is_some(), "columns pass through");
    match s.get("P").unwrap() {
        Column::Vec2(v) => {
            // The wrapped row is no longer flat on y = 0 (the S-curve lifted it).
            assert!(
                v.iter().any(|q| q[1].abs() > 0.3),
                "wrapped off the axis: {v:?}"
            );
        }
        _ => panic!("P"),
    }
}
