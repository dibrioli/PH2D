//! Integration tests for `Graph::validate` — the semantic enforcement that the
//! audit (2026-05-21) flagged as missing: the membrane (`effect::can_feed`) and
//! the port-type compatibility (`port::connects_directly`) were defined but
//! never called on the graph's write/validate path. These tests prove they are
//! now actually enforced.

use ph2d_nodegraph::cook::{EvalCtx, OpResolver};
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::graph::{Edge, Graph, Violation};
use ph2d_nodegraph::node::{NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const FRAME: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const AUDIO: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Audio);

const fn p(name: &'static str, ty: PortType) -> PortSpec {
    PortSpec { name, ty }
}

static PURE_GEN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("v.pure_gen"),
    name: "v.pure_gen",
    inputs: &[],
    outputs: &[p("out", FRAME)],
    effect: Effect::Pure,
    clock: Clock::Frame,
};
static PURE_SINK: NodeManifest = NodeManifest {
    id: NodeTypeId::of("v.pure_sink"),
    name: "v.pure_sink",
    inputs: &[p("in", FRAME)],
    outputs: &[],
    effect: Effect::Pure,
    clock: Clock::Frame,
};
static STATEFUL_SRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("v.stateful_src"),
    name: "v.stateful_src",
    inputs: &[],
    outputs: &[p("out", FRAME)],
    effect: Effect::Stateful,
    clock: Clock::Frame,
};
static AUDIO_SINK: NodeManifest = NodeManifest {
    id: NodeTypeId::of("v.audio_sink"),
    name: "v.audio_sink",
    inputs: &[p("in", AUDIO)],
    outputs: &[],
    effect: Effect::Pure,
    clock: Clock::Audio,
};

macro_rules! op {
    ($ty:ident, $man:ident) => {
        struct $ty;
        impl NodeOp for $ty {
            fn manifest(&self) -> &'static NodeManifest {
                &$man
            }
            fn eval(&self, _ctx: &mut EvalCtx<'_>) {}
        }
    };
}
op!(PureGen, PURE_GEN);
op!(PureSink, PURE_SINK);
op!(StatefulSrc, STATEFUL_SRC);
op!(AudioSink, AUDIO_SINK);

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == PURE_GEN.id => Some(&PureGen),
            t if t == PURE_SINK.id => Some(&PureSink),
            t if t == STATEFUL_SRC.id => Some(&StatefulSrc),
            t if t == AUDIO_SINK.id => Some(&AudioSink),
            _ => None,
        }
    }
}

#[test]
fn valid_graph_passes() {
    let mut g = Graph::new();
    let generator = g.add_node("v.pure_gen");
    let sink = g.add_node("v.pure_sink");
    g.connect(Edge {
        from: (generator, 0),
        to: (sink, 0),
        delayed: false,
    })
    .unwrap();
    assert_eq!(g.validate(&Ops), Ok(()));
}

#[test]
fn membrane_violation_is_caught() {
    // Stateful (push) feeding a Pure (pull) node directly is forbidden — the
    // crossing must be an export, not a plain edge. connect() accepts it
    // structurally; validate() rejects it.
    let mut g = Graph::new();
    let src = g.add_node("v.stateful_src");
    let sink = g.add_node("v.pure_sink");
    g.connect(Edge {
        from: (src, 0),
        to: (sink, 0),
        delayed: false,
    })
    .unwrap();
    let err = g.validate(&Ops).unwrap_err();
    assert_eq!(
        err,
        vec![Violation::Membrane {
            from: (src, 0),
            to: (sink, 0)
        }]
    );
}

#[test]
fn clock_mismatch_is_caught() {
    // Frame output into an Audio input: same value shape, different clock —
    // exactly the leak the algebraic port type exists to prevent.
    let mut g = Graph::new();
    let generator = g.add_node("v.pure_gen");
    let sink = g.add_node("v.audio_sink");
    g.connect(Edge {
        from: (generator, 0),
        to: (sink, 0),
        delayed: false,
    })
    .unwrap();
    let err = g.validate(&Ops).unwrap_err();
    assert_eq!(
        err,
        vec![Violation::TypeMismatch {
            from: (generator, 0),
            to: (sink, 0)
        }]
    );
}

#[test]
fn bad_port_index_is_caught() {
    // connect() does not check port indices (it has no manifests); validate()
    // does. Edge from a non-existent output port 5.
    let mut g = Graph::new();
    let generator = g.add_node("v.pure_gen");
    let sink = g.add_node("v.pure_sink");
    g.connect(Edge {
        from: (generator, 5),
        to: (sink, 0),
        delayed: false,
    })
    .unwrap();
    let err = g.validate(&Ops).unwrap_err();
    assert_eq!(
        err,
        vec![Violation::BadOutputPort {
            node: generator,
            port: 5,
            n_outputs: 1
        }]
    );
}

#[test]
fn unknown_type_is_caught() {
    let mut g = Graph::new();
    let a = g.add_node("v.pure_gen");
    let ghost = g.add_node("v.not_registered");
    g.connect(Edge {
        from: (a, 0),
        to: (ghost, 0),
        delayed: false,
    })
    .unwrap();
    let err = g.validate(&Ops).unwrap_err();
    assert_eq!(err, vec![Violation::UnknownType { node: ghost }]);
}
