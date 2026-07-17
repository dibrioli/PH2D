#![forbid(unsafe_code)]
//! `motion.move` — a Motion **modifier**: adds a constant offset `(dx, dy)` to
//! the `P` (Vec2) attribute of its input stream, scaled per-instance by the
//! multiplicative `falloff` column (§1.2; absent → `1.0`, full effect). Every
//! other column passes through unchanged (count preserved). Pure.
//!
//! Params (read via `ctx.param`): `dx` (0), `dy` (0).
//! `P'_i = P_i + (dx, dy) * falloff_i`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream, par_build};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.move"),
    name: "motion.move",
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
            name: "dx",
            default: 0.0,
        },
        ParamSpec {
            name: "dy",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The multiplicative `falloff` weight for instance `i` (absent column or short
/// column → `1.0`, i.e. full effect). Shared shape across all modifiers.
fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// GPU compute kernel (GPU/M5 Fase 1, ADR-0126): `P' = P + (dx, dy) · falloff`,
/// the exact per-element map of the CPU `eval` (same multiply/add order → same
/// float result up to GPU FMA contraction, covered by the ε parity gate).
/// `ReadWriteExisting` mirrors the CPU's pattern-match: a stream WITHOUT a `P`
/// column passes through untouched (the CPU only rewrites `P` when it exists),
/// so absence means the same thing on both paths.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let mv_f = read_falloff(i);\n\
        let mv_p = read_P(i);\n\
        write_P(i, vec2<f32>(mv_p.x + params.dx * mv_f, mv_p.y + params.dy * mv_f));\n",
    wgsl_lib: "",
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWriteExisting,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
    ],
    params: &["dx", "dy"],
    source_count: None,
    applicable: None,
};

struct MotionMove;

impl NodeOp for MotionMove {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let (dx, dy) = (ctx.param("dx"), ctx.param("dy"));
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            let mut out = Stream::new(n);
            for (name, col) in input.columns() {
                match (name.as_str(), col) {
                    ("P", Column::Vec2(v)) => {
                        // Pure per-instance map → parallel above the threshold
                        // (bit-identical, no reduction). GPU/M5 Fase 0.
                        let moved: Vec<[f32; 2]> = par_build(v.len(), |i| {
                            let p = v[i];
                            let f = falloff_at(input, i);
                            [p[0] + dx * f, p[1] + dy * f]
                        });
                        out.set("P", Column::Vec2(moved));
                    }
                    _ => out.set(name.clone(), col.clone()),
                }
            }
            out
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionMove))?;
    // M1.R1 — UI metadata (a spatial modifier → blue transform, rounded-rect).
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Move",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    // GPU/M5 Fase 1 (ADR-0126): the WGSL lowering, registered on the side.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1): signed X/Y offsets in metres.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "dx",
        label: "Move X",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "dy",
        label: "Move Y",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // Source: 2 instances at (0,0),(1,1) with a falloff column [1, 0.5].
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.move.test.src"),
        name: "motion.move.test.src",
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
                    .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 1.0]]))
                    .with("falloff", Column::Scalar(vec![1.0, 0.5])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionMove),
                _ => None,
            }
        }
    }

    #[test]
    fn offset_is_scaled_by_falloff() {
        let mut g = Graph::new();
        let src = g.add_node("motion.move.test.src");
        let mv = g.add_node("motion.move");
        g.connect(Edge {
            from: (src, 0),
            to: (mv, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(mv, "dx", 10.0);
        g.set_param(mv, "dy", 4.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, mv, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            // i0 f=1: (0,0)+(10,4)=(10,4) ; i1 f=0.5: (1,1)+(5,2)=(6,3)
            Column::Vec2(v) => assert_eq!(v, &vec![[10.0, 4.0], [6.0, 3.0]]),
            _ => panic!("P"),
        }
    }

    #[test]
    fn missing_falloff_means_full_effect() {
        // A source without a falloff column → weight 1 everywhere.
        let s = Stream::new(1).with("P", Column::Vec2(vec![[0.0, 0.0]]));
        assert_eq!(falloff_at(&s, 0), 1.0);
    }
}
