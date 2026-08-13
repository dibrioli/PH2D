//! Os gates do CURL — a geometria do campo (divergence-free, o potencial, o
//! acúmulo em `accel`).
//!
//! Saíram do `lib.rs` no teto de LOC, por assunto: o pai fica com **o que o nó
//! É** e o irmão `cluster_tests.rs` com **o cluster de noise herdado**. Segue
//! FILHO por `#[path]`, então `use super::*` alcança os privados.
/// A spec das gates de geometria: duas oitavas do fBm clássico.
const TEST_SPEC: ph2d_fbm::Spec = ph2d_fbm::Spec {
    octaves: 2,
    lacunarity: 2.0,
    roughness: 0.5,
    ty: ph2d_fbm::NoiseType::Fbm,
};

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("force.curl.test.src"),
    name: "force.curl.test.src",
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
        ctx.emit(Stream::new(3).with(
            "P",
            Column::Vec2(vec![[0.7, 1.3], [-2.1, 0.4], [3.3, -1.8]]),
        ));
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&Src),
            t if t == MANIFEST.id => Some(&ForceCurl),
            _ => None,
        }
    }
}

fn accel_at(playhead: f64) -> Vec<[f32; 2]> {
    let mut g = Graph::new();
    let src = g.add_node("force.curl.test.src");
    let c = g.add_node("force.curl");
    g.connect(Edge {
        from: (src, 0),
        to: (c, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, c, playhead).unwrap();
    match out[0].as_stream().get("accel").unwrap() {
        Column::Vec2(v) => v.clone(),
        _ => panic!("accel"),
    }
}

/// Divergence `∇·v` of a field, sampled with the SAME central-difference
/// step the field itself uses. Matching the stencil matters: the discrete
/// curl's mixed differences cancel exactly at step `EPS`, so any residue is
/// float rounding, not a modelling error. Measuring with a different `h`
/// would report the noise's own curvature, not the field's divergence.
fn divergence(field: impl Fn(f32, f32) -> [f32; 2], x: f32, y: f32) -> f32 {
    let inv = 1.0 / (2.0 * EPS);
    (field(x + EPS, y)[0] - field(x - EPS, y)[0]) * inv
        + (field(x, y + EPS)[1] - field(x, y - EPS)[1]) * inv
}

/// The raw gradient `(∂ψ/∂x, ∂ψ/∂y)` — the naive "push the particle along
/// the noise" field the curl exists to replace.
fn gradient(x: f32, y: f32) -> [f32; 2] {
    let inv = 1.0 / (2.0 * EPS);
    [
        (psi(x + EPS, y, 0.0, 0.0, TEST_SPEC, [0.0, 0.0])
            - psi(x - EPS, y, 0.0, 0.0, TEST_SPEC, [0.0, 0.0]))
            * inv,
        (psi(x, y + EPS, 0.0, 0.0, TEST_SPEC, [0.0, 0.0])
            - psi(x, y - EPS, 0.0, 0.0, TEST_SPEC, [0.0, 0.0]))
            * inv,
    ]
}

/// The property the whole node exists for (Bridson 2007): the field has
/// **zero divergence**, so particles swirl forever and never pile into a
/// sink. The same measurement on the raw gradient — the field you get by
/// sampling noise directly — shows divergence orders of magnitude larger,
/// which is exactly why curl noise is the published answer.
#[test]
fn the_curl_is_divergence_free_and_the_raw_gradient_is_not() {
    let (mut worst_curl, mut worst_grad) = (0.0f32, 0.0f32);
    for k in 0..64 {
        let (x, y) = (k as f32 * 0.37 - 7.0, k as f32 * 0.21 - 4.0);
        // Scale-relative, so a quiet patch of the field cannot flatter us.
        let mag = curl(
            x,
            y,
            0.0,
            0.0,
            ph2d_fbm::Spec {
                octaves: 2,
                ..ph2d_fbm::Spec::default()
            },
            [0.0, 0.0],
        )
        .iter()
        .map(|c| c.abs())
        .sum::<f32>()
        .max(1e-3);
        let d_curl = divergence(
            |a, b| {
                curl(
                    a,
                    b,
                    0.0,
                    0.0,
                    ph2d_fbm::Spec {
                        octaves: 2,
                        ..ph2d_fbm::Spec::default()
                    },
                    [0.0, 0.0],
                )
            },
            x,
            y,
        )
        .abs()
            / mag;
        let d_grad = divergence(gradient, x, y).abs() / mag;
        worst_curl = worst_curl.max(d_curl);
        worst_grad = worst_grad.max(d_grad);
    }
    assert!(
        worst_curl < 1e-2,
        "curl divergence must vanish, worst = {worst_curl}"
    );
    assert!(
        worst_grad > 1.0,
        "the raw gradient DOES diverge (that is the point), worst = {worst_grad}"
    );
}

#[test]
fn instances_feel_different_eddies_and_the_field_drifts() {
    let a = accel_at(0.0);
    assert!(
        a[0] != a[1] || a[1] != a[2],
        "distinct positions sample distinct swirl"
    );
    let b = accel_at(2.0);
    assert!(
        (a[0][0] - b[0][0]).abs() > 1e-6,
        "the field drifts with the playhead"
    );
}

#[test]
fn is_deterministic_for_replay() {
    assert_eq!(accel_at(0.8), accel_at(0.8));
}

#[test]
fn falloff_gates_the_force() {
    // A stream with falloff 0 on the middle instance: it feels nothing.
    static MASK_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("force.curl.test.mask"),
        name: "force.curl.test.mask",
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
    struct Mask;
    impl NodeOp for Mask {
        fn manifest(&self) -> &'static NodeManifest {
            &MASK_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(
                Stream::new(2)
                    .with("P", Column::Vec2(vec![[0.7, 1.3], [0.7, 1.3]]))
                    .with("falloff", Column::Scalar(vec![1.0, 0.0])),
            );
        }
    }
    struct MaskOps;
    impl OpResolver for MaskOps {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == MASK_MAN.id => Some(&Mask),
                t if t == MANIFEST.id => Some(&ForceCurl),
                _ => None,
            }
        }
    }
    let mut g = Graph::new();
    let src = g.add_node("force.curl.test.mask");
    let c = g.add_node("force.curl");
    g.connect(Edge {
        from: (src, 0),
        to: (c, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &MaskOps, c, 0.0).unwrap();
    match out[0].as_stream().get("accel").unwrap() {
        Column::Vec2(v) => {
            assert!(v[0] != [0.0, 0.0], "unmasked instance swirls");
            assert_eq!(v[1], [0.0, 0.0], "falloff 0 → no force");
        }
        _ => panic!("accel"),
    }
}

#[test]
fn registers_and_resolves() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
}
