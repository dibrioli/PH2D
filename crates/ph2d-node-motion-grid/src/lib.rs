#![forbid(unsafe_code)]
//! `motion.grid` — a Motion **generator**: emits a `rows × cols` grid of
//! instances on the `P` (Vec2) attribute, spaced by `spacing` meters. No
//! inputs. Pure (combinational). The instance stream convention is
//! `ph2d-eval-motion`'s (P → world_pos).
//!
//! Params (manifest defaults until per-instance overrides land — node-waves.md):
//! `rows` (3), `cols` (3), `spacing` (1.0).

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.grid"),
    name: "motion.grid",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "rows",
            default: 3.0,
        },
        ParamSpec {
            name: "cols",
            default: 3.0,
        },
        ParamSpec {
            name: "spacing",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

fn param(name: &str) -> f32 {
    MANIFEST
        .params
        .iter()
        .find(|p| p.name == name)
        .map(|p| p.default)
        .unwrap_or(0.0)
}

struct MotionGrid;

impl NodeOp for MotionGrid {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let rows = param("rows").max(0.0) as usize;
        let cols = param("cols").max(0.0) as usize;
        let spacing = param("spacing");
        let mut positions = Vec::with_capacity(rows * cols);
        for r in 0..rows {
            for c in 0..cols {
                positions.push([c as f32 * spacing, r as f32 * spacing]);
            }
        }
        ctx.emit(Stream::new(positions.len()).with("P", Column::Vec2(positions)));
    }
}

pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionGrid))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::Graph;

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionGrid as &dyn NodeOp)
        }
    }

    #[test]
    fn emits_default_3x3_grid() {
        let mut g = Graph::new();
        let n = g.add_node("motion.grid");
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
        let p = out[0].get("P").unwrap();
        assert_eq!(out[0].count(), 9); // 3×3
        match p {
            Column::Vec2(v) => {
                assert_eq!(v[0], [0.0, 0.0]);
                assert_eq!(v[1], [1.0, 0.0]); // col 1, spacing 1.0
                assert_eq!(v[3], [0.0, 1.0]); // row 1
                assert_eq!(v[8], [2.0, 2.0]); // last
            }
            _ => panic!("P must be Vec2"),
        }
    }
}
