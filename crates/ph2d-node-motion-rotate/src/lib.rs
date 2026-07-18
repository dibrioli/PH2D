#![forbid(unsafe_code)]
//! `motion.rotate` — a Motion **modifier**: adds `angle` (**degrees**) to the
//! `rot` (Scalar) attribute of its input stream, scaled per-instance by the
//! multiplicative `falloff` column (§1.2; absent → `1.0`). A stream without a
//! `rot` column starts from `0`. `ph2d-eval-motion` turns `rot` into the render
//! basis (`[cos, sin, -sin, cos]`, converting degrees→radians at that edge);
//! this node only accumulates the scalar, so it stays transcendental-free
//! (HR-5). Every other column passes through unchanged (count preserved). Pure.
//!
//! Degrees is the app's single authored-angle unit (the Painter's `*_angle_deg`
//! fields, the Inspector's `deg` boxes) — no radians anywhere in the authoring
//! surface.
//!
//! Params (read via `ctx.param`): `angle` (0.0). `rot'_i = rot_i + angle * falloff_i`.

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
    id: NodeTypeId::of("motion.rotate"),
    name: "motion.rotate",
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
    params: &[ParamSpec {
        name: "angle",
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

/// The multiplicative `falloff` weight for instance `i` (absent → `1.0`).
fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// GPU compute kernel (GPU/M5 Fase 2, ADR-0126): `rot' = rot + angle·falloff`,
/// the exact per-element map of the CPU `eval` (transcendental-free — the sin/cos
/// of `rot` is the lowering's job, at the render edge, in degrees→radians). No
/// `applicable`: a plain scalar accumulate covers the whole param space.
/// `ReadWrite` on `rot` mirrors the CPU: a stream without a `rot` column starts
/// each instance from the `0.0` identity and the column is always written (the
/// CPU builds `vec![0.0; n]` and `out.set("rot", …)` unconditionally).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        write_rot(i, read_rot(i) + params.angle * read_falloff(i));\n",
    wgsl_lib: "",
    bindings: &[
        ColumnBinding {
            column: "rot",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
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
    params: &["angle"],
    count_law: None,
    applicable: None,
};

struct MotionRotate;

impl NodeOp for MotionRotate {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let angle = ctx.param("angle");
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            // Base per-instance rotation (absent column → 0).
            let base: Vec<f32> = match input.get("rot") {
                Some(Column::Scalar(v)) => v.clone(),
                _ => vec![0.0; n],
            };
            // Pure per-instance map → parallel above the threshold
            // (bit-identical, no reduction). GPU/M5 Fase 0.
            let rotated: Vec<f32> = par_build(n, |i| {
                base.get(i).copied().unwrap_or(0.0) + angle * falloff_at(input, i)
            });
            let mut out = Stream::new(n);
            for (name, col) in input.columns() {
                if name != "rot" {
                    out.set(name.clone(), col.clone());
                }
            }
            out.set("rot", Column::Scalar(rotated));
            out
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionRotate))?;
    // M1.R1 — UI metadata (a spatial modifier → blue transform, rounded-rect).
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Rotate",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    // GPU/M5 Fase 2 (ADR-0126): the WGSL lowering, registered on the side.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1): a signed angle in **degrees** — the unit of the `rot`
/// stream column this node adds to. Painted as a `deg` numeric box.
static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: "angle",
    label: "Angle",
    min: -180.0,
    max: 180.0,
    step: 1.0,
    widget: ParamWidget::Angle,
}];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // Source: 2 instances with an existing rot [1, 2] and falloff [1, 0.5].
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.rotate.test.src"),
        name: "motion.rotate.test.src",
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
                    .with("rot", Column::Scalar(vec![1.0, 2.0]))
                    .with("falloff", Column::Scalar(vec![1.0, 0.5])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionRotate),
                _ => None,
            }
        }
    }

    #[test]
    fn angle_accumulates_scaled_by_falloff() {
        let mut g = Graph::new();
        let src = g.add_node("motion.rotate.test.src");
        let rot = g.add_node("motion.rotate");
        g.connect(Edge {
            from: (src, 0),
            to: (rot, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(rot, "angle", 2.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, rot, 0.0).unwrap();
        match out[0].as_stream().get("rot").unwrap() {
            // i0 f=1: 1 + 2*1 = 3 ; i1 f=0.5: 2 + 2*0.5 = 3
            Column::Scalar(v) => assert_eq!(v, &vec![3.0, 3.0]),
            _ => panic!("rot"),
        }
    }

    #[test]
    fn missing_falloff_means_full_effect() {
        // A source without a falloff column → weight 1 (angle applied in full).
        let s = Stream::new(1).with("P", Column::Vec2(vec![[0.0, 0.0]]));
        assert_eq!(falloff_at(&s, 0), 1.0);
    }
}
