#![forbid(unsafe_code)]
//! `motion.orbit` — a Motion **spatial transform**: rotates each instance's
//! position `P` **around a pivot** by `angle + playhead·speed` (**degrees**), masked
//! per-instance by the multiplicative `falloff` column (§1.2; absent → `1.0`).
//! The one spatial capability the modifier set lacked — `motion.rotate` only
//! spins the sprite's own basis in place; this moves the *position* on a circle
//! about an arbitrary centre (a static reposition with `speed = 0`, or a
//! continuous orbit over time). Reads the playhead → `Effect::Temporal`. Every
//! other column passes through unchanged (count preserved).
//!
//! Per instance: `rotated = pivot + R(θ)·(P − pivot)`, then the falloff blends
//! `P' = lerp(P, rotated, falloff)` so a focus field makes only its region orbit.
//! `R(θ)` uses the transcendental-free corrected-parabolic `cos/sin` (HR-5, see
//! [`trig`]); each frame rotates the pristine upstream `P` by the absolute angle,
//! so nothing drifts.
//!
//! Params (read via `ctx.param`): `pivot_x` (0), `pivot_y` (0), `angle` (0° — a
//! static offset), `speed` (72°/sec — continuous orbit). Degrees is the app's one
//! authored-angle unit; the cycle-based trig converts at the edge (`deg / 360`,
//! an exact IEEE division — deterministic, HR-5).

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod trig;
use trig::cos_sin_cycles;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.orbit"),
    name: "motion.orbit",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Reads the playhead (θ scrolls with `speed`) → pull-side; the rotation math
    // is nonetheless transcendental-free for cross-platform stability.
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "pivot_x",
            default: 0.0,
        },
        ParamSpec {
            name: "pivot_y",
            default: 0.0,
        },
        ParamSpec {
            name: "angle",
            default: 0.0,
        },
        ParamSpec {
            name: "speed",
            default: 72.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Degrees per full turn — the exact divisor that takes an authored angle into
/// the cycle-based trig's unit. IEEE division is correctly rounded, so this is
/// deterministic (HR-5); multiplying by a reciprocal would not be exact.
const DEG_PER_TURN: f32 = 360.0;

/// The multiplicative `falloff` weight for instance `i` (absent → `1.0`).
fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// Rotate `p` about `pivot` by the matrix `(c, s)`, then blend from the original
/// toward the rotated position by `f` (the falloff): `f = 0` keeps `p`, `f = 1`
/// takes the full orbit.
fn orbit_point(p: [f32; 2], pivot: [f32; 2], c: f32, s: f32, f: f32) -> [f32; 2] {
    let dx = p[0] - pivot[0];
    let dy = p[1] - pivot[1];
    let rx = pivot[0] + (c * dx - s * dy);
    let ry = pivot[1] + (s * dx + c * dy);
    [p[0] + (rx - p[0]) * f, p[1] + (ry - p[1]) * f]
}

struct MotionOrbit;

impl NodeOp for MotionOrbit {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let pivot = [ctx.param("pivot_x"), ctx.param("pivot_y")];
        let angle = ctx.param("angle");
        let speed = ctx.param("speed");
        let theta_deg = angle + ctx.playhead() as f32 * speed;
        let (c, s) = cos_sin_cycles(theta_deg / DEG_PER_TURN);
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            let base: Vec<[f32; 2]> = match input.get("P") {
                Some(Column::Vec2(v)) => v.clone(),
                _ => vec![[0.0, 0.0]; n],
            };
            let moved: Vec<[f32; 2]> = (0..n)
                .map(|i| {
                    let p = base.get(i).copied().unwrap_or([0.0, 0.0]);
                    orbit_point(p, pivot, c, s, falloff_at(input, i))
                })
                .collect();
            let mut out = Stream::new(n);
            for (name, col) in input.columns() {
                if name != "P" {
                    out.set(name.clone(), col.clone());
                }
            }
            out.set("P", Column::Vec2(moved));
            out
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionOrbit))?;
    // M1.R1 — UI metadata (a spatial transform → blue transform, rounded-rect).
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Orbit",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1): signed pivot, static angle (degrees, a `deg` box),
/// orbit speed (degrees/sec — a rate is not an angle, so it stays a slider).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "pivot_x",
        label: "Pivot X",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "pivot_y",
        label: "Pivot Y",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "angle",
        label: "Angle",
        min: -360.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "speed",
        label: "Speed",
        min: -720.0,
        max: 720.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // Source: one instance at (1, 0) + a falloff so masking is exercised.
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.orbit.test.src"),
        name: "motion.orbit.test.src",
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
                    .with("P", Column::Vec2(vec![[1.0, 0.0], [1.0, 0.0]]))
                    .with("falloff", Column::Scalar(vec![1.0, 0.0])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionOrbit),
                _ => None,
            }
        }
    }

    fn orbit_p(
        setup: impl FnOnce(&mut Graph, ph2d_nodegraph::graph::NodeId),
        ph: f64,
    ) -> Vec<[f32; 2]> {
        let mut g = Graph::new();
        let src = g.add_node("motion.orbit.test.src");
        let ob = g.add_node("motion.orbit");
        g.connect(Edge {
            from: (src, 0),
            to: (ob, 0),
            delayed: false,
        })
        .unwrap();
        setup(&mut g, ob);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, ob, ph).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    }

    #[test]
    fn quarter_turn_about_origin_maps_x_axis_to_y_axis() {
        // angle 90°, speed 0, pivot origin: (1,0) → ≈(0,1). Instance 1 has
        // falloff 0 → stays put (mask blends back to the original).
        let p = orbit_p(
            |g, ob| {
                g.set_param(ob, "angle", 90.0);
                g.set_param(ob, "speed", 0.0);
            },
            0.0,
        );
        assert!((p[0][0]).abs() < 1e-5, "x → ~0");
        assert!((p[0][1] - 1.0).abs() < 1e-5, "y → ~1");
        assert_eq!(p[1], [1.0, 0.0], "falloff 0 → unmoved");
    }

    #[test]
    fn speed_drives_a_continuous_orbit_over_time() {
        // speed 90°/sec: at t=1 the point has swept a quarter turn (like the
        // static 90° angle above). Different time → different position.
        let a = orbit_p(|g, ob| g.set_param(ob, "speed", 90.0), 0.0);
        let b = orbit_p(|g, ob| g.set_param(ob, "speed", 90.0), 1.0);
        assert_eq!(a[0], [1.0, 0.0], "t=0 → identity (θ=0)");
        assert!((b[0][1] - 1.0).abs() < 1e-5, "t=1 → quarter-turn to y≈1");
    }

    #[test]
    fn a_full_360_degree_angle_is_the_identity() {
        // The degrees→cycles edge is exact: 360° wraps to a whole cycle, so the
        // point lands back on itself (guards the `deg / 360` divisor).
        let p = orbit_p(
            |g, ob| {
                g.set_param(ob, "angle", 360.0);
                g.set_param(ob, "speed", 0.0);
            },
            0.0,
        );
        assert!(
            (p[0][0] - 1.0).abs() < 1e-5 && p[0][1].abs() < 1e-5,
            "360° → identity"
        );
    }

    #[test]
    fn pivot_offsets_the_orbit_centre() {
        // Orbit about (1,0): the point AT the pivot never moves.
        let p = orbit_p(
            |g, ob| {
                g.set_param(ob, "pivot_x", 1.0);
                g.set_param(ob, "angle", 108.0);
                g.set_param(ob, "speed", 0.0);
            },
            0.0,
        );
        assert!(
            (p[0][0] - 1.0).abs() < 1e-5 && p[0][1].abs() < 1e-5,
            "the pivot is fixed"
        );
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
