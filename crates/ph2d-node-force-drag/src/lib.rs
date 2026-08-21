#![forbid(unsafe_code)]
//! `force.drag` — velocity-proportional friction. A Motion **force**: a `Pure`
//! node that adds `−coefficient · vel` into the transient `accel` column
//! (× the multiplicative `falloff` field, plan §1.6); `motion.integrate` turns
//! the accumulated acceleration into motion.
//!
//! Reference behaviour (MiniCavalryV2 `forces/drag.js`, clean-room):
//! `a = −k·v`, a mass-independent exponential decay with time constant
//! `τ = 1/k` (`k = 1` → the velocity falls to ~37% in 1 s; `k = 10` → nearly
//! stopped in 0.1 s). The reference had to sum "upstream + own" velocity to
//! brake other forces because each force integrated privately; here the single
//! integrator owns the one true `vel` column, so drag reads it directly —
//! the composition workaround dissolves by architecture.
//!
//! The classic stable-orbit combo: Vortex + Attractor at the same centre +
//! Drag (without drag a pure vortex spirals outward by centrifugal drift).

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::par_build;
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod accum;
use accum::{add_accel, falloff_at, vec2_at};

/// **O ARRASTO ANISOTRÓPICO** (doc 89 folha 02 — o `Directional Scale` do POP Drag).
///
/// O arrasto era um escalar: `accel += −k·v`. Uma folha que cai **balançando** — que
/// é o caso de uso, não o exótico — precisa de frear mais num eixo que no outro, e
/// isso era **inexprimível**: o `falloff` é um escalar por construção (isotrópico),
/// e nada no catálogo escreve `accel` de um vetor arbitrário.
///
/// ⚠️ **Dois escalares e não um vetor com ângulo, e a escolha tem dono:** o eixo do
/// arrasto de uma folha é o do MUNDO (a gravidade define a vertical), e um ângulo
/// por cima seria uma segunda porta para o que um `motion.rotate` a montante já faz
/// ao layout inteiro. A referência (POP Drag) também expõe um vetor por-eixo.
///
/// ⚠️ **`1`/`1` é o nó que sempre shipou, AO BIT** — e aqui a identidade é
/// ARITMÉTICA e não estrutural: multiplicar por `1.0` é exacto em IEEE-754 para todo
/// finito, então não há braço literal a defender. É o caso raro em que a
/// multiplicação neutra **é** o caminho de antes.
const SCALE_X: &str = "scale_x";
/// O gêmeo vertical do [`SCALE_X`].
const SCALE_Y: &str = "scale_y";

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("force.drag"),
    name: "force.drag",
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
            name: "coefficient",
            default: 1.0,
        },
        // Apendados (doc 89 folha 02). `1`/`1` = o arrasto isotrópico de sempre.
        ParamSpec {
            name: "scale_x",
            default: 1.0,
        },
        ParamSpec {
            name: "scale_y",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (GPU/M5 **Fase 3**, ADR-0126 side channel): `a += −k·v·falloff`,
/// the exact per-element map of the CPU `eval` in the same operand order. No
/// transcendentals, so parity is FMA-ε at worst.
///
/// `accel` is `ReadWrite` with a zero identity — the CPU's `add_accel`
/// ("create at zero when absent, else ADD"), which is what makes a chain of
/// forces accumulate (Houdini POP: microsolvers add, one solver integrates).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let dg_w = max(params.coefficient, 0.0) * read_falloff(i);\n\
        let dg_v = read_vel(i);\n\
        write_accel(i, read_accel(i) + vec2<f32>(\n\
        \x20   -dg_v.x * dg_w * params.scale_x,\n\
        \x20   -dg_v.y * dg_w * params.scale_y));\n",
    wgsl_lib: "",
    bindings: &[
        ColumnBinding {
            column: "accel",
            dim: Dim::Vec2,
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
        ColumnBinding {
            // No velocity column (or no motion yet) → drag is a no-op.
            column: "vel",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &["coefficient", "scale_x", "scale_y"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct ForceDrag;

impl NodeOp for ForceDrag {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let k = ctx.param("coefficient").max(0.0);
        let (sx, sy) = (ctx.param(SCALE_X), ctx.param(SCALE_Y));
        let out = {
            let input = ctx.input(0);
            // Pure per-instance map → parallel above the threshold
            // (bit-identical, no reduction). GPU/M5 Fase 0.
            let contrib: Vec<[f32; 2]> = par_build(input.count(), |i| {
                let vel = vec2_at(input, "vel", i, [0.0, 0.0]);
                let w = k * falloff_at(input, i);
                [-vel[0] * w * sx, -vel[1] * w * sy]
            });
            add_accel(input, &contrib)
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ForceDrag))?;
    // ADR-0155: a force accumulates `accel`; inert without an integrator downstream.
    reg.register_couplings(
        MANIFEST.id,
        &[ph2d_node_registry::Coupling::Produces("accel")],
    );
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Drag",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    // ADR-0130: per-element force: accumulates accel, identity preserved.
    reg.register_dense_window(MANIFEST.id);
    Ok(())
}

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamWidget};

/// O teto que a MÁQUINA (ou o bom senso) impõe, alcançável por DIGITAÇÃO — o slider fica
/// onde a MÃO trabalha (soft/hard do Blender; doc 88 §11). O curso de antes é este número:
/// nada ficou inalcançável, só deixou de ser o que o dedo percorre.
static PARAM_HARD_MAX: &[ParamHardMax] = &[ParamHardMax {
    param: "coefficient",
    max: 20.0,
}];

/// Param UI hints (M1.P1).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: SCALE_X,
        label: "Drag X",
        min: 0.0,
        max: 3.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: SCALE_Y,
        label: "Drag Y",
        min: 0.0,
        max: 3.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "coefficient",
        label: "Coefficient",
        min: 0.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // Two instances: one moving (+2, −1), one at rest — with a half falloff on
    // the mover so the gate is visible too.
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("force.drag.test.src"),
        name: "force.drag.test.src",
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
                    .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]]))
                    .with("vel", Column::Vec2(vec![[2.0, -1.0], [0.0, 0.0]]))
                    .with("falloff", Column::Scalar(vec![0.5, 1.0])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&ForceDrag),
                _ => None,
            }
        }
    }

    #[test]
    fn drag_opposes_velocity_scaled_by_coefficient_and_falloff() {
        let mut g = Graph::new();
        let src = g.add_node("force.drag.test.src");
        let d = g.add_node("force.drag");
        g.connect(Edge {
            from: (src, 0),
            to: (d, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(d, "coefficient", 3.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, d, 0.0).unwrap();
        match out[0].as_stream().get("accel").unwrap() {
            Column::Vec2(v) => {
                // −k·v·falloff = −3·(2,−1)·0.5 = (−3, 1.5).
                assert!((v[0][0] + 3.0).abs() < 1e-5);
                assert!((v[0][1] - 1.5).abs() < 1e-5);
                // No velocity (or no vel column at all) → drag is a no-op.
                assert_eq!(v[1], [0.0, 0.0]);
            }
            _ => panic!("accel"),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}

#[cfg(test)]
#[path = "anisotropy_tests.rs"]
mod anisotropy_tests;
