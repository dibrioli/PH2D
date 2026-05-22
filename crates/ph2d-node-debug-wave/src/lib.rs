#![forbid(unsafe_code)]
//! `debug.wave` — the **canonical worked example** a fan-out agent copies
//! (W1.T5). Unlike `debug.const` (trivial generator), this exercises every
//! pattern a real node needs:
//!
//! - an **input** port (reads the upstream stream's `v` column),
//! - a **parameter** (`gain`, declared in the manifest with a default),
//! - **`Temporal`** effect (it reads `ctx.playhead()` — the pull-side clock),
//! - **`ph2d-expr`** for the per-element math (`eval_column` realizes the field
//!   over the input stream — the "VOP → cook" step),
//! - a **golden input→output test** (the property test ADR-0031 §3 wants).
//!
//! Compute: `out[i] = in.v[i] * gain + sin(t)`.
//!
//! NOTE on params: the manifest declares the param *spec + default*; `eval`
//! reads the live value via [`EvalCtx::param`] — the graph's per-instance
//! override if the author set one ([`ph2d_nodegraph::graph::Graph::set_param`]),
//! else the manifest default. Reading a name the manifest does not declare
//! panics (caught by the golden test), never a silent `0.0`.

use ph2d_expr::{BinOp, Expr, Func, eval_column};
use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const SCALAR_FRAME: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("debug.wave"),
    name: "debug.wave",
    inputs: &[PortSpec {
        name: "in",
        ty: SCALAR_FRAME,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: SCALAR_FRAME,
    }],
    effect: Effect::Temporal, // reads the playhead → pull-side, HR-5-exempt
    clock: Clock::Frame,
    params: &[ParamSpec {
        name: "gain",
        default: 1.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

struct DebugWave;

impl NodeOp for DebugWave {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // out = v * gain + sin(t)   (t = playhead, injected as a param)
        let expr = Expr::bin(
            BinOp::Add,
            Expr::bin(BinOp::Mul, Expr::attr("v"), Expr::param("gain")),
            Expr::call(Func::Sin, vec![Expr::param("t")]),
        );
        let t = ctx.playhead() as f32;
        // `gain` resolves to the graph's per-instance override if set, else the
        // manifest default (`EvalCtx::param`); reading an undeclared name would
        // panic (caught by the golden test), never silently read 0.0.
        let gain = ctx.param("gain");
        let params = |name: &str| match name {
            "gain" => gain,
            "t" => t,
            _ => 0.0,
        };
        let out = {
            let input = ctx.input(0);
            eval_column(&expr, input, &params)
        };
        ctx.emit(Stream::new(out.len()).with("v", Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(DebugWave))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // A minimal in-test source feeding [10, 20, 30] on `v` — the kind of stub a
    // golden test pairs with the node under test.
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("debug.wave.test.src"),
        name: "debug.wave.test.src",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: SCALAR_FRAME,
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
            ctx.emit(Stream::new(3).with("v", Column::Scalar(vec![10.0, 20.0, 30.0])));
        }
    }

    struct Ops {
        src: Src,
        wave: DebugWave,
    }
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&self.src),
                t if t == MANIFEST.id => Some(&self.wave),
                _ => None,
            }
        }
    }

    #[test]
    fn golden_at_playhead_zero() {
        // at t=0, sin(0)=0 and gain=1 → out = in unchanged. Deterministic golden
        // (no transcendental flakiness at the origin).
        let mut g = Graph::new();
        let src = g.add_node("debug.wave.test.src");
        let wave = g.add_node("debug.wave");
        g.connect(Edge {
            from: (src, 0),
            to: (wave, 0),
            delayed: false,
        })
        .unwrap();

        let ops = Ops {
            src: Src,
            wave: DebugWave,
        };
        g.validate(&ops).expect("well-typed");
        let mut cook = Cook::new();
        let out = cook.cook(&g, &ops, wave, 0.0).unwrap();
        match out[0].get("v") {
            Some(Column::Scalar(v)) => assert_eq!(v, &vec![10.0, 20.0, 30.0]),
            other => panic!("expected scalar column, got {other:?}"),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
