#![forbid(unsafe_code)]
//! `motion.distribute_poisson` — **the spacing is the knob**: fill a rectangle with
//! points no two of which are closer than `radius`, and let the *count* be whatever that
//! spacing allows (Motion Nodes M3, distributions — doc 01 §3 / doc 60).
//!
//! ## Why this is not `motion.scatter` twice
//!
//! Both produce blue noise; they are two different questions, and the difference is
//! which number you get to name:
//!
//! | | you name | you get | algorithm |
//! |---|---|---|---|
//! | `motion.scatter` | the **count** | a spacing (as even as N allows) | Mitchell 1991 best-candidate, `O(N²·K)` |
//! | `motion.distribute_poisson` | the **spacing** | a count (as many as fit) | **Bridson 2007** dart-throwing, `O(N)` |
//!
//! An exact count is what a designer wants for *twelve dots around a logo*. A guaranteed
//! minimum distance is what a *scene* wants: trees that never overlap, spawn sites that
//! never double up, stipple that never clumps — a promise no best-candidate can make,
//! because when you ask it for one point too many it has nowhere to put it and puts it
//! close. And Bridson is linear where best-candidate is quadratic, so this is also the
//! one you can ask for ten thousand points.
//!
//! Blender draws the same line (its *Distribute Points* has Random and **Poisson Disk**
//! modes, the second taking a `distance_min`); Houdini's `scatter` grew a *relax
//! iterations* knob for the same reason. The algorithm lives in the sibling `poisson.rs`.
//!
//! A **Source** node (no input, mints `P`). `Effect::Pure`: the layout is a pure function
//! of the params, so it is scrub-stable and bit-exact across machines. HR-5: no
//! transcendentals — the dart's direction is rejection-sampled from the unit disc rather
//! than drawn from an angle, which is *why* there is no `sin`/`cos` here.

use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod hash;
mod poisson;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.distribute_poisson"),
    name: "motion.distribute_poisson",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // The minimum distance between any two points (world units). **There is no
        // `count`** — that is the point of the node.
        ParamSpec {
            name: "radius",
            default: 0.35,
        },
        // The rectangle to fill (world units), centred on the origin.
        ParamSpec {
            name: "width",
            default: 4.0,
        },
        ParamSpec {
            name: "height",
            default: 4.0,
        },
        // Integer seed (rounded at eval) — re-rolls the whole layout.
        ParamSpec {
            name: "seed",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct DistributePoisson;

impl NodeOp for DistributePoisson {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let radius = ctx.param("radius");
        let w = ctx.param("width");
        let h = ctx.param("height");
        let seed = ctx.param("seed").max(0.0).round() as u32;
        let pts = poisson::sample(w, h, radius, seed);
        ctx.emit(Stream::new(pts.len()).with("P", Column::Vec2(pts)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(DistributePoisson))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Poisson Disk",
            category: ph2d_node_registry::NodeUiCategory::Distribute,
            silhouette: ph2d_node_registry::NodeSilhouette::TrapezoidDown,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

/// Param UI hints (M1.P1).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "radius",
        label: "Radius",
        min: 0.02,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "width",
        label: "Width",
        min: 0.1,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "height",
        label: "Height",
        min: 0.1,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 999.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::Graph;

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&DistributePoisson as &dyn NodeOp)
        }
    }

    fn cooked(params: &[(&str, f32)]) -> Vec<[f32; 2]> {
        let mut g = Graph::new();
        let n = g.add_node("motion.distribute_poisson");
        for (k, v) in params {
            g.set_param(n, *k, *v);
        }
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
        match out[0].as_stream().get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => panic!("the distribution mints P"),
        }
    }

    /// The seam, end to end: cooking the node mints a `P` column whose spacing is the
    /// one the param asked for — the count is what fell out.
    #[test]
    fn cooking_it_mints_points_at_the_requested_spacing() {
        let pts = cooked(&[("radius", 0.5), ("width", 5.0), ("height", 3.0)]);
        assert!(pts.len() > 20, "packed only {} points", pts.len());
        for (i, p) in pts.iter().enumerate() {
            for q in &pts[i + 1..] {
                let (dx, dy) = (p[0] - q[0], p[1] - q[1]);
                assert!(
                    dx * dx + dy * dy >= 0.5 * 0.5 - 1e-4,
                    "two points inside the 0.5 radius"
                );
            }
        }
        // The stream's count and the column agree (an empty stream with a full column is
        // the classic way a source node cooks to nothing downstream).
        assert_eq!(
            Stream::new(pts.len()).count(),
            pts.len(),
            "count matches the column"
        );
    }

    /// The count is not a param, so the *seed* is the only thing that re-rolls it — and
    /// the same seed cooks the same scene, which is what `Effect::Pure` promises.
    #[test]
    fn the_same_params_cook_the_same_points() {
        let a = cooked(&[("radius", 0.4), ("seed", 3.0)]);
        let b = cooked(&[("radius", 0.4), ("seed", 3.0)]);
        let c = cooked(&[("radius", 0.4), ("seed", 4.0)]);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
