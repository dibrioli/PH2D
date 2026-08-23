#![forbid(unsafe_code)]
//! `force.vortex` — a tangential force around a fixed centre. A Motion
//! **force**: a `Pure` node that adds its per-instance contribution into the
//! transient `accel` column (× the multiplicative `falloff` field, plan §1.6);
//! `motion.integrate` turns the accumulated acceleration into motion.
//!
//! Reference behaviour (MiniCavalryV2 `forces/vortex.js`, clean-room): purely
//! tangential (no radial component — a free vortex spirals outward by the
//! centrifugal drift; the classic stable-orbit combo is Vortex + Attractor at
//! the same centre + Drag), magnitude `strength · (1 − d/R)` inside radius `R`,
//! zero outside and in the centre dead-zone.
//!
//! **Y-up world** (`ph2d-render::camera`): the reference is Y-down, where
//! rotating the radial vector by +90° reads as clockwise on screen. Here the
//! visual **clockwise** tangent of radial `(dx, dy)` is `(dy, −dx)`;
//! counter-clockwise is `(−dy, dx)`. Anchored by test.

use ph2d_node_registry::{NodeRegistry, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::attr::par_build;
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod accum;
use accum::{add_accel, falloff_at, vec2_at};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Dead-zone radius around the centre (direction is meaningless as d → 0).
const DEAD_ZONE: f32 = 1e-3;

/// **O PERFIL DA BORDA** (doc 89 folha 02) — `0` Linear (a rampa cravada que sempre
/// shipou), `1` Quad, `2` Smooth, `3` Smoother.
///
/// ⚠️ **A célula não veio de uma referência: veio de uma ASSIMETRIA INTERNA.** O
/// irmão `force.attractor` tem estes quatro perfis desde que nasceu; este tinha
/// `(1 − d/R)` **cravado**. A nossa própria família discordava de si, e o artista
/// que aprendeu o dropdown num nó não o encontrava no outro.
///
/// ⚠️ **São os MESMOS quatro polinómios, deliberadamente** — copiados do
/// `force.attractor` (drop-crates não se importam entre si, ADR-0075) e não
/// reinventados: dois `Smooth` que diferissem no terceiro decimal fariam o mesmo
/// rótulo significar duas coisas. Endpoint-exactos (`0→0`, `1→1`), como lá.
///
/// ⚠️ **O `0` é literal:** `curve(0, t)` devolve o próprio `t`, então o Linear é a
/// mesma expressão de antes, ao bit.
///
/// ⚠️ **O que este knob NÃO é:** o corte duro em `d > radius` continua lá. A célula
/// pedia *"Soft Edge"* junto, e ela é outra pergunta — o perfil molda a rampa
/// DENTRO do raio; suavizar o corte é mover a fronteira. Ficou por fazer, e a folha
/// diz isso.
const CURVE: &str = "curve";

/// A rampa da borda em `s ∈ [0,1]` — transcendental-free (HR-5), e byte a byte a do
/// `force.attractor`.
fn curve(kind: i32, s: f32) -> f32 {
    match kind {
        1 => s * s,                                     // Quad
        2 => s * s * (3.0 - 2.0 * s),                   // Smooth (smoothstep)
        3 => s * s * s * (s * (s * 6.0 - 15.0) + 10.0), // Smoother (smootherstep)
        _ => s,                                         // Linear
    }
}

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("force.vortex"),
    name: "force.vortex",
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
            name: "center_x",
            default: 0.0,
        },
        ParamSpec {
            name: "center_y",
            default: 0.0,
        },
        ParamSpec {
            name: "strength",
            default: 4.0,
        },
        ParamSpec {
            name: "radius",
            default: 6.0,
        },
        // Apendado (doc 89 folha 02). `0` = Linear, a rampa cravada que sempre shipou.
        ParamSpec {
            name: "curve",
            default: 0.0,
        },
        ParamSpec {
            name: "clockwise",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (GPU/M5 **Fase 3**, ADR-0126 side channel): the exact
/// per-element map of the CPU `eval` — a tangential push around the centre with
/// a linear edge falloff, gated by the radius and the focus field.
///
/// The `/ d` normalization and the dead zone are the CPU's, verbatim; so is the
/// **sign** of the tangent, which is the porting hazard this node's own test
/// pins (Y-up world vs the Y-down reference): clockwise pushes −Y on the right
/// side. `clockwise` is a `>= 0.5` test, not a rounded enum, so no round hazard.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vx_radius = max(params.radius, VX_DEAD_ZONE);\n\
        let vx_p = read_P(i);\n\
        let vx_dx = vx_p.x - params.center_x;\n\
        let vx_dy = vx_p.y - params.center_y;\n\
        let vx_d = sqrt(vx_dx * vx_dx + vx_dy * vx_dy);\n\
        var vx_c = vec2<f32>(0.0, 0.0);\n\
        if (vx_d >= VX_DEAD_ZONE && vx_d <= vx_radius) {\n\
        \x20   let vx_mag =\n\
        \x20       params.strength * vx_curve(i32(vx_round(params.curve)),\n\
        \x20           1.0 - vx_d / vx_radius) * read_falloff(i) / vx_d;\n\
        \x20   if (params.clockwise >= 0.5) {\n\
        \x20       vx_c = vec2<f32>(vx_dy * vx_mag, -vx_dx * vx_mag);\n\
        \x20   } else {\n\
        \x20       vx_c = vec2<f32>(-vx_dy * vx_mag, vx_dx * vx_mag);\n\
        \x20   }\n\
        }\n\
        write_accel(i, read_accel(i) + vx_c);\n",
    wgsl_lib: "\
        const VX_DEAD_ZONE: f32 = 1e-3;\n\
        fn vx_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (o `round` do WGSL e' half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        // Os MESMOS quatro polinomios do `force.attractor` — a familia tem uma lei so'.\n\
        fn vx_curve(kind: i32, s: f32) -> f32 {\n\
            if (kind == 1) { return s * s; }\n\
            if (kind == 2) { return s * s * (3.0 - 2.0 * s); }\n\
            if (kind == 3) { return s * s * s * (s * (s * 6.0 - 15.0) + 10.0); }\n\
            return s;\n\
        }\n",
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
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &[
        "center_x",
        "center_y",
        "strength",
        "radius",
        "clockwise",
        "curve",
    ],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct ForceVortex;

impl NodeOp for ForceVortex {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let center = [ctx.param("center_x"), ctx.param("center_y")];
        let strength = ctx.param("strength");
        let radius = ctx.param("radius").max(DEAD_ZONE);
        let clockwise = ctx.param("clockwise") >= 0.5;
        let curve_kind = ctx.param(CURVE).round() as i32;
        let out = {
            let input = ctx.input(0);
            // Pure per-instance map → parallel above the threshold
            // (bit-identical, no reduction). GPU/M5 Fase 0.
            let contrib: Vec<[f32; 2]> = par_build(input.count(), |i| {
                let p = vec2_at(input, "P", i, [0.0, 0.0]);
                let dx = p[0] - center[0];
                let dy = p[1] - center[1];
                let d = (dx * dx + dy * dy).sqrt();
                if d < DEAD_ZONE || d > radius {
                    return [0.0, 0.0];
                }
                // O perfil da borda × o campo de foco. ⚠️ `curve(0, t)` devolve o
                // próprio `t`, então o Linear é a MESMA expressão de antes, ao bit.
                let t = curve(curve_kind, 1.0 - d / radius);
                let mag = strength * t * falloff_at(input, i) / d;
                if clockwise {
                    [dy * mag, -dx * mag]
                } else {
                    [-dy * mag, dx * mag]
                }
            });
            add_accel(input, &contrib)
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ForceVortex))?;
    // ADR-0155: a force accumulates `accel`; inert without an integrator downstream.
    reg.register_couplings(
        MANIFEST.id,
        &[ph2d_node_registry::Coupling::Produces("accel")],
    );
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Vortex",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    // ADR-0130: per-element force: accumulates accel, identity preserved.
    reg.register_dense_window(MANIFEST.id);
    Ok(())
}

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamWidget};

/// **O teto DIGITÁVEL do raio, MEDIDO** — bloco Z, doc 91.
///
/// ⚠️ **A cena `=3` deste repo autora `radius = 46` e o campo digitava até `20`.** Sem entrada
/// aqui o digitado para no fim do ARRASTO (`ui.rs:206`), ou seja o app publicava um valor que o
/// artista não conseguia escrever — acusação da sonda
/// `what_the_corpus_authors_and_no_one_can_type`.
///
/// **O recurso é a PRECISÃO** (`CLAUDE.md` §0.0): o raio não satura (um vórtice maior que a cena
/// é uma resposta), então o que acaba é o `f32` — acima daqui somar o `step` do slider (0,1)
/// **não move o número**. Derivado a cada corrida pelo gate
/// `every_precision_bound_param_types_to_the_measured_ceiling` (`ph2d-node-registry-init`).
static PARAM_HARD_MAX: &[ParamHardMax] = &[ParamHardMax {
    param: "radius",
    max: 2_097_152.0 - 0.125,
}];

/// Param UI hints (M1.P1).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "center_x",
        label: "Center X",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "center_y",
        label: "Center Y",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "strength",
        label: "Strength",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "radius",
        label: "Radius",
        min: 0.1,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: CURVE,
        label: "Curve",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Linear", "Quad", "Smooth", "Smoother"],
        },
    },
    ParamUiHint {
        param: "clockwise",
        label: "Clockwise",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
];

/// **What each of this node's numbers IS** (doc 88, Wave A) — never how it is
/// shown. A `Length` is stored in world METRES and the panel resolves the face
/// the artist reads (`px` or `m`) from `ProjectSettings::display_unit`; a node
/// that could pin one would be overriding a setting it does not own.
///
/// Only params whose value is a world COORDINATE or a world DISTANCE are declared
/// here. A weight, a fraction, a rate and a count are left bare on purpose: a unit
/// that is wrong is worse than a unit that is missing, because the artist can read
/// a bare number but a mislabelled one teaches them something false.
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "center_x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "center_y",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "radius",
        unit: ParamUnit::Length,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // One instance at (2, 0): right of the centre, inside R=6.
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("force.vortex.test.src"),
        name: "force.vortex.test.src",
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
            ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[2.0, 0.0], [9.0, 0.0]])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&ForceVortex),
                _ => None,
            }
        }
    }

    fn accel_with(clockwise: f32) -> Vec<[f32; 2]> {
        let mut g = Graph::new();
        let src = g.add_node("force.vortex.test.src");
        let vx = g.add_node("force.vortex");
        g.connect(Edge {
            from: (src, 0),
            to: (vx, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(vx, "clockwise", clockwise);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, vx, 0.0).unwrap();
        match out[0].as_stream().get("accel").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("accel"),
        }
    }

    #[test]
    fn clockwise_in_a_y_up_world_pushes_the_right_side_down() {
        // Y-up anchor: an instance RIGHT of the centre, spun clockwise
        // (visually), must accelerate DOWNWARD (−Y). This is the sign that
        // flips between the Y-down reference and this Y-up world — the exact
        // porting hazard the test pins. At (2,0), R=6: |a| = 4·(1−2/6) = 8/3.
        let a = accel_with(1.0);
        assert!(a[0][0].abs() < 1e-4, "purely tangential: no radial X");
        assert!(
            (a[0][1] + 8.0 / 3.0).abs() < 1e-4,
            "clockwise pushes -Y on the right side, got {:?}",
            a[0]
        );
        // Outside the radius: zero.
        assert_eq!(a[1], [0.0, 0.0]);
    }

    #[test]
    fn counter_clockwise_flips_the_tangent() {
        let a = accel_with(0.0);
        assert!(
            (a[0][1] - 8.0 / 3.0).abs() < 1e-4,
            "counter-clockwise pushes +Y on the right side"
        );
    }

    /// The focus field gates the force (audit 2026-07-10: untested until now):
    /// two instances at the SAME point, one at falloff 0 — only the focused one
    /// is spun. A vortex ignoring the mask would accelerate both equally.
    #[test]
    fn falloff_zero_gates_the_force() {
        static FSRC_MAN: NodeManifest = NodeManifest {
            id: NodeTypeId::of("force.vortex.test.fsrc"),
            name: "force.vortex.test.fsrc",
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
        struct FSrc;
        impl NodeOp for FSrc {
            fn manifest(&self) -> &'static NodeManifest {
                &FSRC_MAN
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(
                    Stream::new(2)
                        .with("P", Column::Vec2(vec![[2.0, 0.0]; 2]))
                        .with("falloff", Column::Scalar(vec![1.0, 0.0])),
                );
            }
        }
        struct FOps;
        impl OpResolver for FOps {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == FSRC_MAN.id => Some(&FSrc),
                    t if t == MANIFEST.id => Some(&ForceVortex),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let src = g.add_node("force.vortex.test.fsrc");
        let vx = g.add_node("force.vortex");
        g.connect(Edge {
            from: (src, 0),
            to: (vx, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(vx, "clockwise", 1.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &FOps, vx, 0.0).unwrap();
        let a = match out[0].as_stream().get("accel").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("accel"),
        };
        assert!((a[0][1] + 8.0 / 3.0).abs() < 1e-4, "focused: full spin");
        assert_eq!(a[1], [0.0, 0.0], "falloff 0: untouched");
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}

#[cfg(test)]
#[path = "curve_tests.rs"]
mod curve_tests;
