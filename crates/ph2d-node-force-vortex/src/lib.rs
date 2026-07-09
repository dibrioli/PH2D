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

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod accum;
use accum::{add_accel, falloff_at, vec2_at};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Dead-zone radius around the centre (direction is meaningless as d → 0).
const DEAD_ZONE: f32 = 1e-3;

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
        ParamSpec {
            name: "clockwise",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
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
        let out = {
            let input = ctx.input(0);
            let contrib: Vec<[f32; 2]> = (0..input.count())
                .map(|i| {
                    let p = vec2_at(input, "P", i, [0.0, 0.0]);
                    let dx = p[0] - center[0];
                    let dy = p[1] - center[1];
                    let d = (dx * dx + dy * dy).sqrt();
                    if d < DEAD_ZONE || d > radius {
                        return [0.0, 0.0];
                    }
                    // Linear edge falloff (reference parity) × the focus field.
                    let mag = strength * (1.0 - d / radius) * falloff_at(input, i) / d;
                    if clockwise {
                        [dy * mag, -dx * mag]
                    } else {
                        [-dy * mag, dx * mag]
                    }
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
    reg.register(Box::new(ForceVortex))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Vortex",
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
        param: "clockwise",
        label: "Clockwise",
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

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
