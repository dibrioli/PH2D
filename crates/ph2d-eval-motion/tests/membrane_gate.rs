//! W2.T3 (headless) — the **membrane arch-gate**, run against the *real*
//! `NodeRegistry` with the real Motion nodes registered.
//!
//! The membrane mechanism (`effect::can_feed`, `port::connects_directly`) and
//! its enforcement point (`Graph::validate`) are unit-proven in
//! `ph2d-nodegraph/tests/validate.rs` with synthetic nodes. This gate proves
//! the next thing the editor relies on: that **validating an authored graph
//! built from the registered nodes** actually rejects the two illegal-edge
//! classes — a `Stateful` (push) node feeding a presentation (pull) Motion node
//! by a plain edge, and a port-type (here dimensionality) mismatch — while a
//! well-typed Motion graph passes. This is the "run `validate` on load" check
//! the W2.T3 editor view will call before cooking; the on-screen view +
//! live-preview + visual smoke are the human (Enio) step, not this test.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::graph::{Edge, Graph, Violation};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
const INST_SCALAR: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// A `Stateful` (gameplay/push) source emitting a Vec2 instance stream. Its
/// port type matches `motion.transform`'s input exactly, so the *only* reason a
/// plain edge into a Motion node is illegal is the membrane (push → pull).
static STATEFUL_SRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("test.membrane.stateful_src"),
    name: "test.membrane.stateful_src",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Stateful,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

/// A `Pure` source emitting a *Scalar* instance stream — well-behaved on the
/// membrane, but its dimensionality does not match `motion.transform`'s Vec2
/// input, so a plain edge is a `TypeMismatch`.
static SCALAR_SRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("test.membrane.scalar_src"),
    name: "test.membrane.scalar_src",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_SCALAR,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

struct StatefulSrc;
impl NodeOp for StatefulSrc {
    fn manifest(&self) -> &'static NodeManifest {
        &STATEFUL_SRC
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[0.0, 0.0]])));
    }
}

struct ScalarSrc;
impl NodeOp for ScalarSrc {
    fn manifest(&self) -> &'static NodeManifest {
        &SCALAR_SRC
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(1).with("v", Column::Scalar(vec![0.0])));
    }
}

/// The runtime registry as it would be at load: the real Motion nodes plus the
/// two adversarial source stubs this gate connects them to.
fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_transform::register(&mut reg).unwrap();
    ph2d_node_motion_clone::register(&mut reg).unwrap();
    reg.register(Box::new(StatefulSrc)).unwrap();
    reg.register(Box::new(ScalarSrc)).unwrap();
    reg
}

#[test]
fn validate_rejects_stateful_feeding_a_motion_node() {
    // A gameplay (Stateful) node feeding a Motion (Pure) node by a plain edge is
    // the membrane crossing the architecture forbids — it must go through an
    // export, not a direct edge. `connect` accepts it structurally (the port
    // types match); `validate` is what rejects it.
    let reg = registry();
    let mut g = Graph::new();
    let src = g.add_node("test.membrane.stateful_src");
    let xf = g.add_node("motion.transform");
    g.connect(Edge {
        from: (src, 0),
        to: (xf, 0),
        delayed: false,
    })
    .unwrap();

    let violations = g
        .validate(&reg)
        .expect_err("membrane crossing must be rejected");
    assert_eq!(
        violations,
        vec![Violation::Membrane {
            from: (src, 0),
            to: (xf, 0),
        }]
    );
}

#[test]
fn validate_rejects_dim_mismatch_into_a_motion_node() {
    // A Scalar instance stream into `motion.transform`'s Vec2 input: same
    // domain + clock, wrong dimensionality. The algebraic port type catches it.
    let reg = registry();
    let mut g = Graph::new();
    let src = g.add_node("test.membrane.scalar_src");
    let xf = g.add_node("motion.transform");
    g.connect(Edge {
        from: (src, 0),
        to: (xf, 0),
        delayed: false,
    })
    .unwrap();

    let violations = g.validate(&reg).expect_err("dim mismatch must be rejected");
    assert_eq!(
        violations,
        vec![Violation::TypeMismatch {
            from: (src, 0),
            to: (xf, 0),
        }]
    );
}

#[test]
fn validate_accepts_the_well_typed_motion_vertical() {
    // The positive control: the real grid → transform → clone graph is all
    // Instances/Vec2/Frame and all Pure, so validate passes — proving the gate
    // rejects the illegal cases above on their merits, not because validate
    // rejects everything.
    let reg = registry();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let xf = g.add_node("motion.transform");
    let clone = g.add_node("motion.clone");
    g.connect(Edge {
        from: (grid, 0),
        to: (xf, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (xf, 0),
        to: (clone, 0),
        delayed: false,
    })
    .unwrap();
    assert_eq!(g.validate(&reg), Ok(()));
}
