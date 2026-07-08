#![forbid(unsafe_code)]
//! `motion.falloff.circle` — a Motion **focus field**: writes the multiplicative
//! `falloff` column (§1.2) as a radial mask centred at `(center_x, center_y)` —
//! `1.0` at the centre fading with a smoothstep edge to `0.0` at `radius`. It
//! *multiplies* into any existing `falloff` so several fields compose, and
//! passes every other column through unchanged (count preserved). Pure.
//!
//! Params (read via `ctx.param`): `center_x` (0), `center_y` (0), `radius` (5),
//! `invert` (0/1 — flips the mask to `1 - f` so the field bites the *outside*).
//! Downstream modifiers read this `falloff` column to scale their effect.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.falloff.circle"),
    name: "motion.falloff.circle",
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
            name: "radius",
            default: 5.0,
        },
        ParamSpec {
            name: "invert",
            default: 0.0,
        },
    ],
    // CPU-only by design (see grid/transform): no Instances-domain WGSL runtime
    // exists yet, and this writes a Vec2-derived scalar column, not a scalar
    // `ph2d-expr` map an `eval_column` could lower.
    lowerings: &[LoweringKind::Cpu],
};

/// Smoothstep `t*t*(3-2t)` on a pre-clamped `t ∈ [0,1]` — transcendental-free
/// (HR-5), so the mask is bit-identical across platforms for the replay hash.
fn smoothstep01(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Radial mask for an instance at distance `d` from the centre: `1` at the
/// centre, a smooth edge to `0` at `radius`. `radius <= 0` degenerates to an
/// empty field (`0` everywhere). `invert` mirrors it to `1 - f`.
fn radial_mask(d: f32, radius: f32, invert: bool) -> f32 {
    let f = if radius > 0.0 {
        1.0 - smoothstep01((d / radius).clamp(0.0, 1.0))
    } else {
        0.0
    };
    if invert { 1.0 - f } else { f }
}

struct MotionFalloffCircle;

impl NodeOp for MotionFalloffCircle {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let (cx, cy) = (ctx.param("center_x"), ctx.param("center_y"));
        let radius = ctx.param("radius");
        let invert = ctx.param("invert") >= 0.5;
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
                let (dx, dy) = (p[0] - cx, p[1] - cy);
                // IEEE sqrt is correctly-rounded / deterministic (HR-5-safe,
                // unlike sin/cos) — the distance is fundamental here.
                let d = (dx * dx + dy * dy).sqrt();
                let base = prev.and_then(|v| v.get(i).copied()).unwrap_or(1.0);
                fall.push(base * radial_mask(d, radius, invert));
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
    reg.register(Box::new(MotionFalloffCircle))?;
    // M1.R1 — UI metadata (a focus field → amber, diamond value silhouette).
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Circle Falloff",
            category: ph2d_node_registry::NodeUiCategory::Focus,
            silhouette: ph2d_node_registry::NodeSilhouette::Diamond,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1): signed centre, positive radius, invert toggle.
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

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

    // Source: 3 instances on a line at x = 0, 5, 10 (y = 0).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.falloff.circle.test.src"),
        name: "motion.falloff.circle.test.src",
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
                t if t == MANIFEST.id => Some(&MotionFalloffCircle),
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
    fn radius5_center0_is_one_at_center_zero_at_edge() {
        let mut g = Graph::new();
        let src = g.add_node("motion.falloff.circle.test.src");
        let foc = g.add_node("motion.falloff.circle");
        g.connect(Edge {
            from: (src, 0),
            to: (foc, 0),
            delayed: false,
        })
        .unwrap();
        // default radius 5, centre (0,0): x=0 → 1, x=5 → 0 (edge), x=10 → 0.
        let f = falloff_of(&g, &Ops, foc);
        assert_eq!(f, vec![1.0, 0.0, 0.0]);
    }

    #[test]
    fn invert_flips_the_mask() {
        let mut g = Graph::new();
        let src = g.add_node("motion.falloff.circle.test.src");
        let foc = g.add_node("motion.falloff.circle");
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
    fn mask_math_is_smooth_and_clamped() {
        // Half radius → smoothstep(0.5)=0.5 → mask 1-0.5=0.5. Beyond radius clamps to 0.
        assert_eq!(radial_mask(0.0, 4.0, false), 1.0);
        assert_eq!(radial_mask(2.0, 4.0, false), 0.5);
        assert_eq!(radial_mask(4.0, 4.0, false), 0.0);
        assert_eq!(radial_mask(8.0, 4.0, false), 0.0); // clamped past the edge
        assert_eq!(radial_mask(1.0, 0.0, false), 0.0); // degenerate radius
    }
}
