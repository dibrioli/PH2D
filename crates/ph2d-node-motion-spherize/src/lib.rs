#![forbid(unsafe_code)]
//! `motion.spherize` — a **radial bulge / pinch** lens deformer: magnify or compress
//! the layout around its centre, like a fisheye (Motion Nodes M3, deformers — doc 01
//! §3 / doc 24). The nonlinear radial counterpart to the affine/projective warps
//! (`motion.transform`, `motion.four_point_warp`): Photoshop's Spherize/Pinch, a
//! magnifying-glass over the field.
//!
//! **Algorithm — a smooth radial magnification.** Each element is displaced along its
//! radius from the layout centroid: `p' = c + (p − c)·scale(r)`, with
//! `scale(r) = 1 + amount·(1 − (r/R)²)` inside radius `R` and `1` outside. `amount > 0`
//! magnifies the centre (**bulge** — things near the middle spread out, identity at the
//! rim); `amount < 0` compresses it (**pinch**). The quadratic falloff makes the rim
//! seamless. Transcendental-free (HR-5): one `sqrt` for the radius, the profile is a
//! polynomial — no trig.
//!
//! **`amount` is animatable** (the value domain): a `value.lfo` swells the field out
//! and sucks it back. Unconnected `amount` reads as `0.5` (a gentle bulge, so a bare
//! node shows something); `amount = 0` is the identity. `radius` (world units) sets the
//! lens size, centred on the layout's centroid. Falloff-masked. `Effect::Pure`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `amount` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Below this a radius is treated as zero (the centre point — no direction to push).
const EPS: f32 = 1e-6;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.spherize"),
    name: "motion.spherize",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // Signed bulge (+) / pinch (−) strength (animatable). Optional: unconnected
        // reads as 0.5 (a gentle bulge).
        PortSpec {
            name: "amount",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // The lens radius (world units), centred on the layout's centroid.
        ParamSpec {
            name: "radius",
            default: 3.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// The `amount`: unconnected (empty) → 0.5; else the first element (broadcast).
fn amount_of(vals: &[f32]) -> f32 {
    vals.first().copied().unwrap_or(0.5)
}

fn falloff_at(vals: &[f32], i: usize) -> f32 {
    match vals.len() {
        0 => 1.0,
        1 => vals[0],
        _ => vals.get(i).copied().unwrap_or(1.0),
    }
}

/// Bulge/pinch every element around the centroid within `radius`, blended per element
/// by `falloff`. A pure function — the whole node.
fn spherize(p: &[[f32; 2]], amount: f32, radius: f32, falloff: &[f32]) -> Vec<[f32; 2]> {
    let n = p.len();
    if n == 0 {
        return Vec::new();
    }
    let mut c = p
        .iter()
        .fold([0.0f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]]);
    c = [c[0] / n as f32, c[1] / n as f32];
    let r_max = radius.max(EPS);
    (0..n)
        .map(|i| {
            let d = [p[i][0] - c[0], p[i][1] - c[1]];
            let r = (d[0] * d[0] + d[1] * d[1]).sqrt();
            if r < EPS || r >= r_max {
                return p[i]; // the centre (no direction) or outside the lens
            }
            let t = r / r_max;
            let scale = 1.0 + amount * (1.0 - t * t);
            let warped = [c[0] + d[0] * scale, c[1] + d[1] * scale];
            let f = falloff_at(falloff, i).clamp(0.0, 1.0);
            [
                p[i][0] + (warped[0] - p[i][0]) * f,
                p[i][1] + (warped[1] - p[i][1]) * f,
            ]
        })
        .collect()
}

struct MotionSpherize;

impl NodeOp for MotionSpherize {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let radius = ctx.param("radius").max(EPS);
        let amount = amount_of(&scalar_col(ctx.input(1), VALUE_COL));
        let input = ctx.input(0);
        let n = input.count();
        let p: Vec<[f32; 2]> = match input.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => vec![[0.0, 0.0]; n],
        };
        let falloff = scalar_col(input, "falloff");
        let out_p = spherize(&p, amount, radius, &falloff);
        let mut out = Stream::new(n);
        for (name, col) in input.columns() {
            if name != "P" {
                out.set(name.clone(), col.clone());
            }
        }
        out.set("P", Column::Vec2(out_p));
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionSpherize))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Spherize",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: "radius",
    label: "Radius",
    min: 0.1,
    max: 20.0,
    step: 0.05,
    widget: ParamWidget::Slider,
}];

#[cfg(test)]
mod tests {
    use super::*;

    /// The radial distance of element `i` from the set's centroid.
    fn radius_of(p: &[[f32; 2]], i: usize) -> f32 {
        let n = p.len() as f32;
        let c = p
            .iter()
            .fold([0.0f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]]);
        let c = [c[0] / n, c[1] / n];
        let (dx, dy) = (p[i][0] - c[0], p[i][1] - c[1]);
        (dx * dx + dy * dy).sqrt()
    }

    fn ring() -> Vec<[f32; 2]> {
        // A centred cross: centre + 4 points at radius 1 along the axes.
        vec![[0.0, 0.0], [1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]]
    }

    /// `amount = 0` is the identity — every element is unchanged.
    #[test]
    fn zero_amount_is_the_identity() {
        let p = ring();
        let out = spherize(&p, 0.0, 3.0, &[]);
        for (o, q) in out.iter().zip(&p) {
            assert!((o[0] - q[0]).abs() < 1e-5 && (o[1] - q[1]).abs() < 1e-5);
        }
    }

    /// Bulge (`amount > 0`) pushes the ring OUTWARD (its radius grows); pinch
    /// (`amount < 0`) pulls it in. FALSIFIED by the identity (radius unchanged).
    #[test]
    fn bulge_pushes_out_and_pinch_pulls_in() {
        let p = ring();
        let base = radius_of(&p, 1); // 1.0
        let bulged = spherize(&p, 0.6, 3.0, &[]);
        let pinched = spherize(&p, -0.6, 3.0, &[]);
        assert!(radius_of(&bulged, 1) > base + 0.05, "bulge pushes out");
        assert!(radius_of(&pinched, 1) < base - 0.05, "pinch pulls in");
    }

    /// The centre element (radius 0) never moves and never NaNs; a point beyond the
    /// lens radius is untouched.
    #[test]
    fn centre_is_stable_and_outside_the_radius_is_untouched() {
        let mut p = ring();
        // Symmetric far points keep the centroid at the origin (radius 5 > lens 3).
        p.push([5.0, 0.0]);
        p.push([-5.0, 0.0]);
        let out = spherize(&p, 0.9, 3.0, &[]);
        assert_eq!(out[0], [0.0, 0.0], "the centre holds (centroid ~origin)");
        assert!(out.iter().all(|q| q[0].is_finite() && q[1].is_finite()));
        // The far points (distance from the centroid > radius) are unchanged.
        assert!(
            (out[5][0] - 5.0).abs() < 1e-4 && (out[6][0] + 5.0).abs() < 1e-4,
            "outside the lens"
        );
    }

    /// Falloff masks the bulge: a falloff-0 element stays put under a strong bulge.
    #[test]
    fn falloff_masks_the_bulge() {
        let p = ring();
        let falloff = vec![1.0, 0.0, 1.0, 1.0, 1.0]; // element 1 masked
        let out = spherize(&p, 0.8, 3.0, &falloff);
        assert!(
            (out[1][0] - 1.0).abs() < 1e-5 && (out[1][1]).abs() < 1e-5,
            "falloff 0 → unchanged: {:?}",
            out[1]
        );
    }

    /// Cooks through the registry, copies columns and bulges P.
    #[test]
    fn registers_and_bulges_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.spherize.test.src"),
            name: "motion.spherize.test.src",
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
                &SRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(
                    Stream::new(3)
                        .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [-1.0, 0.0]]))
                        .with("size", Column::Vec2(vec![[0.4, 0.4]; 3])),
                );
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionSpherize),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let src = g.add_node("motion.spherize.test.src");
        let sp = g.add_node("motion.spherize");
        g.connect(Edge {
            from: (src, 0),
            to: (sp, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, sp, 0.0).unwrap();
        let s = out[0].as_stream();
        assert!(s.get("size").is_some(), "columns pass through");
        match s.get("P").unwrap() {
            // Unconnected amount → 0.5 (bulge): the ±1 points spread past ±1.
            Column::Vec2(v) => assert!(v[1][0] > 1.0 && v[2][0] < -1.0, "bulged: {v:?}"),
            _ => panic!("P"),
        }
    }
}
