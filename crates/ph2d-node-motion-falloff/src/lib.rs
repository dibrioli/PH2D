#![forbid(unsafe_code)]
//! `motion.falloff` — a Motion **focus field**: writes the multiplicative
//! `falloff` column (§1.2) as a mask that downstream modifiers read to scale
//! their effect. Three **shapes** (`shape`): **Circle** (radial disc), **Rect**
//! (axis-aligned box), **Linear** (a directional wipe). Circle/Rect are `1` at the
//! centre fading to `0` at `radius`; Linear ramps `0`→`1` across `±radius` in x.
//! The edge is shaped by a **curve** (`curve`): Linear / Quad / Smooth
//! (smoothstep) / Smoother (smootherstep `6s⁵−15s⁴+10s³`). It *multiplies* into
//! any existing `falloff` so fields compose, and passes every other column
//! through unchanged (count preserved). Pure. Transcendental-free (HR-5): only
//! polynomials + IEEE `sqrt`.
//!
//! Params (read via `ctx.param`): `shape` (0 Circle), `curve` (2 Smooth),
//! `center_x` (0), `center_y` (0), `radius` (5), `invert` (0/1 — flips the mask to
//! `1 − f`). The defaults reproduce the classic smoothstep circle.

mod trig;

use ph2d_node_registry::{NodeRegistry, ParamGate, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.falloff"),
    name: "motion.falloff",
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
            name: "shape",
            default: 0.0,
        },
        ParamSpec {
            name: "curve",
            default: 2.0,
        },
        ParamSpec {
            name: "center_x",
            default: 0.0,
        },
        ParamSpec {
            name: "center_y",
            default: 0.0,
        },
        ParamSpec {
            name: "radius",
            default: 5.0,
        },
        // ⚠️ **Apendado** (doc 89 folha 10, P0): o ângulo do campo, em graus. `0` é
        // o nó que sempre shipou — a rampa Linear só sabia ir na HORIZONTAL, e o
        // Rect só existia alinhado aos eixos.
        ParamSpec {
            name: "rotation",
            default: 0.0,
        },
        ParamSpec {
            name: "invert",
            default: 0.0,
        },
    ],
    // `lowerings` stays `Cpu`: `LoweringKind::Wgsl` is the scalar `eval_column`
    // route (`ph2d-expr`), which this Vec2-derived write does not fit. The GPU
    // lowering it DOES have is the ADR-0126 side channel (`GPU_KERNEL`, via
    // `register_gpu_kernel`) — a separate mechanism that never touches the frozen
    // `NodeManifest`.
    lowerings: &[LoweringKind::Cpu],
};

/// An edge curve on a pre-clamped `s ∈ [0,1]` — transcendental-free (HR-5), so the
/// mask is bit-identical across platforms for the replay hash. `0` Linear · `1`
/// Quad · `2` Smooth (smoothstep `s²(3−2s)`) · `3` Smoother (smootherstep,
/// Perlin `6s⁵−15s⁴+10s³`). Every curve is endpoint-exact (`0→0`, `1→1`).
fn curve(kind: i32, s: f32) -> f32 {
    match kind {
        1 => s * s,                                     // Quad
        2 => s * s * (3.0 - 2.0 * s),                   // Smooth (smoothstep)
        3 => s * s * s * (s * (s * 6.0 - 15.0) + 10.0), // Smoother (smootherstep)
        _ => s,                                         // Linear
    }
}

/// The mask value for an instance at offset `(dx, dy)` from the centre.
/// `shape`: `1` Rect (box), `2` Linear (x wipe), else Circle (disc). Circle/Rect
/// are `1` inside fading to `0` at `radius`; Linear ramps `0→1` across `±radius`.
/// `radius <= 0` degenerates to an empty field. `invert` mirrors it to `1 − f`.
fn field(shape: i32, dx: f32, dy: f32, radius: f32, curve_kind: i32, invert: bool) -> f32 {
    let f = if radius > 0.0 {
        match shape {
            // Rect: Chebyshev (box) distance to the edge.
            1 => {
                let s = (dx.abs().max(dy.abs()) / radius).clamp(0.0, 1.0);
                1.0 - curve(curve_kind, s)
            }
            // Linear: a horizontal ramp, 0 at `x = -radius` → 1 at `x = +radius`.
            2 => {
                let s = (dx / radius * 0.5 + 0.5).clamp(0.0, 1.0);
                curve(curve_kind, s)
            }
            // Circle: radial disc. IEEE `sqrt` is deterministic (HR-5-safe).
            _ => {
                let d = (dx * dx + dy * dy).sqrt();
                let s = (d / radius).clamp(0.0, 1.0);
                1.0 - curve(curve_kind, s)
            }
        }
    } else {
        0.0
    };
    if invert { 1.0 - f } else { f }
}

/// GPU compute kernel (GPU/M5 Fase 2, ADR-0126): a straight WGSL port of
/// [`field`] × [`curve`] multiplied into the existing `falloff` — same
/// polynomials, same IEEE `sqrt`/`floor` (HR-5), so parity holds within float
/// ULPs. No `applicable`: it covers every `shape`/`curve`. The `shape`/`curve`
/// enums are routed through `fl_round` (round-half-AWAY, matching Rust's
/// `f32::round`; WGSL's builtin `round` is half-even and would pick a different
/// branch at `x.5` — [[feedback_cpu_gpu_rounding_conventions_diverge]]), and
/// `invert` through the CPU's own `>= 0.5` threshold. `ReadWrite` on `falloff`
/// mirrors the CPU: a stream without a `falloff` column starts from the `1.0`
/// identity (fields multiply) and the column is always written; `P` reads its
/// `0` identity when absent (the CPU's `positions.get(i).unwrap_or([0,0])`).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let fl_p = read_P(i);\n\
        // Para o FRAME do campo — a MESMA transcricao que o `field.box` faz.\n\
        let fl_ox = fl_p.x - params.center_x;\n\
        let fl_oy = fl_p.y - params.center_y;\n\
        let fl_b = fl_cos_sin(params.rotation / 360.0);\n\
        let fl_v = fl_field(\n\
            i32(fl_round(params.shape)),\n\
            fl_ox * fl_b.x + fl_oy * fl_b.y,\n\
            -fl_ox * fl_b.y + fl_oy * fl_b.x,\n\
            params.radius,\n\
            i32(fl_round(params.curve)),\n\
            params.invert >= 0.5);\n\
        write_falloff(i, read_falloff(i) * fl_v);\n",
    wgsl_lib: "\
        fn fl_sin_cycles(phase: f32) -> f32 {\n\
            // A senoide parabolica corrigida (ver trig.rs) — o MESMO polinomio\n\
            // que a CPU, entao a paridade vale; a fase e em ciclos (graus/360).\n\
            let ff = phase - floor(phase);\n\
            var p: f32;\n\
            if (ff < 0.5) { let u = ff * 2.0; p = 4.0 * u * (1.0 - u); }\n\
            else { let u = (ff - 0.5) * 2.0; p = -4.0 * u * (1.0 - u); }\n\
            return 0.225 * (p * abs(p) - p) + p;\n\
        }\n\
        fn fl_cos_sin(phase: f32) -> vec2<f32> {\n\
            return vec2<f32>(fl_sin_cycles(phase + 0.25), fl_sin_cycles(phase));\n\
        }\n\
        fn fl_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn fl_curve(kind: i32, s: f32) -> f32 {\n\
            if (kind == 1) { return s * s; }\n\
            if (kind == 2) { return s * s * (3.0 - 2.0 * s); }\n\
            if (kind == 3) { return s * s * s * (s * (s * 6.0 - 15.0) + 10.0); }\n\
            return s;\n\
        }\n\
        fn fl_field(shape: i32, dx: f32, dy: f32, radius: f32, curve_kind: i32, invert: bool) -> f32 {\n\
            var f: f32;\n\
            if (radius > 0.0) {\n\
                if (shape == 1) {\n\
                    let s = clamp(max(abs(dx), abs(dy)) / radius, 0.0, 1.0);\n\
                    f = 1.0 - fl_curve(curve_kind, s);\n\
                } else if (shape == 2) {\n\
                    let s = clamp(dx / radius * 0.5 + 0.5, 0.0, 1.0);\n\
                    f = fl_curve(curve_kind, s);\n\
                } else {\n\
                    let d = sqrt(dx * dx + dy * dy);\n\
                    let s = clamp(d / radius, 0.0, 1.0);\n\
                    f = 1.0 - fl_curve(curve_kind, s);\n\
                }\n\
            } else {\n\
                f = 0.0;\n\
            }\n\
            if (invert) { return 1.0 - f; }\n\
            return f;\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [1.0; 4],
            port: 0,
        },
    ],
    params: &[
        "shape", "curve", "center_x", "center_y", "radius", "rotation", "invert",
    ],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct MotionFalloff;

impl NodeOp for MotionFalloff {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let (cx, cy) = (ctx.param("center_x"), ctx.param("center_y"));
        let radius = ctx.param("radius");
        let shape = ctx.param("shape").round() as i32;
        let curve_kind = ctx.param("curve").round() as i32;
        let invert = ctx.param("invert") >= 0.5;
        // A base de rotação, calculada UMA vez (constante por cook). ⚠️ É a MESMA
        // aproximação que o `field.box` usa (`trig.rs` copiado verbatim de lá), e
        // tem de ser: os dois são campos espaciais que o artista gira, e um `30°`
        // que significasse ângulos diferentes em dois nós seria a falha de duas
        // portas na sua forma mais quieta — nada na tela diria qual está certo.
        let (rc, rs) = trig::cos_sin_cycles(ctx.param("rotation") / 360.0);
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            // Existing per-instance falloff (fields multiply); absent → 1.
            let prev = match input.get("falloff") {
                Some(Column::Scalar(v)) => Some(v.as_slice()),
                _ => None,
            };
            let positions: &[[f32; 2]] = match input.get("P") {
                Some(Column::Vec2(v)) => v.as_slice(),
                _ => &[],
            };
            let mut fall = Vec::with_capacity(n);
            for i in 0..n {
                let p = positions.get(i).copied().unwrap_or([0.0, 0.0]);
                let (ox, oy) = (p[0] - cx, p[1] - cy);
                // Para o FRAME do campo: gira o ponto por −rotação (a inversa de
                // girar o campo). Em `rotation = 0` isto reduz LITERALMENTE ao
                // `(ox, oy)` que shipava — `cos_sin_cycles(0)` é `(1, 0)` EXATO,
                // então `ox·1 + oy·0` é `ox` em IEEE-754.
                let (dx, dy) = (ox * rc + oy * rs, -ox * rs + oy * rc);
                let base = prev.and_then(|v| v.get(i).copied()).unwrap_or(1.0);
                fall.push(base * field(shape, dx, dy, radius, curve_kind, invert));
            }
            let mut out = Stream::new(n);
            for (name, col) in input.columns() {
                if name != "falloff" {
                    out.set(name.clone(), col.clone());
                }
            }
            out.set("falloff", Column::Scalar(fall));
            out
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionFalloff))?;
    // M1.R1 — UI metadata (a focus field → amber, diamond value silhouette).
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Falloff",
            category: ph2d_node_registry::NodeUiCategory::Focus,
            silhouette: ph2d_node_registry::NodeSilhouette::Diamond,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    // ⚠️ Um CÍRCULO é isotrópico: girá-lo não move um texel, então o knob seria
    // morto ali. Ele aparece no Rect (uma caixa orientada) e no Linear (a rampa
    // num ângulo qualquer, que é o P0 da folha 10).
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    // GPU/M5 Fase 2 (ADR-0126): the WGSL lowering, registered on the side.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1): named Shape / Curve selectors, signed centre, positive
/// radius, invert checkbox — never number sliders for the enums.
static PARAM_HINTS: &[ParamUiHint] = &[
    // O ângulo do campo. ⚠️ A faixa cobre a volta inteira: uma rampa a 190° é a de
    // 10° ao contrário, e recusar metade do círculo faria o artista procurar um
    // `invert` para dizer o que o ângulo já diz.
    ParamUiHint {
        param: "rotation",
        label: "Rotation",
        min: -180.0,
        max: 180.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "shape",
        label: "Shape",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Circle", "Rect", "Linear"],
        },
    },
    ParamUiHint {
        param: "curve",
        label: "Curve",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Linear", "Quad", "Smooth", "Smoother"],
        },
    },
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
        param: "radius",
        label: "Radius",
        min: 0.0,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "invert",
        label: "Invert",
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
/// O ângulo só é lido pelas formas que TÊM direção (`1` Rect · `2` Linear).
static PARAM_GATES: &[ParamGate] = &[ParamGate {
    param: "rotation",
    when: "shape",
    values: &[1, 2],
}];

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
#[path = "rotation_tests.rs"]
mod rotation_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

    // Source: 3 instances on a line at x = 0, 5, 10 (y = 0).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.falloff.test.src"),
        name: "motion.falloff.test.src",
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
                Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0], [5.0, 0.0], [10.0, 0.0]])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionFalloff),
                _ => None,
            }
        }
    }

    fn falloff_of(g: &Graph, ops: &Ops, target: NodeId) -> Vec<f32> {
        let mut cook = Cook::new();
        let out = cook.cook(g, ops, target, 0.0).unwrap();
        match out[0].as_stream().get("falloff").unwrap() {
            Column::Scalar(v) => v.clone(),
            _ => panic!("falloff must be a Scalar column"),
        }
    }

    #[test]
    fn default_circle_is_one_at_center_zero_at_edge() {
        let mut g = Graph::new();
        let src = g.add_node("motion.falloff.test.src");
        let foc = g.add_node("motion.falloff");
        g.connect(Edge {
            from: (src, 0),
            to: (foc, 0),
            delayed: false,
        })
        .unwrap();
        // Defaults: Circle + Smooth, radius 5, centre (0,0): x=0 → 1, x=5 → 0, x=10 → 0.
        assert_eq!(falloff_of(&g, &Ops, foc), vec![1.0, 0.0, 0.0]);
    }

    /// Fields COMPOSE multiplicatively (audit 2026-07-10: the promise at the
    /// `base * field` site was untested): an upstream `falloff` column is
    /// multiplied by this field, never overwritten — two stacked focus nodes
    /// intersect their regions.
    #[test]
    fn a_prior_falloff_column_is_multiplied_not_overwritten() {
        static FSRC_MAN: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.falloff.test.fsrc"),
            name: "motion.falloff.test.fsrc",
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
                        .with("P", Column::Vec2(vec![[0.0, 0.0], [10.0, 0.0]]))
                        .with("falloff", Column::Scalar(vec![0.5, 0.8])),
                );
            }
        }
        struct FOps;
        impl OpResolver for FOps {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == FSRC_MAN.id => Some(&FSrc),
                    t if t == MANIFEST.id => Some(&MotionFalloff),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let src = g.add_node("motion.falloff.test.fsrc");
        let foc = g.add_node("motion.falloff");
        g.connect(Edge {
            from: (src, 0),
            to: (foc, 0),
            delayed: false,
        })
        .unwrap();
        // Defaults (Circle, radius 5, centre origin): field = 1 at x=0, 0 at
        // x=10. Composed with the carried [0.5, 0.8]: 0.5·1 and 0.8·0.
        let mut cook = Cook::new();
        let out = cook.cook(&g, &FOps, foc, 0.0).unwrap();
        match out[0].as_stream().get("falloff").unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![0.5, 0.0]),
            _ => panic!("falloff"),
        }
    }

    #[test]
    fn invert_flips_the_mask() {
        let mut g = Graph::new();
        let src = g.add_node("motion.falloff.test.src");
        let foc = g.add_node("motion.falloff");
        g.connect(Edge {
            from: (src, 0),
            to: (foc, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(foc, "invert", 1.0);
        // 1 - mask: x=0 → 0, x=5 → 1, x=10 → 1.
        assert_eq!(falloff_of(&g, &Ops, foc), vec![0.0, 1.0, 1.0]);
    }

    #[test]
    fn linear_shape_ramps_across_x() {
        let mut g = Graph::new();
        let src = g.add_node("motion.falloff.test.src");
        let foc = g.add_node("motion.falloff");
        g.connect(Edge {
            from: (src, 0),
            to: (foc, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(foc, "shape", 2.0); // Linear
        g.set_param(foc, "curve", 0.0); // Linear curve → a pure ramp
        g.set_param(foc, "radius", 10.0);
        // s = x/10·0.5+0.5: x=0 → 0.5, x=5 → 0.75, x=10 → 1.0 (a left→right wipe).
        assert_eq!(falloff_of(&g, &Ops, foc), vec![0.5, 0.75, 1.0]);
    }

    #[test]
    fn curves_are_smooth_and_endpoint_exact() {
        // Linear / Quad / Smooth / Smoother all map 0→0 and 1→1; the midpoint
        // differs (Linear .5, Quad .25, Smooth/Smoother symmetric .5).
        for k in 0..=3 {
            assert_eq!(curve(k, 0.0), 0.0, "curve {k} at 0");
            assert_eq!(curve(k, 1.0), 1.0, "curve {k} at 1");
        }
        assert_eq!(curve(0, 0.5), 0.5); // Linear
        assert_eq!(curve(1, 0.5), 0.25); // Quad
        assert_eq!(curve(2, 0.5), 0.5); // Smoothstep symmetric
        assert!((curve(3, 0.5) - 0.5).abs() < 1e-6); // Smootherstep symmetric
    }

    #[test]
    fn rect_reaches_the_diagonal_corner_further_than_the_circle() {
        // At (3,3) with radius 5, curve Linear: Rect uses Chebyshev (max axis) →
        // s = 3/5 = .6 → 1−.6 = .4; Circle uses Euclidean → s = √18/5 ≈ .8485 →
        // 1−.8485 ≈ .1515. The box keeps more field into the corner.
        let rect = field(1, 3.0, 3.0, 5.0, 0, false);
        let circle = field(0, 3.0, 3.0, 5.0, 0, false);
        assert!((rect - 0.4).abs() < 1e-6);
        assert!((circle - 0.151_471_86).abs() < 1e-4);
        assert!(rect > circle);
    }

    #[test]
    fn degenerate_radius_is_empty() {
        assert_eq!(field(0, 0.0, 0.0, 0.0, 2, false), 0.0);
        assert_eq!(field(2, 0.0, 0.0, 0.0, 2, false), 0.0);
    }
}
