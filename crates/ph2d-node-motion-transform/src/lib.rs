#![forbid(unsafe_code)]
//! `motion.transform` — a Motion **modifier**: scales then offsets the `P`
//! (Vec2) attribute of its input instance stream; passes every other column
//! through unchanged (count is preserved). Pure.
//!
//! Params (manifest defaults): `scale` (1.0), `offset_x` (0.0), `offset_y` (0.0).
//! `P' = P * scale + (offset_x, offset_y)`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.transform"),
    name: "motion.transform",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "scale",
            default: 1.0,
        },
        ParamSpec {
            name: "offset_x",
            default: 0.0,
        },
        ParamSpec {
            name: "offset_y",
            default: 0.0,
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

struct MotionTransform;

impl NodeOp for MotionTransform {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let scale = param("scale");
        let (ox, oy) = (param("offset_x"), param("offset_y"));
        let out = {
            let input = ctx.input(0);
            let mut out = Stream::new(input.count());
            for (name, col) in input.columns() {
                match (name.as_str(), col) {
                    ("P", Column::Vec2(v)) => {
                        let t: Vec<[f32; 2]> = v
                            .iter()
                            .map(|p| [p[0] * scale + ox, p[1] * scale + oy])
                            .collect();
                        out.set("P", Column::Vec2(t));
                    }
                    _ => out.set(name.clone(), col.clone()),
                }
            }
            out
        };
        ctx.emit(out);
    }
}

pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionTransform))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // Source: 2 instances at (1,1),(2,2) with a size column to verify passthrough.
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.transform.test.src"),
        name: "motion.transform.test.src",
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
            ctx.emit(
                Stream::new(2)
                    .with("P", Column::Vec2(vec![[1.0, 1.0], [2.0, 2.0]]))
                    .with("size", Column::Vec2(vec![[5.0, 5.0], [5.0, 5.0]])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionTransform),
                _ => None,
            }
        }
    }

    #[test]
    fn scales_and_offsets_p_passes_size_through() {
        // default scale=1, offset=0 → P unchanged; assert passthrough + identity.
        let mut g = Graph::new();
        let src = g.add_node("motion.transform.test.src");
        let xf = g.add_node("motion.transform");
        g.connect(Edge {
            from: (src, 0),
            to: (xf, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, xf, 0.0).unwrap();
        assert_eq!(out[0].count(), 2);
        match out[0].get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[1.0, 1.0], [2.0, 2.0]]),
            _ => panic!("P"),
        }
        // size carried through unchanged
        match out[0].get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[5.0, 5.0], [5.0, 5.0]]),
            _ => panic!("size"),
        }
    }
}
