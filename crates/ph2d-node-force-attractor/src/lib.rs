#![forbid(unsafe_code)]
//! `force.attractor` — a radial force toward (or away from) a fixed target
//! point. A Motion **force**: a `Pure` node that adds its per-instance
//! contribution into the transient `accel` column (× the multiplicative
//! `falloff` field, plan §1.6); `motion.integrate` turns the accumulated
//! acceleration into motion. It holds no state and never moves anything by
//! itself — wire it inside the integrator's `pre` loop.
//!
//! Reference behaviour (MiniCavalryV2 `forces/attractor.js`, clean-room):
//! magnitude `strength · curve(1 − d/R)` inside radius `R`, zero outside and in
//! the dead-zone at the centre; `repel` flips the sign. The reference's free
//! `(1−d/R)^power` is transcendental for non-integer powers (HR-5), so the
//! shaping is the app's falloff-curve vocabulary instead: Linear / Quad /
//! Smooth / Smoother — deterministic polynomials, endpoint-exact.
//!
//! Params: `target_x`/`target_y` (world), `strength` (accel at the centre,
//! world-units/s²), `radius` (influence extent), `curve` (0 Linear · 1 Quad ·
//! 2 Smooth · 3 Smoother), `repel` (0/1).

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod accum;
use accum::{add_accel, falloff_at, vec2_at};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Dead-zone radius around the target: inside it the direction is numerically
/// meaningless (d → 0), so the force is zero (the reference's `d < 1` px in a
/// ~10-unit world).
const DEAD_ZONE: f32 = 1e-3;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("force.attractor"),
    name: "force.attractor",
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
            name: "target_x",
            default: 0.0,
        },
        ParamSpec {
            name: "target_y",
            default: 0.0,
        },
        ParamSpec {
            name: "strength",
            default: 5.0,
        },
        ParamSpec {
            name: "radius",
            default: 4.0,
        },
        ParamSpec {
            name: "curve",
            default: 2.0, // Smooth
        },
        ParamSpec {
            name: "repel",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// An edge curve on a pre-clamped `s ∈ [0,1]` — the same transcendental-free
/// vocabulary as `motion.falloff` (HR-5). Endpoint-exact (`0→0`, `1→1`).
fn curve(kind: i32, s: f32) -> f32 {
    match kind {
        0 => s,                                         // Linear
        1 => s * s,                                     // Quad
        2 => s * s * (3.0 - 2.0 * s),                   // Smooth (smoothstep)
        _ => s * s * s * (s * (s * 6.0 - 15.0) + 10.0), // Smoother (smootherstep)
    }
}

struct ForceAttractor;

impl NodeOp for ForceAttractor {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let target = [ctx.param("target_x"), ctx.param("target_y")];
        let strength = ctx.param("strength");
        let radius = ctx.param("radius").max(DEAD_ZONE);
        let kind = ctx.param("curve").round() as i32;
        let sign = if ctx.param("repel") >= 0.5 { -1.0 } else { 1.0 };
        let out = {
            let input = ctx.input(0);
            let contrib: Vec<[f32; 2]> = (0..input.count())
                .map(|i| {
                    let p = vec2_at(input, "P", i, [0.0, 0.0]);
                    let dx = target[0] - p[0];
                    let dy = target[1] - p[1];
                    let d = (dx * dx + dy * dy).sqrt();
                    if d < DEAD_ZONE || d > radius {
                        return [0.0, 0.0];
                    }
                    let w = curve(kind, (1.0 - d / radius).clamp(0.0, 1.0));
                    let mag = strength * w * sign * falloff_at(input, i);
                    [(dx / d) * mag, (dy / d) * mag]
                })
                .collect();
            add_accel(input, &contrib)
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ForceAttractor))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Attractor",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "target_x",
        label: "Target X",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "target_y",
        label: "Target Y",
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
        param: "repel",
        label: "Repel",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

    // Instances at x = 2 (inside R=4), x = 9 (outside), and at the target.
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("force.attractor.test.src"),
        name: "force.attractor.test.src",
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
                Stream::new(3)
                    .with("P", Column::Vec2(vec![[2.0, 0.0], [9.0, 0.0], [0.0, 0.0]]))
                    .with("falloff", Column::Scalar(vec![1.0, 1.0, 1.0])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&ForceAttractor),
                _ => None,
            }
        }
    }

    fn accel_with(params: &[(&str, f32)]) -> Vec<[f32; 2]> {
        let mut g = Graph::new();
        let src = g.add_node("force.attractor.test.src");
        let att = g.add_node("force.attractor");
        g.connect(Edge {
            from: (src, 0),
            to: (att, 0),
            delayed: false,
        })
        .unwrap();
        for (name, v) in params {
            g.set_param(att, *name, *v);
        }
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, att, 0.0).unwrap();
        match out[0].as_stream().get("accel").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("accel"),
        }
    }

    #[test]
    fn pulls_toward_the_target_inside_the_radius_only() {
        let a = accel_with(&[("curve", 0.0)]); // Linear: legible closed form
        // x=2, target 0, R=4: w = 1 − 2/4 = 0.5 → accel = −5·0.5 = −2.5 in X.
        assert!((a[0][0] + 2.5).abs() < 1e-4, "inside: pulled toward target");
        assert_eq!(a[0][1], 0.0);
        // x=9 is outside R=4 → untouched.
        assert_eq!(a[1], [0.0, 0.0], "outside the radius: zero");
        // At the target (dead zone) → zero, not NaN.
        assert_eq!(a[2], [0.0, 0.0], "dead zone: zero");
    }

    #[test]
    fn repel_flips_the_sign() {
        let a = accel_with(&[("curve", 0.0), ("repel", 1.0)]);
        assert!((a[0][0] - 2.5).abs() < 1e-4, "repel pushes away");
    }

    #[test]
    fn falloff_column_gates_the_force() {
        // Same graph but the src emits falloff 0.5 on instance 0 → half force.
        static HALF_MAN: NodeManifest = NodeManifest {
            id: NodeTypeId::of("force.attractor.test.half"),
            name: "force.attractor.test.half",
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
        struct Half;
        impl NodeOp for Half {
            fn manifest(&self) -> &'static NodeManifest {
                &HALF_MAN
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(
                    Stream::new(1)
                        .with("P", Column::Vec2(vec![[2.0, 0.0]]))
                        .with("falloff", Column::Scalar(vec![0.5])),
                );
            }
        }
        struct HalfOps;
        impl OpResolver for HalfOps {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == HALF_MAN.id => Some(&Half),
                    t if t == MANIFEST.id => Some(&ForceAttractor),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let src = g.add_node("force.attractor.test.half");
        let att = g.add_node("force.attractor");
        g.connect(Edge {
            from: (src, 0),
            to: (att, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(att, "curve", 0.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &HalfOps, att, 0.0).unwrap();
        match out[0].as_stream().get("accel").unwrap() {
            Column::Vec2(v) => assert!((v[0][0] + 1.25).abs() < 1e-4, "falloff 0.5 halves it"),
            _ => panic!("accel"),
        }
    }

    #[test]
    fn two_attractors_accumulate() {
        let mut g = Graph::new();
        let src = g.add_node("force.attractor.test.src");
        let a1: NodeId = g.add_node("force.attractor");
        let a2: NodeId = g.add_node("force.attractor");
        g.connect(Edge {
            from: (src, 0),
            to: (a1, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (a1, 0),
            to: (a2, 0),
            delayed: false,
        })
        .unwrap();
        for n in [a1, a2] {
            g.set_param(n, "curve", 0.0);
        }
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, a2, 0.0).unwrap();
        match out[0].as_stream().get("accel").unwrap() {
            Column::Vec2(v) => assert!((v[0][0] + 5.0).abs() < 1e-4, "chained forces sum"),
            _ => panic!("accel"),
        }
    }

    #[test]
    fn curves_are_endpoint_exact() {
        for k in 0..4 {
            assert_eq!(curve(k, 0.0), 0.0, "curve {k} at 0");
            assert!((curve(k, 1.0) - 1.0).abs() < 1e-6, "curve {k} at 1");
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
