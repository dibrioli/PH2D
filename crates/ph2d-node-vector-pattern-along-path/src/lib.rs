#![forbid(unsafe_code)]
//! `vector.pattern-along-path` — repeat a **shape** network along a **path**
//! network (ADR-0058 §2.2 node #8; the 18th geometry node, deferred from W4).
//! Binary geometry node: input `0` = the shape to repeat, input `1` = the path
//! to follow. The engine lives in [`engine`].
//!
//! `Effect::Pure` (renderer-consumed; memory
//! `project_node_effect_pure_for_renderer_consumed`). Params: `count` (copies,
//! capped via [`param_as_count`]), `align` (`>0.5` rotates each copy to the path
//! tangent), `scale` (uniform per-copy scale).
//!
//! ## Scope (this is the *vector* flavour)
//!
//! This node repeats a **vector** shape — pure geometry→geometry, no Painter
//! dependency. The raster *brush*-along-path bridge (stamping a
//! `ph2d-painter-brush` along the path) is the separate, Painter-gated half of
//! the W8 "+ brush bridge" line and is **not** wired here.

mod engine;

pub use engine::{MAX_COUNT, pattern_along_path};

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, param_as_count,
};
use ph2d_nodegraph::port::Clock;
use ph2d_vector_doc::VectorNetwork;
use ph2d_vector_graph::{VECTOR_PORT, VectorEvalExt};

/// The static contract of this node type (ADR-0031 / ADR-0058 §2.2).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("vector.pattern-along-path"),
    name: "vector.pattern-along-path",
    inputs: &[
        PortSpec {
            name: "shape",
            ty: VECTOR_PORT,
        },
        PortSpec {
            name: "path",
            ty: VECTOR_PORT,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: VECTOR_PORT,
    }],
    effect: Effect::Pure,
    clock: Clock::Static,
    params: &[
        ParamSpec {
            name: "count",
            default: 10.0,
        },
        ParamSpec {
            name: "align",
            default: 1.0,
        },
        ParamSpec {
            name: "scale",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct VectorPatternAlongPath;

impl NodeOp for VectorPatternAlongPath {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let count = param_as_count(ctx.param("count"), MAX_COUNT);
        let align = ctx.param("align") > 0.5;
        let scale = ctx.param("scale");
        let empty = VectorNetwork::empty();
        let shape = ctx.input_network(0).unwrap_or(&empty);
        let path = ctx.input_network(1).unwrap_or(&empty);
        let out = engine::pattern_along_path(shape, path, count, align, scale);
        ctx.emit_network(out);
    }
}

/// Register this node with the runtime registry (codegen entry point).
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(VectorPatternAlongPath))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};
    use ph2d_vector_doc::{Segment, Vertex, primitives};

    fn shape_src() -> VectorNetwork {
        let mut net = primitives::rect(glam::Vec2::new(-1.0, -1.0), glam::Vec2::new(1.0, 1.0));
        net.deterministic = true;
        net
    }

    fn path_src() -> VectorNetwork {
        let mut net = VectorNetwork::empty();
        net.vertices
            .push(Vertex::auto(0, glam::Vec2::new(0.0, 0.0)));
        net.vertices
            .push(Vertex::auto(1, glam::Vec2::new(120.0, 0.0)));
        net.segments.push(Segment::straight(0, 0, 1));
        net.deterministic = true;
        net
    }

    static SHAPE_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("vector.test.shape"),
        name: "vector.test.shape",
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
    static PATH_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("vector.test.path"),
        name: "vector.test.path",
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

    struct ShapeSrc;
    impl NodeOp for ShapeSrc {
        fn manifest(&self) -> &'static NodeManifest {
            &SHAPE_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit_network(shape_src());
        }
    }
    struct PathSrc;
    impl NodeOp for PathSrc {
        fn manifest(&self) -> &'static NodeManifest {
            &PATH_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit_network(path_src());
        }
    }

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SHAPE_MAN.id => Some(&ShapeSrc),
                t if t == PATH_MAN.id => Some(&PathSrc),
                t if t == MANIFEST.id => Some(&VectorPatternAlongPath),
                _ => None,
            }
        }
    }

    #[test]
    fn pattern_through_a_real_cook_repeats_shape_along_path() {
        let mut g = Graph::new();
        let shape = g.add_node("vector.test.shape");
        let path = g.add_node("vector.test.path");
        let pat = g.add_node("vector.pattern-along-path");
        g.set_param(pat, "count", 6.0);
        g.connect(Edge {
            from: (shape, 0),
            to: (pat, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (path, 0),
            to: (pat, 1),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, pat, 0.0).unwrap();
        let net = out[0]
            .as_any()
            .and_then(|x| x.downcast_ref::<VectorNetwork>())
            .expect("output carries a VectorNetwork");
        assert!(net.validate().is_ok());
        assert_eq!(net.regions.len(), 6, "6 copies along the path");
    }
}
