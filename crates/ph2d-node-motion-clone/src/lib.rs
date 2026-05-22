#![forbid(unsafe_code)]
//! `motion.clone` — a Motion **cloner**: multiplies its input instance stream
//! into `count` copies, each offset on `P` by `copy_index * (step_x, step_y)`.
//! Other columns are replicated unchanged. This is a **stream multiplier**
//! (1 node → N×in instances) — NOT entity spawning; it has no ECS analogue
//! (ADR-0035). Output count = `in_count * count`. Pure.
//!
//! Params (manifest defaults): `count` (3), `step_x` (2.0), `step_y` (0.0).

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.clone"),
    name: "motion.clone",
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
            name: "count",
            default: 3.0,
        },
        ParamSpec {
            name: "step_x",
            default: 2.0,
        },
        ParamSpec {
            name: "step_y",
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

/// Replicate a column `k` times (copy 0, copy 1, ... — element order within a
/// copy preserved), matching the `P` offset loop below.
fn replicate(col: &Column, k: usize) -> Column {
    fn rep<T: Clone>(v: &[T], k: usize) -> Vec<T> {
        let mut out = Vec::with_capacity(v.len() * k);
        for _ in 0..k {
            out.extend_from_slice(v);
        }
        out
    }
    match col {
        Column::Scalar(v) => Column::Scalar(rep(v, k)),
        Column::Vec2(v) => Column::Vec2(rep(v, k)),
        Column::Vec3(v) => Column::Vec3(rep(v, k)),
        Column::Vec4(v) => Column::Vec4(rep(v, k)),
    }
}

struct MotionClone;

impl NodeOp for MotionClone {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let k = (param("count").max(1.0)) as usize; // at least one copy
        let (sx, sy) = (param("step_x"), param("step_y"));
        let out = {
            let input = ctx.input(0);
            let in_count = input.count();
            let mut out = Stream::new(in_count * k);
            for (name, col) in input.columns() {
                match (name.as_str(), col) {
                    ("P", Column::Vec2(v)) => {
                        let mut nv = Vec::with_capacity(in_count * k);
                        for copy in 0..k {
                            let (dx, dy) = (copy as f32 * sx, copy as f32 * sy);
                            for p in v {
                                nv.push([p[0] + dx, p[1] + dy]);
                            }
                        }
                        out.set("P", Column::Vec2(nv));
                    }
                    _ => out.set(name.clone(), replicate(col, k)),
                }
            }
            out
        };
        ctx.emit(out);
    }
}

pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionClone))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.clone.test.src"),
        name: "motion.clone.test.src",
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
            ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[0.0, 0.0]])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionClone),
                _ => None,
            }
        }
    }

    #[test]
    fn multiplies_stream_with_per_copy_offset() {
        // 1 instance × default count 3, step_x 2 → 3 instances at x=0,2,4.
        let mut g = Graph::new();
        let src = g.add_node("motion.clone.test.src");
        let clone = g.add_node("motion.clone");
        g.connect(Edge {
            from: (src, 0),
            to: (clone, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, clone, 0.0).unwrap();
        assert_eq!(out[0].count(), 3);
        match out[0].get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[0.0, 0.0], [2.0, 0.0], [4.0, 0.0]]),
            _ => panic!("P"),
        }
    }
}
