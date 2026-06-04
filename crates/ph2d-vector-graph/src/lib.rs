#![forbid(unsafe_code)]
//! The vector domain's glue between the `ph2d-nodegraph` substrate and the
//! `ph2d-vector-doc` data model — the typed boundary for geometry nodes
//! (ADR-0058 + **ADR-0058-amendment-1**).
//!
//! The substrate carries a node output port's value as a [`CookValue`]: an
//! instance [`Stream`](ph2d_nodegraph::attr::Stream) for motion/field, or an
//! `Opaque(Arc<dyn Any>)` for a domain-specific rich value. A geometry edge
//! carries a [`VectorNetwork`] behind that opaque channel. To keep the
//! substrate domain-agnostic (it has **zero** deps and is shared by
//! motion/field/signal), `ph2d-nodegraph` never names `VectorNetwork`; this
//! crate provides the typed accessors that wrap/downcast it.
//!
//! A vector node crate (`ph2d-node-vector-*`) depends on this crate and:
//! - declares its geometry output port with [`VECTOR_PORT`];
//! - emits its result with [`VectorEvalExt::emit_network`];
//! - reads an upstream network with [`VectorEvalExt::input_network`].
//!
//! It never touches `Arc<dyn Any>` directly.

use std::sync::Arc;

use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use ph2d_vector_doc::VectorNetwork;

/// The canonical port type for a vector geometry edge: domain
/// [`Domain::Vector`], cooked once ([`Clock::Static`] — a source is static
/// until a param edit re-cooks it). The [`Dim`] axis is **not meaningful** for
/// an opaque whole-network value (there is no per-element width); [`Dim::Scalar`]
/// is the canonical sentinel so every vector port shares one [`PortType`] and
/// thus connects (`connects_directly` matches on all three axes). Use this for
/// every geometry input/output rather than re-spelling the sentinel.
pub const VECTOR_PORT: PortType = PortType::new(Domain::Vector, Dim::Scalar, Clock::Static);

/// Typed geometry accessors over the substrate's opaque value channel. The
/// node author calls these instead of `EvalCtx::{input_any, emit_any}` +
/// `Arc<dyn Any>` downcasting.
pub trait VectorEvalExt {
    /// The [`VectorNetwork`] on input `port`, or `None` if that input is
    /// unconnected or does not carry a geometry value. (A wrongly-typed edge is
    /// already barred at connect time by [`VECTOR_PORT`]'s domain check, so a
    /// connected geometry input downcasts successfully.)
    fn input_network(&self, port: usize) -> Option<&VectorNetwork>;

    /// Emit `net` as this node's next output port value (call once per output
    /// port, in declared order — same discipline as `EvalCtx::emit`).
    fn emit_network(&mut self, net: VectorNetwork);
}

impl VectorEvalExt for EvalCtx<'_> {
    fn input_network(&self, port: usize) -> Option<&VectorNetwork> {
        self.input_any(port)?.downcast_ref::<VectorNetwork>()
    }

    fn emit_network(&mut self, net: VectorNetwork) {
        self.emit_any(Arc::new(net));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::effect::Effect;
    use ph2d_nodegraph::graph::{Edge, Graph};
    use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};

    // A source that emits a one-vertex-free network, and a passthrough that
    // reads it back via `input_network` — proves the opaque geometry value
    // survives a real cook (emit_network → CookValue::Opaque → input_network).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("vector.test.src"),
        name: "vector.test.src",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: VECTOR_PORT,
        }],
        effect: Effect::Pure,
        clock: Clock::Static,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Src;
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            let mut net = VectorNetwork::empty();
            net.deterministic = true;
            net.vertices
                .push(ph2d_vector_doc::Vertex::auto(0, glam::Vec2::new(3.0, 4.0)));
            ctx.emit_network(net);
        }
    }

    static PASS_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("vector.test.pass"),
        name: "vector.test.pass",
        inputs: &[PortSpec {
            name: "in",
            ty: VECTOR_PORT,
        }],
        // Emits a Scalar stream carrying the input network's vertex count — a
        // probe so the test can read a plain value out of the cook.
        outputs: &[PortSpec {
            name: "count",
            ty: ph2d_nodegraph::port::PortType::new(Domain::Instances, Dim::Scalar, Clock::Static),
        }],
        effect: Effect::Pure,
        clock: Clock::Static,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Pass;
    impl NodeOp for Pass {
        fn manifest(&self) -> &'static NodeManifest {
            &PASS_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            let n = ctx.input_network(0).map_or(0.0, |net| net.vertices.len() as f32);
            ctx.emit(Stream::new(1).with("v", Column::Scalar(vec![n])));
        }
    }

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == PASS_MAN.id => Some(&Pass),
                _ => None,
            }
        }
    }

    #[test]
    fn geometry_survives_a_real_cook() {
        // src(network with 1 vertex) → pass(reads input_network) → count=1.
        let mut g = Graph::new();
        let src = g.add_node("vector.test.src");
        let pass = g.add_node("vector.test.pass");
        g.connect(Edge {
            from: (src, 0),
            to: (pass, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, pass, 0.0).unwrap();
        match out[0].as_stream().get("v") {
            Some(Column::Scalar(v)) => assert_eq!(v, &vec![1.0]),
            other => panic!("expected the vertex count, got {other:?}"),
        }
    }

    #[test]
    fn input_network_is_none_when_unconnected() {
        // pass with no upstream → input_network(0) is None → count 0.
        let mut g = Graph::new();
        let pass = g.add_node("vector.test.pass");
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, pass, 0.0).unwrap();
        match out[0].as_stream().get("v") {
            Some(Column::Scalar(v)) => assert_eq!(v, &vec![0.0]),
            other => panic!("expected 0, got {other:?}"),
        }
    }
}
